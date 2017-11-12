extern crate rustc_version;
extern crate xdg;

use std::env;
use std::ffi::OsStr;
use std::process::{self, Command};

#[path = "../dirs.rs"]
mod dirs;

fn main() {
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

fn run_afl<I, S>(args: I, cmd: &str)
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let cmd_path = dirs::afl().join("bin").join(cmd);
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
         -C panic=abort \
         -l afl-llvm-rt \
         -L {}",
        dirs::afl_llvm_rt().display()
    );
    let status = Command::new(cargo_path)
        .args(args) // skip `cargo` and `afl`
        .env("RUSTFLAGS", rustflags)
        .status()
        .unwrap();
    process::exit(status.code().unwrap_or(1));
}
