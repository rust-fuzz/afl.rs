use lazy_static::lazy_static;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

static AFL_SRC_PATH: &str = "AFLplusplus";

// https://github.com/rust-fuzz/afl.rs/issues/148
#[cfg(target_os = "macos")]
static AR_CMD: &str = "/usr/bin/ar";
#[cfg(not(target_os = "macos"))]
static AR_CMD: &str = "ar";

lazy_static! {
    static ref BASE: Option<PathBuf> = {
        if env::var("DOCS_RS").is_ok() {
            Some(PathBuf::from(env::var("OUT_DIR").unwrap()))
        } else {
            None
        }
    };
}

#[path = "src/common.rs"]
mod common;

fn main() {
    build_afl(&common::afl_dir(BASE.as_deref()));
    build_afl_llvm_runtime();
}

fn build_afl(out_dir: &Path) {
    let mut command = Command::new("make");
    command
        .current_dir(AFL_SRC_PATH)
        .args(&["clean", "all", "install"])
        // skip the checks for the legacy x86 afl-gcc compiler
        .env("AFL_NO_X86", "1")
        // build just the runtime to avoid troubles with Xcode clang on macOS
        .env("NO_BUILD", "1")
        .env("DESTDIR", out_dir)
        .env("PREFIX", "");
    if std::env::var("DEBUG").as_deref() == Ok("false") {
        command.env_remove("DEBUG");
    }
    let status = command.status().expect("could not run 'make'");
    assert!(status.success());
}

fn build_afl_llvm_runtime() {
    std::fs::copy(
        Path::new(&AFL_SRC_PATH).join("afl-compiler-rt.o"),
        common::object_file_path(BASE.as_deref()),
    )
    .expect("Couldn't copy object file");

    let status = Command::new(AR_CMD)
        .arg("r")
        .arg(common::archive_file_path(BASE.as_deref()))
        .arg(common::object_file_path(BASE.as_deref()))
        .status()
        .expect("could not run 'ar'");
    assert!(status.success());
}
