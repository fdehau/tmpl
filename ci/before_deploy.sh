#!/bin/bash

set -e

make_tarball() {
  local tmpdir="$(mktemp -d)"
  local name="tmpl-$TRAVIS_TAG-$TARGET"
  local staging="$tmpdir/$name"
  local out_dir="$(pwd)/deployment"

  mkdir -p $staging
  mkdir -p $out_dir

  cp "target/$TARGET/release/tmpl" "$staging/tmpl"
  cp {README.md,LICENSE} "$staging/"
  cd "$tmpdir" && tar czf "$out_dir/$name.tar.gz" "$name"
  rm -rf "$tmpdir"
}

# Build release artifacts
cargo build --target "$TARGET" --release
make_tarball
