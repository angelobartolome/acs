use wasm_bindgen::prelude::wasm_bindgen;

use crate::constraint_catalog::Vocabulary;
use crate::sketch_solve::solve_sketch_json_in;

/// Solves one sketch written in ACS's native constraint vocabulary, with the
/// `P3DSketch_Solve` contract (see `sketch_solve::solve_sketch_json_in`).
/// Always returns a response; a request that couldn't be read comes back with
/// `status: "invalid"` and an `error`.
#[wasm_bindgen(js_name = acsSolveSketch)]
pub fn acs_solve_sketch(request_json: &str) -> String {
    solve_sketch_json_in(Vocabulary::Native, request_json).unwrap_or_else(|error| error)
}

/// Solves one sketch written in the PlaneGCS dialect (the constraint types
/// GCS-style clients send), with the same contract and responses as `P3DSketch_Solve`.
#[wasm_bindgen(js_name = acsSolveSketchPlaneGcs)]
pub fn acs_solve_sketch_planegcs(request_json: &str) -> String {
    solve_sketch_json_in(Vocabulary::PlaneGcs, request_json).unwrap_or_else(|error| error)
}

/// Every native constraint type `acsSolveSketch` accepts, one object per
/// variant: `[{ "type", "fields": [{ "name", "kind" }], "extension"? }]`.
#[wasm_bindgen(js_name = acsConstraintCatalog)]
pub fn acs_constraint_catalog() -> String {
    Vocabulary::Native.catalog_json()
}

/// Every PlaneGCS dialect type `acsSolveSketchPlaneGcs` accepts, in the
/// same shape as `acsConstraintCatalog`.
#[wasm_bindgen(js_name = acsPlaneGcsConstraintCatalog)]
pub fn acs_planegcs_constraint_catalog() -> String {
    Vocabulary::PlaneGcs.catalog_json()
}
