#!/usr/bin/env sh

set -eux

for dir in $(ls -A examples)
do
  if [ -d examples/$dir ]
  then
    cd examples/$dir
    cargo +nightly miri run
    ../../ci.sh
    cd ../..
  fi
done
