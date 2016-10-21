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
    // Ensure we're working with version 3.8.x
    let output = Command::new(&llvm_config_path)
                         .arg("--version")
                         .output()
                         .expect("Could not execute llvm-config");
    if !output.stdout.starts_with(b"3.8") {
        panic!("Found llvm-config, but is not a compatible version. afl.rs \
                requires version LLVM 3.8. Either set $LLVM_CONFIG or ensure \
                `llvm-config` version 3.8 is in your $PATH.")
    }
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
    //println!("cargo:rustc-link-search=/usr/local/Cellar/llvm38/3.8.0/lib/llvm-3.8/lib");
    println!("cargo:rustc-link-search=/Users/coreyf/Development/rust/rust/./build/x86_64-apple-darwin/llvm/build/lib/");
    println!("cargo:rustc-link-lib=LLVMLTO");
    println!("cargo:rustc-link-lib=LLVMObjCARCOpts");
    println!("cargo:rustc-link-lib=LLVMSymbolize");
    println!("cargo:rustc-link-lib=LLVMDebugInfoPDB");
    println!("cargo:rustc-link-lib=LLVMDebugInfoDWARF");
    println!("cargo:rustc-link-lib=LLVMMIRParser");
    println!("cargo:rustc-link-lib=LLVMLibDriver");
    println!("cargo:rustc-link-lib=LLVMOption");
    println!("cargo:rustc-link-lib=LLVMTableGen");
    println!("cargo:rustc-link-lib=LLVMOrcJIT");
    println!("cargo:rustc-link-lib=LLVMPasses");
    println!("cargo:rustc-link-lib=LLVMipo");
    println!("cargo:rustc-link-lib=LLVMVectorize");
    println!("cargo:rustc-link-lib=LLVMLinker");
    println!("cargo:rustc-link-lib=LLVMIRReader");
    println!("cargo:rustc-link-lib=LLVMAsmParser");
    println!("cargo:rustc-link-lib=LLVMX86Disassembler");
    println!("cargo:rustc-link-lib=LLVMX86AsmParser");
    println!("cargo:rustc-link-lib=LLVMX86CodeGen");
    println!("cargo:rustc-link-lib=LLVMSelectionDAG");
    println!("cargo:rustc-link-lib=LLVMAsmPrinter");
    println!("cargo:rustc-link-lib=LLVMX86Desc");
    println!("cargo:rustc-link-lib=LLVMMCDisassembler");
    println!("cargo:rustc-link-lib=LLVMX86Info");
    println!("cargo:rustc-link-lib=LLVMX86AsmPrinter");
    println!("cargo:rustc-link-lib=LLVMX86Utils");
    println!("cargo:rustc-link-lib=LLVMMCJIT");
    println!("cargo:rustc-link-lib=LLVMLineEditor");
    println!("cargo:rustc-link-lib=LLVMDebugInfoCodeView");
    println!("cargo:rustc-link-lib=LLVMInterpreter");
    println!("cargo:rustc-link-lib=LLVMExecutionEngine");
    println!("cargo:rustc-link-lib=LLVMRuntimeDyld");
    println!("cargo:rustc-link-lib=LLVMCodeGen");
    println!("cargo:rustc-link-lib=LLVMTarget");
    println!("cargo:rustc-link-lib=LLVMScalarOpts");
    println!("cargo:rustc-link-lib=LLVMInstCombine");
    println!("cargo:rustc-link-lib=LLVMInstrumentation");
    println!("cargo:rustc-link-lib=LLVMProfileData");
    println!("cargo:rustc-link-lib=LLVMObject");
    println!("cargo:rustc-link-lib=LLVMMCParser");
    println!("cargo:rustc-link-lib=LLVMTransformUtils");
    println!("cargo:rustc-link-lib=LLVMMC");
    println!("cargo:rustc-link-lib=LLVMBitWriter");
    println!("cargo:rustc-link-lib=LLVMBitReader");
    println!("cargo:rustc-link-lib=LLVMAnalysis");
    println!("cargo:rustc-link-lib=LLVMCore");
    println!("cargo:rustc-link-lib=LLVMSupport");
    println!("cargo:rustc-link-lib=z");
    println!("cargo:rustc-link-lib=pthread");
    println!("cargo:rustc-link-lib=ffi");
    println!("cargo:rustc-link-lib=edit");
    println!("cargo:rustc-link-lib=curses");
    println!("cargo:rustc-link-lib=m");
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
