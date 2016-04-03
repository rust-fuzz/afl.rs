#!/bin/sh -e

export AFL_NO_UI=1

cd afl-plugin
cargo build --verbose
cd ..
cargo build --verbose
cargo build --example hello
cargo build --example deferred-init
cargo build --example integer-overflow
cd afl-sys
cargo build
cd ..
cargo test
