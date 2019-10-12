# Changes

## 0.5.0

* [Add a fuzz! version that doesn't hook panics](https://github.com/rust-fuzz/afl.rs/pull/154)

## 0.4.4

* [Add build support for AFL on ARM](https://github.com/rust-fuzz/afl.rs/pull/157)

## 0.4.3

* [Only enable -fuse-ld=gold on Linux.](https://github.com/rust-fuzz/afl.rs/pull/147)

## 0.4.2

* [Work around linking issues from rust-fuzz/afl.rs#141, rust-lang/rust#53945](https://github.com/rust-fuzz/afl.rs/pull/144)

## 0.4.1

* [Set RUSTDOCFLAGS to avoid issues with linkage for doctests](https://github.com/rust-fuzz/afl.rs/pull/143)

## 0.4.0

* [Run AFL in persistent mode, use `panic::set_hook()`, add ASAN/TSAN flags, deprecate non-persistent functions, `RUSTFLAGS` no longer get overwritten](https://github.com/rust-fuzz/afl.rs/pull/137)

## 0.3.2

* [Commit cargo.lock since we now distribute a binary](https://github.com/rust-fuzz/afl.rs/commit/fc80199080f36ea0c249e1a4bb827370dcefebc2)

## 0.3.1

* [Bump clap-rs to fix `cargo-afl fuzz --help` bug.](https://github.com/rust-fuzz/afl.rs/issues/121)

## 0.3.0

* [Prefer panic catching strategy over adjusting panic strategy.](https://github.com/rust-fuzz/afl.rs/pull/123)
* [Incorporate commit hash into directory structure.](https://github.com/rust-fuzz/afl.rs/pull/125)

## 0.2.3

* [Print error message if archive file (to be linked) can't be found.](https://github.com/rust-fuzz/afl.rs/commit/d65c9cbc7f679aae87b0ad92d7e2496ee4e09e55)

## 0.2.2

* [Use more generic C compiler binary name](https://github.com/rust-fuzz/afl.rs/commit/f1369aadc2352510d2af42d23480324800960d26)
* [More descriptive panic messages](https://github.com/rust-fuzz/afl.rs/commit/7f0114c0a0d42e1487f5e573e949b12f8932f42c)

## 0.2.1

* [Introduce more helpful CLI using clap-rs](https://github.com/rust-fuzz/afl.rs/commit/c9537eabd412591b91e328f41451c4aba199c684)

## 0.2.0

* [Rewrite of afl.rs; introduction of cargo-afl](https://github.com/rust-fuzz/afl.rs/pull/116)

## 0.1.5

* Don't enforce LLVM version 3.8

## 0.1.4

* Merged in upstream changes for LLVM files: afl-llvm-pass.so.cc, afl-llvm-rt.o.c
* Check check for `llvm-config-3.8` in `PATH` in addition to `llvm-config`
* Utilities for reading from standard input and handling panics: `afl::handle_*`
* Initial writing for "The afl.rs Book"

## 0.1.3

* [Don't pass extra values to C afl-fuzz `main`.](https://github.com/frewsxcv/afl.rs/pull/62)

## 0.1.2

* [Add afl-sys crate](https://github.com/frewsxcv/afl.rs/pull/51)
* [Introduce `cargo afl-fuzz`](https://github.com/frewsxcv/afl.rs/pull/60)
