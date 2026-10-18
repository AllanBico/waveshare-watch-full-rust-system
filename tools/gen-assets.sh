#!/usr/bin/env sh
# Regenerate compiled-in UI assets. Pass --check to verify without writing.
set -eu

exec cargo +stable run \
    --manifest-path tools/assetgen/Cargo.toml \
    --target x86_64-unknown-linux-gnu \
    --quiet -- "$@"
