use clap::crate_version;

use std::env;
use std::ffi::{OsStr, OsString};
use std::io;
use std::process::{self, Command, ExitStatus, Stdio};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[path = "../common.rs"]
mod common;

fn main() {
    if !common::archive_file_path(None).exists() {
        let version = common::afl_rustc_version();
        eprintln!(
            "AFL LLVM runtime is not built with Rust {version}, run `cargo \
             install --force afl` to build it."
        );
        process::exit(1);
    }

    let app_matches = clap_app().get_matches();
    // This unwrap is okay because we set SubcommandRequiredElseHelp at the top level, and afl is
    // the only subcommand
    let afl_matches = app_matches.subcommand_matches("afl").unwrap();

    match afl_matches.subcommand() {
        Some(("analyze", sub_matches)) => {
            let args = sub_matches
                .get_many::<OsString>("afl-analyze args")
                .unwrap_or_default();
            run_afl(args, "afl-analyze", None);
        }
        Some(("cmin", sub_matches)) => {
            let args = sub_matches
                .get_many::<OsString>("afl-cmin args")
                .unwrap_or_default();
            run_afl(args, "afl-cmin", None);
        }
        Some(("fuzz", sub_matches)) => {
            let mut args = sub_matches
                .get_many::<OsString>("afl-fuzz args")
                .unwrap_or_default();
            // We use next recursively on the args iterator, until we hit "--".
            // We are then able to append `-c0` to the AFL++ arguments.
            let mut front_args = vec![];
            let separator = OsString::from("--");
            let cmplog_flag = OsString::from("-c0");
            for next_value in args.by_ref() {
                if *next_value == separator {
                    front_args.push(&cmplog_flag);
                    break;
                }
                front_args.push(next_value);
            }
            let args = front_args.into_iter().chain(args);
            let timeout = sub_matches.get_one::<u64>("max_total_time").copied();
            if timeout.is_some() {
                eprintln!(
                    "`--max_total_time` is deprecated and will be removed in a \
                     future version of afl.rs. Please use `-V seconds`."
                );
            }
            run_afl(args, "afl-fuzz", timeout);
        }
        Some(("gotcpu", sub_matches)) => {
            let args = sub_matches
                .get_many::<OsString>("afl-gotcpu args")
                .unwrap_or_default();
            run_afl(args, "afl-gotcpu", None);
        }
        Some(("plot", sub_matches)) => {
            let args = sub_matches
                .get_many::<OsString>("afl-plot args")
                .unwrap_or_default();
            run_afl(args, "afl-plot", None);
        }
        Some(("showmap", sub_matches)) => {
            let args = sub_matches
                .get_many::<OsString>("afl-showmap args")
                .unwrap_or_default();
            run_afl(args, "afl-showmap", None);
        }
        Some(("tmin", sub_matches)) => {
            let args = sub_matches
                .get_many::<OsString>("afl-tmin args")
                .unwrap_or_default();
            run_afl(args, "afl-tmin", None);
        }
        Some(("whatsup", sub_matches)) => {
            let args = sub_matches
                .get_many::<OsString>("afl-whatsup args")
                .unwrap_or_default();
            run_afl(args, "afl-whatsup", None);
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
                .after_help(
                    "In addition to the subcommands above, Cargo subcommands are also \
                 supported (see `cargo help` for a list of all Cargo subcommands).",
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

fn run_timeout_terminate(mut cmd: Command, timeout: Option<u64>) -> Result<ExitStatus, io::Error> {
    let timeout = match timeout {
        Some(timeout) => Duration::from_secs(timeout),
        None => return cmd.status(),
    };
    let start_time = Instant::now();

    let mut child = cmd.spawn()?;
    let pid = child.id();

    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let (stop_mutex, condvar) = &*pair;
    let thread_handle = {
        let pair = pair.clone();
        thread::spawn(move || -> Result<(), io::Error> {
            // This thread will wait until the child process has exited, or the
            // timeout has elapsed, whichever comes first. If the timeout
            // elapses, and the process is still running, it will send SIGTERM
            // to the child process.

            let (stop_mutex, condvar) = &*pair;
            let mut stop = stop_mutex.lock().unwrap();
            loop {
                let elapsed = start_time.elapsed();
                if elapsed >= timeout {
                    break;
                }

                let dur = timeout - elapsed;
                let results = condvar.wait_timeout(stop, dur).unwrap();
                stop = results.0;
                if *stop {
                    // Blocking waitid call on the main thread has returned,
                    // thus the child process has terminated
                    return Ok(());
                }
                if results.1.timed_out() {
                    break;
                }
            }

            // Since the waitid call on the main thread is using WNOWAIT, the
            // child process won't be cleaned up (until after this thread
            // exits and the main thread calls wait) and thus its PID won't be
            // reused by another, unrelated process.
            unsafe {
                #[allow(clippy::cast_possible_wrap)]
                let ret = libc::kill(pid as i32, libc::SIGTERM);
                if ret == -1 {
                    return Err(io::Error::last_os_error());
                }
            }

            Ok(())
        })
    };

    unsafe {
        // Block until the child process terminates, but leave it in a waitable
        // state still
        let ret = libc::waitid(
            libc::P_PID,
            pid,
            std::ptr::null_mut(),
            libc::WEXITED | libc::WNOWAIT,
        );
        if ret == -1 {
            return Err(io::Error::last_os_error());
        }
    }
    {
        let mut stop = stop_mutex.lock().unwrap();
        *stop = true;
    }
    // Tell the timeout thread to stop, wake it, and wait for it to exit
    condvar.notify_one();
    thread_handle.join().unwrap()?;

    // Clean up zombie and get exit status (this won't block, because the child
    // process has terminated and is still waitable)
    child.wait()
}

fn run_afl<I, S>(args: I, tool: &str, timeout: Option<u64>)
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let cmd_path = common::afl_dir(None).join("bin").join(tool);
    let mut cmd = Command::new(cmd_path);
    cmd.args(args);
    let status = run_timeout_terminate(cmd, timeout).unwrap();
    #[cfg(target_os = "macos")]
    if tool == "afl-fuzz" && !status.success() {
        let sudo_cmd_path = common::afl_dir(None).join("bin").join("afl-system-config");
        eprintln!(
            "
If you see an error message like `shmget() failed` above, try running the following command:

    sudo {}

Note: You will be prompted to enter your password.",
            sudo_cmd_path.display()
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
         -C llvm-args=-sanitizer-coverage-trace-divs \
         -C opt-level=3 \
         -C target-cpu=native "
    );

    if cfg!(not(feature = "no_cfg_fuzzing")) {
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
        "analyze", "cmin", "fuzz", "gotcpu", "plot", "showmap", "tmin", "whatsup",
    ];

    #[test]
    fn subcommands_allow_invalid_utf8() {
        for &subcommand in SUBCOMMANDS.iter() {
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
        for &subcommand in SUBCOMMANDS.iter() {
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

        for &subcommand in SUBCOMMANDS.iter() {
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

        for &subcommand in SUBCOMMANDS.iter() {
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

        for &subcommand in SUBCOMMANDS.iter() {
            assert!(
                !String::from_utf8(cargo_afl(&[subcommand, "-V"]).output().unwrap().stdout)
                    .unwrap()
                    .starts_with("cargo-afl")
            );
        }
    }

    #[test]
    fn max_total_time_is_deprecated() {
        assert!(String::from_utf8(
            cargo_afl(&["fuzz", "--max_total_time=0"])
                .output()
                .unwrap()
                .stderr
        )
        .unwrap()
        .starts_with("`--max_total_time` is deprecated"));
    }

    fn cargo_afl<T: AsRef<OsStr>>(args: &[T]) -> Command {
        let mut command = command();
        command.arg("afl").args(args);
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
