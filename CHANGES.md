# Changes

## 0.12.5

* [build.rs: Remove DEBUG environment variable](https://github.com/rust-fuzz/afl.rs/pull/248)
* [Check for `DOCS_RS` environment variable in build.rs](https://github.com/rust-fuzz/afl.rs/pull/249)

## 0.12.4

* [Set rustc-args instead of rustdoc-args](https://github.com/rust-fuzz/afl.rs/commit/125af5fa32f13e1ecaab0e219eecca286ee9d8e8)

## 0.12.3

* [Fix docs.rs documentation generation](https://github.com/rust-fuzz/afl.rs/pull/231)

## 0.12.2

* [Handle new LLVM pass manager on rustc 1.59](https://github.com/rust-fuzz/afl.rs/pull/220)

## 0.12.1

* [Use arbitrary::Unstructured instead of arbitrary::RingBuffer](https://github.com/rust-fuzz/afl.rs/pull/211)
* [Stop is_nightly from spewing to stderr](https://github.com/rust-fuzz/afl.rs/pull/212)

## 0.12.0

* [Update to AFLplusplus 4.0.0c](https://github.com/rust-fuzz/afl.rs/pull/206)

## 0.11.1

* [Handle old LLVM pass manager on rustc 1.57](https://github.com/rust-fuzz/afl.rs/pull/197)

## 0.11.0

* [Update rustc_version to 0.4](https://github.com/rust-fuzz/afl.rs/pull/188)
* [Update AFLplusplus to 3.14c](https://github.com/rust-fuzz/afl.rs/pull/189)
* [Update for new LLVM pass manager](https://github.com/rust-fuzz/afl.rs/pull/193)

## 0.10.1

* [Get docs building on docs.rs](https://github.com/rust-fuzz/afl.rs/pull/185)

## 0.10.0

* [Qualify uses of `__fuzz!` macro](https://github.com/rust-fuzz/afl.rs/pull/174)
* [update to AFL++ 3.01a && enable persistent shared memory fuzzing](https://github.com/rust-fuzz/afl.rs/pull/180)
* [Remove deprecated functions: `read_stdio_bytes` and `read_stdio_string`](https://github.com/rust-fuzz/afl.rs/commit/08db0b0afbf20eb20e09e3dd0397e6adcfe33def)

## 0.9.0

* [MacOS – Hard-code the path to `ar` as `/usr/bin/ar`](https://github.com/rust-fuzz/afl.rs/pull/171)

## 0.8.0

* [Migrate from AFL to AFLplusplus](https://github.com/rust-fuzz/afl.rs/pull/169)

## 0.7.0

* [Add option to kill afl-fuzz after a time limit](https://github.com/rust-fuzz/afl.rs/pull/162)
* [Add opt-in resettable-lazy-static.rs feature](https://github.com/rust-fuzz/afl.rs/pull/166)

## 0.6.0

* [Accept `FnMut` instead of `Fn`](https://github.com/rust-fuzz/afl.rs/pull/165)
* [Eliminate `fuzz`/`fuzz_nohook` redundancy](https://github.com/rust-fuzz/afl.rs/pull/161)

## 0.5.2

* [Expose `common` in the public API](https://github.com/rust-fuzz/afl.rs/pull/159)

## 0.5.1

* [Fix broken OS detection](https://github.com/rust-fuzz/afl.rs/pull/153)

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
