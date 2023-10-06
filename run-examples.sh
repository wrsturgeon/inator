#!/usr/bin/env sh

set -eux

for dir in $(ls -A examples)
do
  cd examples/$dir
  cargo build
  cargo fmt
  ../../ci.sh
  cargo +nightly miri run
  cd ../..
done
