use std::path::{Path, PathBuf};

use xdg;
use rustc_version;

fn xdg_dir() -> xdg::BaseDirectories {
    // TODO incorporate crate version?
    let prefix = Path::new("afl.rs")
        .join(rustc_version::version().unwrap().to_string());
    xdg::BaseDirectories::with_prefix(prefix).unwrap()
}

pub fn afl() -> PathBuf {
    xdg_dir().create_data_directory("afl").unwrap()
}

pub fn afl_llvm_rt() -> PathBuf {
    xdg_dir().create_data_directory("afl-llvm-rt").unwrap()
}
