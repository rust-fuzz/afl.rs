use std::env;
use std::path::{Path, PathBuf};

use xdg;

fn xdg_dir() -> xdg::BaseDirectories {
    let prefix = Path::new("afl.rs")
        .join(afl_rustc_version())
        .join(pkg_version());
    xdg::BaseDirectories::with_prefix(prefix).unwrap()
}

fn data_dir(dir_name: &str) -> PathBuf {
    // In the build script, at build time, use OUT_DIR.
    // At runtime, use a XDG data directory.
    // It is idiomatic to use OUT_DIR in build scripts,
    // and in some environments (e.g., docsrs builds)
    // that may be the only place we can write to.
    env::var("OUT_DIR").map_or_else(
        |_| xdg_dir().create_data_directory(dir_name).unwrap(),
        |dir| {
            let path = Path::new(&dir).join(dir_name);
            std::fs::create_dir_all(&path).unwrap();
            path
        }
    )
}

const SHORT_COMMIT_HASH_LEN: usize = 7;

pub fn afl_rustc_version() -> String {
    let version_meta = rustc_version::version_meta().unwrap();
    let mut ret = String::from("rustc-");
    ret.push_str(&version_meta.semver.to_string());
    if let Some(commit_hash) = version_meta.commit_hash {
        ret.push('-');
        ret.push_str(&commit_hash[..SHORT_COMMIT_HASH_LEN]);
    }
    ret
}

fn pkg_version() -> String {
    let mut ret = String::from("afl.rs-");

    let version = env!("CARGO_PKG_VERSION");
    assert!(!version.is_empty());

    ret.push_str(version);
    ret
}

pub fn afl_dir() -> PathBuf {
    data_dir("afl")
}

pub fn afl_llvm_rt_dir() -> PathBuf {
    data_dir("afl-llvm-rt")
}

#[allow(dead_code)]
pub fn object_file_path() -> PathBuf {
    afl_llvm_rt_dir().join("libafl-llvm-rt.o")
}

#[allow(dead_code)]
pub fn archive_file_path() -> PathBuf {
    afl_llvm_rt_dir().join("libafl-llvm-rt.a")
}
