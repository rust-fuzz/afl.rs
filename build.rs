extern crate rustc_version;
extern crate xdg;

use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

static AFL_SRC_PATH: &str = "afl-2.52b";

#[path = "src/common.rs"]
mod common;

fn main() {
    build_afl(&common::afl_dir());
    build_afl_llvm_runtime();
}

fn build_afl(out_dir: &Path) {
    let status = Command::new("make")
        .current_dir(AFL_SRC_PATH)
        .args(&["clean", "all", "install"])
        // Rely on LLVM’s built-in execution tracing feature instead of AFL’s
        // LLVM passi instrumentation.
        .env("AFL_TRACE_PC", "1")
        .env("DESTDIR", out_dir)
        .env("PREFIX", "")
        .status()
        .expect("could not run 'make'");
    assert!(status.success());
}

fn build_afl_llvm_runtime() {
    let status = Command::new("cc")
        .current_dir(AFL_SRC_PATH)
        .arg("-c")
        .arg("-O1")
        .arg("-fPIC")
        .arg("-fno-omit-frame-pointer")
        .arg("llvm_mode/afl-llvm-rt.o.c")
        .arg("-fpermissive")
        .args(&[OsStr::new("-o"), common::object_file_path().as_os_str()])
        .status()
        .expect("could not run 'gcc'");
    assert!(status.success());

    let status = Command::new("ar")
        .arg("r")
        .arg(common::archive_file_path())
        .arg(common::object_file_path())
        .status()
        .expect("could not run 'ar'");
    assert!(status.success());
}
