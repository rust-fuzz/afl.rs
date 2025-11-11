//! Utilities needed by `cargo-afl`'s `config` module and the `update` test

use anyhow::{Result, bail, ensure};
use std::path::Path;
use std::process::{Command, ExitStatus};

const AFL_SRC_PATH: &str = "AFLplusplus";
const AFLPLUSPLUS_URL: &str = "https://github.com/AFLplusplus/AFLplusplus";

pub fn update_to_stable_or_tag(aflplusplus_dir: &Path, tag: Option<&str>) -> Result<()> {
    if is_repo(aflplusplus_dir)? {
        let success = Command::new("git")
            .arg("fetch")
            .current_dir(aflplusplus_dir)
            .status()
            .as_ref()
            .is_ok_and(ExitStatus::success);
        ensure!(success, "could not run 'git fetch'");
    } else {
        remove_aflplusplus_dir(aflplusplus_dir).unwrap_or_default();
        let success = Command::new("git")
            .args([
                "clone",
                AFLPLUSPLUS_URL,
                &*aflplusplus_dir.to_string_lossy(),
            ])
            .status()
            .as_ref()
            .is_ok_and(ExitStatus::success);
        ensure!(success, "could not run 'git clone'");
    }

    let mut command = Command::new("git");
    command.arg("checkout");
    if let Some(tag) = tag {
        command.arg(tag);
    } else {
        command.arg("origin/stable");
    }
    command.current_dir(aflplusplus_dir);
    let success = command.status().as_ref().is_ok_and(ExitStatus::success);
    ensure!(success, "could not run 'git checkout'");

    Ok(())
}

pub fn remove_aflplusplus_dir(aflplusplus_dir: &Path) -> Result<()> {
    std::fs::remove_dir_all(aflplusplus_dir).map_err(Into::into)
}

#[allow(clippy::disallowed_macros)]
pub fn copy_aflplusplus_submodule(aflplusplus_dir: &Path) -> Result<()> {
    let afl_src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(AFL_SRC_PATH);
    let afl_src_dir_str = &afl_src_dir.to_string_lossy();

    let Some(aflplusplus_dir_parent) = aflplusplus_dir.parent() else {
        bail!("could not get AFLplusplus dir parent");
    };
    debug_assert_eq!(aflplusplus_dir_parent.join(AFL_SRC_PATH), aflplusplus_dir);

    let success = Command::new("cp")
        .args([
            "-P", // preserve symlinks
            "-R", // copy directories recursively
            afl_src_dir_str,
            &*aflplusplus_dir_parent.to_string_lossy(),
        ])
        .status()
        .as_ref()
        .is_ok_and(ExitStatus::success);
    ensure!(
        success,
        "could not copy directory `{}`",
        afl_src_dir.display()
    );

    Ok(())
}

// smoelius: `dot_git` will refer to an ASCII text file if it was copied from the AFLplusplus
// submodule from afl.rs's source tree.
pub fn is_repo(aflplusplus_dir: &Path) -> Result<bool> {
    let dot_git = aflplusplus_dir.join(".git");
    if dot_git.try_exists()? {
        Ok(dot_git.is_dir())
    } else {
        Ok(false)
    }
}
