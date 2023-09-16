#!/usr/bin/env sh

set -eux

rustup update
rustup toolchain install nightly
rustup component add miri --toolchain nightly

cargo fmt --check
cargo clippy --all-targets --no-default-features
cargo clippy --all-targets --all-features

export MIRIFLAGS=-Zmiri-backtrace=1
export RUST_BACKTRACE=1
cargo run --example 2>&1 | grep '^ ' | xargs -n 1 cargo +nightly miri run --no-default-features --example
cargo +nightly miri test --no-default-features --examples
cargo test --examples --no-default-features
cargo test --examples --all-features
