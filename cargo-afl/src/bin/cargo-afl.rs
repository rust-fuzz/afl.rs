use clap::crate_version;

use std::env;
use std::ffi::{OsStr, OsString};
use std::process::{self, Command, Stdio};

#[path = "../common.rs"]
mod common;

fn main() {
    if !common::archive_file_path(None).exists() {
        let version = common::afl_rustc_version();
        eprintln!(
            "AFL LLVM runtime is not built with Rust {version}, run `cargo \
             install --force cargo-afl` to build it."
        );
        process::exit(1);
    }

    let app_matches = clap_app().get_matches();
    // This unwrap is okay because we set SubcommandRequiredElseHelp at the top level, and afl is
    // the only subcommand
    let afl_matches = app_matches.subcommand_matches("afl").unwrap();

    match afl_matches.subcommand() {
        Some(("addseeds", sub_matches)) => {
            let args = sub_matches
                .get_many::<OsString>("afl-addseeds args")
                .unwrap_or_default();
            run_afl(args, "afl-addseeds");
        }
        Some(("analyze", sub_matches)) => {
            let args = sub_matches
                .get_many::<OsString>("afl-analyze args")
                .unwrap_or_default();
            run_afl(args, "afl-analyze");
        }
        Some(("cmin", sub_matches)) => {
            let args = sub_matches
                .get_many::<OsString>("afl-cmin args")
                .unwrap_or_default();
            run_afl(args, "afl-cmin");
        }
        Some(("fuzz", sub_matches)) => {
            let args = sub_matches
                .get_many::<OsString>("afl-fuzz args")
                .unwrap_or_default();
            // We prepend -c0 to the AFL++ arguments
            let cmplog_flag = vec![OsString::from("-c0")];
            let args = cmplog_flag.iter().chain(args);
            run_afl(args, "afl-fuzz");
        }
        Some(("gotcpu", sub_matches)) => {
            let args = sub_matches
                .get_many::<OsString>("afl-gotcpu args")
                .unwrap_or_default();
            run_afl(args, "afl-gotcpu");
        }
        Some(("plot", sub_matches)) => {
            let args = sub_matches
                .get_many::<OsString>("afl-plot args")
                .unwrap_or_default();
            run_afl(args, "afl-plot");
        }
        Some(("showmap", sub_matches)) => {
            let args = sub_matches
                .get_many::<OsString>("afl-showmap args")
                .unwrap_or_default();
            run_afl(args, "afl-showmap");
        }
        Some(("system-config", sub_matches)) => {
            let args = sub_matches
                .get_many::<OsString>("afl-system-config args")
                .unwrap_or_default();
            run_afl(args, "afl-system-config");
        }
        Some(("tmin", sub_matches)) => {
            let args = sub_matches
                .get_many::<OsString>("afl-tmin args")
                .unwrap_or_default();
            run_afl(args, "afl-tmin");
        }
        Some(("whatsup", sub_matches)) => {
            let args = sub_matches
                .get_many::<OsString>("afl-whatsup args")
                .unwrap_or_default();
            run_afl(args, "afl-whatsup");
        }
        Some((subcommand, sub_matches)) => {
            let args = sub_matches.get_many::<OsString>("").unwrap_or_default();
            run_cargo(subcommand, args);
        }
        // unreachable due to SubcommandRequiredElseHelp on "afl" subcommand
        None => unreachable!(),
    }
}

#[allow(clippy::too_many_lines)]
fn clap_app() -> clap::Command {
    use clap::{value_parser, Arg, Command};

    let help = "In addition to the subcommands above, Cargo subcommands are also \
                      supported (see `cargo help` for a list of all Cargo subcommands).";

    Command::new("cargo afl")
        .display_name("cargo")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("afl")
                .version(crate_version!())
                .subcommand_required(true)
                .arg_required_else_help(true)
                .allow_external_subcommands(true)
                .external_subcommand_value_parser(value_parser!(OsString))
                .override_usage("cargo afl [SUBCOMMAND or Cargo SUBCOMMAND]")
                .after_help(help)
                .subcommand(
                    Command::new("addseeds")
                        .about("Invoke afl-addseeds")
                        .allow_hyphen_values(true)
                        .disable_help_subcommand(true)
                        .disable_help_flag(true)
                        .disable_version_flag(true)
                        .arg(
                            Arg::new("afl-addseeds args")
                                .value_parser(value_parser!(OsString))
                                .num_args(0..),
                        ),
                )
                .subcommand(
                    Command::new("analyze")
                        .about("Invoke afl-analyze")
                        .allow_hyphen_values(true)
                        .disable_help_subcommand(true)
                        .disable_help_flag(true)
                        .disable_version_flag(true)
                        .arg(
                            Arg::new("afl-analyze args")
                                .value_parser(value_parser!(OsString))
                                .num_args(0..),
                        ),
                )
                .subcommand(
                    Command::new("cmin")
                        .about("Invoke afl-cmin")
                        .allow_hyphen_values(true)
                        .disable_help_subcommand(true)
                        .disable_help_flag(true)
                        .disable_version_flag(true)
                        .arg(
                            Arg::new("afl-cmin args")
                                .value_parser(value_parser!(OsString))
                                .num_args(0..),
                        ),
                )
                .subcommand(
                    Command::new("fuzz")
                        .about("Invoke afl-fuzz")
                        .allow_hyphen_values(true)
                        .disable_help_subcommand(true)
                        .disable_help_flag(true)
                        .disable_version_flag(true)
                        .arg(
                            Arg::new("max_total_time")
                                .long("max_total_time")
                                .num_args(1)
                                .value_parser(value_parser!(u64))
                                .help("Maximum amount of time to run the fuzzer"),
                        )
                        .arg(
                            Arg::new("afl-fuzz args")
                                .value_parser(value_parser!(OsString))
                                .num_args(0..),
                        ),
                )
                .subcommand(
                    Command::new("gotcpu")
                        .about("Invoke afl-gotcpu")
                        .allow_hyphen_values(true)
                        .disable_help_subcommand(true)
                        .disable_help_flag(true)
                        .disable_version_flag(true)
                        .arg(
                            Arg::new("afl-gotcpu args")
                                .value_parser(value_parser!(OsString))
                                .num_args(0..),
                        ),
                )
                .subcommand(
                    Command::new("plot")
                        .about("Invoke afl-plot")
                        .allow_hyphen_values(true)
                        .disable_help_subcommand(true)
                        .disable_help_flag(true)
                        .disable_version_flag(true)
                        .arg(
                            Arg::new("afl-plot args")
                                .value_parser(value_parser!(OsString))
                                .num_args(0..),
                        ),
                )
                .subcommand(
                    Command::new("showmap")
                        .about("Invoke afl-showmap")
                        .allow_hyphen_values(true)
                        .disable_help_subcommand(true)
                        .disable_help_flag(true)
                        .disable_version_flag(true)
                        .arg(
                            Arg::new("afl-showmap args")
                                .value_parser(value_parser!(OsString))
                                .num_args(0..),
                        ),
                )
                .subcommand(
                    Command::new("system-config")
                        .about("Invoke afl-system-config (beware, called with sudo!)")
                        .allow_hyphen_values(true)
                        .disable_help_subcommand(true)
                        .disable_help_flag(true)
                        .disable_version_flag(true)
                        .arg(
                            Arg::new("afl-system-config args")
                                .value_parser(value_parser!(OsString))
                                .num_args(0..),
                        ),
                )
                .subcommand(
                    Command::new("tmin")
                        .about("Invoke afl-tmin")
                        .allow_hyphen_values(true)
                        .disable_help_subcommand(true)
                        .disable_help_flag(true)
                        .disable_version_flag(true)
                        .arg(
                            Arg::new("afl-tmin args")
                                .value_parser(value_parser!(OsString))
                                .num_args(0..),
                        ),
                )
                .subcommand(
                    Command::new("whatsup")
                        .about("Invoke afl-whatsup")
                        .allow_hyphen_values(true)
                        .disable_help_subcommand(true)
                        .disable_help_flag(true)
                        .disable_version_flag(true)
                        .arg(
                            Arg::new("afl-whatsup args")
                                .value_parser(value_parser!(OsString))
                                .num_args(0..),
                        ),
                ),
        )
}

fn run_afl<I, S>(args: I, tool: &str)
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let no_sudo = env::var("NO_SUDO").is_ok();
    let cmd_path = common::afl_dir(None).join("bin").join(tool);
    let mut cmd = if !no_sudo && tool == "afl-system-config" {
        let mut cmd = Command::new("sudo");
        cmd.args([OsStr::new("--reset-timestamp"), cmd_path.as_os_str()]);
        eprintln!("Running: {cmd:?}");
        cmd
    } else {
        Command::new(cmd_path)
    };
    cmd.args(args);

    let status = cmd.status().unwrap();

    if tool == "afl-fuzz" && !status.success() {
        eprintln!(
            "
If you see an error message like `shmget() failed` above, try running the following command:

    cargo-afl afl system-config

Note: You might be prompted to enter your password as root privileges are required and hence sudo is run within this command."
        );
    }
    process::exit(status.code().unwrap_or(1));
}

fn run_cargo<I, S>(subcommand: &str, args: I)
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    #![allow(clippy::similar_names)]

    let cargo_path = env::var("CARGO").expect("Could not determine `cargo` path");

    // add some flags to sanitizers to make them work with Rust code
    let asan_options = env::var("ASAN_OPTIONS").unwrap_or_default();
    let asan_options =
        format!("detect_odr_violation=0:abort_on_error=1:symbolize=0:{asan_options}");

    let tsan_options = env::var("TSAN_OPTIONS").unwrap_or_default();
    let tsan_options = format!("report_signal_unsafe=0:{tsan_options}");

    // The new LLVM pass manager was enabled in rustc 1.59.
    let version_meta = rustc_version::version_meta().unwrap();
    let passes = if (version_meta.semver.minor >= 59 || is_nightly())
        && version_meta.llvm_version.map_or(true, |v| v.major >= 13)
    {
        "sancov-module"
    } else {
        "sancov"
    };

    // `-C codegen-units=1` is needed to work around link errors
    // https://github.com/rust-fuzz/afl.rs/pull/193#issuecomment-933550430
    let mut rustflags = format!(
        "-C debug-assertions \
         -C overflow_checks \
         -C passes={passes} \
         -C codegen-units=1 \
         -C llvm-args=-sanitizer-coverage-level=3 \
         -C llvm-args=-sanitizer-coverage-trace-pc-guard \
         -C llvm-args=-sanitizer-coverage-prune-blocks=0 \
         -C llvm-args=-sanitizer-coverage-trace-compares \
         -C opt-level=3 \
         -C target-cpu=native "
    );

    let no_cfg_fuzzing = env::var("AFL_NO_CFG_FUZZING").is_ok();
    if no_cfg_fuzzing {
        rustflags.push_str("--cfg no_fuzzing ");
        // afl-fuzz is sensitive to AFL_ env variables. Let's remove this particular one - it did it's job
        env::remove_var("AFL_NO_CFG_FUZZING");
    } else {
        rustflags.push_str("--cfg fuzzing ");
    }

    if cfg!(target_os = "linux") {
        // work around https://github.com/rust-fuzz/afl.rs/issues/141 /
        // https://github.com/rust-lang/rust/issues/53945, can be removed once
        // those are fixed.
        rustflags.push_str("-Clink-arg=-fuse-ld=gold ");
    }

    // RUSTFLAGS are not used by rustdoc, instead RUSTDOCFLAGS are used. Since
    // doctests will try to link against afl-llvm-rt, set up RUSTDOCFLAGS to
    // have doctests built the same as other code to avoid issues with doctests.
    let mut rustdocflags = rustflags.clone();

    rustflags.push_str(&format!(
        "-l afl-llvm-rt \
         -L {} ",
        common::afl_llvm_rt_dir(None).display()
    ));

    // add user provided flags
    rustflags.push_str(&env::var("RUSTFLAGS").unwrap_or_default());
    rustdocflags.push_str(&env::var("RUSTDOCFLAGS").unwrap_or_default());

    let status = Command::new(cargo_path)
        .arg(subcommand)
        .args(args)
        .env("RUSTFLAGS", &rustflags)
        .env("RUSTDOCFLAGS", &rustdocflags)
        .env("ASAN_OPTIONS", asan_options)
        .env("TSAN_OPTIONS", tsan_options)
        .status()
        .unwrap();
    process::exit(status.code().unwrap_or(1));
}

fn is_nightly() -> bool {
    Command::new("rustc")
        .args(["-Z", "help"])
        .stderr(Stdio::null())
        .status()
        .unwrap()
        .success()
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use assert_cmd::Command;
    use std::os::unix::ffi::OsStringExt;

    #[test]
    fn test_app() {
        clap_app().debug_assert();
    }

    #[test]
    fn display_name() {
        assert!(
            String::from_utf8(cargo_afl(&["-V"]).output().unwrap().stdout)
                .unwrap()
                .starts_with("cargo-afl")
        );
    }

    #[test]
    fn afl_required_else_help() {
        assert_eq!(
            String::from_utf8(command().arg("--help").output().unwrap().stdout).unwrap(),
            String::from_utf8(command().output().unwrap().stderr).unwrap()
        );
    }

    #[test]
    fn subcommand_required_else_help() {
        assert_eq!(
            String::from_utf8(cargo_afl(&["--help"]).output().unwrap().stdout).unwrap(),
            String::from_utf8(cargo_afl::<&OsStr>(&[]).output().unwrap().stderr).unwrap()
        );
    }

    #[test]
    fn external_subcommands_allow_invalid_utf8() {
        let _arg_matches = clap_app()
            .try_get_matches_from([
                OsStr::new("cargo"),
                OsStr::new("afl"),
                OsStr::new("test"),
                &invalid_utf8(),
            ])
            .unwrap();
    }

    const SUBCOMMANDS: &[&str] = &[
        "addseeds",
        "analyze",
        "cmin",
        "fuzz",
        "gotcpu",
        "plot",
        "showmap",
        "system-config",
        "tmin",
        "whatsup",
    ];

    #[test]
    fn subcommands_allow_invalid_utf8() {
        for &subcommand in SUBCOMMANDS {
            let _arg_matches = clap_app()
                .try_get_matches_from([
                    OsStr::new("cargo"),
                    OsStr::new("afl"),
                    OsStr::new(subcommand),
                    &invalid_utf8(),
                ])
                .unwrap();
        }
    }

    #[test]
    fn subcommands_allow_hyphen_values() {
        for &subcommand in SUBCOMMANDS {
            let _arg_matches = clap_app()
                .try_get_matches_from(["cargo", "afl", subcommand, "-i", "--input"])
                .unwrap();
        }
    }

    #[test]
    fn subcommands_help_subcommand_disabled() {
        assert!(
            String::from_utf8(cargo_afl(&["help"]).output().unwrap().stdout)
                .unwrap()
                .starts_with("Usage:")
        );

        for &subcommand in SUBCOMMANDS {
            assert!(
                !String::from_utf8(cargo_afl(&[subcommand, "help"]).output().unwrap().stdout)
                    .unwrap()
                    .starts_with("Usage:")
            );
        }
    }

    #[test]
    fn subcommands_help_flag_disabled() {
        assert!(
            String::from_utf8(cargo_afl(&["--help"]).output().unwrap().stdout)
                .unwrap()
                .starts_with("Usage:")
        );

        for &subcommand in SUBCOMMANDS {
            assert!(!String::from_utf8(
                cargo_afl(&[subcommand, "--help"]).output().unwrap().stdout
            )
            .unwrap()
            .starts_with("Usage:"));
        }
    }

    #[test]
    fn subcommands_version_flag_disabled() {
        assert!(
            String::from_utf8(cargo_afl(&["-V"]).output().unwrap().stdout)
                .unwrap()
                .starts_with("cargo-afl")
        );

        for &subcommand in SUBCOMMANDS {
            assert!(
                !String::from_utf8(cargo_afl(&[subcommand, "-V"]).output().unwrap().stdout)
                    .unwrap()
                    .starts_with("cargo-afl")
            );
        }
    }

    fn cargo_afl<T: AsRef<OsStr>>(args: &[T]) -> Command {
        let mut command = command();
        command.arg("afl").args(args).env("NO_SUDO", "1");
        command
    }

    fn command() -> Command {
        Command::cargo_bin("cargo-afl").unwrap()
    }

    fn invalid_utf8() -> OsString {
        OsString::from_vec(vec![0xfe])
    }

    #[test]
    fn invalid_utf8_is_invalid() {
        assert!(String::from_utf8(invalid_utf8().into_vec()).is_err());
    }
}
