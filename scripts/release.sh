#!/usr/bin/env bash
# Builds the release artifacts: the static library behind the sketch solver C ABI
# (include/p3d_sketch_solver.h, `--features c-abi`) per Apple platform, plus the
# WASM npm package. Everything lands in dist/:
#
#   acs-<version>-macos-arm64.tar.gz          aarch64-apple-darwin
#   acs-<version>-ios-arm64.tar.gz            aarch64-apple-ios
#   acs-<version>-ios-arm64-simulator.tar.gz  aarch64-apple-ios-sim
#   acs-<version>.tgz                         wasm-pack --target web, npm pack
#
# Each archive unpacks to acs/{include/p3d_sketch_solver.h, lib/libp3d_sketch_solver.a,
# VERSION}. <version> is the crate version from Cargo.toml, so every file name carries
# it and several versions can sit side by side in one release.
# Needs rustup (targets are added if missing), Xcode for the iOS SDKs, wasm-pack and npm.
set -euo pipefail

cd "$(dirname "$0")/.."
ROOT="$PWD"
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
DIST="$ROOT/dist"
export MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-14.0}"
export IPHONEOS_DEPLOYMENT_TARGET="${IPHONEOS_DEPLOYMENT_TARGET:-18.0}"

mkdir -p "$DIST"

build() {
  local target="$1" platform="$2"
  echo "Building the static library for $platform ($target)..."
  rustup target list --installed | grep -qx "$target" || rustup target add "$target"
  cargo build --release --lib --features c-abi --target "$target"

  local staging="$DIST/_release-$platform"
  local stage="$staging/acs"
  rm -rf "$staging"
  mkdir -p "$stage/include" "$stage/lib"
  cp include/p3d_sketch_solver.h "$stage/include/"
  cp "target/$target/release/libacs.a" "$stage/lib/libp3d_sketch_solver.a"
  echo "$VERSION" > "$stage/VERSION"
  local archive="acs-$VERSION-$platform.tar.gz"
  tar -czf "$DIST/$archive" -C "$staging" acs
  rm -rf "$staging"
  echo "Packaged dist/$archive"
}

build aarch64-apple-darwin macos-arm64
build aarch64-apple-ios ios-arm64
build aarch64-apple-ios-sim ios-arm64-simulator

echo "Building the WASM npm package..."
wasm-pack build --release --target web --out-dir pkg
npm pack ./pkg --pack-destination "$DIST"

echo "Done:"
ls -lh "$DIST"/acs-"$VERSION"-*.tar.gz "$DIST"/acs-"$VERSION".tgz
echo "Upload them to a release (the version is in every file name), e.g.:"
echo "  gh release upload <tag> -R <owner>/<repo> dist/acs-$VERSION-*.tar.gz dist/acs-$VERSION.tgz"
