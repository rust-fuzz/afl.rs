# Setup

## Rust (nightly-2016-07-30)

For the time being, afl.rs requires a specific version of nightly Rust. To facilitate this requirement, using [rustup.rs](https://rustup.rs/) is recommended. Install it, then run:

```
rustup default nightly-2016-07-30
```

## C++ compiler

afl.rs will compile some C++ in order to install the necessary instrumentation. A C++ compiler that supports C++11 is required.

### OS X

OS X ships with g++, nothing needs to be done for OS X users.

### Linux

```
sudo apt-get update
sudo apt-get install g++ g++-multilib g++-4.9 g++-4.9-multilib libstdc++-4.8-dev
```

## LLVM 3.8

afl.rs will need to compile against LLVM 3.8 in order to setup instrumentation properly.

### OS X

```
brew update
brew install homebrew/versions/llvm38
```

### Linux

```
sudo apt-get update
sudo apt-get install llvm-3.8
```

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
