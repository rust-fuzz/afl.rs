// Copyright 2015 Keegan McAllister.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// See `LICENSE` in this repository.

#![deny(warnings)]

use std::process::Command;
use std::env;

fn main() {
    assert!(Command::new("bash").arg("build.bash")
        .status().unwrap().success());

    println!("cargo:rustc-link-search=native={}",
        env::var("OUT_DIR").unwrap());
    println!("cargo:rustc-link-lib=static=afl_cov");
}
