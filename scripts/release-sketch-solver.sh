#!/usr/bin/env bash
# Builds the release artifacts: the static library behind the sketch solver C ABI
# (include/p3d_sketch_solver.h, `--features c-abi`) per Apple platform, plus the
# WASM npm package for the Frontend. Everything lands in dist/:
#
#   sketch-solver-<version>-macos-arm64.tar.gz          aarch64-apple-darwin
#   sketch-solver-<version>-ios-arm64.tar.gz            aarch64-apple-ios
#   sketch-solver-<version>-ios-arm64-simulator.tar.gz  aarch64-apple-ios-sim
#   acs-<crate version>.tgz                              wasm-pack --target web, npm pack
#
# Each sketch-solver archive unpacks to sketch-solver/{include/p3d_sketch_solver.h,
# lib/libp3d_sketch_solver.a, VERSION}: the layout a consumer's setup script
# extracts into Vendor/SketchSolver (the same names the GCS build used).
#
# <version> is SKETCH_SOLVER_VERSION, or the crate version from Cargo.toml.
# Needs rustup (targets are added if missing), Xcode for the iOS SDKs, wasm-pack and npm.
set -euo pipefail

cd "$(dirname "$0")/.."
ROOT="$PWD"
CRATE_VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
VERSION="${SKETCH_SOLVER_VERSION:-$CRATE_VERSION}"
DIST="$ROOT/dist"
# Deployment targets match the GCS build's.
export MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-14.0}"
export IPHONEOS_DEPLOYMENT_TARGET="${IPHONEOS_DEPLOYMENT_TARGET:-18.0}"

mkdir -p "$DIST"

build() {
  local target="$1" platform="$2"
  echo "Building the sketch solver for $platform ($target)..."
  rustup target list --installed | grep -qx "$target" || rustup target add "$target"
  cargo build --release --lib --features c-abi --target "$target"

  local staging="$DIST/_release-$platform"
  local stage="$staging/sketch-solver"
  rm -rf "$staging"
  mkdir -p "$stage/include" "$stage/lib"
  cp include/p3d_sketch_solver.h "$stage/include/"
  cp "target/$target/release/libacs.a" "$stage/lib/libp3d_sketch_solver.a"
  echo "$VERSION" > "$stage/VERSION"
  local archive="sketch-solver-$VERSION-$platform.tar.gz"
  tar -czf "$DIST/$archive" -C "$staging" sketch-solver
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
ls -lh "$DIST"/sketch-solver-"$VERSION"-*.tar.gz "$DIST"/acs-"$CRATE_VERSION".tgz
echo "Publish them as one release, e.g.:"
echo "  gh release create acs-$VERSION -R <owner>/<repo> --title \"ACS $VERSION\" \\"
echo "    dist/sketch-solver-$VERSION-*.tar.gz dist/acs-$CRATE_VERSION.tgz"
