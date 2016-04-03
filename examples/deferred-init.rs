// Copyright 2015 Keegan McAllister.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// See `LICENSE` in this repository.

// Fuzz me with AFL_DEFER_FORKSRV=1 for a huge speedup.

#![feature(plugin)]

#![plugin(afl_plugin)]

extern crate afl;

use std::thread;
use std::time::Duration;

fn main() {
    println!("An eternity in...");
    thread::sleep(Duration::from_secs(1));

    unsafe {
        afl::init();
    }

    afl::handle_string(|input| {
        println!("the blink of an eye.");
        if input.starts_with("x") {
            println!("going...");
            if input.starts_with("xy") {
                println!("going...");
                if input.starts_with("xyz") {
                    panic!("gone!");
                }
            }
        }
    });
}
