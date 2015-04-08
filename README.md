# Fuzzing Rust with american-fuzzy-lop

This package allows you to find bugs in Rust code, particularly [`unsafe`][unsafe] Rust
code, using [american-fuzzy-lop][].

*(Pending [this patch to `rustc`](https://github.com/rust-lang/rust/pull/24207).)*

![Screenshot of afl: 3 crashes (1 unique) found in 1 minute 43 seconds][screenshot]

This was performed on one core of an [i7-4790K][] at 4.8 GHz. The code under
test is [`examples/hello.rs`][example] in this repository.

Once you've added this library as a [Cargo][] dependency, you can enable afl
instrumentation with

```rust
#![feature(plugin)]

#![plugin(afl_coverage_plugin)]

extern crate afl_coverage;
```

This will produce a binary that you can pass to `afl-fuzz` in the usual manner.
afl instrumentation adds some run-time overhead, so it's a good candidate for
[conditional compilation][], perhaps through a [Cargo feature][].

**Note**: `afl.rs` needs to compile against a version of LLVM that matches
`rustc`'s. The easy solution (if you can wait on a slow build) is to [build
`rustc` from source][from source] and put it in your `$PATH`. Then `afl.rs`'s
[build script][] will find `llvm-config` automatically. Otherwise, the environment
variable `$LLVM_CONFIG` should hold the path to `llvm-config` when you build
`afl.rs`.

It does *not* require `clang++`; it will use `$CXX` with a fallback to `g++`.
Your C++ compiler must support C++11.

`afl.rs` uses an [LLVM pass][] based on [László Szekeres's work][mailing-list].

If you've changed the afl config variable `SHM_ENV_VAR`, `MAP_SIZE`, or
`FORKSRV_FD`, you need to change `src/config.h` to match.

[conditional compilation]: http://doc.rust-lang.org/reference.html#conditional-compilation
[american-fuzzy-lop]: http://lcamtuf.coredump.cx/afl/
[Cargo feature]: http://doc.crates.io/manifest.html#the-[features]-section
[build script]: https://github.com/kmcallister/afl.rs/blob/master/plugin/build.bash
[mailing-list]: https://groups.google.com/forum/#!msg/afl-users/gpa_igE8G50/uLAmT6v-bQEJ
[from source]: https://github.com/rust-lang/rust#building-from-source
[screenshot]: https://camo.githubusercontent.com/ce24f64d6226654988de3ec2bf7719255d91beb6/687474703a2f2f692e696d6775722e636f6d2f7067784c337a722e706e67
[LLVM pass]: https://github.com/kmcallister/afl.rs/blob/master/plugin/src/afl_cov.cc
[i7-4790k]: http://ark.intel.com/products/80807/Intel-Core-i7-4790K-Processor-8M-Cache-up-to-4_40-GHz
[example]: https://github.com/kmcallister/afl.rs/blob/master/examples/hello.rs
[unsafe]: http://doc.rust-lang.org/book/unsafe.html
[Cargo]: http://doc.crates.io/
