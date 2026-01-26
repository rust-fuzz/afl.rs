# Changes

## 0.17.1

- [Fix a bug affecting afl-cmin](https://github.com/rust-fuzz/afl.rs/pull/686)
- [Upgrade AFLplusplus to commit afbcb07e](https://github.com/rust-fuzz/afl.rs/pull/688)

## 0.17.0

- [Change how `cargo-afl` manages AFL++ source code](https://github.com/rust-fuzz/afl.rs/pull/605)

## 0.16.0

- [Add AFL++ IJON functionality](https://github.com/rust-fuzz/afl.rs/pull/655)

## 0.15.24

- [Update AFLplusplus to 4.34c](https://github.com/rust-fuzz/afl.rs/pull/660)

## 0.15.23

- Internal refactor. The changes should not be noticeable to end users. If you experience breakage, please open an [issue](https://github.com/rust-fuzz/afl.rs/issues).

## 0.15.22

- Go back to conventional publishing. Attempting to use trusted publishing caused the AFLplusplus submodule to not be included.

## 0.15.21

- [Enable link time optimization and strip symbols for release builds](https://github.com/rust-fuzz/afl.rs/pull/634)

## 0.15.20

- [Update AFLplusplus to 4.33c](https://github.com/rust-fuzz/afl.rs/pull/624)

## 0.15.19

- [Remove images from published crates.io source code](https://github.com/rust-fuzz/afl.rs/pull/618)

## 0.15.18

- [Update AFLplusplus to 4.32c](https://github.com/rust-fuzz/afl.rs/pull/612)

## 0.15.17

- The version of AFL++ on crates.io was 4.21a, not 4.31c. This release correct the error.

  Note that if AFL++ is built for your default Rust toolchain, then `cargo afl --version` will show AFL++'s version:

  ```
  $ cargo afl --version
  cargo-afl 0.15.17 (AFL++ version 4.31c)
                                   ^^^^^
  ```

## 0.15.16

- [Eliminate use of gold linker](https://github.com/rust-fuzz/afl.rs/pull/597)

## 0.15.15

- [Update AFLplusplus to 4.31c](https://github.com/rust-fuzz/afl.rs/pull/581)

  Version 0.15.14 claimed that it updated AFLplusplus to 4.31c. However, commit [e586a66](https://github.com/rust-fuzz/afl.rs/commit/e586a66aadc36977501257ee8b8201d61a452021) incorrectly undid the update. This release corrects the error.

## 0.15.14

- [Update AFLplusplus to 4.31c](https://github.com/rust-fuzz/afl.rs/pull/575)

## 0.15.13

- [Update AFLplusplus to 4.30c](https://github.com/rust-fuzz/afl.rs/pull/560)

## 0.15.12

- [Reorder passes and add split-switches-pass to AFL++ instrumentation](https://github.com/rust-fuzz/afl.rs/pull/554)

## 0.15.11

- [Introduce `AFLRS_REQUIRE_PLUGINS` and `AFL_OPT_LEVEL` environment variable options](https://github.com/rust-fuzz/afl.rs/pull/540)

## 0.15.10

- [Remove unnecessary `libc` dependency](https://github.com/rust-fuzz/afl.rs/commit/2f7c215caef827013b2b1418ab72eaadccd97ebe)
- [Fix typo in error message](https://github.com/rust-fuzz/afl.rs/commit/626b605d5757dc5b2cd9e422ca61d49a6165b7a1)
- [Improve error messages](https://github.com/rust-fuzz/afl.rs/commit/ead58d0422b1b16615627bb3620c1ee50fed92d9)
- [Eliminate reliance on `fs_extra`; copy AFL++ directory with shell](https://github.com/rust-fuzz/afl.rs/pull/506)

## 0.15.9

- [Don't require `--force` when enabling/disabling plugins](https://github.com/rust-fuzz/afl.rs/pull/498)
- [Display AFL++ version and whether plugins are in use](https://github.com/rust-fuzz/afl.rs/pull/500)

## 0.15.8

- [Update AFLplusplus to 4.21c](https://github.com/rust-fuzz/afl.rs/pull/492)

## 0.15.7

- [Use `expr` metavariable in `fuzz!` macro](https://github.com/rust-fuzz/afl.rs/pull/490)

## 0.15.6

- [Update AFLplusplus to commit fd71341 (latest on branch stable) to address a performance regression](https://github.com/rust-fuzz/afl.rs/pull/487)

## 0.15.5

- [Update AFLplusplus to 4.20c](https://github.com/rust-fuzz/afl.rs/pull/480)

## 0.15.4

- [Don't panic in `config::config`](https://github.com/rust-fuzz/afl.rs/pull/472)

## 0.15.3

- [Update AFLplusplus to 4.10c](https://github.com/rust-fuzz/afl.rs/pull/457)

## 0.15.2

- [Do not pass `-C passes=...` when nightly is used and plugins are compiled](https://github.com/rust-fuzz/afl.rs/pull/449)

## 0.15.1

- [Allow setting `__afl_persistent_loop` argument (support for #433)](https://github.com/rust-fuzz/afl.rs/pull/437)
- [Update AFLplusplus to 4.09c](https://github.com/rust-fuzz/afl.rs/pull/438)

## 0.15.0

- [Call existing panic hook before aborting](https://github.com/rust-fuzz/afl.rs/pull/426)
- [Add `config` subcommand](https://github.com/rust-fuzz/afl.rs/pull/421)

## 0.14.5

- [Add plugins feature](https://github.com/rust-fuzz/afl.rs/pull/392)

## 0.14.4

- [Add addseeds command](https://github.com/rust-fuzz/afl.rs/pull/407)

## 0.14.3

- [Fix running AFL with no 'fuzzing' flag](https://github.com/rust-fuzz/afl.rs/pull/398)

## 0.14.2

- [Update error message in cargo-afl.rs](https://github.com/rust-fuzz/afl.rs/pull/395)

## 0.14.1

- No functional changes. Version 0.14.1 was needed because version 0.14.0 failed to include the AFLplusplus submodule.

## 0.14.0

- [Make afl-system-config available as system-config](https://github.com/rust-fuzz/afl.rs/pull/371)
- [Complete separation of `afl` and `cargo-afl`](https://github.com/rust-fuzz/afl.rs/pull/375)
- [Remove `reset_lazy_static` feature](https://github.com/rust-fuzz/afl.rs/pull/376)

## 0.13.5

- [Update AFLplusplus to commit ad2af7c (latest on branch stable)](https://github.com/rust-fuzz/afl.rs/pull/374)

## 0.13.4

- [Update AFLplusplus to 4.08c](https://github.com/rust-fuzz/afl.rs/pull/367)

## 0.13.3

- [Fix etc symlinks](https://github.com/rust-fuzz/afl.rs/pull/352)
- [Add more transition messages](https://github.com/rust-fuzz/afl.rs/pull/354)

## 0.13.2

- [Add transitional message](https://github.com/rust-fuzz/afl.rs/pull/348)

## 0.13.1

- [Update AFLplusplus to 4.07c](https://github.com/rust-fuzz/afl.rs/pull/344)

## 0.13.0

- [Add mini CmpLog](https://github.com/rust-fuzz/afl.rs/pull/324)
- [Remove --max-total-time](https://github.com/rust-fuzz/afl.rs/pull/333)

## 0.12.17

- [Adjust build script output](https://github.com/rust-fuzz/afl.rs/pull/317)
- [Unconditionally remove `DEBUG` environment variable](https://github.com/rust-fuzz/afl.rs/pull/321)
- [Update AFLplusplus to 4.06c](https://github.com/rust-fuzz/afl.rs/pull/322)

## 0.12.16

- [Add optional feature 'no_cfg_fuzzing'](https://github.com/rust-fuzz/afl.rs/pull/306)

## 0.12.15

- [Bump tempfile from 3.3.0 to 3.4.0](https://github.com/rust-fuzz/afl.rs/pull/302)

## 0.12.14

- [Fix broken installation (#299)](https://github.com/rust-fuzz/afl.rs/pull/300)

## 0.12.13

- [Do not store object files inside $CARGO_HOME](https://github.com/rust-fuzz/afl.rs/pull/297)

## 0.12.12

- [Remove debuginfo=0 from default compiler options](https://github.com/rust-fuzz/afl.rs/pull/291)

## 0.12.11

- [Add --max_total_time deprecation message](https://github.com/rust-fuzz/afl.rs/pull/278)
- [Update AFLplusplus to 4.05c](https://github.com/rust-fuzz/afl.rs/pull/289)

## 0.12.10

- [Update AFLplusplus to 4.04c](https://github.com/rust-fuzz/afl.rs/pull/267)

## 0.12.9

- [Update AFLplusplus to 4.03c](https://github.com/rust-fuzz/afl.rs/pull/260)
- [Upgrade to Clap 4](https://github.com/rust-fuzz/afl.rs/pull/263)

## 0.12.8

- [Copy if AFLplusplus is not a git repository](https://github.com/rust-fuzz/afl.rs/commit/ff8d1c8c970cd5977b3efed74a78af9a49b315f4)

## 0.12.7

- [Build AFL in a temporary directory in `OUT_DIR`](https://github.com/rust-fuzz/afl.rs/pull/254)

## 0.12.6

- [Build AFL in a temporary directory on docs.rs](https://github.com/rust-fuzz/afl.rs/pull/250)
- [Add help for `shmget() failed` error message](https://github.com/rust-fuzz/afl.rs/pull/253)
- [Update AFLpluplus to 4.02c](https://github.com/rust-fuzz/afl.rs/pull/251)

## 0.12.5

- [build.rs: Remove DEBUG environment variable](https://github.com/rust-fuzz/afl.rs/pull/248)
- [Check for `DOCS_RS` environment variable in build.rs](https://github.com/rust-fuzz/afl.rs/pull/249)

## 0.12.4

- [Set rustc-args instead of rustdoc-args](https://github.com/rust-fuzz/afl.rs/commit/125af5fa32f13e1ecaab0e219eecca286ee9d8e8)

## 0.12.3

- [Fix docs.rs documentation generation](https://github.com/rust-fuzz/afl.rs/pull/231)

## 0.12.2

- [Handle new LLVM pass manager on rustc 1.59](https://github.com/rust-fuzz/afl.rs/pull/220)

## 0.12.1

- [Use arbitrary::Unstructured instead of arbitrary::RingBuffer](https://github.com/rust-fuzz/afl.rs/pull/211)
- [Stop is_nightly from spewing to stderr](https://github.com/rust-fuzz/afl.rs/pull/212)

## 0.12.0

- [Update to AFLplusplus 4.00c](https://github.com/rust-fuzz/afl.rs/pull/206)

## 0.11.1

- [Handle old LLVM pass manager on rustc 1.57](https://github.com/rust-fuzz/afl.rs/pull/197)

## 0.11.0

- [Update rustc_version to 0.4](https://github.com/rust-fuzz/afl.rs/pull/188)
- [Update AFLplusplus to 3.14c](https://github.com/rust-fuzz/afl.rs/pull/189)
- [Update for new LLVM pass manager](https://github.com/rust-fuzz/afl.rs/pull/193)

## 0.10.1

- [Get docs building on docs.rs](https://github.com/rust-fuzz/afl.rs/pull/185)

## 0.10.0

- [Qualify uses of `__fuzz!` macro](https://github.com/rust-fuzz/afl.rs/pull/174)
- [update to AFL++ 3.01a && enable persistent shared memory fuzzing](https://github.com/rust-fuzz/afl.rs/pull/180)
- [Remove deprecated functions: `read_stdio_bytes` and `read_stdio_string`](https://github.com/rust-fuzz/afl.rs/commit/08db0b0afbf20eb20e09e3dd0397e6adcfe33def)

## 0.9.0

- [MacOS – Hard-code the path to `ar` as `/usr/bin/ar`](https://github.com/rust-fuzz/afl.rs/pull/171)

## 0.8.0

- [Migrate from AFL to AFLplusplus](https://github.com/rust-fuzz/afl.rs/pull/169)

## 0.7.0

- [Add option to kill afl-fuzz after a time limit](https://github.com/rust-fuzz/afl.rs/pull/162)
- [Add opt-in resettable-lazy-static.rs feature](https://github.com/rust-fuzz/afl.rs/pull/166)

## 0.6.0

- [Accept `FnMut` instead of `Fn`](https://github.com/rust-fuzz/afl.rs/pull/165)
- [Eliminate `fuzz`/`fuzz_nohook` redundancy](https://github.com/rust-fuzz/afl.rs/pull/161)

## 0.5.2

- [Expose `common` in the public API](https://github.com/rust-fuzz/afl.rs/pull/159)

## 0.5.1

- [Fix broken OS detection](https://github.com/rust-fuzz/afl.rs/pull/153)

## 0.5.0

- [Add a fuzz! version that doesn't hook panics](https://github.com/rust-fuzz/afl.rs/pull/154)

## 0.4.4

- [Add build support for AFL on ARM](https://github.com/rust-fuzz/afl.rs/pull/157)

## 0.4.3

- [Only enable -fuse-ld=gold on Linux.](https://github.com/rust-fuzz/afl.rs/pull/147)

## 0.4.2

- [Work around linking issues from rust-fuzz/afl.rs#141, rust-lang/rust#53945](https://github.com/rust-fuzz/afl.rs/pull/144)

## 0.4.1

- [Set RUSTDOCFLAGS to avoid issues with linkage for doctests](https://github.com/rust-fuzz/afl.rs/pull/143)

## 0.4.0

- [Run AFL in persistent mode, use `panic::set_hook()`, add ASAN/TSAN flags, deprecate non-persistent functions, `RUSTFLAGS` no longer get overwritten](https://github.com/rust-fuzz/afl.rs/pull/137)

## 0.3.2

- [Commit cargo.lock since we now distribute a binary](https://github.com/rust-fuzz/afl.rs/commit/fc80199080f36ea0c249e1a4bb827370dcefebc2)

## 0.3.1

- [Bump clap-rs to fix `cargo-afl fuzz --help` bug.](https://github.com/rust-fuzz/afl.rs/issues/121)

## 0.3.0

- [Prefer panic catching strategy over adjusting panic strategy.](https://github.com/rust-fuzz/afl.rs/pull/123)
- [Incorporate commit hash into directory structure.](https://github.com/rust-fuzz/afl.rs/pull/125)

## 0.2.3

- [Print error message if archive file (to be linked) can't be found.](https://github.com/rust-fuzz/afl.rs/commit/d65c9cbc7f679aae87b0ad92d7e2496ee4e09e55)

## 0.2.2

- [Use more generic C compiler binary name](https://github.com/rust-fuzz/afl.rs/commit/f1369aadc2352510d2af42d23480324800960d26)
- [More descriptive panic messages](https://github.com/rust-fuzz/afl.rs/commit/7f0114c0a0d42e1487f5e573e949b12f8932f42c)

## 0.2.1

- [Introduce more helpful CLI using clap-rs](https://github.com/rust-fuzz/afl.rs/commit/c9537eabd412591b91e328f41451c4aba199c684)

## 0.2.0

- [Rewrite of afl.rs; introduction of cargo-afl](https://github.com/rust-fuzz/afl.rs/pull/116)

## 0.1.5

- Don't enforce LLVM version 3.8

## 0.1.4

- Merged in upstream changes for LLVM files: afl-llvm-pass.so.cc, afl-llvm-rt.o.c
- Check check for `llvm-config-3.8` in `PATH` in addition to `llvm-config`
- Utilities for reading from standard input and handling panics: `afl::handle_*`
- Initial writing for "The afl.rs Book"

## 0.1.3

- [Don't pass extra values to C afl-fuzz `main`.](https://github.com/frewsxcv/afl.rs/pull/62)

## 0.1.2

- [Add afl-sys crate](https://github.com/frewsxcv/afl.rs/pull/51)
- [Introduce `cargo afl-fuzz`](https://github.com/frewsxcv/afl.rs/pull/60)
