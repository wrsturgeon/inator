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

if [ -d examples ]
then
  ls examples | xargs -n 1 ./ci-example.sh || exit 1
fi

cargo +nightly miri test --no-default-features
cargo +nightly miri test --no-default-features --examples
cargo +nightly miri test -r --no-default-features
cargo +nightly miri test -r --no-default-features --examples
cargo test -r --all-features
cargo test -r --all-features --examples
