#!/usr/bin/env sh
# Run unit tests on the host, where std and a test harness are available.
#
# Uses the stable toolchain on purpose: stable ignores `build-std`, while the
# esp toolchain would rebuild `core` and clash with the host's prebuilt std.
# `firmware` is a bare-metal binary and cannot be host-tested.
set -eu

exec cargo +stable test --workspace --exclude firmware --target x86_64-unknown-linux-gnu "$@"
