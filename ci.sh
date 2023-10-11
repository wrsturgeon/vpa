#!/usr/bin/env sh

set -eux

# Update our workbench
rustup update
rustup toolchain install nightly
rustup component add miri --toolchain nightly
cargo install cargo-careful

# Call the rest of CI
./nix-ci.sh
