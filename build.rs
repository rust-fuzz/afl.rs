// Copyright 2015 Keegan McAllister.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// See `LICENSE` in this repository.

#![deny(warnings)]

extern crate gcc;

use std::env;

fn main() {
    gcc::Config::new()
        .file("src/afl_cov_rt.c")
        .flag("-O2").flag("-fPIC")
        .flag("-Wall") // can't use -Werror due to constructor(0)
        .compile("libafl_cov_rt.a");

    println!("cargo:rustc-link-search=native={}",
        env::var("OUT_DIR").unwrap());
}
