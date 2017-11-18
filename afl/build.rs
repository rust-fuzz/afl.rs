// Copyright 2015 Keegan McAllister.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// See `LICENSE` in this repository.

#![deny(warnings)]

extern crate cc;

use std::env;

fn main() {
    cc::Build::new()
        .file("src/afl-llvm-rt.o.c")
        .opt_level(3)
        .flag("-w")
        .flag("-fPIC")
        .compile("libafl-llvm-rt.a");

    println!("cargo:rustc-link-search=native={}",
        env::var("OUT_DIR").unwrap());
}
