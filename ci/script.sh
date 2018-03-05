#!/bin/bash

set -e

# Build in debug
cargo build --target "$TARGET"

# Test
cargo test --target "$TARGET"
