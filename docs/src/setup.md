# Setup

At the time of writing, the recommended approach when using afl.rs is to use a prebuilt Docker image. For more information about why this necessary, read the section following this one.

To install Docker, see the instructions in the following link:

[docker.com/getdocker](https://docker.com/getdocker)

Once you have installed Docker, retrieve the afl.rs image:

```
docker pull corey/afl.rs
```

## Why is Docker necessary?

*Note: This is optional reading. Don't worry if you're confused by anything in this section.*

AFL is a form of coverage-guided fuzzing (i.e. AFL requires insight into what code branches have been hit). In order to accomplish this, afl.rs includes a plugin for LLVM called an *[LLVM pass]*. This is accomplished via [a C++ file][afl.rs llvm pass] that afl.rs compiles and links against LLVM. Since Rust does *not* expose its LLVM internals, the user of afl.rs will have to either: compile the pass using tools that are ABI compatible with the Rust binary they're using or compile Rust from source. Neither of these options are trivial for the user. This guide used to recommend the former strategy, but [this caused issues][issues]. To get around this, this guide now recommends a Dockerfile which has rustc and afl.rs that are ABI-compatible.

[LLVM pass]: http://llvm.org/docs/WritingAnLLVMPass.html
[afl.rs llvm pass]: https://github.com/frewsxcv/afl.rs/blob/master/afl-plugin/afl-llvm-pass.so.cc
[issues]: https://github.com/frewsxcv/afl.rs/issues/57

## AFL

### OS X

```
brew update
brew install afl-fuzz
```

### Linux

Many operating system package repositories offer AFL:

```
sudo apt-get update
sudo apt-get install afl
```

If your operating system does not offer AFL, download and compile it [from the tarball provided on the AFL website](http://lcamtuf.coredump.cx/afl/).
