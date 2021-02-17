extern crate rustc_version;
extern crate xdg;

use std::path::Path;
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
    build_afl(&common::afl_dir());
    build_afl_llvm_runtime();
}

fn build_afl(out_dir: &Path) {
    let mut command = Command::new("make");
    command
        .current_dir(AFL_SRC_PATH)
        .args(&["clean", "all", "install"])
        .env("DESTDIR", out_dir)
        .env("PREFIX", "");
    // sets AFL_NO_X86 to compile for ARM arch
    if cfg!(target_arch = "arm") {
        command.env("AFL_NO_X86", "1");
    }
    let status = command.status().expect("could not run 'make'");
    assert!(status.success());
}

fn build_afl_llvm_runtime() {
    std::fs::copy(
        Path::new(&AFL_SRC_PATH).join("afl-compiler-rt.o"),
        common::object_file_path(),
    )
    .expect("Couldn't copy object file");

    let status = Command::new(AR_CMD)
        .arg("r")
        .arg(common::archive_file_path())
        .arg(common::object_file_path())
        .status()
        .expect("could not run 'ar'");
    assert!(status.success());
}
