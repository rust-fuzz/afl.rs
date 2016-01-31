// Copyright 2015 Keegan McAllister.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// See `LICENSE` in this repository.

// Fuzz me with AFL_RS_CRASH_ON_PANIC=1.

#![feature(plugin)]

#![plugin(afl_plugin)]

extern crate afl;

use std::io::{self, Read};

fn main() {
    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input).unwrap();

    if input.starts_with(b"x") {
        println!("going...");
        if input.starts_with(b"xy") {
            println!("going...");
            if input.starts_with(b"xyz") {
                panic!("gone!");
            }
        }
    }
}
