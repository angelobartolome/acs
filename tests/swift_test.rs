//! The typed sketch API from Swift: builds the library with UniFFI, generates
//! its Swift (`uniffi-bindgen`, configured by `uniffi.toml`) and compiles and
//! runs `tests/swift/main.swift` against them, the generated header and the C
//! ABI's in one `ACS` module, as `ACS.xcframework` exports them. macOS only.
//! Run with `cargo test --features uniffi --test swift_test`.
#![cfg(all(feature = "uniffi", target_os = "macos"))]

use std::path::Path;
use std::process::Command;

fn run(command: &mut Command, what: &str) {
    let status = command.status().unwrap_or_else(|e| panic!("{what}: {e}"));
    assert!(status.success(), "{what} failed");
}

#[test]
fn swift_solves_through_the_generated_bindings() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tmp = Path::new(env!("CARGO_TARGET_TMPDIR"));
    let target_dir = tmp.join("swift-target");
    let generated = tmp.join("swift-generated");
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());

    // The library with both the C ABI and UniFFI, as the release builds it.
    run(
        Command::new(&cargo)
            .current_dir(root)
            .args(["build", "--lib", "--features", "c-abi,uniffi", "--target-dir"])
            .arg(&target_dir),
        "building the library",
    );
    let _ = std::fs::remove_dir_all(&generated);
    run(
        Command::new(&cargo)
            .current_dir(root)
            .args(["run", "--features", "uniffi-cli", "--bin", "uniffi-bindgen", "--target-dir"])
            .arg(&target_dir)
            .args(["--", "generate", "--language", "swift", "--library"])
            .arg(target_dir.join("debug/libacs.dylib"))
            .arg("--out-dir")
            .arg(&generated),
        "generating the Swift bindings",
    );
    // The module ACS: UniFFI's header and the C ABI's.
    std::fs::copy(root.join("include/p3d_sketch_solver.h"), generated.join("p3d_sketch_solver.h")).unwrap();
    std::fs::write(
        generated.join("module.modulemap"),
        "module ACS {\n    header \"acs_uniffi.h\"\n    header \"p3d_sketch_solver.h\"\n    export *\n}\n",
    )
    .unwrap();

    let exe = tmp.join("swift_smoke_test");
    run(
        Command::new("swiftc")
            .arg("-module-name")
            .arg("SwiftSmokeTest")
            .arg(generated.join("ACSSketch.swift"))
            .arg(root.join("tests/swift/main.swift"))
            .arg("-I")
            .arg(&generated)
            .arg(target_dir.join("debug/libacs.a"))
            .arg("-o")
            .arg(&exe),
        "compiling the Swift smoke test",
    );
    let out = Command::new(&exe).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "the Swift smoke test failed: {stdout}");
    assert_eq!(stdout.trim(), "ok");
}
