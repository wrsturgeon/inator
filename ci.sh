#!/usr/bin/env sh

set -eux

rustup update
rustup toolchain install nightly
rustup component add miri --toolchain nightly

cd automata
export QUICKCHECK_TESTS=100
. ../ci-local.sh
cd ..

cd macros
export QUICKCHECK_TESTS=1000000
. ../ci-local.sh
cd ..

export QUICKCHECK_TESTS=1000000
. ./ci-local.sh
