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
