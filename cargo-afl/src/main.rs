use clap::{crate_version, CommandFactory, FromArgMatches, Parser};
use std::collections::HashMap;
use std::env;
use std::ffi::{OsStr, OsString};
use std::process::{self, Command, Stdio};

mod common;
mod config;

const HELP: &str = "In addition to the subcommands above, Cargo subcommands are also \
supported (see `cargo help` for a list of all Cargo subcommands).";

#[derive(Parser)]
#[clap(
    display_name = "cargo",
    subcommand_required = true,
    arg_required_else_help = true
)]
struct Args {
    #[clap(subcommand)]
    subcmd: CargoSubcommand,
}

#[derive(Parser)]
enum CargoSubcommand {
    Afl(AflArgs),
}

#[derive(Parser)]
#[clap(
    version = crate_version!(),
    allow_hyphen_values = true,
    arg_required_else_help = true,
    override_usage = "cargo afl [SUBCOMMAND or Cargo SUBCOMMAND]",
    after_help = HELP,
)]
struct AflArgs {
    #[clap(subcommand)]
    subcmd: Option<AflSubcommand>,

    args: Vec<OsString>,
}

macro_rules! construct_afl_subcommand_variants {
    // base (i.e., final) case
    (
        {
            $($constructed_variants:tt)*
        } // no more materials
    ) => {
        #[derive(Parser)]
        enum AflSubcommand {
            $($constructed_variants)*
        }
    };
    // inductive case, with args type
    (
        {
            $($constructed_variants:tt)*
        } $variant:ident ( $about:literal, $args_ty:ty ), $($unused_materials:tt)*
    ) => {
        construct_afl_subcommand_variants! {
            {
                $($constructed_variants)*
                #[clap(
                    about = $about,
                    arg_required_else_help = true,
                )]
                $variant($args_ty),
            } $($unused_materials)*
        }
    };
    // inductive case, without args type
    (
        {
            $($constructed_variants:tt)*
        } $variant:ident ( $about:literal ), $($unused_materials:tt)*
    ) => {
        construct_afl_subcommand_variants! {
            {
                $($constructed_variants)*
                #[clap(
                    about = $about,
                    allow_hyphen_values = true,
                    disable_help_subcommand = true,
                    disable_help_flag = true,
                    disable_version_flag = true,
                )]
                $variant { args: Vec<OsString> },
            } $($unused_materials)*
        }
    };
}

macro_rules! declare_afl_subcommand_enum {
    ($($materials:tt)*) => {
        construct_afl_subcommand_variants! {
            {} $($materials)*
        }
    };
}

declare_afl_subcommand_enum! {
    Addseeds("Invoke afl-addseeds"),
    Analyze("Invoke afl-analyze"),
    Cmin("Invoke afl-cmin"),
    Config("Build or rebuild AFL++", config::Args),
    Fuzz("Invoke afl-fuzz"),
    Gotcpu("Invoke afl-gotcpu"),
    Plot("Invoke afl-plot"),
    Showmap("Invoke afl-showmap"),
    SystemConfig("Invoke afl-system-config (beware, called with sudo!)"),
    Tmin("Invoke afl-tmin"),
    Whatsup("Invoke afl-whatsup"),
}

fn main() {
    let command = command_with_afl_version();

    let afl_args = match Args::from_arg_matches(&command.get_matches()).unwrap() {
        Args {
            subcmd: CargoSubcommand::Afl(afl_args),
        } => afl_args,
    };

    if !matches!(afl_args.subcmd, Some(AflSubcommand::Config(..)))
        && !common::archive_file_path().unwrap().exists()
    {
        let version = common::afl_rustc_version().unwrap();
        eprintln!(
            "AFL LLVM runtime was not built for Rust {version}; run `cargo \
             afl config --build` to build it."
        );
        process::exit(1);
    }

    match &afl_args.subcmd {
        Some(AflSubcommand::Addseeds { args }) => {
            run_afl("afl-addseeds", args);
        }
        Some(AflSubcommand::Analyze { args }) => {
            run_afl("afl-analyze", args);
        }
        Some(AflSubcommand::Config(args)) => {
            config::config(args).unwrap();
        }
        Some(AflSubcommand::Cmin { args }) => {
            run_afl("afl-cmin", args);
        }
        Some(AflSubcommand::Fuzz { args }) => {
            // We prepend -c0 to the AFL++ arguments
            let cmplog_flag = [OsString::from("-c0")];
            let args = cmplog_flag.iter().chain(args);
            run_afl("afl-fuzz", args);
        }
        Some(AflSubcommand::Gotcpu { args }) => {
            run_afl("afl-gotcpu", args);
        }
        Some(AflSubcommand::Plot { args }) => {
            run_afl("afl-plot", args);
        }
        Some(AflSubcommand::Showmap { args }) => {
            run_afl("afl-showmap", args);
        }
        Some(AflSubcommand::SystemConfig { args }) => {
            run_afl("afl-system-config", args);
        }
        Some(AflSubcommand::Tmin { args }) => {
            run_afl("afl-tmin", args);
        }
        Some(AflSubcommand::Whatsup { args }) => {
            run_afl("afl-whatsup", args);
        }
        None => {
            run_cargo(afl_args.args);
        }
    }
}

fn command_with_afl_version() -> clap::Command {
    let mut command = Args::command();

    (|| -> Option<()> {
        let afl_version = afl_version()?;
        let with_plugins = common::plugins_available().ok()?;

        let subcmd = command.find_subcommand_mut("afl").unwrap();
        let ver = format!(
            "{} (AFL++ version {}{})",
            subcmd.get_version().unwrap(),
            afl_version,
            if with_plugins { " with plugins" } else { "" }
        );
        *subcmd = subcmd.clone().version(ver);
        Some(())
    })()
    .unwrap_or_default();

    command
}

fn afl_version() -> Option<String> {
    const PREFIX: &str = "afl-fuzz++";
    let afl_fuzz_path = common::afl_dir().unwrap().join("bin/afl-fuzz");
    let output = Command::new(afl_fuzz_path).output().ok()?;
    let stdout = String::from_utf8(output.stdout).ok()?;
    let index = stdout.find(PREFIX)?;
    Some(
        stdout[index + PREFIX.len()..]
            .chars()
            .take_while(|c| !c.is_ascii_whitespace())
            .collect(),
    )
}

fn run_afl<I, S>(tool: &str, args: I)
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let no_sudo = env::var("NO_SUDO").is_ok();
    let cmd_path = common::afl_dir().unwrap().join("bin").join(tool);
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

    cargo afl system-config

Note: You might be prompted to enter your password as root privileges are required and hence sudo is run within this command."
        );
    }
    process::exit(status.code().unwrap_or(1));
}

fn run_cargo<I, S>(args: I)
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

    let binding = common::afl_llvm_dir().unwrap();
    let p = binding.display();

    let mut rustflags = String::from(
        "-C debug-assertions \
             -C overflow_checks \
             -C codegen-units=1 \
             -C opt-level=3 \
             -C target-cpu=native ",
    );
    let mut environment_variables = HashMap::<&str, String>::new();
    environment_variables.insert("ASAN_OPTIONS", asan_options);
    environment_variables.insert("TSAN_OPTIONS", tsan_options);

    if common::plugins_available().unwrap() {
        // Make sure we are on nightly for the -Z flags
        assert!(
            rustc_version::version_meta().unwrap().channel == rustc_version::Channel::Nightly,
            "cargo-afl must be compiled with nightly for CMPLOG and other advanced AFL++ features"
        );

        rustflags.push_str(&format!(
            "-Z llvm-plugins={p}/cmplog-instructions-pass.so  \
            -Z llvm-plugins={p}/cmplog-routines-pass.so \
            -Z llvm-plugins={p}/cmplog-switches-pass.so \
            -Z llvm-plugins={p}/SanitizerCoveragePCGUARD.so \
            -Z llvm-plugins={p}/afl-llvm-dict2file.so
            "
        ));

        environment_variables.insert("AFL_QUIET", "1".to_string());
    } else {
        rustflags.push_str(&format!(
            "-C passes={passes} \
            -C llvm-args=-sanitizer-coverage-level=3 \
            -C llvm-args=-sanitizer-coverage-trace-pc-guard \
            -C llvm-args=-sanitizer-coverage-prune-blocks=0 \
            -C llvm-args=-sanitizer-coverage-trace-compares
            ",
        ));
    }

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
        common::afl_llvm_dir().unwrap().display()
    ));

    // add user provided flags
    rustflags.push_str(&env::var("RUSTFLAGS").unwrap_or_default());
    rustdocflags.push_str(&env::var("RUSTDOCFLAGS").unwrap_or_default());

    environment_variables.insert("RUSTFLAGS", rustflags);
    environment_variables.insert("RUSTDOCFLAGS", rustdocflags);

    let status = Command::new(cargo_path)
        .args(args)
        .envs(&environment_variables)
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
    use std::process::Output;

    #[test]
    fn test_app() {
        command_with_afl_version().debug_assert();
    }

    #[test]
    fn display_name() {
        let output = cargo_afl(&["-V"]).output().unwrap();
        assert_success(&output, None);
        assert!(String::from_utf8(output.stdout)
            .unwrap()
            .starts_with("cargo-afl"));
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
    fn external_subcommands_allow_invalid_utf8() {
        let _arg_matches = Args::try_parse_from([
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
            let _arg_matches = Args::try_parse_from([
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
            let _arg_matches =
                Args::try_parse_from(["cargo", "afl", subcommand, "-i", "--input"]).unwrap();
        }
    }

    #[test]
    fn subcommands_help_subcommand_disabled() {
        let output = cargo_afl(&["help"]).output().unwrap();
        assert_success(&output, None);
        assert!(String::from_utf8(output.stdout)
            .unwrap()
            .starts_with("Usage:"));

        for &subcommand in SUBCOMMANDS {
            let output = cargo_afl(&[subcommand, "help"]).output().unwrap();
            assert_failure(&output, Some(subcommand));
            assert!(!String::from_utf8(output.stdout)
                .unwrap()
                .starts_with("Usage:"));
        }
    }

    #[test]
    fn subcommands_help_flag_disabled() {
        let output = cargo_afl(&["--help"]).output().unwrap();
        assert_success(&output, None);
        assert!(String::from_utf8(output.stdout)
            .unwrap()
            .starts_with("Usage:"));

        for &subcommand in SUBCOMMANDS {
            let output = cargo_afl(&[subcommand, "--help"]).output().unwrap();
            // smoelius: `afl-addseeds` and `afl-system-config` have `--help` flags.
            if subcommand == "addseeds" || subcommand == "system-config" {
                assert_success(&output, Some(subcommand));
            } else {
                assert_failure(&output, Some(subcommand));
            }
            assert!(!String::from_utf8(output.stdout)
                .unwrap()
                .starts_with("Usage:"));
        }
    }

    #[test]
    fn subcommands_version_flag_disabled() {
        let output = cargo_afl(&["-V"]).output().unwrap();
        assert_success(&output, None);
        assert!(String::from_utf8(output.stdout)
            .unwrap()
            .starts_with("cargo-afl"));

        for &subcommand in SUBCOMMANDS {
            let output = cargo_afl(&[subcommand, "-V"]).output().unwrap();
            assert_failure(&output, Some(subcommand));
            assert!(!String::from_utf8(output.stdout)
                .unwrap()
                .starts_with("cargo-afl"));
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

    fn invalid_utf8() -> OsString {
        OsString::from_vec(vec![0xfe])
    }

    #[test]
    fn invalid_utf8_is_invalid() {
        assert!(String::from_utf8(invalid_utf8().into_vec()).is_err());
    }
}
