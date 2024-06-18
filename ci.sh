#!/bin/bash

set -e

# Build
cargo build

# Check formatting
cargo fmt -- --check || (printf "ERROR! cargo fmt failed! Consider running \"cargo fmt --\" to automatically format your code.\n\n" && exit 1)

# Run Clippy
cargo clippy -- -D warnings

# Run tests
cargo test

echo "CI checks passed successfully."
