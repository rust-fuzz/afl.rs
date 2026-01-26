use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin_cmd;
use cargo_afl_common::SUBCOMMANDS;
use std::ffi::OsStr;
use std::process::Output;

#[test]
fn display_name() {
    let output = cargo_afl(&["-V"]).output().unwrap();
    assert_success(&output, None);
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .starts_with("cargo-afl")
    );
}

#[test]
fn afl_required_else_help() {
    let lhs = command().arg("--help").output().unwrap();
    let rhs = command().output().unwrap();
    assert_success(&lhs, None);
    assert_failure(&rhs, None);
    assert_eq!(
        String::from_utf8(lhs.stdout).unwrap(),
        String::from_utf8(rhs.stderr).unwrap()
    );
}

#[test]
fn subcommand_required_else_help() {
    let lhs = cargo_afl(&["--help"]).output().unwrap();
    let rhs = cargo_afl::<&OsStr>(&[]).output().unwrap();
    assert_success(&lhs, None);
    assert_failure(&rhs, None);
    assert_eq!(
        String::from_utf8(lhs.stdout).unwrap(),
        String::from_utf8(rhs.stderr).unwrap()
    );
}

#[test]
fn subcommands_help_subcommand_disabled() {
    let output = cargo_afl(&["help"]).output().unwrap();
    assert_success(&output, None);
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .starts_with("Usage:")
    );

    for &subcommand in SUBCOMMANDS {
        let output = cargo_afl(&[subcommand, "help"]).output().unwrap();
        assert_failure(&output, Some(subcommand));
        assert!(
            !String::from_utf8(output.stdout)
                .unwrap()
                .starts_with("Usage:")
        );
    }
}

#[test]
fn subcommands_help_flag_disabled() {
    let output = cargo_afl(&["--help"]).output().unwrap();
    assert_success(&output, None);
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .starts_with("Usage:")
    );

    for &subcommand in SUBCOMMANDS {
        let output = cargo_afl(&[subcommand, "--help"]).output().unwrap();
        // smoelius: `afl-addseeds`, `cmin`, and `afl-system-config` have `--help` flags.
        if ["addseeds", "cmin", "system-config"].contains(&subcommand) {
            assert_success(&output, Some(subcommand));
        } else {
            assert_failure(&output, Some(subcommand));
        }
        assert!(
            !String::from_utf8(output.stdout)
                .unwrap()
                .starts_with("Usage:")
        );
    }
}

#[test]
fn subcommands_version_flag_disabled() {
    let output = cargo_afl(&["-V"]).output().unwrap();
    assert_success(&output, None);
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .starts_with("cargo-afl")
    );

    for &subcommand in SUBCOMMANDS {
        let output = cargo_afl(&[subcommand, "-V"]).output().unwrap();
        assert_failure(&output, Some(subcommand));
        assert!(
            !String::from_utf8(output.stdout)
                .unwrap()
                .starts_with("cargo-afl")
        );
    }
}

#[test]
fn tag_requires_update() {
    let output = cargo_afl(&["config", "--tag", "v4.33c"]).output().unwrap();
    assert_failure(&output, None);
    assert!(String::from_utf8(output.stderr).unwrap().contains(
        "error: the following required arguments were not provided:
  --update"
    ));
}

fn cargo_afl<T: AsRef<OsStr>>(args: &[T]) -> Command {
    let mut command = command();
    command.arg("afl").args(args).env("NO_SUDO", "1");
    command
}

fn command() -> Command {
    cargo_bin_cmd!("cargo-afl")
}

#[track_caller]
fn assert_success(output: &Output, subcommand: Option<&str>) {
    assert!(
        output.status.success(),
        "{}",
        if let Some(subcommand) = subcommand {
            format!("{subcommand} failed")
        } else {
            String::new()
        }
    );
}

#[track_caller]
fn assert_failure(output: &Output, subcommand: Option<&str>) {
    assert!(
        !output.status.success(),
        "{}",
        if let Some(subcommand) = subcommand {
            format!("{subcommand} succeeded")
        } else {
            String::new()
        }
    );
}
