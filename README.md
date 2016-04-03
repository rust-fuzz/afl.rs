# afl.rs

Fuzzing [Rust][] code with [american fuzzy lop (AFL)][american-fuzzy-lop]

<img src="https://i.imgur.com/RUZRyTO.gif" width="563" height="368" alt="Screen recording of afl">

Screen recording of AFL running on Rust code. The code under test is [`examples/hello.rs`][example] in this repository.

## What is it?

[Fuzz testing][] is a software testing technique used to find security and stability issues by providing pseudo-random data as input to the software. [American fuzzy lop][american-fuzzy-lop] is a popular, effective, and modern fuzz testing tool. This library, afl.rs, allows one to run AFL on code written in [the Rust programming language][rust].

## Requirements

* Nightly build of Rust from any time after January 24, 2016 ([this issue](https://github.com/rust-lang/rust/pull/31176) prevented compatibility with previous builds of Rust)
* C++ compiler that supports C++11
* afl.rs needs to compile against a version of LLVM that matches `rustc`'s. The easy solution (if you can wait on a slow build) is to [build `rustc` from source][from source] and put it in your `PATH`. Then afl.rs's [build script][] will find `llvm-config` automatically. Otherwise, the environment variable `LLVM_CONFIG` should hold the path to `llvm-config` when you build afl.rs.

Because of these relatively strict requirements, there is a Vagrantfile provided that assists in bootstraping an afl.rs compatible environment. View the README in the [`vagrant/`](vagrant) directory for more information.

## Using it

First, add this project as a [Cargo][] dependency:

```toml
[dependencies]
afl = "0.1"
afl-plugin = "0.1"
```

Then you can add afl instrumentation to one or more crates:

```rust
#![feature(plugin)]
#![plugin(afl_plugin)]
```

You will also need a test executable that exercises the instrumented functions,
in a deterministic way based on input from stdin. This executable should link
the `afl` run-time library:

```rust
extern crate afl;
```

This will produce a binary that you can pass to `afl-fuzz` in the usual manner.
afl instrumentation adds some run-time overhead, so it's a good candidate for
[conditional compilation][], perhaps through a [Cargo feature][]:

```toml
# You may need to add `optional = true` to the above dependencies.
[features]
afl = ["afl-plugin", "afl"]
```

```rust
// Active only with `cargo [...] --feature afl`
#![cfg_attr(feature = "afl", feature(plugin))]
#![cfg_attr(feature = "afl", plugin(afl_plugin))]
```

C++ code will be compiled by default with `g++`, though one can specify a different C++ compiler by setting the `CXX` environment variable to point to a different compiler binary.

## Tweakables

To look for logic errors in safe Rust code, use the `no-landing-pads` rustc flag
upon compilation of the AFL entrypoint.  This causes the fuzzer to treat any
Rust panic as a crash. Examples of usage:

* `rustc -Z no-landing-pads`
* `cargo rustc -- -Z no-landing-pads`

If your program has a slow set-up phase that does not depend on the input data,
you can set `AFL_DEFER_FORKSRV=1` for a substantial speed-up, provided that you
insert a call to `afl::init()` after the set-up and before any
dependence on input. There are various other caveats, described in the section
"Bonus feature: deferred instrumentation" in `llvm_mode/README.llvm`
distributed with afl. See also [`examples/deferred-init.rs`][example-defer] in
this repository.

See the afl documentation for other configuration variables. Some of these are
set at compile time in `config.h`. For the most part they only affect
`afl-fuzz` itself, and will work fine with this library. However, if you change
`SHM_ENV_VAR`, `MAP_SIZE`, or `FORKSRV_FD`, you should update this library's
`src/config.h` to match.

## Trophy case

* brotli-rs: [#2](https://github.com/ende76/brotli-rs/issues/2), [#3](https://github.com/ende76/brotli-rs/issues/3), [#4](https://github.com/ende76/brotli-rs/issues/4), [#5](https://github.com/ende76/brotli-rs/issues/5), [#6](https://github.com/ende76/brotli-rs/issues/6), [#7](https://github.com/ende76/brotli-rs/issues/7), [#8](https://github.com/ende76/brotli-rs/issues/8), [#9](https://github.com/ende76/brotli-rs/issues/9), [#10](https://github.com/ende76/brotli-rs/issues/10), [#11](https://github.com/ende76/brotli-rs/issues/11), [#12](https://github.com/ende76/brotli-rs/issues/12)
* flac: [#3](https://github.com/sourrust/flac/issues/3)
* httparse: [#9](https://github.com/seanmonstar/httparse/issues/9)
* image: [#414](https://github.com/PistonDevelopers/image/issues/414), [#473](https://github.com/PistonDevelopers/image/issues/473), [#474](https://github.com/PistonDevelopers/image/issues/474), [#477](https://github.com/PistonDevelopers/image/issues/477)
* mp4parse-rust: [#2](https://github.com/mozilla/mp4parse-rust/issues/2), [#4](https://github.com/mozilla/mp4parse-rust/issues/4), [#5](https://github.com/mozilla/mp4parse-rust/issues/5), [#6](https://github.com/mozilla/mp4parse-rust/issues/6)
* rustc: [#24275](https://github.com/rust-lang/rust/issues/24275), [#24276](https://github.com/rust-lang/rust/issues/24276)
* rust-url: [#108](https://github.com/servo/rust-url/pull/108)
* regex: [#84](https://github.com/rust-lang/regex/issues/84)
* rust-asn1: [#32](https://github.com/alex/rust-asn1/issues/32)
* rustc-serialize: [#109](https://github.com/rust-lang/rustc-serialize/issues/109), [#110](https://github.com/rust-lang/rustc-serialize/issues/110)
* serde: [#75](https://github.com/serde-rs/serde/issues/75), [#77](https://github.com/serde-rs/serde/issues/77), [#82](https://github.com/serde-rs/serde/issues/82)
* tar-rs: [#23](https://github.com/alexcrichton/tar-rs/issues/23)
* xml-rs: [#93](https://github.com/netvl/xml-rs/issues/93)
* Logic errors in [tendril](https://github.com/kmcallister/tendril) and its [html5ever](https://github.com/servo/html5ever) integration

These bugs aren't nearly as serious as the [memory-safety issues afl has
discovered](http://lcamtuf.coredump.cx/afl/#bugs) in C and C++ projects.
That's because Rust is memory-safe by default, but also because not many people
have tried afl.rs yet! Over time we will update this section with the most
interesting bugs, whether they're logic errors or memory-safety problems
arising from `unsafe` code. Pull requests are welcome!

[conditional compilation]: https://doc.rust-lang.org/reference.html#conditional-compilation
[american-fuzzy-lop]: http://lcamtuf.coredump.cx/afl/
[Cargo feature]: http://doc.crates.io/manifest.html#the-[features]-section
[example-defer]: https://github.com/frewsxcv/afl.rs/blob/master/examples/deferred-init.rs
[build script]: https://github.com/frewsxcv/afl.rs/blob/master/plugin/build.bash
[from source]: https://github.com/rust-lang/rust#building-from-source
[LLVM pass]: https://github.com/frewsxcv/afl.rs/blob/master/plugin/src/afl-llvm-pass.o.cc
[example]: https://github.com/frewsxcv/afl.rs/blob/master/examples/hello.rs
[Cargo]: http://doc.crates.io/
[unresolved issue]: https://github.com/frewsxcv/afl.rs/issues/11
[fuzz testing]: https://en.wikipedia.org/wiki/Fuzz_testing
[Rust]: https://www.rust-lang.org
