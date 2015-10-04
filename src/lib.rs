// Copyright 2015 Keegan McAllister.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// See `LICENSE` in this repository.

#![feature(core_intrinsics, rt)]

extern crate libc;

use std::{env, rt, intrinsics, any};
use libc::exit;

#[cfg(not(test))]
#[link(name="afl-llvm-rt", kind="static")]
extern "C" {
    fn __afl_manual_init();
}

#[no_mangle]
pub unsafe extern "C" fn __afl_rs_init() {
    fn crash(_msg: &(any::Any + Send), _file: &'static str, _line: u32) {
        unsafe {
            intrinsics::abort();
        }
    }
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
