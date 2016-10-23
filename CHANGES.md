# Changes

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
