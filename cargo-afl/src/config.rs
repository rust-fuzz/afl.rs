#![deny(clippy::disallowed_macros, clippy::expect_used, clippy::unwrap_used)]

use anyhow::{Context, Result, bail, ensure};
use clap::Parser;
use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

use super::common;

const AFL_SRC_PATH: &str = "AFLplusplus";

// https://github.com/rust-fuzz/afl.rs/issues/148
#[cfg(target_os = "macos")]
static AR_CMD: &str = "/usr/bin/ar";
#[cfg(not(target_os = "macos"))]
static AR_CMD: &str = "ar";

#[allow(clippy::struct_excessive_bools)]
#[derive(Default, Parser)]
#[clap(after_help = "\
If you are using rustup, you can build AFL++ for a specific TOOLCHAIN as follows:

    cargo +TOOLCHAIN afl config --build")]
pub struct Args {
    #[clap(long, help = "Build AFL++ for the default toolchain")]
    pub build: bool,

    #[clap(long, help = "Rebuild AFL++ if it was already built")]
    pub force: bool,

    #[clap(long, help = "Enable building of LLVM plugins")]
    pub plugins: bool,

    #[clap(long, help = "Show build output")]
    pub verbose: bool,
}

pub fn config(args: &Args) -> Result<()> {
    let archive_file_path = common::archive_file_path()?;
    if !args.force && archive_file_path.exists() && args.plugins == common::plugins_available()? {
        let version = common::afl_rustc_version()?;
        bail!(
            "AFL LLVM runtime was already built for Rust {version}; run `cargo afl config --build \
             --force` to rebuild it."
        );
    }

    let afl_src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(AFL_SRC_PATH);
    let afl_src_dir_str = &afl_src_dir.to_string_lossy();

    let tempdir = tempfile::tempdir().with_context(|| "could not create temporary directory")?;

    if afl_src_dir.join(".git").is_dir() {
        let success = Command::new("git")
            .args(["clone", afl_src_dir_str, &*tempdir.path().to_string_lossy()])
            .status()
            .as_ref()
            .is_ok_and(ExitStatus::success);
        ensure!(success, "could not run 'git'");
    } else {
        let success = Command::new("cp")
            .args([
                "-P", // preserve symlinks
                "-R", // copy directories recursively
                afl_src_dir_str,
                &*tempdir.path().to_string_lossy(),
            ])
            .status()
            .as_ref()
            .is_ok_and(ExitStatus::success);
        ensure!(success, "could not copy directory {afl_src_dir:?}");
    }

    let work_dir = tempdir.path().join(AFL_SRC_PATH);

    build_afl(args, &work_dir)?;
    build_afl_llvm_runtime(args, &work_dir)?;

    if args.plugins {
        copy_afl_llvm_plugins(args, &work_dir)?;
    }

    let afl_dir = common::afl_dir()?;
    let Some(dir) = afl_dir.parent().map(Path::to_path_buf) else {
        bail!("could not get afl dir parent");
    };
    eprintln!("Artifacts written to {}", dir.display());

    Ok(())
}

fn build_afl(args: &Args, work_dir: &Path) -> Result<()> {
    // if you had already installed cargo-afl previously you **must** clean AFL++
    // smoelius: AFL++ is now copied to a temporary directory before being built. So `make clean`
    // is no longer necessary.
    let afl_dir = common::afl_dir()?;
    let mut command = Command::new("make");
    command
        .current_dir(work_dir)
        .arg("install")
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
    ensure!(success, "could not run 'make install'");

    Ok(())
}

fn build_afl_llvm_runtime(args: &Args, work_dir: &Path) -> Result<()> {
    let object_file_path = common::object_file_path()?;
    let _: u64 = std::fs::copy(work_dir.join("afl-compiler-rt.o"), &object_file_path)
        .with_context(|| "could not copy object file")?;

    let archive_file_path = common::archive_file_path()?;
    let mut command = Command::new(AR_CMD);
    command
        .arg("r")
        .arg(archive_file_path)
        .arg(object_file_path);

    if !args.verbose {
        command.stdout(Stdio::null());
        command.stderr(Stdio::null());
    }

    let success = command.status().as_ref().is_ok_and(ExitStatus::success);
    ensure!(success, "could not run 'ar'");

    Ok(())
}

fn copy_afl_llvm_plugins(_args: &Args, work_dir: &Path) -> Result<()> {
    // Iterate over the files in the directory.
    for result in work_dir
        .read_dir()
        .with_context(|| format!("could not read {work_dir:?}"))?
    {
        let entry = result.with_context(|| format!("could not read `DirEntry` in {work_dir:?}"))?;
        let file_name = entry.file_name();

        // Get the file extension. Only copy the files that are shared objects.
        if Path::new(&file_name).extension() == Some(OsStr::new("so")) {
            // Attempt to copy the shared object file.
            let afl_llvm_dir = common::afl_llvm_dir()?;
            let _: u64 = std::fs::copy(work_dir.join(&file_name), afl_llvm_dir.join(&file_name))
                .with_context(|| format!("could not copy shared object file {file_name:?}"))?;
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
