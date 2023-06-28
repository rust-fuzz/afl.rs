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
        .unwrap()
        || env::var("TESTING_INSTALL").is_ok();

    let building_cargo_afl = env::var("CARGO_PKG_NAME") == Ok(String::from("cargo-afl"));

    let building_on_docs_rs = env::var("DOCS_RS").is_ok();

    let out_dir = env::var("OUT_DIR").unwrap();

    if installing && !building_cargo_afl {
        println!("cargo:warning=You appear to be installing the `cargo-afl` binary with:");
        println!("cargo:warning=    cargo install afl");
        println!("cargo:warning=A future version of afl.rs will require you to use:");
        println!("cargo:warning=    cargo install cargo-afl");
        println!("cargo:warning=You can use the new command now, if you like.");
        println!(
            "cargo:warning=Note: If the binary is already installed, you may need to add --force."
        );
    }

    // smoelius: Build AFLplusplus in a temporary directory when installing or when building on docs.rs.
    let work_dir = if installing || building_on_docs_rs {
        let tempdir = tempfile::tempdir_in(&out_dir).unwrap();
        if Path::new(AFL_SRC_PATH).join(".git").is_dir() {
            let status = Command::new("git")
                .args(["clone", AFL_SRC_PATH, &*tempdir.path().to_string_lossy()])
                .status()
                .expect("could not run 'git'");
            assert!(status.success());
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
        tempdir.into_path()
    } else {
        PathBuf::from(AFL_SRC_PATH)
    };

    let base = if building_on_docs_rs {
        Some(PathBuf::from(out_dir))
    } else {
        None
    };

    // smoelius: Lock `work_dir` until the build script exits.
    #[cfg(unix)]
    let _file = sys::lock_path(&work_dir).unwrap();

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
        .env("PREFIX", "")
        .env_remove("DEBUG");
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

#[cfg(unix)]
mod sys {
    use std::fs::File;
    use std::io::{Error, Result};
    use std::os::unix::io::AsRawFd;
    use std::path::Path;

    pub fn lock_path(path: &Path) -> Result<File> {
        let file = File::open(path)?;
        lock_exclusive(&file)?;
        Ok(file)
    }

    // smoelius: `lock_exclusive` and `flock` were copied from:
    // https://github.com/rust-lang/cargo/blob/ae91d4ed41da98bdfa16041dbc6cd30287920120/src/cargo/util/flock.rs

    fn lock_exclusive(file: &File) -> Result<()> {
        flock(file, libc::LOCK_EX)
    }

    fn flock(file: &File, flag: libc::c_int) -> Result<()> {
        let ret = unsafe { libc::flock(file.as_raw_fd(), flag) };
        if ret < 0 {
            Err(Error::last_os_error())
        } else {
            Ok(())
        }
    }
}
