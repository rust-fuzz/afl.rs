// Copyright 2015 Keegan McAllister.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// See `LICENSE` in this repository.

#[cfg(not(test))]
#[link(name="afl-llvm-rt", kind="static")]
extern "C" {
    fn __afl_manual_init();
}

#[cfg(not(test))]
/// Initialize the afl runtime.
///
/// Only needed when the env var `AFL_DEFER_FORKSRV` is set.
///
/// It's undefined behavior to call this function from multiple
/// threads. You almost certainly need to call it before any
/// additional threads are created.
///
/// However, calling this more than once in a single-threaded
/// program, or calling it when not running with
/// `AFL_DEFER_FORKSRV` is safe and a no-op.
pub unsafe fn init() {
    __afl_manual_init();
}

#[cfg(test)]
mod test {
    use std::process::{Command, Stdio};
    use std::fs;
    use std::thread;
    use std::time;

    extern crate libc;
    extern crate tempdir;

    // TODO: enable this test for Linux
    #[cfg(target_os = "macos")]
    #[test]
    fn test_cargo_afl_fuzz() {
        let temp_dir = tempdir::TempDir::new("aflrs").expect("Could not create temporary directory");
        let temp_dir_path = temp_dir.path();
        let mut child = Command::new("target/debug/cargo-afl-fuzz")
            .stdout(Stdio::inherit())
            .arg("-i")
            .arg(".")
            .arg("-o")
            .arg(temp_dir_path)
            .arg("target/debug/examples/hello")
            .spawn()
            .expect("Could not run cargo-afl-fuzz");
        thread::sleep(time::Duration::from_secs(7));
        child.kill().unwrap();
        let mut entries = fs::read_dir(temp_dir_path.join("queue")).expect("Could not read AFL out directory");
        assert!(entries.next().is_some());
    }
}
