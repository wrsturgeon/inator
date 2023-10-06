#!/usr/bin/env sh

set -eux

export MIRIFLAGS=-Zmiri-backtrace=1

rustup update
rustup toolchain install nightly
rustup component add miri --toolchain nightly

cargo fmt --check
cargo clippy --all-targets --no-default-features
cargo clippy --all-targets --all-features

cargo test -r --all-features
cargo test -r --all-features --examples

cargo +nightly miri test --no-default-features
cargo +nightly miri test --no-default-features --examples
cargo +nightly miri test -r --no-default-features
cargo +nightly miri test -r --no-default-features --examples

./run-examples.sh
