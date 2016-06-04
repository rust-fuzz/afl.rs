// Copyright 2015 Keegan McAllister.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// See `LICENSE` in this repository.

#![feature(plugin)]

#![plugin(afl_plugin)]

// Integer overflow bug.
// Loosely based on:
//   https://github.com/sandstorm-io/capnproto/blob/master/security-advisories/2015-03-02-0-c%2B%2B-integer-overflow.md

extern crate afl;
extern crate byteorder;

use std::{mem, io, slice};
use std::io::Read;
use byteorder::{ReadBytesExt, LittleEndian, Error};

fn main() {
    let mut stdin = io::stdin();

    // First, the element size.
    let bytes_per_element = stdin.read_u32::<LittleEndian>().unwrap();

    loop {
        let element_count = match stdin.read_u32::<LittleEndian>() {
            Err(Error::UnexpectedEOF) => break,
            Err(e) => panic!(e),
            Ok(n) => n,
        };

        let total_size = element_count.wrapping_mul(bytes_per_element);
        assert!(total_size <= (1 << 20)); // 1MB limit
        let total_size = total_size as usize;

        let mut buf: Vec<u8> = Vec::with_capacity(total_size);

        let dest: &mut [u8] = unsafe {
            mem::transmute(slice::from_raw_parts_mut(
                buf.as_mut_ptr(),
                (element_count as usize) * (bytes_per_element as usize),
            ))
        };

        match stdin.by_ref().read(dest) {
            Ok(n) if n == total_size => {
                unsafe {
                    buf.set_len(n);
                    println!("full read: {:?}", buf);
                }
            }
            Ok(n) => println!("partial read: got {}, expected {}", n, total_size),
            Err(_) => println!("error!"),
        }
    }
}
