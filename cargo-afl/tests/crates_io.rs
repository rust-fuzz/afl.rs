use assert_cmd::{Command, cargo::cargo_bin_cmd};
use predicates::prelude::*;
use std::{env, fs::OpenOptions, io::Write, path::Path};
use tempfile::tempdir;
use yare::parameterized;

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

    cargo_bin_cmd!("cargo-afl")
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

#[parameterized(
    install_without_plugins_then_force = { false, Test::Force },
    install_without_plugins_then_change_plugins = { false, Test::ChangePlugins },
    install_with_plugins_then_force = { true, Test::Force },
    install_with_plugins_then_change_plugins = { true, Test::ChangePlugins },
)]
fn install_and_config(install_with_plugins: bool, test: Test) {
    let is_nightly =
        rustc_version::version_meta().unwrap().channel == rustc_version::Channel::Nightly;

    // smoelius: Plugins are currently not tested on macOS.
    if (install_with_plugins || test == Test::ChangePlugins)
        && (!is_nightly || cfg!(target_os = "macos"))
    {
        return;
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

fn cargo_afl_build_command(home: &Path, cargo_afl: &Path) -> Command {
    let mut command = Command::new(cargo_afl);
    command.args(["afl", "config", "--build"]).env("HOME", home);
    command
}

#[parameterized(
    afl = { "afl" },
    cargo_afl = { "cargo-afl" },
    cargo_afl_common = { "cargo-afl-common" },
)]
fn package(subdir: &str) {
    Command::new("cargo")
        .args(["package", "--allow-dirty"])
        .current_dir(Path::new("..").join(subdir))
        .assert()
        .success();
}
