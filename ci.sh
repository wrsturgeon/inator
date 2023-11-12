#!/usr/bin/env sh

set -ex # `-u` set below

if [ -z "${QUICKCHECK_TESTS}" ]
  then
  if [ "${GITHUB_REF##*/}" = "main" ]
  then
    export QUICKCHECK_TESTS=1000000
  else
    export QUICKCHECK_TESTS=100
  fi
fi

set -u

# Update our workbench
rustup self update || :
rustup update || :
rustup toolchain install nightly || :
rustup component add clippy miri rustfmt
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
for i in $(seq 2 8)
do
  export QUICKCHECK_GENERATOR_SIZE=$(expr ${i} '*' '(' ${i} - 1 ')')
  QUICKCHECK_TESTS=$(expr ${QUICKCHECK_TESTS} / 50) \
  cargo test --all-features
  QUICKCHECK_TESTS=$(expr ${QUICKCHECK_TESTS} / 10) \
  cargo test -r --all-features && \
  cargo test -r --all-features --examples
done

# Run examples
set +e
export EXAMPLES=$(cargo run --example 2>&1 | grep '^ ')
set -e
if [ ! -z "$EXAMPLES" ]
then
  echo $EXAMPLES | xargs -n 1 cargo miri run --example
fi

# Examples that are crates themselves
for dir in $(ls -A examples)
do
  if [ -d examples/$dir ]
  then
    cd examples/$dir
    cargo miri run
    cargo miri test
    cargo fmt --check
    cd ../..
  fi
done

# Extremely slow (but lovely) UB checks
cargo miri test --no-default-features
cargo miri test --no-default-features --examples
cargo miri test -r --no-default-features
cargo miri test -r --no-default-features --examples

# Recurse on the automata library
if [ -d automata ]
then
  cd automata
  ../ci.sh
  cd ..
fi

# Nix build status
git add -A
nix build

# Check for remaining `FIXME`s
grep -Rnw . --exclude-dir=target --exclude-dir=.git --exclude-dir='*JSONTestSuite*' --exclude=ci.sh -e FIXME && exit 1 || : # next line checks result

# Print remaining `TODO`s
grep -Rnw . --exclude-dir=target --exclude-dir=.git --exclude-dir='*JSONTestSuite*' --exclude=ci.sh -e TODO || :
