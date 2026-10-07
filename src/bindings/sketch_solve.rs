//! `acsSolveSketch` and `acsConstraintCatalog`: the JSON API
//! ([`crate::sketch_solve::solve_sketch_json`]) and the constraint catalog
//! ([`crate::constraint_catalog::catalog_json`]) for JavaScript.

use wasm_bindgen::prelude::wasm_bindgen;

use crate::constraint_catalog::catalog_json;
use crate::sketch_solve::solve_sketch_json;

/// Solves one sketch, with the `P3DSketch_Solve` contract (see
/// `sketch_solve::solve_sketch_json`). Always returns a response; a request
/// that couldn't be read comes back with `status: "invalid"` and an `error`.
#[wasm_bindgen(js_name = acsSolveSketch)]
pub fn acs_solve_sketch(request_json: &str) -> String {
    solve_sketch_json(request_json).unwrap_or_else(|error| error)
}

/// Every constraint type `acsSolveSketch` accepts, one object per variant:
/// `[{ "type", "fields": [{ "name", "index"?, "kind" }], "extension"?,
/// "internal"? }]`.
#[wasm_bindgen(js_name = acsConstraintCatalog)]
pub fn acs_constraint_catalog() -> String {
    catalog_json()
}
