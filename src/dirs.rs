use std::path::{Path, PathBuf};

use xdg;
use rustc_version;

fn xdg_dir() -> xdg::BaseDirectories {
    let prefix = Path::new("afl.rs")
        .join(rustc_version())
        .join(pkg_version());
    xdg::BaseDirectories::with_prefix(prefix).unwrap()
}

fn rustc_version() -> String {
    let mut ret = String::from("rustc-");
    ret.push_str(&rustc_version::version().unwrap().to_string());
    ret
}

fn pkg_version() -> String {
    let mut ret = String::from("afl.rs-");

    let version = env!("CARGO_PKG_VERSION");
    assert!(!version.is_empty());

    ret.push_str(version);
    ret
}

pub fn afl() -> PathBuf {
    xdg_dir().create_data_directory("afl").unwrap()
}

pub fn afl_llvm_rt() -> PathBuf {
    xdg_dir().create_data_directory("afl-llvm-rt").unwrap()
}
