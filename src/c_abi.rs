//! The sketch solver C ABI (`include/p3d_sketch_solver.h`), exported from
//! the static library when the `c-abi` feature is on.
//!
//! `P3DSketch_Solve` runs [`solve_sketch_json`]: 0 with its `Ok` response, 1
//! with its `Err` (`status: "invalid"`) response. `P3DSpline_Curve` runs
//! [`spline_curve_json`] the same way (`Err`: `{version, error}`). A null or
//! non-UTF-8 request and a panic inside the solver are malformed too; nothing
//! unwinds into C. Responses are allocated here and released by
//! `P3DSketch_Free`.

use std::ffi::{CStr, CString, c_char, c_int};
use std::panic::{self, AssertUnwindSafe};

use serde_json::json;

use crate::sketch_solve::{curve_error, solve_sketch_json, spline_curve_json};

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

/// Runs `answer` on the request C passed and hands its response back: 0 with
/// `Ok`, 1 with `Err`, and 1 with `reject`'s response for a null or
/// non-UTF-8 request or a panic; 2 (nothing written) when `response_json`
/// is null.
///
/// # Safety
///
/// As the entry points: `request_json` null or a NUL-terminated string,
/// `response_json` null or valid for a pointer write.
unsafe fn respond(
    request_json: *const c_char,
    response_json: *mut *mut c_char,
    answer: fn(&str) -> Result<String, String>,
    reject: fn(&str) -> String,
) -> c_int {
    if response_json.is_null() {
        return 2;
    }
    let answered = panic::catch_unwind(AssertUnwindSafe(|| {
        if request_json.is_null() {
            return Err(reject("request is null"));
        }
        // SAFETY: the caller passes a NUL-terminated string.
        let request = unsafe { CStr::from_ptr(request_json) }
            .to_str()
            .map_err(|_| reject("request is not valid UTF-8"))?;
        answer(request)
    }));
    let (code, response) = match answered {
        Ok(Ok(response)) => (0, response),
        Ok(Err(response)) => (1, response),
        Err(payload) => (
            1,
            reject(&format!("internal error: {}", panic_message(&*payload))),
        ),
    };
    // serde_json escapes NUL inside strings, so a response never contains one.
    let response = CString::new(response)
        .unwrap_or_else(|_| CString::new(reject("response contained NUL")).unwrap_or_default());
    // SAFETY: checked non-null above; the caller makes it valid for writes.
    unsafe { *response_json = response.into_raw() };
    code
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
    // SAFETY: the caller's guarantees, passed on.
    unsafe { respond(request_json, response_json, solve_sketch_json, invalid) }
}

/// A Spline's curve from the handles in `request_json`, without solving
/// ([`spline_curve_json`]). Returns 0 with `{version, curve}`, non-zero with
/// `{version, error}` for a malformed request or handles and knots that make
/// no curve. `*response_json` is always set (unless `response_json` itself
/// is null, which returns non-zero) and must be released with
/// [`P3DSketch_Free`].
///
/// # Safety
///
/// `request_json` must be null or a NUL-terminated string, and `response_json`
/// null or valid for a pointer write.
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C" fn P3DSpline_Curve(
    request_json: *const c_char,
    response_json: *mut *mut c_char,
) -> c_int {
    // SAFETY: the caller's guarantees, passed on.
    unsafe { respond(request_json, response_json, spline_curve_json, curve_error) }
}

/// Releases a string returned by [`P3DSketch_Solve`] or [`P3DSpline_Curve`].
/// Null is a no-op.
///
/// # Safety
///
/// `p` must be null or a pointer `P3DSketch_Solve` or `P3DSpline_Curve`
/// returned and that hasn't been released yet.
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C" fn P3DSketch_Free(p: *mut c_char) {
    if !p.is_null() {
        // SAFETY: `p` came from `CString::into_raw` in `respond`.
        drop(unsafe { CString::from_raw(p) });
    }
}
