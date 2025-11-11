use anyhow::Result;
use assert_cmd::cargo::cargo_bin_cmd;
use std::path::Path;
use tempfile::tempdir;

#[allow(dead_code)]
#[path = "../src/common/mod.rs"]
mod common;

use common::config::{
    copy_aflplusplus_submodule, is_repo, remove_aflplusplus_dir, update_to_stable_or_tag,
};

#[derive(Clone, Copy, Debug)]
enum State {
    Nonexistent,
    Submodule,
    Tag(&'static str),
    Stable,
}

const TESTCASES: &[(State, State, &[&str])] = &[
    // smoelius: There is currently no way to update to the submodule.
    // (State::Nonexistent, State::Submodule, &[]),
    (
        State::Nonexistent,
        State::Tag("v4.33c"),
        &[
            #[cfg(not(target_os = "macos"))]
            "Note: switching to 'v4.33c'.",
            "HEAD is now at",
        ],
    ),
    (
        State::Nonexistent,
        State::Stable,
        &[
            #[cfg(not(target_os = "macos"))]
            "Note: switching to 'origin/stable'.",
            "HEAD is now at",
        ],
    ),
    (
        State::Submodule,
        State::Tag("v4.33c"),
        &[
            #[cfg(not(target_os = "macos"))]
            "Note: switching to 'v4.33c'.",
            "HEAD is now at",
        ],
    ),
    (
        State::Submodule,
        State::Stable,
        &[
            #[cfg(not(target_os = "macos"))]
            "Note: switching to 'origin/stable'.",
            "HEAD is now at",
        ],
    ),
    // smoelius: It should be possible to go from a tag to the stable version.
    (
        State::Tag("v4.33c"),
        State::Stable,
        &["Previous HEAD position was", "HEAD is now at"],
    ),
    // smoelius: It should be possible to go from the stable version to a tag.
    (
        State::Stable,
        State::Tag("v4.33c"),
        &["Previous HEAD position was", "HEAD is now at"],
    ),
];

#[test]
fn update() {
    let mut base_dir = common::xdg_base_dir();

    for &(before, after, line_prefixes) in TESTCASES {
        eprintln!("{before:?} -> {after:?}");

        let tempdir = tempdir().unwrap();

        // smoelius: Based on https://github.com/whitequark/rust-xdg/issues/44, the recommended
        // way of testing with a fake value of `XDG_DATA_HOME` seems to be manually overwriting
        // the `data_home` field in `xdg::BaseDirectories`.
        base_dir.data_home = Some(tempdir.path().to_path_buf());

        let aflplusplus_dir = common::aflplusplus_dir_from_base_dir(&base_dir).unwrap();

        assert!(aflplusplus_dir.starts_with(tempdir.path()));

        set_aflplusplus_dir_contents(before, &aflplusplus_dir).unwrap();

        let mut command = cargo_bin_cmd!("cargo-afl");
        command.args(["afl", "config", "--update"]);
        command.env("XDG_DATA_HOME", tempdir.path());
        match after {
            State::Nonexistent | State::Submodule => unreachable!(),
            State::Tag(tag) => {
                command.args(["--tag", tag]);
            }
            State::Stable => {}
        }
        let output = command.output().unwrap();
        assert!(output.status.success());
        let stderr = String::from_utf8(output.stderr).unwrap();
        contains_expected_line_prefixes(&stderr, line_prefixes);
    }
}

fn set_aflplusplus_dir_contents(state: State, aflplusplus_dir: &Path) -> Result<()> {
    let result = match state {
        State::Nonexistent => remove_aflplusplus_dir(aflplusplus_dir),
        State::Submodule => copy_aflplusplus_submodule(aflplusplus_dir),
        State::Tag(tag) => update_to_stable_or_tag(aflplusplus_dir, Some(tag)),
        State::Stable => update_to_stable_or_tag(aflplusplus_dir, None),
    };
    // smoelius: Sanity.
    assert!(
        is_repo(aflplusplus_dir)
            .is_ok_and(|value| value == matches!(state, State::Tag(_) | State::Stable))
    );
    result
}

fn contains_expected_line_prefixes(stderr: &str, mut line_prefixes: &[&str]) {
    for line in stderr.lines() {
        if line_prefixes
            .first()
            .is_some_and(|prefix| line.starts_with(prefix))
        {
            line_prefixes = &line_prefixes[1..];
        }
    }
    assert!(
        line_prefixes.is_empty(),
        "Could not find line prefix {line_prefixes:?}:\n```\n{stderr}```"
    );
}
