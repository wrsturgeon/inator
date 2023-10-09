#!/usr/bin/env sh

set -eux

export RUST_BACKTRACE=1
export MIRIFLAGS=-Zmiri-backtrace=1

# Update our workbench
rustup update
rustup toolchain install nightly
rustup component add miri --toolchain nightly
cargo install cargo-careful

# Housekeeping
cargo fmt --check
cargo clippy --all-targets --no-default-features
cargo clippy --all-targets --all-features

# Non-property tests
cargo +nightly careful test --no-default-features
cargo +nightly careful test --no-default-features --examples

# Property tests
cargo test -r --all-features
cargo test -r --all-features --examples

# Extremely slow (but lovely) UB checks
cargo +nightly careful test -r --no-default-features
cargo +nightly careful test -r --no-default-features --examples
cargo +nightly miri test --no-default-features
cargo +nightly miri test --no-default-features --examples
cargo +nightly miri test -r --no-default-features
cargo +nightly miri test -r --no-default-features --examples

# Run examples
./run-examples.sh

# Check for remaining `FIXME`s
grep -Rnw . -e FIXME # next line checks result
if [ $? -eq 0 ]
then
  exit 1
fi

# Print remaining `TODO`s
grep -Rnw . -e TODO
