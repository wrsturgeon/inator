#!/usr/bin/env sh

set -eux

rustup update
rustup toolchain install nightly
rustup component add miri --toolchain nightly

pushd automata
export QUICKCHECK_TESTS=1000
. ../ci-local.sh
popd

pushd macros
export QUICKCHECK_TESTS=1000000
. ../ci-local.sh
popd

export QUICKCHECK_TESTS=1000000
. ./ci-local.sh
