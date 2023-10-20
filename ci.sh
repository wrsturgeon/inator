#!/usr/bin/env sh

set -eux

if [ "${GITHUB_REF##*/}" = "main" ]
then
  export QUICKCHECK_TESTS=1000000
else
  export QUICKCHECK_TESTS=1000
fi

if [ -d automata ]
then
  cd automata
  ../ci.sh
  cd ..
  exit 0 # <-- TODO: remove
fi

# Update our workbench
rustup update || :
rustup toolchain install nightly || :
rustup component add miri --toolchain nightly

# Housekeeping
cargo fmt --check
cargo clippy --all-targets --no-default-features
cargo clippy --all-targets --all-features

# Non-property tests
cargo install cargo-careful || :
cargo +nightly careful test --no-default-features \
|| cargo +nightly miri test --no-default-features
cargo +nightly careful test --no-default-features --examples \
|| cargo +nightly miri test --no-default-features --examples
cargo +nightly careful test -r --no-default-features \
|| cargo +nightly miri test -r --no-default-features
cargo +nightly careful test -r --no-default-features --examples \
|| cargo +nightly miri test -r --no-default-features --examples

# Property tests
cargo test -r --all-features
cargo test -r --all-features --examples

# Extremely slow (but lovely) UB checks
cargo +nightly miri test --no-default-features
cargo +nightly miri test --no-default-features --examples
cargo +nightly miri test -r --no-default-features
cargo +nightly miri test -r --no-default-features --examples

# Run examples
set +e
export EXAMPLES=$(cargo run --example 2>&1 | grep '^ ')
set -e
if [ ! -z "$EXAMPLES" ]
then
  echo $EXAMPLES | xargs -n 1 cargo +nightly miri run --example
  echo $EXAMPLES | xargs -n 1 cargo +nightly careful run --all-features --example
fi
if [ -f run-examples.sh ]
then
  ./run-examples.sh
fi

# Nix build status
git add -A
nix build

# Check for remaining `FIXME`s
grep -Rnw . --exclude-dir=target --exclude-dir=.git --exclude=ci.sh -e FIXME && exit 1 || : # next line checks result

# Print remaining `TODO`s
grep -Rnw . --exclude-dir=target --exclude-dir=.git --exclude=ci.sh -e TODO || :
