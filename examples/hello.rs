// Copyright 2015 Keegan McAllister.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// See `LICENSE` in this repository.

#![feature(plugin)]

#![plugin(afl_coverage_plugin)]

extern crate afl_coverage;

use std::io::{self, Read};

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    if input.starts_with("x") {
        println!("going...");
        if input.starts_with("xy") {
            println!("going...");
            if input.starts_with("xyz") {
                println!("gone!");
                unsafe {
                    let x: *mut usize = 0 as *mut usize;
                    *x = 0xBEEF;
                }
            }
        }
    }
}
