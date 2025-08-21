use assert_cmd::Command;
use predicates::prelude::*;
use std::{env, fs::OpenOptions, io::Write, path::Path};
use tempfile::tempdir;

const BUILD_MSGS: &[&str] = &[
    "You appear to be building `afl` not under `cargo-afl`.",
    "Perhaps you used `cargo build` instead of `cargo afl build`?",
];

#[ctor::ctor]
fn init() {
    unsafe {
        env::set_var("CARGO_TERM_COLOR", "never");
    }
}

#[test]
fn build() {
    let tempdir = tempdir().unwrap();

    Command::new("cargo")
        .current_dir(&tempdir)
        .args(["init", "--name", "a"])
        .assert()
        .success();

    Command::new("cargo")
        .current_dir(&tempdir)
        .args(["build"])
        .env("TESTING_BUILD", "1")
        .assert()
        .success()
        .stderr(
            predicates::str::contains(BUILD_MSGS[0])
                .not()
                .and(predicates::str::contains(BUILD_MSGS[1]).not()),
        );

    let mut file = OpenOptions::new()
        .append(true)
        .open(tempdir.path().join("Cargo.toml"))
        .unwrap();
    writeln!(
        file,
        r#"afl = {{ path = "{}" }}"#,
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../afl")
            .display()
    )
    .unwrap();

    Command::new("cargo")
        .current_dir(&tempdir)
        .args(["build"])
        .env("TESTING_BUILD", "1")
        .assert()
        .success()
        .stderr(
            predicates::str::contains(BUILD_MSGS[0]).and(predicates::str::contains(BUILD_MSGS[1])),
        );

    Command::cargo_bin("cargo-afl")
        .unwrap()
        .current_dir(&tempdir)
        .args(["afl", "build"])
        .env("TESTING_BUILD", "1")
        .assert()
        .success()
        .stderr(
            predicates::str::contains(BUILD_MSGS[0])
                .not()
                .and(predicates::str::contains(BUILD_MSGS[1]).not()),
        );
}

#[derive(PartialEq)]
enum Test {
    Force,
    ChangePlugins,
}

#[test]
fn install_and_config() {
    let is_nightly =
        rustc_version::version_meta().unwrap().channel == rustc_version::Channel::Nightly;

    for (install_with_plugins, test) in [false, true]
        .into_iter()
        .zip([Test::Force, Test::ChangePlugins])
    {
        // smoelius: Plugins are currently not tested on macOS.
        if (install_with_plugins || test == Test::ChangePlugins)
            && (!is_nightly || cfg!(target_os = "macos"))
        {
            continue;
        }

        let temp_home = tempdir().unwrap();
        let temp_cargo_home = tempdir().unwrap();

        let cargo_afl = temp_cargo_home.path().join("bin/cargo-afl");

        assert!(!cargo_afl.exists());

        let mut command = Command::new("cargo");
        command
            .args(["install", "--path", "../cargo-afl"])
            .env("HOME", temp_home.path())
            .env("CARGO_HOME", temp_cargo_home.path())
            .env("TESTING_INSTALL", "1");
        if install_with_plugins {
            command.arg("--features=plugins");
        }
        command.assert().success();

        Command::new(&cargo_afl)
            .args(["afl", "--help"])
            .assert()
            .success();

        // smoelius: Verify that `cargo afl config --build` fails since AFL++ was already built.
        let mut command = cargo_afl_build_command(temp_home.path(), &cargo_afl);
        if install_with_plugins {
            command.arg("--plugins");
        }
        command.assert().failure().stderr(
            predicates::str::is_match(
                "AFL LLVM runtime was already built for Rust [^;]*; run `cargo \
                 afl config --build --force` to rebuild it\\.",
            )
            .unwrap(),
        );

        // smoelius: Verify that `--force` or a change in `--plugins` is needed to rebuild AFL++.
        let mut command = cargo_afl_build_command(temp_home.path(), &cargo_afl);
        match test {
            Test::Force => {
                command.arg("--force");
            }
            Test::ChangePlugins => {
                if !install_with_plugins {
                    command.arg("--plugins");
                }
            }
        }
        command.assert().success();
    }
}

fn cargo_afl_build_command(home: &Path, cargo_afl: &Path) -> Command {
    let mut command = Command::new(cargo_afl);
    command.args(["afl", "config", "--build"]).env("HOME", home);
    command
}

#[test]
fn package() {
    for subdir in ["afl", "cargo-afl"] {
        Command::new("cargo")
            .args(["package", "--allow-dirty"])
            .current_dir(Path::new("..").join(subdir))
            .assert()
            .success();
    }
}
