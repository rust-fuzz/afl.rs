extern crate rustc_version;
extern crate xdg;

use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

static AFL_SRC_PATH: &str = "afl-2.52b";

#[path = "src/dirs.rs"]
mod dirs;

fn main() {
    build_afl(&dirs::afl());
    build_afl_llvm_runtime(&dirs::afl_llvm_rt());
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

fn build_afl_llvm_runtime(out_dir: &Path) {
    let object_file_path = out_dir.join("libafl-llvm-rt.o");
    let archive_file_path = out_dir.join("libafl-llvm-rt.a");

    let status = Command::new("cc")
        .current_dir(AFL_SRC_PATH)
        .arg("-c")
        .arg("-O1")
        .arg("-fPIC")
        .arg("-fno-omit-frame-pointer")
        .arg("llvm_mode/afl-llvm-rt.o.c")
        .arg("-fpermissive")
        .args(&[OsStr::new("-o"), object_file_path.as_os_str()])
        .status()
        .expect("could not run 'gcc'");
    assert!(status.success());

    let status = Command::new("ar")
        .arg("r")
        .arg(archive_file_path)
        .arg(object_file_path)
        .status()
        .expect("could not run 'ar'");
    assert!(status.success());
}
