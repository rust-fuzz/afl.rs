<h1 align="center">
  <a href="https://github.com/frewsxcv/afl.rs/issues/66"><img src="etc/logo.gif" alt="afl.rs logo"></a>
  <br>
  afl.rs
</h1>

<h4 align="center">Fuzzing <a href="https://www.rust-lang.org">Rust</a> code with <a href="http://lcamtuf.coredump.cx/afl/">american fuzzy lop (AFL)</a></h4>

## What is it?

[Fuzz testing][] is a software testing technique used to find security and stability issues by providing pseudo-random data as input to the software. [American fuzzy lop][american-fuzzy-lop] is a popular, effective, and modern fuzz testing tool. This library, afl.rs, allows one to run AFL on code written in [the Rust programming language][rust].

## What does it look like?

<img src="etc/screencap.gif" width="563" height="368" alt="Screen recording of afl">

Screen recording of AFL running on Rust code. The code under test is [`examples/hello.rs`][example] in this repository.

## Book

Documentation for afl.rs can be found here:

[The afl.rs Book](https://frewsxcv.github.io/afl.rs/)

## Trophy case

* brotli-rs: [#2](https://github.com/ende76/brotli-rs/issues/2), [#3](https://github.com/ende76/brotli-rs/issues/3), [#4](https://github.com/ende76/brotli-rs/issues/4), [#5](https://github.com/ende76/brotli-rs/issues/5), [#6](https://github.com/ende76/brotli-rs/issues/6), [#7](https://github.com/ende76/brotli-rs/issues/7), [#8](https://github.com/ende76/brotli-rs/issues/8), [#9](https://github.com/ende76/brotli-rs/issues/9), [#10](https://github.com/ende76/brotli-rs/issues/10), [#11](https://github.com/ende76/brotli-rs/issues/11), [#12](https://github.com/ende76/brotli-rs/issues/12)
* cpp_demangle: [#41](https://github.com/fitzgen/cpp_demangle/pull/41)
* flac: [#3](https://github.com/sourrust/flac/issues/3)
* httparse: [#9](https://github.com/seanmonstar/httparse/issues/9)
* image: [#414](https://github.com/PistonDevelopers/image/issues/414), [#473](https://github.com/PistonDevelopers/image/issues/473), [#474](https://github.com/PistonDevelopers/image/issues/474), [#477](https://github.com/PistonDevelopers/image/issues/477)
* jpeg-decoder: [#38](https://github.com/kaksmet/jpeg-decoder/issues/38), [#50](https://github.com/kaksmet/jpeg-decoder/issues/50)
* mp3-metadata: [#9](https://github.com/GuillaumeGomez/mp3-metadata/pull/9)
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
[Cargo feature]: http://doc.crates.io/manifest.html#the-[features]-section
[example-defer]: https://github.com/frewsxcv/afl.rs/blob/master/examples/deferred-init.rs
[LLVM pass]: https://github.com/frewsxcv/afl.rs/blob/master/plugin/src/afl-llvm-pass.o.cc
[example]: https://github.com/frewsxcv/afl.rs/blob/master/examples/hello.rs
[Cargo]: http://doc.crates.io/
[unresolved issue]: https://github.com/frewsxcv/afl.rs/issues/11
[fuzz testing]: https://en.wikipedia.org/wiki/Fuzz_testing
[rustup]: https://rustup.rs/
[american-fuzzy-lop]: http://lcamtuf.coredump.cx/afl/
[rust]: https://www.rust-lang.org
