use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::process::Command;

static AFL_SRC_PATH: &str = "afl-2.52b";

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_dir_path = Path::new(&out_dir).join("afl");

    create_dir(&out_dir_path);
    create_dir(&out_dir_path.join("bin"));
    create_dir(&out_dir_path.join("link"));

    build_afl(&out_dir_path);
    build_afl_llvm_runtime(&out_dir_path);
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
        .unwrap();
    assert!(status.success());
}

fn build_afl_llvm_runtime(out_dir: &Path) {
    let object_file_path = out_dir.join("link").join("libafl-llvm-rt.o");
    let archive_file_path = out_dir.join("link").join("libafl-llvm-rt.a");

    let status = Command::new("gcc")
        .current_dir(AFL_SRC_PATH)
        .arg("-c")
        .arg("-O1")
        .arg("-fPIC")
        .arg("-fno-omit-frame-pointer")
        .arg("llvm_mode/afl-llvm-rt.o.c")
        .arg("-fpermissive")
        .args(&[OsStr::new("-o"), object_file_path.as_os_str()])
        .status()
        .unwrap();
    assert!(status.success());

    let status = Command::new("ar")
        .arg("r")
        .arg(archive_file_path)
        .arg(object_file_path)
        .status()
        .unwrap();
    assert!(status.success());
}

fn create_dir(path: &Path) {
    if !path.exists() {
        fs::create_dir(&path).unwrap();
    }
}
