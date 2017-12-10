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

    let rustflags = &format!(
        "-C llvm-args=-sanitizer-coverage-level=3 \
         -C llvm-args=-sanitizer-coverage-trace-pc-guard \
         -C passes=sancov \
         -l afl-llvm-rt \
         -L {}",
        common::afl_llvm_rt_dir().display()
    );
    let status = Command::new(cargo_path)
        .args(args) // skip `cargo` and `afl`
        .env("RUSTFLAGS", rustflags)
        .status()
        .unwrap();
    process::exit(status.code().unwrap_or(1));
}
