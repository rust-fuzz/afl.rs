# Fuzzing Rust with american-fuzzy-lop

This package allows you to find bugs in Rust code using [american-fuzzy-lop][].

![Screenshot of afl: 3 crashes (1 unique) found in 1 minute 43 seconds][screenshot]

This was performed on one core of an [i7-4790K][] at 4.8 GHz. The code under
test is [`examples/hello.rs`][example] in this repository.

## Using it

First, add this project as a [Cargo][] dependency:

```toml
[dependencies.afl-coverage-plugin]
git = "https://github.com/kmcallister/afl.rs"

[dependencies.afl-coverage]
git = "https://github.com/kmcallister/afl.rs"
```

Then you can add afl instrumentation to one or more crates:

```rust
#![feature(plugin)]
#![plugin(afl_coverage_plugin)]
```

You will also need a test executable that exercises the instrumented functions,
in a deterministic way based on input from stdin. This executable should link
the `afl_coverage` run-time library:

```rust
extern crate afl_coverage;
```

This will produce a binary that you can pass to `afl-fuzz` in the usual manner.
afl instrumentation adds some run-time overhead, so it's a good candidate for
[conditional compilation][], perhaps through a [Cargo feature][].

To look for logic errors in safe Rust code, set the environment variable
`$AFL_RS_CRASH_ON_PANIC` when you invoke `afl-fuzz`. This causes the fuzzer
to treat any Rust panic as a crash.

## Building it

`afl.rs` needs to compile against a version of LLVM that matches `rustc`'s. The
easy solution (if you can wait on a slow build) is to [build `rustc` from
source][from source] and put it in your `$PATH`. Then `afl.rs`'s [build
script][] will find `llvm-config` automatically. Otherwise, the environment
variable `$LLVM_CONFIG` should hold the path to `llvm-config` when you build
`afl.rs`.

It does *not* require `clang++`; it will use `$CXX` with a fallback to `g++`.
Your C++ compiler must support C++11.

If you've changed the afl config variable `SHM_ENV_VAR`, `MAP_SIZE`, or
`FORKSRV_FD`, you need to change `src/config.h` to match.

`afl.rs` uses an [LLVM pass][] based on [László Szekeres's work][mailing-list].

## Trophy case

* httparse: [#9](https://github.com/seanmonstar/httparse/issues/9)
* image: [#414](https://github.com/PistonDevelopers/image/issues/414)
* rustc: [#24275](https://github.com/rust-lang/rust/issues/24275), [#24276](https://github.com/rust-lang/rust/issues/24276)
* rust-url: [#108](https://github.com/servo/rust-url/pull/108)
* regex: [#84](https://github.com/rust-lang/regex/issues/84)
* rustc-serialize: [#109](https://github.com/rust-lang/rustc-serialize/issues/109), [#110](https://github.com/rust-lang/rustc-serialize/issues/110)
* serde: [#75](https://github.com/serde-rs/serde/issues/75)
* tar-rs: [#23](https://github.com/alexcrichton/tar-rs/issues/23)
* xml-rs: [#93](https://github.com/netvl/xml-rs/issues/93)
* Logic errors in [tendril](https://github.com/kmcallister/tendril) and its [html5ever](https://github.com/servo/html5ever) integration

These bugs aren't nearly as serious as the [memory-safety issues afl has
discovered](http://lcamtuf.coredump.cx/afl/#bugs) in C and C++ projects.
That's because Rust is memory-safe by default, but also because not many people
have tried afl.rs yet! Over time we will update this section with the most
interesting bugs, whether they're logic errors or memory-safety problems
arising from `unsafe` code. Pull requests are welcome!

[conditional compilation]: http://doc.rust-lang.org/reference.html#conditional-compilation
[american-fuzzy-lop]: http://lcamtuf.coredump.cx/afl/
[Cargo feature]: http://doc.crates.io/manifest.html#the-[features]-section
[build script]: https://github.com/kmcallister/afl.rs/blob/master/plugin/build.bash
[mailing-list]: https://groups.google.com/forum/#!msg/afl-users/gpa_igE8G50/uLAmT6v-bQEJ
[from source]: https://github.com/rust-lang/rust#building-from-source
[screenshot]: http://i.imgur.com/SbjNZKr.png
[LLVM pass]: https://github.com/kmcallister/afl.rs/blob/master/plugin/src/afl_cov.cc
[i7-4790k]: http://ark.intel.com/products/80807/Intel-Core-i7-4790K-Processor-8M-Cache-up-to-4_40-GHz
[example]: https://github.com/kmcallister/afl.rs/blob/master/examples/hello.rs
[unsafe]: http://doc.rust-lang.org/book/unsafe-code.html
[Cargo]: http://doc.crates.io/
