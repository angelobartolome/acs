//! The sketch solver C ABI (`include/p3d_sketch_solver.h`), called through
//! raw pointers as C does, and from C itself (`tests/c/smoke_test.c`).
//! Run with `cargo test --features c-abi`.
#![cfg(feature = "c-abi")]

use std::ffi::{CStr, CString, c_char};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use acs::c_abi::{P3DSketch_Free, P3DSketch_Solve};
use acs::sketch_solve::solve_sketch_json;
use serde_json::{Value, json};

/// Calls `P3DSketch_Solve` with `request` (null when `None`) and returns its
/// return code and response, released with `P3DSketch_Free`.
fn call_raw(request: Option<&[u8]>) -> (i32, String) {
    let owned = request.map(|bytes| CString::new(bytes).unwrap());
    let ptr = owned.as_ref().map_or(std::ptr::null(), |c| c.as_ptr());
    let mut response: *mut c_char = std::ptr::null_mut();
    let code = unsafe { P3DSketch_Solve(ptr, &mut response) };
    assert!(!response.is_null(), "response must always be set");
    let text = unsafe { CStr::from_ptr(response) }
        .to_str()
        .unwrap()
        .to_string();
    unsafe { P3DSketch_Free(response) };
    (code, text)
}

fn call(request: &str) -> (i32, String) {
    call_raw(Some(request.as_bytes()))
}

fn assert_invalid(response: &str) -> Value {
    let resp: Value = serde_json::from_str(response).expect("response should be JSON");
    assert_eq!(resp["version"], 1);
    assert_eq!(resp["status"], "invalid");
    assert!(resp["error"].is_string());
    resp
}

/// Requests from the JSON API tests: understood ones and malformed ones.
fn understood_requests() -> Vec<String> {
    vec![
        json!({ "version": 1, "primitives": [
            { "id": "p1", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
            { "id": "p2", "type": "point", "x": 3.0, "y": 4.0, "fixed": false },
            { "id": "k1", "type": "distance", "a": "p1", "b": "p2", "value": 10.0 }
        ]})
        .to_string(),
        json!({ "version": 1, "maxIterations": 0, "primitives": [
            { "id": "p1", "type": "point", "x": 0.0, "y": 0.0, "fixed": false },
            { "id": "p2", "type": "point", "x": 5.0, "y": 3.0, "fixed": false },
            { "id": "l1", "type": "line", "p1_id": "p1", "p2_id": "p2" },
            { "id": "h", "type": "horizontal", "line": "l1" }
        ]})
        .to_string(),
        json!({ "version": 1, "primitives": [] }).to_string(),
    ]
}

fn malformed_requests() -> Vec<String> {
    vec![
        "not json".to_string(),
        json!({ "version": 2, "primitives": [] }).to_string(),
        json!({ "version": 1 }).to_string(),
        json!({ "version": 1, "primitives": [
            { "id": "p1", "type": "point", "x": 0.0, "y": 0.0 },
            { "id": "k1", "type": "no_such_constraint", "p1_id": "p1" }
        ]})
        .to_string(),
    ]
}

#[test]
fn understood_request_returns_zero_and_the_json_api_response() {
    for request in understood_requests() {
        let (code, response) = call(&request);
        assert_eq!(code, 0, "{request}");
        assert_eq!(response, solve_sketch_json(&request).unwrap());
    }
}

#[test]
fn malformed_request_returns_non_zero_and_the_json_api_error_response() {
    for request in malformed_requests() {
        let (code, response) = call(&request);
        assert_ne!(code, 0, "{request}");
        assert_invalid(&response);
        assert_eq!(response, solve_sketch_json(&request).unwrap_err());
    }
}

#[test]
fn null_request_is_malformed() {
    let (code, response) = call_raw(None);
    assert_ne!(code, 0);
    assert_invalid(&response);
}

#[test]
fn non_utf8_request_is_malformed() {
    let (code, response) = call_raw(Some(b"{\"version\": 1, \"primitives\": [\xff]}"));
    assert_ne!(code, 0);
    assert_invalid(&response);
}

#[test]
fn null_response_pointer_is_rejected_without_crashing() {
    let request = CString::new(understood_requests()[0].clone()).unwrap();
    let code = unsafe {
        P3DSketch_Solve(request.as_ptr(), std::ptr::null_mut())
    };
    assert_ne!(code, 0);
}

#[test]
fn freeing_null_is_a_no_op() {
    unsafe { P3DSketch_Free(std::ptr::null_mut()) };
}

// ── The C smoke test ───────────────────────────────────────────────────────

/// Builds the static library with the `c-abi` feature (`cargo test` builds
/// only the rlib) into its own target directory, returning `libacs.a`.
fn build_static_lib() -> PathBuf {
    let target_dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("c-abi-target");
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let status = Command::new(cargo)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(["build", "--lib", "--features", "c-abi", "--target-dir"])
        .arg(&target_dir)
        .status()
        .expect("cargo should run");
    assert!(status.success(), "building the static library failed");
    target_dir.join("debug/libacs.a")
}

/// Compiles `tests/c/smoke_test.c` against the shipped header and the static
/// library.
fn build_c_smoke_test() -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lib = build_static_lib();
    let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join("c_abi_smoke_test");

    let mut cc = Command::new(std::env::var("CC").unwrap_or_else(|_| "cc".into()));
    cc.arg(root.join("tests/c/smoke_test.c"))
        .arg("-I")
        .arg(root.join("include"))
        .arg(&lib)
        .arg("-o")
        .arg(&out);
    // The native libraries Rust's std needs (`rustc --print native-static-libs`);
    // on macOS, cc already links libSystem.
    if !cfg!(target_os = "macos") {
        cc.args([
            "-lgcc_s",
            "-lutil",
            "-lrt",
            "-lpthread",
            "-lm",
            "-ldl",
            "-lc",
        ]);
    }
    let status = cc.status().expect("cc should run");
    assert!(status.success(), "compiling the C smoke test failed");
    out
}

/// Runs the C smoke test on `request` (stdin) with `args` and returns its exit
/// code (`P3DSketch_Solve`'s return value) and stdout (the response).
fn run_c(exe: &Path, args: &[&str], request: &str) -> (i32, String) {
    let mut child = Command::new(exe)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(request.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    let code = out.status.code().expect("the C smoke test must not crash");
    (code, String::from_utf8(out.stdout).unwrap())
}

#[test]
fn c_smoke_test_gets_the_json_api_responses() {
    let exe = build_c_smoke_test();
    for request in understood_requests() {
        let (code, response) = run_c(&exe, &[], &request);
        assert_eq!(code, 0, "{request}");
        assert_eq!(response, solve_sketch_json(&request).unwrap());
    }
    for request in malformed_requests() {
        let (code, response) = run_c(&exe, &[], &request);
        assert_ne!(code, 0, "{request}");
        assert_eq!(response, solve_sketch_json(&request).unwrap_err());
    }

    // A null request, from C.
    let out = Command::new(&exe).arg("--null").output().unwrap();
    assert_ne!(out.status.code().expect("must not crash"), 0);
    assert_invalid(&String::from_utf8(out.stdout).unwrap());
}
