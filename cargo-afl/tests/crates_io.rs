use assert_cmd::Command;
use predicates::prelude::*;
use std::{env, fs::OpenOptions, io::Write, path::Path};
use tempfile::tempdir;

const BUILD_MSG: &str = "\
warning: You appear to be building `afl` not under `cargo-afl`.
warning: Perhaps you used `cargo build` instead of `cargo afl build`?
";

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
        .stderr(predicates::str::contains(BUILD_MSG).not());

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
        .stderr(predicates::str::contains(BUILD_MSG));

    Command::cargo_bin("cargo-afl")
        .unwrap()
        .current_dir(&tempdir)
        .args(["afl", "build"])
        .env("TESTING_BUILD", "1")
        .assert()
        .success()
        .stderr(predicates::str::contains(BUILD_MSG).not());
}

#[test]
fn install() {
    let tempdir = tempdir().unwrap();

    let cargo_afl = tempdir.path().join("bin/cargo-afl");

    assert!(!cargo_afl.exists());

    Command::new("cargo")
        .args(["install", "--path", "../cargo-afl"])
        .env("CARGO_HOME", tempdir.path())
        .env("TESTING_INSTALL", "1")
        .assert()
        .success();

    Command::new(cargo_afl)
        .args(["afl", "--help"])
        .assert()
        .success();
}

#[test]
fn publish() {
    for subdir in ["afl", "cargo-afl"] {
        Command::new("cargo")
            .args(["publish", "--allow-dirty", "--dry-run", "--no-verify"])
            .current_dir(Path::new("..").join(subdir))
            .assert()
            .success();
    }
}
