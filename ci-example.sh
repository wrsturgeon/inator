#!/usr/bin/env sh

set -eux

cd examples/$1
cargo build
cargo fmt
../../ci.sh
cargo +nightly miri run
cd ../.. 