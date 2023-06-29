use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;
use tempfile::tempdir;

#[path = "../src/common.rs"]
mod common;

struct Test {
    subdir: &'static str,
    should_contain_msgs: bool,
}

static INSTALL_MSG: &str = "warning: You appear to be installing the `cargo-afl` binary with:
warning:     cargo install afl
warning: A future version of afl.rs will require you to use:
warning:     cargo install cargo-afl
warning: You can use the new command now, if you like.
warning: Note: If the binary is already installed, you may need to add --force.
";

static TESTS: &[Test] = &[
    Test {
        subdir: "afl",
        should_contain_msgs: true,
    },
    Test {
        subdir: "cargo-afl",
        should_contain_msgs: false,
    },
];

#[test]
fn install() {
    for &Test {
        subdir,
        should_contain_msgs,
    } in TESTS
    {
        let tempdir = tempdir().unwrap();

        let cargo_afl = tempdir.path().join("bin/cargo-afl");

        assert!(!cargo_afl.exists());

        let assert = Command::new("cargo")
            .args([
                "install",
                "--path",
                &Path::new("..").join(subdir).to_string_lossy(),
            ])
            .env("CARGO_HOME", tempdir.path())
            .env("CARGO_TERM_COLOR", "never")
            .env("TESTING_INSTALL", "1")
            .assert()
            .success();

        if should_contain_msgs {
            assert.stderr(predicates::str::contains(INSTALL_MSG));
        } else {
            assert.stderr(predicates::str::contains(INSTALL_MSG).not());
        }

        let assert = Command::new(cargo_afl)
            .args(["afl", "--help"])
            .assert()
            .success();

        if should_contain_msgs {
            assert.stdout(predicates::str::contains(common::HELP_MSG));
        } else {
            assert.stdout(predicates::str::contains(common::HELP_MSG).not());
        }
    }
}

#[test]
fn publish() {
    for &Test { subdir, .. } in TESTS {
        Command::new("cargo")
            .args(["publish", "--allow-dirty", "--dry-run", "--no-verify"])
            .current_dir(Path::new("..").join(subdir))
            .assert()
            .success();
    }
}
