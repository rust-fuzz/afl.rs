// Copyright 2015 Keegan McAllister.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// See `LICENSE` in this repository.

#![feature(plugin)]

#![plugin(afl_plugin)]

extern crate afl;

fn main() {
    afl::handle_string(|input| {
        if input.starts_with("x") {
            println!("going...");
            if input.starts_with("xy") {
                println!("going...");
                if input.starts_with("xyz") {
                    panic!("gone!");
                }
            }
        }
    })
}
