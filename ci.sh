#!/usr/bin/env sh

set -eux

export MIRIFLAGS=-Zmiri-backtrace=1

# Update our workbench
rustup update || :
rustup toolchain install nightly || :
rustup component add miri --toolchain nightly

# Housekeeping
cargo fmt --check
cargo clippy --all-targets --no-default-features
cargo clippy --all-targets --all-features

# Non-property tests
cargo install cargo-careful
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
set +e
export EXAMPLES=$(cargo run --example 2>&1 | grep '^ ')
set -e
if [ ! -z "$EXAMPLES" ]
then
  echo $EXAMPLES | xargs -n 1 cargo +nightly miri run --example
fi
if [ -f run-examples.sh ]
then
  ./run-examples.sh
fi

# Check for remaining `FIXME`s
grep -Rnw . --exclude-dir=target --exclude-dir=.git --exclude=ci.sh -e FIXME && exit 1 || : # next line checks result

# Print remaining `TODO`s
grep -Rnw . --exclude-dir=target --exclude-dir=.git --exclude=ci.sh -e TODO || :
