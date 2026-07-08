use serde_json::json;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(js_name = acsSolveSketch)]
pub fn acs_solve_sketch(input_json: &str) -> String {
    match crate::sketch_solve::solve_sketch_primitives_json(input_json) {
        Ok(s) => s,
        Err(e) => json!({
            "ok": false,
            "status": "failed",
            "solveStatus": 2,
            "primitives": Vec::<serde_json::Value>::new(),
            "skipped_constraint_ids": Vec::<String>::new(),
            "conflicting_constraint_ids": Vec::<String>::new(),
            "fully_constrained_ids": Vec::<String>::new(),
            "error": e,
        })
        .to_string(),
    }
}
