#!/bin/bash

set -e

host() {
  case "$TRAVIS_OS_NAME" in
    linux)
      echo x86_64-unknown-linux-gnu
      ;;
    osx)
      echo x86_64-apple-darwin
      ;;
  esac
}

# install rustup
curl https://sh.rustup.rs -sSf | \
  sh -s -- -y --default-toolchain="$TRAVIS_RUST_VERSION"
rustc -V
cargo -V

# install target
if [ $(host) != "$TARGET" ]; then
  rustup target add $TARGET
fi
