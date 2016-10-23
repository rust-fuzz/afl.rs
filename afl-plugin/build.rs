// Copyright 2015 Keegan McAllister.
// Copyright 2016 Corey Farwell
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// See `LICENSE` in this repository.

extern crate gcc;
extern crate quale;

use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::process::Command;

/// Get the filesystem path to the llvm-config executable
fn llvm_config_path() -> OsString {
    let llvm_config_path = if let Some(path) = env::var_os("LLVM_CONFIG") {
        path
    } else if let Some(path) = quale::which("llvm-config-3.8") {
        path.into()
    } else if let Some(path) = quale::which("llvm-config") {
        path.into()
    } else {
        panic!("Could not find LLVM. Either set $LLVM_CONFIG or ensure \
                `llvm-config` is in your $PATH. Consult the README for more \
                information.");
    };
    File::open(&llvm_config_path).unwrap_or_else(|error| {
        panic!("Cannot access {:?} to determine your LLVM configuration: {}",
               llvm_config_path,
               error);
    });
    llvm_config_path
}

/// Using llvm-config, determine the flags we'll pass to the C++ compiler
fn cxx_flags<T: AsRef<OsStr>>(llvm_config_path: T) -> Vec<String> {
    let output = Command::new(llvm_config_path)
                     .arg("--cxxflags")
                     .output()
                     .unwrap_or_else(|e| panic!("failed to execute process: {}", e));
    if !output.status.success() {
        panic!("{:?}", output);
    }
    let output_string = unsafe { String::from_utf8_unchecked(output.stdout) };
    output_string.trim()
                 .split(' ')
                 .filter(|s| !s.is_empty())
                 .map(|s| s.to_owned())
                 .collect::<Vec<_>>()
}

fn main() {
    let llvm_config_path = llvm_config_path();
    let cxx_flags = cxx_flags(llvm_config_path);
    let mut config = gcc::Config::new();
    config.cpp(true);
    config.opt_level(2);
    config.file("afl-llvm-pass.so.cc");
    config.flag("-fPIC");
    config.flag("-Wall");
    config.flag("-Werror");
    config.flag("-fno-rtti");
    config.flag("-c");
    for flag in cxx_flags {
        config.flag(&flag);
    }
    config.compile("libafl-llvm-pass.a")
}
