use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

static AFL_SRC_PATH: &str = "AFLplusplus";

// https://github.com/rust-fuzz/afl.rs/issues/148
#[cfg(target_os = "macos")]
static AR_CMD: &str = "/usr/bin/ar";
#[cfg(not(target_os = "macos"))]
static AR_CMD: &str = "ar";

#[path = "src/common.rs"]
mod common;

fn main() {
    let (work_dir, base) = if env::var("DOCS_RS").is_ok() {
        let tempdir = tempfile::tempdir().unwrap();
        let status = Command::new("git")
            .args(["clone", AFL_SRC_PATH, &*tempdir.path().to_string_lossy()])
            .status()
            .expect("could not run 'git'");
        assert!(status.success());
        (
            tempdir.into_path(),
            Some(PathBuf::from(env::var("OUT_DIR").unwrap())),
        )
    } else {
        (PathBuf::from(AFL_SRC_PATH), None)
    };
    build_afl(&work_dir, base.as_deref());
    build_afl_llvm_runtime(&work_dir, base.as_deref());
}

fn build_afl(work_dir: &Path, base: Option<&Path>) {
    let mut command = Command::new("make");
    command
        .current_dir(work_dir)
        .args(&["clean", "all", "install"])
        // skip the checks for the legacy x86 afl-gcc compiler
        .env("AFL_NO_X86", "1")
        // build just the runtime to avoid troubles with Xcode clang on macOS
        .env("NO_BUILD", "1")
        .env("DESTDIR", common::afl_dir(base))
        .env("PREFIX", "");
    if std::env::var("DEBUG").as_deref() == Ok("false") {
        command.env_remove("DEBUG");
    }
    let status = command.status().expect("could not run 'make'");
    assert!(status.success());
}

fn build_afl_llvm_runtime(work_dir: &Path, base: Option<&Path>) {
    std::fs::copy(
        work_dir.join("afl-compiler-rt.o"),
        common::object_file_path(base),
    )
    .expect("Couldn't copy object file");

    let status = Command::new(AR_CMD)
        .arg("r")
        .arg(common::archive_file_path(base))
        .arg(common::object_file_path(base))
        .status()
        .expect("could not run 'ar'");
    assert!(status.success());
}
