#!/usr/bin/env sh

set -eux

export MIRIFLAGS=-Zmiri-backtrace=1

# Housekeeping
cargo fmt --check
cargo clippy --all-targets --no-default-features
cargo clippy --all-targets --all-features

# Non-property tests
cargo +nightly careful test --no-default-features
cargo +nightly careful test --no-default-features --examples

# Nix build status
git add -A
nix build

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
grep -Rnw . --exclude-dir=target -e FIXME # next line checks result
if [ $? -eq 0 ]
then
  exit 1
fi

# Print remaining `TODO`s
grep -Rnw . --exclude-dir=target -e TODO
