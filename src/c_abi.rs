//! The sketch solver C ABI (`include/p3d_sketch_solver.h`), exported from
//! the static library when the `c-abi` feature is on.
//!
//! `P3DSketch_Solve` runs [`solve_planegcs_sketch_json`]: the PlaneGCS dialect
//! unless the request names another `vocabulary` (`"native"`), so existing
//! clients are unaffected. 0 with its `Ok` response, 1
//! with its `Err` (`status: "invalid"`) response. A null or non-UTF-8 request
//! and a panic inside the solver are malformed too; nothing unwinds into C.
//! Responses are allocated here and released by `P3DSketch_Free`.

use std::ffi::{CStr, CString, c_char, c_int};
use std::panic::{self, AssertUnwindSafe};

use serde_json::json;

use crate::sketch_solve::solve_planegcs_sketch_json;

fn invalid(error: &str) -> String {
    json!({ "version": 1, "status": "invalid", "error": error }).to_string()
}

fn panic_message(payload: &(dyn std::any::Any + Send)) -> &str {
    payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("unknown panic")
}

/// Solves the sketch in `request_json`. Returns 0 when the request was
/// understood (whatever the solve status) and non-zero for a malformed one.
/// `*response_json` is always set (unless `response_json` itself is null, which
/// returns non-zero) and must be released with [`P3DSketch_Free`].
///
/// # Safety
///
/// `request_json` must be null or a NUL-terminated string, and `response_json`
/// null or valid for a pointer write.
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C" fn P3DSketch_Solve(
    request_json: *const c_char,
    response_json: *mut *mut c_char,
) -> c_int {
    if response_json.is_null() {
        return 2;
    }
    let solved = panic::catch_unwind(AssertUnwindSafe(|| {
        if request_json.is_null() {
            return Err(invalid("request is null"));
        }
        // SAFETY: the caller passes a NUL-terminated string.
        let request = unsafe { CStr::from_ptr(request_json) }
            .to_str()
            .map_err(|_| invalid("request is not valid UTF-8"))?;
        solve_planegcs_sketch_json(request)
    }));
    let (code, response) = match solved {
        Ok(Ok(response)) => (0, response),
        Ok(Err(response)) => (1, response),
        Err(payload) => (
            1,
            invalid(&format!("internal error: {}", panic_message(&*payload))),
        ),
    };
    // serde_json escapes NUL inside strings, so a response never contains one.
    let response = CString::new(response).unwrap_or_else(|_| {
        c"{\"version\":1,\"status\":\"invalid\",\"error\":\"response contained NUL\"}".to_owned()
    });
    // SAFETY: checked non-null above; the caller makes it valid for writes.
    unsafe { *response_json = response.into_raw() };
    code
}

/// Releases a string returned by [`P3DSketch_Solve`]. Null is a no-op.
///
/// # Safety
///
/// `p` must be null or a pointer `P3DSketch_Solve` returned and that hasn't
/// been released yet.
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C" fn P3DSketch_Free(p: *mut c_char) {
    if !p.is_null() {
        // SAFETY: `p` came from `CString::into_raw` in `P3DSketch_Solve`.
        drop(unsafe { CString::from_raw(p) });
    }
}
