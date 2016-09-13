#!/bin/sh -e

cd afl-plugin
cargo build --verbose
cd ..
cargo build --verbose
cargo build --example hello
cargo build --example deferred-init
cargo build --example integer-overflow
cargo build --example panic
cd afl-sys
cargo build
cd ..
cargo test
