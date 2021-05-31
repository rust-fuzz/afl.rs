use std::path::{Path, PathBuf};

use xdg;

fn xdg_dir() -> xdg::BaseDirectories {
    let prefix = Path::new("afl.rs")
        .join(afl_rustc_version())
        .join(pkg_version());
    xdg::BaseDirectories::with_prefix(prefix).unwrap()
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

// Place directories inside the crate when building for docs.rs. 
// (Modifying system paths are forbidden.)

#[cfg(docsrs)]
pub fn afl_dir() -> PathBuf {
    let path = PathBuf::from("./afl");
    std::fs::create_dir_all(&path).unwrap();
    path
}

#[cfg(not(docsrs))]
pub fn afl_dir() -> PathBuf {
    xdg_dir().create_data_directory("afl").unwrap()
}

#[cfg(docsrs)]
pub fn afl_llvm_rt_dir() -> PathBuf {
    let path = PathBuf::from("./afl-llvm-rt");
    std::fs::create_dir_all(&path).unwrap();
    path
}

#[cfg(not(docsrs))]
pub fn afl_llvm_rt_dir() -> PathBuf {
    xdg_dir().create_data_directory("afl-llvm-rt").unwrap()
}

#[allow(dead_code)]
pub fn object_file_path() -> PathBuf {
    afl_llvm_rt_dir().join("libafl-llvm-rt.o")
}

#[allow(dead_code)]
pub fn archive_file_path() -> PathBuf {
    afl_llvm_rt_dir().join("libafl-llvm-rt.a")
}
