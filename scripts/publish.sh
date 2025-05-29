#! /bin/bash

# set -x
set -euo pipefail

for X in afl cargo-afl; do
    pushd "$X"
    cargo publish "$@"
    popd
done
