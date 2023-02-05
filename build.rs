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
    let installing = home::cargo_home()
        .map(|path| Path::new(env!("CARGO_MANIFEST_DIR")).starts_with(path))
        .unwrap();

    // smoelius: Build AFLplusplus in a temporary directory when installing or when building on docs.rs.
    let (work_dir, base) = if installing || env::var("DOCS_RS").is_ok() {
        let out_dir = env::var("OUT_DIR").unwrap();
        let tempdir = tempfile::tempdir_in(&out_dir).unwrap();
        if Path::new(AFL_SRC_PATH).join(".git").is_dir() {
            let output = Command::new("git")
                .args(["clone", AFL_SRC_PATH, &*tempdir.path().to_string_lossy()])
                .output()
                .expect("could not run 'git'");
            assert!(output.status.success(), "{output:#?}");
        } else {
            fs_extra::dir::copy(
                AFL_SRC_PATH,
                tempdir.path(),
                &fs_extra::dir::CopyOptions {
                    content_only: true,
                    ..Default::default()
                },
            )
            .unwrap();
        }
        (tempdir.into_path(), Some(PathBuf::from(out_dir)))
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
        .args(["clean", "all", "install"])
        // skip the checks for the legacy x86 afl-gcc compiler
        .env("AFL_NO_X86", "1")
        // build just the runtime to avoid troubles with Xcode clang on macOS
        .env("NO_BUILD", "1")
        .env("DESTDIR", common::afl_dir(base))
        .env("PREFIX", "");
    if std::env::var("DEBUG").as_deref() == Ok("false") {
        command.env_remove("DEBUG");
    }
    let output = command.output().expect("could not run 'make'");
    assert!(output.status.success(), "{output:#?}");
}

fn build_afl_llvm_runtime(work_dir: &Path, base: Option<&Path>) {
    std::fs::copy(
        work_dir.join("afl-compiler-rt.o"),
        common::object_file_path(base),
    )
    .expect("Couldn't copy object file");

    let output = Command::new(AR_CMD)
        .arg("r")
        .arg(common::archive_file_path(base))
        .arg(common::object_file_path(base))
        .output()
        .expect("could not run 'ar'");
    assert!(output.status.success(), "{output:#?}");
}
