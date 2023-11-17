use clap::Parser;
use std::ffi::OsStr;
use std::path::Path;
use std::process::{self, Command, Stdio};

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

pub fn config(args: &Args) {
    if !args.force && common::archive_file_path(None).exists() {
        let version = common::afl_rustc_version();
        eprintln!(
            "AFL LLVM runtime was already built for Rust {version}; run `cargo \
             afl config --build --force` to rebuild it."
        );
        process::exit(1);
    }

    let afl_src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(AFL_SRC_PATH);
    let afl_src_dir_str = &afl_src_dir.to_string_lossy();

    let tempdir = tempfile::tempdir().unwrap();

    if afl_src_dir.join(".git").is_dir() {
        let status = Command::new("git")
            .args(["clone", afl_src_dir_str, &*tempdir.path().to_string_lossy()])
            .status()
            .expect("could not run 'git'");
        assert!(status.success());
    } else {
        fs_extra::dir::copy(
            afl_src_dir,
            tempdir.path(),
            &fs_extra::dir::CopyOptions {
                content_only: true,
                ..Default::default()
            },
        )
        .unwrap();
    }

    let work_dir = tempdir.path();

    build_afl(args, work_dir, None);
    build_afl_llvm_runtime(args, work_dir, None);

    if args.plugins {
        copy_afl_llvm_plugins(args, work_dir, None);
    }

    eprintln!(
        "Artifacts written to {}",
        common::afl_dir(None).parent().unwrap().display()
    );
}

fn build_afl(args: &Args, work_dir: &Path, base: Option<&Path>) {
    // if you had already installed cargo-afl previously you **must** clean AFL++
    // smoelius: AFL++ is now copied to a temporary directory before being built. So `make clean`
    // is no longer necessary.
    let mut command = Command::new("make");
    command
        .current_dir(work_dir)
        .arg("install")
        // skip the checks for the legacy x86 afl-gcc compiler
        .env("AFL_NO_X86", "1")
        .env("DESTDIR", common::afl_dir(base))
        .env("PREFIX", "")
        .env_remove("DEBUG");

    if args.plugins {
        let llvm_config = check_llvm_and_get_config();
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

    let status = command.status().expect("could not run 'make install'");
    assert!(status.success());
}

fn build_afl_llvm_runtime(args: &Args, work_dir: &Path, base: Option<&Path>) {
    std::fs::copy(
        work_dir.join("afl-compiler-rt.o"),
        common::object_file_path(base),
    )
    .expect("Couldn't copy object file");

    let mut command = Command::new(AR_CMD);
    command
        .arg("r")
        .arg(common::archive_file_path(base))
        .arg(common::object_file_path(base));

    if !args.verbose {
        command.stdout(Stdio::null());
        command.stderr(Stdio::null());
    }

    let status = command.status().expect("could not run 'ar'");
    assert!(status.success());
}

fn copy_afl_llvm_plugins(_args: &Args, work_dir: &Path, base: Option<&Path>) {
    // Iterate over the files in the directory.
    for result in work_dir.read_dir().unwrap() {
        let entry = result.unwrap();
        let file_name = entry.file_name();

        // Get the file extension. Only copy the files that are shared objects.
        if Path::new(&file_name).extension() == Some(OsStr::new("so")) {
            // Attempt to copy the shared object file.
            std::fs::copy(
                work_dir.join(&file_name),
                common::afl_llvm_dir(base).join(&file_name),
            )
            .unwrap_or_else(|error| {
                panic!("Couldn't copy shared object file {file_name:?}: {error}")
            });
        }
    }
}

fn check_llvm_and_get_config() -> String {
    // Make sure we are on nightly for the -Z flags
    assert!(
        rustc_version::version_meta().unwrap().channel == rustc_version::Channel::Nightly,
        "cargo-afl must be compiled with nightly for the plugins feature"
    );
    let version_meta = rustc_version::version_meta().unwrap();
    let llvm_version = version_meta.llvm_version.unwrap().major.to_string();

    // Fetch the llvm version of the rust toolchain and set the LLVM_CONFIG environment variable to the same version
    // This is needed to compile the llvm plugins (needed for cmplog) from afl with the right LLVM version
    let llvm_config = if cfg!(target_os = "macos") {
        "llvm-config".to_string()
    } else {
        format!("llvm-config-{llvm_version}")
    };

    // check if llvm tools are installed and with the good version for the plugin compilation
    let mut command = Command::new(llvm_config.clone());
    command.args(["--version"]);
    let out = command
        .output()
        .unwrap_or_else(|_| panic!("could not run {llvm_config} --version"));

    let version = String::from_utf8(out.stdout)
        .expect("could not convert llvm-config --version output to utf8");
    let major = version
        .split('.')
        .next()
        .expect("could not get major from llvm-config --version output");
    assert!(major == llvm_version);

    llvm_config
}
