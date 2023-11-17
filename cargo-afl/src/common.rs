use std::env;
use std::path::{Path, PathBuf};

fn xdg_dir() -> xdg::BaseDirectories {
    let prefix = Path::new("afl.rs")
        .join(afl_rustc_version())
        .join(pkg_version());
    xdg::BaseDirectories::with_prefix(prefix).unwrap()
}

fn data_dir(dir_name: &str) -> PathBuf {
    // For docs.rs builds, use OUT_DIR.
    // For other cases, use a XDG data directory.
    // It is necessary to use OUT_DIR for docs.rs builds,
    // as that is the only place where we can write to.
    // The Cargo documentation recommends that build scripts
    // place their generated files at OUT_DIR too, but we
    // don't change that for now for normal builds.
    // smoelius: AFL++ is no longer built on docs.rs.
    xdg_dir().create_data_directory(dir_name).unwrap()
}

const SHORT_COMMIT_HASH_LEN: usize = 7;

#[must_use]
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

#[allow(dead_code)]
#[must_use]
pub fn afl_dir() -> PathBuf {
    data_dir("afl")
}

#[allow(dead_code)]
#[must_use]
pub fn afl_llvm_dir() -> PathBuf {
    data_dir("afl-llvm")
}

#[allow(dead_code)]
#[must_use]
pub fn object_file_path() -> PathBuf {
    afl_llvm_dir().join("libafl-llvm-rt.o")
}

#[allow(dead_code)]
#[must_use]
pub fn archive_file_path() -> PathBuf {
    afl_llvm_dir().join("libafl-llvm-rt.a")
}
