<h1 align="center">
  <a href="https://github.com/frewsxcv/afl.rs/issues/66"><img src="etc/logo.gif" alt="afl.rs logo"></a>
  <br>
  afl.rs
</h1>

<h4 align="center">Fuzzing <a href="https://www.rust-lang.org">Rust</a> code with <a href="https://aflplus.plus/">AFLplusplus</a></h4>

**Notice:** Version 0.14.0 of afl.rs requires you to install the `cargo-afl` binary with:

```sh
cargo install cargo-afl
```

If the binary is already installed, you may need to add `--force`.

## What is it?

[Fuzz testing][] is a software testing technique used to find security and stability issues by providing pseudo-random data as input to the software. [AFLplusplus][] is a popular, effective, and modern fuzz testing tool based on [AFL][american-fuzzy-lop]. This library, afl.rs, allows one to run AFLplusplus on code written in [the Rust programming language][rust].

## Documentation

Documentation can be found in the [Rust Fuzz Book](https://rust-fuzz.github.io/book/afl.html).

## What does it look like?

<img src="etc/screencap.gif" width="563" height="368" alt="Screen recording of afl">

Screen recording of AFL running on Rust code.

## Hints

Before starting to fuzz, you should reconfigure your system for optimal
performance and better crash detection. This can be done with `cargo-afl afl system-config`.
But this subcommand requires root, so it uses sudo internally. Hence, you might need to enter
your password.

By default, the AFL++ [CMPLOG](https://github.com/AFLplusplus/AFLplusplus/blob/stable/instrumentation/README.cmplog.md)
feature is activated, which helps to achieve good code coverage.
However, it is not beneficial to activate CMPLOG on more than two instances.
So if you run multiple AFL++ instances on your fuzzing target, you can disable CMPLOG by specifying the command line parameter '-c -'.

This [document](https://github.com/AFLplusplus/AFLplusplus/blob/stable/docs/fuzzing_in_depth.md)
will familiarize you with AFL++ features to help in running a successful fuzzing campaign.

By default, 'fuzzing' config is set when `cargo-afl` is used to build. If you want to prevent this, just set the
environment variable `AFL_NO_CFG_FUZZING` to `1` when building.

[conditional compilation]: https://doc.rust-lang.org/reference.html#conditional-compilation
[Cargo feature]: http://doc.crates.io/manifest.html#the-[features]-section
[example-defer]: https://github.com/frewsxcv/afl.rs/blob/master/examples/deferred-init.rs
[LLVM pass]: https://github.com/frewsxcv/afl.rs/blob/master/plugin/src/afl-llvm-pass.o.cc
[example]: https://github.com/frewsxcv/afl.rs/blob/master/afl/examples/hello.rs
[Cargo]: http://doc.crates.io/
[unresolved issue]: https://github.com/frewsxcv/afl.rs/issues/11
[fuzz testing]: https://en.wikipedia.org/wiki/Fuzz_testing
[rustup]: https://rustup.rs/
[american-fuzzy-lop]: http://lcamtuf.coredump.cx/afl/
[AFLplusplus]: https://aflplus.plus/
[rust]: https://www.rust-lang.org
