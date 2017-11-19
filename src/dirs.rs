use std::path::{Path, PathBuf};

use xdg;
use rustc_version;

fn xdg_dir() -> xdg::BaseDirectories {
    let prefix = Path::new("afl.rs")
        .join(rustc_version::version().unwrap().to_string())
        .join(pkg_version());
    xdg::BaseDirectories::with_prefix(prefix).unwrap()
}

fn pkg_version() -> &'static str {
    let version = env!("CARGO_PKG_VERSION");
    assert!(!version.is_empty());
    version
}

pub fn afl() -> PathBuf {
    xdg_dir().create_data_directory("afl").unwrap()
}

pub fn afl_llvm_rt() -> PathBuf {
    xdg_dir().create_data_directory("afl-llvm-rt").unwrap()
}
