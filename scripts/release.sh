#!/usr/bin/env bash
# Builds the release artifacts: the static library behind the sketch solver C ABI
# (include/p3d_sketch_solver.h, `--features c-abi`) per Apple platform, plus the
# WASM npm package. Everything lands in dist/:
#
#   acs-<version>-macos-arm64.tar.gz          aarch64-apple-darwin
#   acs-<version>-ios-arm64.tar.gz            aarch64-apple-ios
#   acs-<version>-ios-arm64-simulator.tar.gz  aarch64-apple-ios-sim
#   acs-<version>-xcframework.zip             ACS.xcframework: the same three as frameworks
#   acs-<version>.tgz                         wasm-pack --target web, npm pack (package acs-solver)
#
# Each archive unpacks to acs/{include/p3d_sketch_solver.h, lib/libp3d_sketch_solver.a,
# VERSION}. <version> is the crate version from Cargo.toml, so every file name carries
# it and several versions can sit side by side in one release.
#
# ACS.xcframework holds a dynamic ACS.framework per Apple platform (the module `ACS`, with the
# same header). An app links it in place of the static library to keep Rust's exception
# personality routine in an image of its own: beside Swift's, C++'s and Objective-C's it is a
# fourth, one more than compact unwind can encode in one image.
#
# Usage: scripts/release.sh [macos-arm64 | ios-arm64 | ios-arm64-simulator | xcframework | wasm ...]
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
[ ${#PIECES[@]} -gt 0 ] || PIECES=(macos-arm64 ios-arm64 ios-arm64-simulator xcframework wasm)
PLATFORMS=()
BUILD_WASM=false
BUILD_XCFRAMEWORK=false
for piece in "${PIECES[@]}"; do
  if [ "$piece" = wasm ]; then
    BUILD_WASM=true
  elif [ "$piece" = xcframework ]; then
    BUILD_XCFRAMEWORK=true
  elif target_of "$piece" > /dev/null; then
    PLATFORMS+=("$piece")
  else
    echo "Unknown piece '$piece'; expected macos-arm64, ios-arm64, ios-arm64-simulator, xcframework or wasm" >&2
    exit 2
  fi
done

# The XCFramework is built from every Apple target.
TARGET_PLATFORMS=(${PLATFORMS[@]+"${PLATFORMS[@]}"})
if $BUILD_XCFRAMEWORK; then
  for platform in macos-arm64 ios-arm64 ios-arm64-simulator; do
    [[ " ${TARGET_PLATFORMS[*]-} " == *" $platform "* ]] || TARGET_PLATFORMS+=("$platform")
  done
fi

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

if [ ${#TARGET_PLATFORMS[@]} -gt 0 ]; then
  # One cargo invocation builds every requested Apple target, scheduling all of them across the
  # cores and sharing the host artifacts (build scripts, proc macros).
  target_args=()
  for platform in "${TARGET_PLATFORMS[@]}"; do
    target="$(target_of "$platform")"
    rustup target list --installed | grep -qx "$target" || rustup target add "$target"
    target_args+=(--target "$target")
  done
  if $BUILD_XCFRAMEWORK; then
    # The framework binaries' install names, set when the dynamic library links: rewriting them
    # afterwards with install_name_tool leaves a LINKEDIT that ld refuses to link against.
    export CARGO_TARGET_AARCH64_APPLE_DARWIN_RUSTFLAGS="-C link-arg=-Wl,-install_name,@rpath/ACS.framework/Versions/A/ACS"
    export CARGO_TARGET_AARCH64_APPLE_IOS_RUSTFLAGS="-C link-arg=-Wl,-install_name,@rpath/ACS.framework/ACS"
    export CARGO_TARGET_AARCH64_APPLE_IOS_SIM_RUSTFLAGS="-C link-arg=-Wl,-install_name,@rpath/ACS.framework/ACS"
    # Cargo strips release dylibs' debug info, leaving a string pool ld refuses to link against
    # ("mis-aligned LINKEDIT string pool"); unstripped they are some 50 KB larger.
    export CARGO_PROFILE_RELEASE_STRIP=false
  fi
  echo "Building the libraries for ${TARGET_PLATFORMS[*]}..."
  cargo build --release --lib --features c-abi "${target_args[@]}"
fi

if [ ${#PLATFORMS[@]} -gt 0 ]; then

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

# Writes ACS.framework for <platform> into <dir> from cargo's dynamic library: flat on iOS,
# versioned (Versions/A, with symlinks) on macOS, as each platform expects.
make_framework() {
  local platform="$1" dir="$2"
  local target framework contents plist plist_platform min_os_key min_os
  target="$(target_of "$platform")"
  framework="$dir/ACS.framework"
  if [ "$platform" = macos-arm64 ]; then
    contents="$framework/Versions/A"
    mkdir -p "$contents/Resources"
    plist="$contents/Resources/Info.plist"
    plist_platform=MacOSX
    min_os_key=LSMinimumSystemVersion
    min_os="$MACOSX_DEPLOYMENT_TARGET"
  else
    contents="$framework"
    mkdir -p "$contents"
    plist="$contents/Info.plist"
    [ "$platform" = ios-arm64 ] && plist_platform=iPhoneOS || plist_platform=iPhoneSimulator
    min_os_key=MinimumOSVersion
    min_os="$IPHONEOS_DEPLOYMENT_TARGET"
  fi
  mkdir -p "$contents/Headers" "$contents/Modules"
  cp include/p3d_sketch_solver.h "$contents/Headers/"
  cat > "$contents/Modules/module.modulemap" <<MODULEMAP
framework module ACS {
    header "p3d_sketch_solver.h"
    export *
}
MODULEMAP
  cp "target/$target/release/libacs.dylib" "$contents/ACS"
  cat > "$plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleDevelopmentRegion</key>
	<string>en</string>
	<key>CFBundleExecutable</key>
	<string>ACS</string>
	<key>CFBundleIdentifier</key>
	<string>io.github.angelobartolome.acs</string>
	<key>CFBundleInfoDictionaryVersion</key>
	<string>6.0</string>
	<key>CFBundleName</key>
	<string>ACS</string>
	<key>CFBundlePackageType</key>
	<string>FMWK</string>
	<key>CFBundleShortVersionString</key>
	<string>$VERSION</string>
	<key>CFBundleVersion</key>
	<string>$VERSION</string>
	<key>CFBundleSupportedPlatforms</key>
	<array>
		<string>$plist_platform</string>
	</array>
	<key>$min_os_key</key>
	<string>$min_os</string>
</dict>
</plist>
PLIST
  if [ "$platform" = macos-arm64 ]; then
    ln -s A "$framework/Versions/Current"
    for entry in ACS Headers Modules Resources; do
      ln -s "Versions/Current/$entry" "$framework/$entry"
    done
  fi
}

if $BUILD_XCFRAMEWORK; then
  staging="$DIST/_release-xcframework"
  rm -rf "$staging"
  framework_args=()
  for platform in macos-arm64 ios-arm64 ios-arm64-simulator; do
    make_framework "$platform" "$staging/$platform"
    framework_args+=(-framework "$staging/$platform/ACS.framework")
  done
  xcodebuild -create-xcframework "${framework_args[@]}" -output "$staging/ACS.xcframework" > /dev/null
  archive="acs-$VERSION-xcframework.zip"
  rm -f "$DIST/$archive"
  # ditto keeps the macOS framework's symlinks, which SwiftPM's binary targets need.
  (cd "$staging" && ditto -c -k --keepParent ACS.xcframework "$DIST/$archive")
  rm -rf "$staging"
  echo "Packaged dist/$archive"
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
  ls -lh "$DIST"/acs-"$VERSION"-*.tar.gz "$DIST"/acs-"$VERSION"-xcframework.zip "$DIST"/acs-"$VERSION".tgz
  echo "Upload them to a release (the version is in every file name), e.g.:"
  echo "  gh release upload <tag> -R <owner>/<repo> dist/acs-$VERSION-*.tar.gz dist/acs-$VERSION-xcframework.zip dist/acs-$VERSION.tgz"
fi
