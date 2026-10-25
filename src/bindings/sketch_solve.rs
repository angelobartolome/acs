//! `acsSolveSketch`, `acsSplineCurve` and `acsConstraintCatalog`: the JSON
//! API ([`crate::sketch_solve::solve_sketch_json`]), a Spline's curve
//! ([`crate::sketch_solve::spline_curve_json`]) and the constraint catalog
//! ([`crate::constraint_catalog::catalog_json`]) for JavaScript.

use wasm_bindgen::prelude::wasm_bindgen;

use crate::constraint_catalog::catalog_json;
use crate::sketch_solve::{solve_sketch_json, spline_curve_json};

/// Solves one sketch, with the `P3DSketch_Solve` contract (see
/// `sketch_solve::solve_sketch_json`). Always returns a response; a request
/// that couldn't be read comes back with `status: "invalid"` and an `error`.
#[wasm_bindgen(js_name = acsSolveSketch)]
pub fn acs_solve_sketch(request_json: &str) -> String {
    solve_sketch_json(request_json).unwrap_or_else(|error| error)
}

/// A Spline's curve from its handles, without solving (see
/// `sketch_solve::spline_curve_json`): the `curve` a solve reports, or a
/// response with an `error` naming why there is none.
#[wasm_bindgen(js_name = acsSplineCurve)]
pub fn acs_spline_curve(request_json: &str) -> String {
    spline_curve_json(request_json).unwrap_or_else(|error| error)
}

/// Every constraint type `acsSolveSketch` accepts, one object per variant:
/// `[{ "type", "fields": [{ "name", "index"?, "kind" }], "extension"?,
/// "internal"? }]`.
#[wasm_bindgen(js_name = acsConstraintCatalog)]
pub fn acs_constraint_catalog() -> String {
    catalog_json()
}

#[cfg(target_arch = "wasm32")]
pub use typed::*;

/// The typed API ([`crate::sketch_types`]) for JavaScript, with TypeScript
/// declarations in the package's `.d.ts`: objects in and out, no JSON strings.
#[cfg(target_arch = "wasm32")]
mod typed {
    use serde::{Deserialize, Serialize};
    use tsify::{Ts, Tsify};
    use wasm_bindgen::prelude::wasm_bindgen;

    use crate::sketch_types::{solve, spline_curve, Curve, SketchError, SolveRequest, SolveResponse, Xy};

    /// `acsSolve`'s answer: the response, or the `status: "invalid"` one.
    #[derive(Serialize, Tsify)]
    #[serde(untagged)]
    // Flattened fields (`horizontal`'s line or points, …) serialize as maps,
    // which would reach JavaScript as `Map`s.
    #[tsify(hashmap_as_object)]
    pub enum SolveResult {
        /// The request was understood, whatever the solve status.
        Solved(SolveResponse),
        /// It couldn't be read.
        Invalid(SolveRejection),
    }

    /// `{version: 1, status: "invalid", error, constraintId?}`.
    #[derive(Serialize, Tsify)]
    #[serde(rename_all = "camelCase")]
    pub struct SolveRejection {
        /// Always 1.
        pub version: u32,
        /// Always `"invalid"`.
        pub status: InvalidStatus,
        /// What's wrong.
        pub error: String,
        /// The constraint it's about, if any.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub constraint_id: Option<String>,
    }

    /// The `status` of a [`SolveRejection`].
    #[derive(Serialize, Tsify)]
    #[serde(rename_all = "snake_case")]
    pub enum InvalidStatus {
        /// `"invalid"`.
        Invalid,
    }

    /// `acsCurve`'s request: `{version: 1, handles, interpolated, knots?}`.
    #[derive(Deserialize, Tsify)]
    pub struct CurveRequest {
        /// Always 1.
        pub version: u32,
        /// The handles' positions, `[x, y]` each.
        #[tsify(type = "[number, number][]")]
        pub handles: Vec<Xy>,
        /// `true`: fit points; `false`: control points.
        pub interpolated: bool,
        /// Control points only: a clamped knot vector.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub knots: Option<Vec<f64>>,
    }

    /// `acsCurve`'s answer: `{version: 1, curve}` or `{version: 1, error}`.
    #[derive(Serialize, Tsify)]
    #[serde(untagged)]
    pub enum CurveResult {
        /// The curve.
        Curve {
            /// Always 1.
            version: u32,
            /// The curve.
            curve: Curve,
        },
        /// Why there is none.
        Error {
            /// Always 1.
            version: u32,
            /// What's wrong.
            error: String,
        },
    }

    fn rejection(error: String, constraint_id: Option<String>) -> SolveResult {
        SolveResult::Invalid(SolveRejection { version: 1, status: InvalidStatus::Invalid, error, constraint_id })
    }

    /// Solves one sketch: `acsSolveSketch` over objects, typed by the
    /// package's declarations. Always answers; a request that couldn't be read
    /// comes back with `status: "invalid"`.
    #[wasm_bindgen(js_name = acsSolve)]
    pub fn acs_solve(request: Ts<SolveRequest>) -> Ts<SolveResult> {
        let result = match request.to_rust() {
            Err(e) => rejection(format!("invalid request: {e}"), None),
            Ok(request) => match solve(request) {
                Ok(response) => SolveResult::Solved(response),
                Err(SketchError::Invalid { message, constraint_id }) => rejection(message, constraint_id),
            },
        };
        Ts::from_rust(&result).expect("a result always converts")
    }

    /// A Spline's curve from its handles, without solving: `acsSplineCurve`
    /// over objects.
    #[wasm_bindgen(js_name = acsCurve)]
    pub fn acs_curve(request: Ts<CurveRequest>) -> Ts<CurveResult> {
        let result = match request.to_rust() {
            Err(e) => Err(format!("invalid request: {e}")),
            Ok(r) if r.version != 1 => Err("unsupported request version".to_string()),
            Ok(r) => spline_curve(r.handles, r.interpolated, r.knots).map_err(|e| e.to_string()),
        };
        let result = match result {
            Ok(curve) => CurveResult::Curve { version: 1, curve },
            Err(error) => CurveResult::Error { version: 1, error },
        };
        Ts::from_rust(&result).expect("a result always converts")
    }
}
