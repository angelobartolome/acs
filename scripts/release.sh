#!/usr/bin/env bash
# Builds the release artifacts: the static library behind the sketch solver C ABI
# (include/p3d_sketch_solver.h, `--features c-abi`) per Apple platform, plus the
# WASM npm package. Everything lands in dist/:
#
#   acs-<version>-macos-arm64.tar.gz          aarch64-apple-darwin
#   acs-<version>-ios-arm64.tar.gz            aarch64-apple-ios
#   acs-<version>-ios-arm64-simulator.tar.gz  aarch64-apple-ios-sim
#   acs-<version>.tgz                         wasm-pack --target web, npm pack (package acs-solver)
#
# Each archive unpacks to acs/{include/p3d_sketch_solver.h, lib/libp3d_sketch_solver.a,
# VERSION}. <version> is the crate version from Cargo.toml, so every file name carries
# it and several versions can sit side by side in one release.
#
# Usage: scripts/release.sh [macos-arm64 | ios-arm64 | ios-arm64-simulator | wasm ...]
# With no argument it builds everything, in parallel: one cargo invocation for every Apple
# target, and the WASM package alongside it. CI builds one piece per job.
# Needs rustup (targets are added if missing), Xcode for the iOS SDKs (Apple pieces), and
# wasm-pack and npm (wasm).
set -euo pipefail

cd "$(dirname "$0")/.."
ROOT="$PWD"
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
DIST="$ROOT/dist"
export MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-14.0}"
export IPHONEOS_DEPLOYMENT_TARGET="${IPHONEOS_DEPLOYMENT_TARGET:-18.0}"

target_of() {
  case "$1" in
    macos-arm64) echo aarch64-apple-darwin ;;
    ios-arm64) echo aarch64-apple-ios ;;
    ios-arm64-simulator) echo aarch64-apple-ios-sim ;;
    *) return 1 ;;
  esac
}

PIECES=("$@")
[ ${#PIECES[@]} -gt 0 ] || PIECES=(macos-arm64 ios-arm64 ios-arm64-simulator wasm)
PLATFORMS=()
BUILD_WASM=false
for piece in "${PIECES[@]}"; do
  if [ "$piece" = wasm ]; then
    BUILD_WASM=true
  elif target_of "$piece" > /dev/null; then
    PLATFORMS+=("$piece")
  else
    echo "Unknown piece '$piece'; expected macos-arm64, ios-arm64, ios-arm64-simulator or wasm" >&2
    exit 2
  fi
done

mkdir -p "$DIST"

# The WASM package builds in the background, in its own target directory: cargo locks a
# target directory, so sharing one would serialize the two builds.
WASM_PID=""
if $BUILD_WASM; then
  echo "Building the WASM npm package (in the background)..."
  CARGO_TARGET_DIR="$ROOT/target/wasm-release" wasm-pack build --release --target web --out-dir pkg \
    > "$DIST/_wasm-pack.log" 2>&1 &
  WASM_PID=$!
fi

if [ ${#PLATFORMS[@]} -gt 0 ]; then
  # One cargo invocation builds every requested Apple target, scheduling all of them across the
  # cores and sharing the host artifacts (build scripts, proc macros).
  target_args=()
  for platform in "${PLATFORMS[@]}"; do
    target="$(target_of "$platform")"
    rustup target list --installed | grep -qx "$target" || rustup target add "$target"
    target_args+=(--target "$target")
  done
  echo "Building the static library for ${PLATFORMS[*]}..."
  cargo build --release --lib --features c-abi "${target_args[@]}"

  for platform in "${PLATFORMS[@]}"; do
    target="$(target_of "$platform")"
    staging="$DIST/_release-$platform"
    stage="$staging/acs"
    rm -rf "$staging"
    mkdir -p "$stage/include" "$stage/lib"
    cp include/p3d_sketch_solver.h "$stage/include/"
    cp "target/$target/release/libacs.a" "$stage/lib/libp3d_sketch_solver.a"
    echo "$VERSION" > "$stage/VERSION"
    archive="acs-$VERSION-$platform.tar.gz"
    tar -czf "$DIST/$archive" -C "$staging" acs
    rm -rf "$staging"
    echo "Packaged dist/$archive"
  done
fi

if [ -n "$WASM_PID" ]; then
  if ! wait "$WASM_PID"; then
    cat "$DIST/_wasm-pack.log" >&2
    echo "wasm-pack failed" >&2
    exit 1
  fi
  rm -f "$DIST/_wasm-pack.log"
  # npm names the tarball after the package (acs-solver); keep the release's acs-<version> names.
  npm pack ./pkg --pack-destination "$DIST"
  mv "$DIST/acs-solver-$VERSION.tgz" "$DIST/acs-$VERSION.tgz"
  echo "Packaged dist/acs-$VERSION.tgz"
fi

echo "Done."
if [ $# -eq 0 ]; then
  ls -lh "$DIST"/acs-"$VERSION"-*.tar.gz "$DIST"/acs-"$VERSION".tgz
  echo "Upload them to a release (the version is in every file name), e.g.:"
  echo "  gh release upload <tag> -R <owner>/<repo> dist/acs-$VERSION-*.tar.gz dist/acs-$VERSION.tgz"
fi
