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
    env::set_var("CARGO_TERM_COLOR", "never");
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

#[test]
fn install_and_config() {
    let temp_home = tempdir().unwrap();
    let temp_cargo_home = tempdir().unwrap();

    let cargo_afl = temp_cargo_home.path().join("bin/cargo-afl");

    assert!(!cargo_afl.exists());

    Command::new("cargo")
        .args(["install", "--path", "../cargo-afl"])
        .env("HOME", temp_home.path())
        .env("CARGO_HOME", temp_cargo_home.path())
        .env("TESTING_INSTALL", "1")
        .assert()
        .success();

    Command::new(&cargo_afl)
        .args(["afl", "--help"])
        .assert()
        .success();

    // smoelius: Verify that `--force` is needed to rebuild AFL++.
    Command::new(&cargo_afl)
        .args(["afl", "config", "--build"])
        .env("HOME", temp_home.path())
        .assert()
        .failure()
        .stderr(
            predicates::str::is_match(
                "AFL LLVM runtime was already built for Rust [^;]*; run `cargo \
                 afl config --build --force` to rebuild it\\.",
            )
            .unwrap(),
        );

    Command::new(cargo_afl)
        .args(["afl", "config", "--build", "--force"])
        .env("HOME", temp_home.path())
        .assert()
        .success();
}

#[test]
fn publish() {
    for subdir in ["afl", "cargo-afl"] {
        Command::new("cargo")
            .args(["publish", "--allow-dirty", "--dry-run"])
            .current_dir(Path::new("..").join(subdir))
            .assert()
            .success();
    }
}
