#!/bin/sh -ex

export AFL_NO_UI=1

cd afl-plugin
cargo build --verbose
cd ../afl-sys
cargo build
cd ../afl
cargo build --verbose
cargo build --example hello
cargo build --example deferred-init
cargo build --example integer-overflow
cargo test
cd crate-tests/jpeg-decoder/
cargo build
