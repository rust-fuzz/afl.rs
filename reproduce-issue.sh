#!/bin/sh
docker build -t corey/afl.rs .
docker build -t afl.rs -f ci/Dockerfile   .
docker run -t afl.rs bash -c "cd afl/crate-tests/jpeg-decoder && cargo build"
