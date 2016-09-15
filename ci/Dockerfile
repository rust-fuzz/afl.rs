FROM ubuntu:16.04

RUN apt-get update && apt-get install -y \
  clang \
  curl \
  file \
  g++ \
  g++-multilib \
  g++-4.9 \
  g++-4.9-multilib \
  libstdc++-4.8-dev \
  llvm-3.8

RUN curl -sSf https://static.rust-lang.org/rustup.sh | sh -s -- --channel=nightly --disable-sudo

WORKDIR /opt/afl.rs

ADD . .
