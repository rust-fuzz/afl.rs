use clap::{crate_version, value_t};

use std::env;
use std::ffi::OsStr;
use std::io;
use std::process::{self, Command, ExitStatus};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[path = "../common.rs"]
mod common;

fn main() {
    if !common::archive_file_path().exists() {
        let version = common::afl_rustc_version();
        eprintln!(
            "AFL LLVM runtime is not built with Rust {}, run `cargo \
             install --force afl` to build it.",
            version
        );
        process::exit(1);
    }

    let app_matches = clap_app().get_matches();
    // This unwrap is okay because we set SubcommandRequiredElseHelp at the top level, and afl is
    // the only subcommand
    let afl_matches = app_matches.subcommand_matches("afl").unwrap();

    match afl_matches.subcommand() {
        ("analyze", Some(sub_matches)) => {
            let args = sub_matches
                .values_of_os("afl-analyze args")
                .unwrap_or_default();
            run_afl(args, "afl-analyze", None);
        }
        ("cmin", Some(sub_matches)) => {
            let args = sub_matches
                .values_of_os("afl-cmin args")
                .unwrap_or_default();
            run_afl(args, "afl-cmin", None);
        }
        ("fuzz", Some(sub_matches)) => {
            let args = sub_matches
                .values_of_os("afl-fuzz args")
                .unwrap_or_default();
            let timeout = sub_matches
                .value_of("max_total_time")
                .map(|_| value_t!(sub_matches, "max_total_time", u64).unwrap_or_else(|e| e.exit()));
            run_afl(args, "afl-fuzz", timeout);
        }
        ("gotcpu", Some(sub_matches)) => {
            let args = sub_matches
                .values_of_os("afl-gotcpu args")
                .unwrap_or_default();
            run_afl(args, "afl-gotcpu", None);
        }
        ("plot", Some(sub_matches)) => {
            let args = sub_matches
                .values_of_os("afl-plot args")
                .unwrap_or_default();
            run_afl(args, "afl-plot", None);
        }
        ("showmap", Some(sub_matches)) => {
            let args = sub_matches
                .values_of_os("afl-showmap args")
                .unwrap_or_default();
            run_afl(args, "afl-showmap", None);
        }
        ("tmin", Some(sub_matches)) => {
            let args = sub_matches
                .values_of_os("afl-tmin args")
                .unwrap_or_default();
            run_afl(args, "afl-tmin", None);
        }
        ("whatsup", Some(sub_matches)) => {
            let args = sub_matches
                .values_of_os("afl-whatsup args")
                .unwrap_or_default();
            run_afl(args, "afl-whatsup", None);
        }
        (subcommand, Some(sub_matches)) => {
            let args = sub_matches.values_of_os("").unwrap_or_default();
            run_cargo(subcommand, args);
        }
        // unreachable due to SubcommandRequiredElseHelp on "afl" subcommand
        (_, None) => unreachable!(),
    }
}

fn clap_app() -> clap::App<'static, 'static> {
    use clap::{
        App,
        AppSettings::{
            AllowExternalSubcommands, AllowLeadingHyphen, DisableHelpFlags, DisableHelpSubcommand,
            DisableVersion, SubcommandRequiredElseHelp,
        },
        Arg, SubCommand,
    };

    App::new("cargo afl")
        .bin_name("cargo")
        .setting(SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("afl")
                .version(crate_version!())
                .setting(SubcommandRequiredElseHelp)
                .setting(AllowExternalSubcommands)
                .usage("cargo afl [SUBCOMMAND or Cargo SUBCOMMAND]")
                .after_help(
                    "In addition to the subcommands above, Cargo subcommands are also \
                 supported (see `cargo help` for a list of all Cargo subcommands).",
                )
                .subcommand(
                    SubCommand::with_name("analyze")
                        .about("Invoke afl-analyze")
                        .setting(AllowLeadingHyphen)
                        .setting(DisableHelpSubcommand)
                        .setting(DisableHelpFlags)
                        .setting(DisableVersion)
                        .arg(Arg::with_name("afl-analyze args").multiple(true)),
                )
                .subcommand(
                    SubCommand::with_name("cmin")
                        .about("Invoke afl-cmin")
                        .setting(AllowLeadingHyphen)
                        .setting(DisableHelpSubcommand)
                        .setting(DisableHelpFlags)
                        .setting(DisableVersion)
                        .arg(Arg::with_name("afl-cmin args").multiple(true)),
                )
                .subcommand(
                    SubCommand::with_name("fuzz")
                        .about("Invoke afl-fuzz")
                        .setting(AllowLeadingHyphen)
                        .setting(DisableHelpSubcommand)
                        .setting(DisableHelpFlags)
                        .setting(DisableVersion)
                        .arg(
                            Arg::with_name("max_total_time")
                                .long("max_total_time")
                                .takes_value(true)
                                .help("Maximum amount of time to run the fuzzer"),
                        )
                        .arg(Arg::with_name("afl-fuzz args").multiple(true)),
                )
                .subcommand(
                    SubCommand::with_name("gotcpu")
                        .about("Invoke afl-gotcpu")
                        .setting(AllowLeadingHyphen)
                        .setting(DisableHelpSubcommand)
                        .setting(DisableHelpFlags)
                        .setting(DisableVersion)
                        .arg(Arg::with_name("afl-gotcpu args").multiple(true)),
                )
                .subcommand(
                    SubCommand::with_name("plot")
                        .about("Invoke afl-plot")
                        .setting(AllowLeadingHyphen)
                        .setting(DisableHelpSubcommand)
                        .setting(DisableHelpFlags)
                        .setting(DisableVersion)
                        .arg(Arg::with_name("afl-plot args").multiple(true)),
                )
                .subcommand(
                    SubCommand::with_name("showmap")
                        .about("Invoke afl-showmap")
                        .setting(AllowLeadingHyphen)
                        .setting(DisableHelpSubcommand)
                        .setting(DisableHelpFlags)
                        .setting(DisableVersion)
                        .arg(Arg::with_name("afl-showmap args").multiple(true)),
                )
                .subcommand(
                    SubCommand::with_name("tmin")
                        .about("Invoke afl-tmin")
                        .setting(AllowLeadingHyphen)
                        .setting(DisableHelpSubcommand)
                        .setting(DisableHelpFlags)
                        .setting(DisableVersion)
                        .arg(Arg::with_name("afl-tmin args").multiple(true)),
                )
                .subcommand(
                    SubCommand::with_name("whatsup")
                        .about("Invoke afl-whatsup")
                        .setting(AllowLeadingHyphen)
                        .setting(DisableHelpSubcommand)
                        .setting(DisableHelpFlags)
                        .setting(DisableVersion)
                        .arg(Arg::with_name("afl-whatsup args").multiple(true)),
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
                let elapsed = Instant::now() - start_time;
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
                let ret = libc::kill(pid as i32, libc::SIGTERM);
                if ret == -1 {
                    Err(io::Error::last_os_error())?
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
            Err(io::Error::last_os_error())?
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

fn run_afl<I, S>(args: I, cmd: &str, timeout: Option<u64>)
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let cmd_path = common::afl_dir().join("bin").join(cmd);
    let mut cmd = Command::new(cmd_path);
    cmd.args(args);
    let status = run_timeout_terminate(cmd, timeout).unwrap();
    process::exit(status.code().unwrap_or(1));
}

fn run_cargo<I, S>(subcommand: &str, args: I)
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let cargo_path = env!("CARGO");

    // add some flags to sanitizers to make them work with Rust code
    let asan_options = env::var("ASAN_OPTIONS").unwrap_or_default();
    let asan_options = format!("detect_odr_violation=0:{}", asan_options);

    let tsan_options = env::var("TSAN_OPTIONS").unwrap_or_default();
    let tsan_options = format!("report_signal_unsafe=0:{}", tsan_options);

    let mut rustflags = format!(
        "--cfg fuzzing \
         -C debug-assertions \
         -C overflow_checks \
         -C passes=sancov \
         -C llvm-args=-sanitizer-coverage-level=3 \
         -C llvm-args=-sanitizer-coverage-trace-pc-guard \
         -C llvm-args=-sanitizer-coverage-prune-blocks=0 \
         -C opt-level=3 \
         -C target-cpu=native \
         -C debuginfo=0 \
         -l afl-llvm-rt \
         -L {} ",
        common::afl_llvm_rt_dir().display()
    );

    // RUSTFLAGS are not used by rustdoc, instead RUSTDOCFLAGS are used. Since
    // doctests will try to link against afl-llvm-rt, set up RUSTDOCFLAGS to
    // have doctests built the same as other code to avoid issues with doctests.
    let mut rustdocflags = format!(
        "--cfg fuzzing \
         -C debug-assertions \
         -C overflow_checks \
         -C passes=sancov \
         -C llvm-args=-sanitizer-coverage-level=3 \
         -C llvm-args=-sanitizer-coverage-trace-pc-guard \
         -C llvm-args=-sanitizer-coverage-prune-blocks=0 \
         -C opt-level=3 \
         -C target-cpu=native \
         -C debuginfo=0 \
         -L {} ",
        common::afl_llvm_rt_dir().display()
    );

    if cfg!(target_os = "linux") {
        // work around https://github.com/rust-fuzz/afl.rs/issues/141 /
        // https://github.com/rust-lang/rust/issues/53945, can be removed once
        // those are fixed.
        rustflags.push_str("-Clink-arg=-fuse-ld=gold ");
        rustdocflags.push_str("-Clink-arg=-fuse-ld=gold ");
    }

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
