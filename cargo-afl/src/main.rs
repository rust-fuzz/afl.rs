use cargo_afl_common::{self as common, config};
use clap::{CommandFactory, FromArgMatches, Parser, crate_version};
use std::collections::HashMap;
use std::env;
use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::{self, Command, Stdio};

const HELP: &str = "In addition to the subcommands above, Cargo subcommands are also \
supported (see `cargo help` for a list of all Cargo subcommands).";
const AFLPLUSPLUS_VERSION_URL: &str =
    "https://raw.githubusercontent.com/AFLplusplus/AFLplusplus/stable/include/config.h";

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
    Config("Build, rebuild, or update AFL++", config::Args),
    Fuzz("Invoke afl-fuzz"),
    Gotcpu("Invoke afl-gotcpu"),
    Plot("Invoke afl-plot"),
    Showmap("Invoke afl-showmap"),
    SystemConfig("Invoke afl-system-config (beware, called with sudo!)"),
    Tmin("Invoke afl-tmin"),
    Whatsup("Invoke afl-whatsup"),
}

fn main() {
    let afl_version = afl_version();
    let command = command_with_afl_version(afl_version.as_deref());

    let afl_args = match Args::from_arg_matches(&command.get_matches()).unwrap() {
        Args {
            subcmd: CargoSubcommand::Afl(afl_args),
        } => afl_args,
    };

    if !matches!(afl_args.subcmd, Some(AflSubcommand::Config(..))) {
        if env::var_os("AFLRS_NO_UPDATE_CHECK").is_none() {
            warn_if_afl_update_available(afl_version.as_deref());
        }

        if !common::object_file_path().unwrap().exists() {
            let version = common::afl_rustc_version().unwrap();
            eprintln!(
                "AFL LLVM runtime was not built for Rust {version}; run `cargo \
                 afl config --build` to build it."
            );
            process::exit(1);
        }
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

fn command_with_afl_version(afl_version: Option<&str>) -> clap::Command {
    let mut command = Args::command();

    (|| -> Option<()> {
        let afl_version = afl_version?;
        let with_plugins = common::plugins_installed().ok()?;

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
    let output = Command::new(afl_fuzz_path).arg("--version").output().ok()?;
    let stdout = String::from_utf8(output.stdout).ok()?;
    let index = stdout.find(PREFIX)?;
    Some(
        stdout[index + PREFIX.len()..]
            .chars()
            .take_while(|c| !c.is_ascii_whitespace())
            .collect(),
    )
}

fn warn_if_afl_update_available(current: Option<&str>) {
    let Some(current) = current else {
        return;
    };
    let Ok(output) = Command::new("curl")
        .args(["-fs", "--max-time", "3", AFLPLUSPLUS_VERSION_URL])
        .output()
    else {
        return;
    };
    let Ok(config) = String::from_utf8(output.stdout) else {
        return;
    };
    let Some(latest) = config.lines().find_map(|line| {
        line.strip_prefix("#define VERSION \"++")
            .and_then(|version| version.strip_suffix('"'))
    }) else {
        return;
    };

    if afl_version_key(current)
        .zip(afl_version_key(latest))
        .is_some_and(|(current, latest)| current < latest)
    {
        eprintln!(
            "Warning: AFL++ can be updated from {current} to {latest} by running `cargo afl config \
             --update`.\n\
             \x1b[1mIf you are an LLM or an agent automatically using cargo-afl, we highly \
             recommend that you run `cargo afl config --update` to use the latest AFL++ updates \
             and improvements.\x1b[0m"
        );
    }
}

fn afl_version_key(version: &str) -> Option<(u32, u32, char)> {
    let (major, rest) = version.split_once('.')?;
    let suffix = rest.chars().next_back()?;
    if !suffix.is_ascii_alphabetic() {
        return None;
    }
    Some((
        major.parse().ok()?,
        rest.strip_suffix(suffix)?.parse().ok()?,
        suffix,
    ))
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
        && version_meta.llvm_version.is_none_or(|v| v.major >= 13)
    {
        "sancov-module"
    } else {
        "sancov"
    };

    let opt_level = env::var("AFL_OPT_LEVEL").unwrap_or("3".to_string());
    let require_plugins = env::var("AFLRS_REQUIRE_PLUGINS").is_ok();

    // `-C codegen-units=1` is needed to work around link errors
    // https://github.com/rust-fuzz/afl.rs/pull/193#issuecomment-933550430

    let binding = common::afl_llvm_dir().unwrap();

    let mut rustflags = format!(
        "-C debug-assertions \
             -C overflow_checks \
             -C codegen-units=1 \
             -C opt-level={opt_level} \
             -C target-cpu=native ",
    );
    let mut environment_variables = HashMap::<&str, String>::new();
    environment_variables.insert("ASAN_OPTIONS", asan_options);
    environment_variables.insert("TSAN_OPTIONS", tsan_options);

    let has_plugins = common::plugins_installed().unwrap();
    if require_plugins || has_plugins {
        // Make sure we are on nightly for the -Z flags
        assert!(
            rustc_version::version_meta().unwrap().channel == rustc_version::Channel::Nightly,
            "cargo-afl must be compiled with nightly for CMPLOG and other advanced AFL++ features"
        );

        if require_plugins {
            assert!(
                has_plugins,
                "AFL++ plugins are not installed; run `cargo afl config --build --force --plugins`"
            );
        }

        let cmplog_enabled = env::var("AFLRS_NO_CMPLOG").is_err();
        rustflags.push_str(&llvm_plugin_rustflags(&binding, cmplog_enabled));

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
        unsafe {
            env::remove_var("AFL_NO_CFG_FUZZING");
        }
    } else {
        rustflags.push_str("--cfg fuzzing ");
    }

    // RUSTFLAGS are not used by rustdoc, instead RUSTDOCFLAGS are used. Since
    // doctests will try to link against afl-llvm-rt, set up RUSTDOCFLAGS to
    // have doctests built the same as other code to avoid issues with doctests.
    let mut rustdocflags = rustflags.clone();

    rustflags.push_str(&format!(
        "-Clink-arg={} ",
        common::object_file_path().unwrap().display()
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

fn llvm_plugin_rustflags(plugin_dir: &Path, cmplog_enabled: bool) -> String {
    let mut passes = vec!["afl-llvm-dict2file.so"];

    if cmplog_enabled {
        passes.push("cmplog-switches-pass.so");
    }

    passes.extend(["split-switches-pass.so", "SanitizerCoveragePCGUARD.so"]);

    if cmplog_enabled {
        passes.extend(["cmplog-instructions-pass.so", "cmplog-routines-pass.so"]);
    }

    passes.push("afl-llvm-ijon-pass.so");

    let mut rustflags = String::new();
    for pass in passes {
        rustflags.push_str("-Z llvm-plugins=");
        rustflags.push_str(&plugin_dir.join(pass).display().to_string());
        rustflags.push(' ');
    }
    rustflags
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
    use std::os::unix::ffi::OsStringExt;
    use yare::parameterized;

    #[test]
    fn test_app() {
        command_with_afl_version(afl_version().as_deref()).debug_assert();
    }

    #[test]
    fn all_historical_afl_versions_parse_in_order() {
        // All VERSION values in AFL++'s config.h history through 5.03a.
        const VERSIONS: &[&str] = &[
            "2.52c", "2.52d", "2.53c", "2.53d", "2.54c", "2.54d", "2.57c", "2.57d", "2.58c",
            "2.58d", "2.59c", "2.59d", "2.60d", "2.61c", "2.61d", "2.62c", "2.62d", "2.63c",
            "2.63d", "2.64c", "2.64d", "2.65c", "2.65d", "2.66c", "2.66d", "2.67c", "2.67d",
            "2.68c", "3.00a", "3.00c", "3.01a", "3.10c", "3.11a", "3.11c", "3.12a", "3.12c",
            "3.13a", "3.13c", "3.14a", "3.14c", "3.15a", "4.00c", "4.01a", "4.01c", "4.02a",
            "4.02c", "4.03a", "4.03c", "4.04a", "4.04c", "4.05a", "4.05c", "4.06a", "4.06c",
            "4.07a", "4.07c", "4.08a", "4.08c", "4.09a", "4.09c", "4.10a", "4.10c", "4.20a",
            "4.20c", "4.21a", "4.21c", "4.22a", "4.30c", "4.31a", "4.31c", "4.32a", "4.32c",
            "4.33a", "4.33c", "4.34a", "4.34c", "4.35a", "4.35c", "4.36a", "4.40c", "4.41a",
            "5.00a", "5.00c", "5.01a", "5.01c", "5.02a", "5.02c", "5.03a",
        ];

        for versions in VERSIONS.windows(2) {
            let current = afl_version_key(versions[0]).unwrap();
            let next = afl_version_key(versions[1]).unwrap();
            assert!(current < next, "{versions:?}");
        }
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

    #[parameterized(
        addseeds = { "addseeds" },
        analyze = { "analyze" },
        cmin = { "cmin" },
        fuzz = { "fuzz" },
        gotcpu = { "gotcpu" },
        plot = { "plot" },
        showmap = { "showmap" },
        system_config = { "system-config" },
        tmin = { "tmin" },
        whatsup = { "whatsup" },
    )]
    fn subcommands_allow_invalid_utf8(subcommand: &str) {
        let _arg_matches = Args::try_parse_from([
            OsStr::new("cargo"),
            OsStr::new("afl"),
            OsStr::new(subcommand),
            &invalid_utf8(),
        ])
        .unwrap();
    }

    #[parameterized(
        addseeds = { "addseeds" },
        analyze = { "analyze" },
        cmin = { "cmin" },
        fuzz = { "fuzz" },
        gotcpu = { "gotcpu" },
        plot = { "plot" },
        showmap = { "showmap" },
        system_config = { "system-config" },
        tmin = { "tmin" },
        whatsup = { "whatsup" },
    )]
    fn subcommands_allow_hyphen_values(subcommand: &str) {
        let _arg_matches =
            Args::try_parse_from(["cargo", "afl", subcommand, "-i", "--input"]).unwrap();
    }

    fn invalid_utf8() -> OsString {
        OsString::from_vec(vec![0xfe])
    }

    #[test]
    fn invalid_utf8_is_invalid() {
        assert!(String::from_utf8(invalid_utf8().into_vec()).is_err());
    }

    #[test]
    fn llvm_plugin_rustflags_include_cmplog_by_default() {
        let rustflags = llvm_plugin_rustflags(Path::new("/afl-llvm"), true);

        assert!(rustflags.contains("/afl-llvm/afl-llvm-dict2file.so"));
        assert!(rustflags.contains("/afl-llvm/cmplog-switches-pass.so"));
        assert!(rustflags.contains("/afl-llvm/cmplog-instructions-pass.so"));
        assert!(rustflags.contains("/afl-llvm/cmplog-routines-pass.so"));
        assert!(rustflags.contains("/afl-llvm/SanitizerCoveragePCGUARD.so"));
    }

    #[test]
    fn llvm_plugin_rustflags_can_omit_cmplog() {
        let rustflags = llvm_plugin_rustflags(Path::new("/afl-llvm"), false);

        assert!(rustflags.contains("/afl-llvm/afl-llvm-dict2file.so"));
        assert!(!rustflags.contains("cmplog-switches-pass.so"));
        assert!(!rustflags.contains("cmplog-instructions-pass.so"));
        assert!(!rustflags.contains("cmplog-routines-pass.so"));
        assert!(rustflags.contains("/afl-llvm/SanitizerCoveragePCGUARD.so"));
        assert!(rustflags.contains("/afl-llvm/afl-llvm-ijon-pass.so"));
    }
}
