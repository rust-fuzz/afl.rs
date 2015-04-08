#!/bin/bash

# Copyright 2015 Keegan McAllister.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# See `LICENSE` in this repository.

set -e

: ${CXX:=g++}
: ${AR:=ar}
# $OUT_DIR is set by Cargo

if [ "$LLVM_CONFIG" == "" ]; then
    echo '[*] $LLVM_CONFIG not set. Will assume you built rustc from source.'
    LLVM_CONFIG=$(dirname $(which rustc))/../../llvm/Release+Asserts/bin/llvm-config
fi

if ! [ -x "$LLVM_CONFIG" ]; then
    echo "[-] Expected but did not find llvm-config at $LLVM_CONFIG"
    exit 1
fi

set -x

CXXFLAGS="$($LLVM_CONFIG --cxxflags) -O2 -fPIC -Wall -Werror -fno-rtti"

$CXX $CXXFLAGS -c src/afl_cov.cc -o $OUT_DIR/afl_cov.o
$AR rcs $OUT_DIR/libafl_cov.a $OUT_DIR/afl_cov.o
