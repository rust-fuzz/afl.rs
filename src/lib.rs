// Copyright 2015 Keegan McAllister.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// See `LICENSE` in this repository.

#![feature(core, std_misc)]

extern crate libc;

use std::{env, rt, intrinsics, any};
use libc::exit;

#[cfg(not(test))]
#[link(name="afl-llvm-rt", kind="static")]
extern "C" { }

#[no_mangle]
pub unsafe extern "C" fn __afl_rs_init() {
    fn crash(_msg: &(any::Any + Send), _file: &'static str, _line: u32) {
        unsafe {
            intrinsics::abort();
        }
    }

    if env::var_os("AFL_RS_CRASH_ON_PANIC").is_some() {
        if !rt::unwind::register(crash) {
            exit(1);
        }
    }
}
