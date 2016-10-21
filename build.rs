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
        .file("src/afl-llvm-rt.o.c")
        .opt_level(3)
        .flag("-w")
        .flag("-fPIC")
        .compile("libafl-llvm-rt.a");

    println!("cargo:rustc-link-search=native={}",
        env::var("OUT_DIR").unwrap());
    println!("cargo:rustc-link-search=/usr/local/Cellar/llvm38/3.8.0/lib/llvm-3.8/lib");
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
}
