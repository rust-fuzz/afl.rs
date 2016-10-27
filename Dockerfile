FROM ubuntu:16.04

RUN apt-get update && apt-get install -y \
  curl \
  file \
  g++ \
  g++-multilib \
  g++-4.9 \
  g++-4.9-multilib \
  libstdc++-4.8-dev \
  llvm-3.8 \
  make

RUN curl -sSf https://static.rust-lang.org/rustup.sh | sh -s -- --channel=nightly --date=2016-07-30 --disable-sudo

RUN curl http://lcamtuf.coredump.cx/afl/releases/afl-2.34b.tgz > afl-2.34b.tgz && \
  tar xf afl-2.34b.tgz && \
  cd afl-2.34b && \
  make && \
  make install && \
  cd .. \
  rm -rf afl-2.34.tgz

ENV CXX /usr/bin/g++-4.9

WORKDIR /source
