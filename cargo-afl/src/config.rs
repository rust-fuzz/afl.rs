use crate::common;
use crate::common::config::{copy_aflplusplus_submodule, is_repo, update_to_stable_or_tag};
use anyhow::{Context, Result, bail, ensure};
use clap::Parser;
use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

#[allow(clippy::struct_excessive_bools)]
#[derive(Default, Parser)]
#[clap(after_help = "\
If you are using rustup, you can build AFL++ for a specific TOOLCHAIN as follows:

    cargo +TOOLCHAIN afl config --build")]
pub struct Args {
    #[clap(long, help = "Build AFL++ for the default toolchain")]
    pub build: bool,

    #[clap(
        long,
        help = "Rebuild AFL++ if it was already built. Note: AFL++ will be built without plugins \
                if `--plugins` is not passed."
    )]
    pub force: bool,

    #[clap(long, help = "Enable building of LLVM plugins")]
    pub plugins: bool,

    #[clap(
        long,
        help = "Update to <TAG> instead of the latest stable version",
        requires = "update"
    )]
    pub tag: Option<String>,

    #[clap(
        long,
        help = "Update AFL++ to the latest stable version (preserving plugins, if applicable)"
    )]
    pub update: bool,

    #[clap(long, help = "Show build output")]
    pub verbose: bool,
}

pub fn config(args: &Args) -> Result<()> {
    let object_file_path = common::object_file_path()?;

    if !args.force
        && !args.update
        && object_file_path.exists()
        && args.plugins == common::plugins_installed()?
    {
        let version = common::afl_rustc_version()?;
        bail!(
            "AFL LLVM runtime was already built for Rust {version}; run `cargo afl config --build \
             --force` to rebuild it."
        );
    }

    // smoelius: If updating and AFL++ was built with plugins before, build with plugins again.
    let args = Args {
        plugins: if args.update {
            common::plugins_installed().is_ok_and(|is_true| is_true)
        } else {
            args.plugins
        },
        tag: args.tag.clone(),
        ..*args
    };

    let aflplusplus_dir =
        common::aflplusplus_dir().with_context(|| "could not determine AFLplusplus directory")?;

    // smoelius: The AFLplusplus directory could be in one of three possible states:
    //
    // 1. Nonexistent
    // 2. Initialized with a copy of the AFLplusplus submodule from afl.rs's source tree
    // 3. Cloned from `AFLPLUSPLUS_URL`
    //
    // If we are not updating and the AFLplusplus directory is nonexistent: initialize the directory
    // with a copy of the AFLplusplus submodule from afl.rs's source tree (the `else` case in the
    // next `if` statement).
    //
    // If we are updating and the AFLplusplus directory is a copy of the AFLplusplus submodule from
    // afl.rs's source tree: remove it and create a new directory by cloning AFL++ (the `else` case
    // in `update_to_stable_or_tag`).
    //
    // Finally, if we are updating: check out either `origin/stable` or the tag that was passed.
    if args.update {
        let rev_prev = if is_repo(&aflplusplus_dir)? {
            rev(&aflplusplus_dir).map(Some)?
        } else {
            None
        };

        update_to_stable_or_tag(&aflplusplus_dir, args.tag.as_deref())?;

        let rev_curr = rev(&aflplusplus_dir)?;

        if rev_prev == Some(rev_curr) && !args.force {
            eprintln!("Nothing to do. Pass `--force` to force rebuilding.");
            return Ok(());
        }
    } else if !aflplusplus_dir.join(".git").try_exists()? {
        copy_aflplusplus_submodule(&aflplusplus_dir)?;
    }

    build_afl(&args, &aflplusplus_dir)?;
    build_afl_llvm_runtime(&args, &aflplusplus_dir)?;

    if args.plugins {
        copy_afl_llvm_plugins(&args, &aflplusplus_dir)?;
    }

    let afl_dir = common::afl_dir()?;
    let Some(afl_dir_parent) = afl_dir.parent() else {
        bail!("could not get afl dir parent");
    };
    eprintln!("Artifacts written to {}", afl_dir_parent.display());

    Ok(())
}

fn rev(dir: &Path) -> Result<String> {
    let mut command = Command::new("git");
    command.args(["rev-parse", "HEAD"]);
    command.current_dir(dir);
    let output = command
        .output()
        .with_context(|| "could not run `git rev-parse`")?;
    ensure!(output.status.success(), "`git rev-parse` failed");
    String::from_utf8(output.stdout).map_err(Into::into)
}

fn build_afl(args: &Args, work_dir: &Path) -> Result<()> {
    // if you had already installed cargo-afl previously you **must** clean AFL++
    let afl_dir = common::afl_dir()?;
    let mut command = Command::new("make");
    command
        .current_dir(work_dir)
        .args(["clean", "install"])
        // skip the checks for the legacy x86 afl-gcc compiler
        .env("AFL_NO_X86", "1")
        .env("DESTDIR", afl_dir)
        .env("PREFIX", "")
        .env_remove("DEBUG");

    if args.plugins {
        let llvm_config = check_llvm_and_get_config()?;
        command.env("LLVM_CONFIG", llvm_config);
    } else {
        // build just the runtime to avoid troubles with Xcode clang on macOS
        // smoelius: `NO_BUILD=1` also makes `cargo build` much faster.
        command.env("NO_BUILD", "1");
    }

    if !args.verbose {
        command.stdout(Stdio::null());
        command.stderr(Stdio::null());
    }

    let success = command.status().as_ref().is_ok_and(ExitStatus::success);
    ensure!(
        success,
        "could not run 'make clean install' in {}",
        work_dir.display()
    );

    Ok(())
}

fn build_afl_llvm_runtime(_args: &Args, work_dir: &Path) -> Result<()> {
    let object_file_path = common::object_file_path()?;
    let _: u64 = std::fs::copy(work_dir.join(common::OBJECT_FILE_NAME), &object_file_path)
        .with_context(|| "could not copy object file")?;

    Ok(())
}

fn copy_afl_llvm_plugins(_args: &Args, work_dir: &Path) -> Result<()> {
    // Iterate over the files in the directory.
    for result in work_dir
        .read_dir()
        .with_context(|| format!("could not read `{}`", work_dir.display()))?
    {
        let entry = result
            .with_context(|| format!("could not read `DirEntry` in `{}`", work_dir.display()))?;
        let file_name = entry.file_name();

        // Get the file extension. Only copy the files that are shared objects.
        if Path::new(&file_name).extension() == Some(OsStr::new("so")) {
            // Attempt to copy the shared object file.
            let afl_llvm_dir = common::afl_llvm_dir()?;
            let _: u64 = std::fs::copy(work_dir.join(&file_name), afl_llvm_dir.join(&file_name))
                .with_context(|| {
                    format!(
                        "could not copy shared object file `{}`",
                        file_name.display()
                    )
                })?;
        }
    }

    Ok(())
}

fn check_llvm_and_get_config() -> Result<String> {
    // Make sure we are on nightly for the -Z flags
    let version_meta = rustc_version::version_meta()?;
    if version_meta.channel != rustc_version::Channel::Nightly {
        bail!("cargo-afl must be compiled with nightly for the plugins feature");
    }
    let Some(llvm_version) = version_meta
        .llvm_version
        .map(|llvm_version| llvm_version.major.to_string())
    else {
        bail!("could not get llvm version");
    };

    // Fetch the llvm version of the rust toolchain and set the LLVM_CONFIG environment variable to the same version
    // This is needed to compile the llvm plugins (needed for cmplog) from afl with the right LLVM version
    let llvm_config = if cfg!(target_os = "macos") {
        "llvm-config".to_string()
    } else {
        format!("llvm-config-{llvm_version}")
    };

    // check if llvm tools are installed and with the good version for the plugin compilation
    let mut command = Command::new(&llvm_config);
    command.args(["--version"]);
    let out = command
        .output()
        .with_context(|| format!("could not run {llvm_config} --version"))?;

    let version = String::from_utf8(out.stdout)
        .with_context(|| format!("could not convert {llvm_config} --version output to utf8"))?;
    let Some(major) = version.split('.').next() else {
        bail!("could not get major from {llvm_config} --version output");
    };
    if major != llvm_version {
        bail!(
            "{llvm_config} --version output does not contain expected major version \
             ({llvm_version})",
        );
    }

    Ok(llvm_config)
}
