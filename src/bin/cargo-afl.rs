#[macro_use]
extern crate clap;
extern crate rustc_version;
extern crate xdg;

use std::env;
use std::ffi::OsStr;
use std::process::{self, Command};

#[path = "../common.rs"]
mod common;

fn main() {
    if !common::archive_file_path().exists() {
        let version = common::rustc_version();
        eprintln!("AFL LLVM runtime is not built with Rust {}, run `cargo \
                   install --force afl` to build it.", version);
        process::exit(1);
    }

    let _ = clap_app().get_matches();

    let mut args = env::args().skip(2).peekable(); // skip `cargo` and `afl`
    match args.peek().map(|s| &**s) {
        Some("analyze") => run_afl(args, "afl-analyze"),
        Some("cmin") => run_afl(args, "afl-cmin"),
        Some("fuzz") => run_afl(args, "afl-fuzz"),
        Some("gotcpu") => run_afl(args, "afl-got-cpu"),
        Some("plot") => run_afl(args, "afl-plot"),
        Some("showmap") => run_afl(args, "afl-showmap"),
        Some("tmin") => run_afl(args, "afl-tmin"),
        Some("whatsup") => run_afl(args, "afl-whatsup"),
        _ => run_cargo(args),
    }
}

fn clap_app() -> clap::App<'static, 'static> {
    clap::App::new("cargo afl").bin_name("cargo").subcommand(
        clap::SubCommand::with_name("afl")
            .version(crate_version!())
            .setting(clap::AppSettings::ArgRequiredElseHelp)
            .setting(clap::AppSettings::TrailingVarArg)
            .setting(clap::AppSettings::AllowExternalSubcommands)
            .usage("cargo afl [SUBCOMMAND or Cargo SUBCOMMAND]")
            .after_help(
                "In addition to the subcommands above, Cargo subcommands are also \
                     supported (see `cargo help` for a list of all Cargo subcommands).",
            )
            .subcommand(
                clap::SubCommand::with_name("analyze")
                    .about("Invoke afl-analyze")
                    .setting(clap::AppSettings::AllowLeadingHyphen)
                    .setting(clap::AppSettings::DisableHelpSubcommand)
                    .setting(clap::AppSettings::DisableVersion)
                    .arg(clap::Arg::with_name("h").short("h").hidden(true))
                    .arg(clap::Arg::with_name("help").long("help").hidden(true))
                    .arg(clap::Arg::with_name("afl-analyze args").multiple(true)),
            )
            .subcommand(
                clap::SubCommand::with_name("cmin")
                    .about("Invoke afl-cmin")
                    .setting(clap::AppSettings::AllowLeadingHyphen)
                    .setting(clap::AppSettings::DisableHelpSubcommand)
                    .setting(clap::AppSettings::DisableVersion)
                    .arg(clap::Arg::with_name("h").short("h").hidden(true))
                    .arg(clap::Arg::with_name("help").long("help").hidden(true))
                    .arg(clap::Arg::with_name("afl-cmin args").multiple(true)),
            )
            .subcommand(
                clap::SubCommand::with_name("fuzz")
                    .about("Invoke afl-fuzz")
                    .setting(clap::AppSettings::AllowLeadingHyphen)
                    .setting(clap::AppSettings::DisableHelpSubcommand)
                    .setting(clap::AppSettings::DisableVersion)
                    .arg(clap::Arg::with_name("h").short("h").hidden(true))
                    .arg(clap::Arg::with_name("help").long("help").hidden(true))
                    .arg(clap::Arg::with_name("afl-fuzz args").multiple(true)),
            )
            .subcommand(
                clap::SubCommand::with_name("gotcpu")
                    .about("Invoke afl-gotcpu")
                    .setting(clap::AppSettings::AllowLeadingHyphen)
                    .setting(clap::AppSettings::DisableHelpSubcommand)
                    .setting(clap::AppSettings::DisableVersion)
                    .arg(clap::Arg::with_name("h").short("h").hidden(true))
                    .arg(clap::Arg::with_name("help").long("help").hidden(true))
                    .arg(clap::Arg::with_name("afl-gotcpu args").multiple(true)),
            )
            .subcommand(
                clap::SubCommand::with_name("plot")
                    .about("Invoke afl-plot")
                    .setting(clap::AppSettings::AllowLeadingHyphen)
                    .setting(clap::AppSettings::DisableHelpSubcommand)
                    .setting(clap::AppSettings::DisableVersion)
                    .arg(clap::Arg::with_name("h").short("h").hidden(true))
                    .arg(clap::Arg::with_name("help").long("help").hidden(true))
                    .arg(clap::Arg::with_name("afl-plot args").multiple(true)),
            )
            .subcommand(
                clap::SubCommand::with_name("showmap")
                    .about("Invoke afl-showmap")
                    .setting(clap::AppSettings::AllowLeadingHyphen)
                    .setting(clap::AppSettings::DisableHelpSubcommand)
                    .setting(clap::AppSettings::DisableVersion)
                    .arg(clap::Arg::with_name("h").short("h").hidden(true))
                    .arg(clap::Arg::with_name("help").long("help").hidden(true))
                    .arg(clap::Arg::with_name("afl-showmap args").multiple(true)),
            )
            .subcommand(
                clap::SubCommand::with_name("tmin")
                    .about("Invoke afl-tmin")
                    .setting(clap::AppSettings::AllowLeadingHyphen)
                    .setting(clap::AppSettings::DisableHelpSubcommand)
                    .setting(clap::AppSettings::DisableVersion)
                    .arg(clap::Arg::with_name("h").short("h").hidden(true))
                    .arg(clap::Arg::with_name("help").long("help").hidden(true))
                    .arg(clap::Arg::with_name("afl-tmin args").multiple(true)),
            )
            .subcommand(
                clap::SubCommand::with_name("whatsup")
                    .about("Invoke afl-whatsup")
                    .setting(clap::AppSettings::AllowLeadingHyphen)
                    .setting(clap::AppSettings::DisableHelpSubcommand)
                    .setting(clap::AppSettings::DisableVersion)
                    .arg(clap::Arg::with_name("h").short("h").hidden(true))
                    .arg(clap::Arg::with_name("help").long("help").hidden(true))
                    .arg(clap::Arg::with_name("afl-whatsup args").multiple(true)),
            ),
    )
}

fn run_afl<I, S>(args: I, cmd: &str)
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let cmd_path = common::afl_dir().join("bin").join(cmd);
    let status = Command::new(cmd_path)
        .args(args.into_iter().skip(1)) // skip afl sub-command
        .status()
        .unwrap();
    process::exit(status.code().unwrap_or(1));
}

fn run_cargo<I, S>(args: I)
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
        rustflags.push_str("-Clink-arg=-fuse-ld=gold");
        rustdocflags.push_str("-Clink-arg=-fuse-ld=gold");
    }

    // add user provided flags
    rustflags.push_str(&env::var("RUSTFLAGS").unwrap_or_default());
    rustdocflags.push_str(&env::var("RUSTDOCFLAGS").unwrap_or_default());

    let status = Command::new(cargo_path)
        .args(args) // skip `cargo` and `afl`
        .env("RUSTFLAGS", &rustflags)
        .env("RUSTDOCFLAGS", &rustdocflags)
        .env("ASAN_OPTIONS", asan_options)
        .env("TSAN_OPTIONS", tsan_options)
        .status()
        .unwrap();
    process::exit(status.code().unwrap_or(1));
}
