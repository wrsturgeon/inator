#!/usr/bin/env sh

set -ex # `-u` set below

if [ -z "${QUICKCHECK_TESTS}" ]
  then
  if [ "${GITHUB_REF##*/}" = "main" ]
  then
    export QUICKCHECK_TESTS=1000000
  else
    export QUICKCHECK_TESTS=1000
  fi
fi

set -u

# Recurse on the automata library
if [ -d automata ]
then
  cd automata
  ../ci.sh
  cd ..
fi

# Update our workbench
rustup update || :
rustup toolchain install nightly || :
rustup component add miri --toolchain nightly
git submodule update --init --recursive --remote

# Housekeeping
cargo fmt --check
cargo clippy --all-targets --no-default-features
cargo clippy --all-targets --all-features

# Non-property tests
cargo test --no-default-features
cargo test --no-default-features --examples
cargo test -r --no-default-features
cargo test -r --no-default-features --examples

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
fi

# Examples that are crates themselves
for dir in $(ls -A examples)
do
  if [ -d examples/$dir ]
  then
    cd examples/$dir
    cargo +nightly miri run
    cargo test
    cd ../..
  fi
done

# Nix build status
git add -A
nix build

# Check for remaining `FIXME`s
grep -Rnw . --exclude-dir=target --exclude-dir=.git --exclude-dir='*JSONTestSuite*' --exclude=ci.sh -e FIXME && exit 1 || : # next line checks result

# Print remaining `TODO`s
grep -Rnw . --exclude-dir=target --exclude-dir=.git --exclude-dir='*JSONTestSuite*' --exclude=ci.sh -e TODO || :
