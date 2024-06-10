#! /bin/bash

# set -x
set -euo pipefail

if [[ $# -ne 0 ]]; then
    echo "$0: expect no arguments" >&2
    exit 1
fi

for X in afl cargo-afl; do
    pushd "$X"
    cargo publish
    popd
done
