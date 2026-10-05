//! `distance` between a point and a circle: the signed gap `|p − c| − r`
//! (outside), or with `internal: true`, `r − |p − c|` (inside).

use acs::sketch_solve::solve_sketch_json;
use serde_json::{Value, json};

/// A fixed radius-2 circle at the origin and a point `p` held on y = 0,
/// starting at `(x, 0)`, plus `constraint`.
fn solve(x: f64, constraint: Value) -> Value {
    let request = json!({ "version": 1, "primitives": [
        { "id": "o", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "c", "type": "circle", "c_id": "o", "radius": 2.0, "fixed": true },
        { "id": "p", "type": "point", "x": x, "y": 0.0 },
        { "id": "py", "type": "y", "point": "p", "value": 0.0 },
        constraint
    ]})
    .to_string();
    let out = solve_sketch_json(&request).unwrap_or_else(|e| panic!("rejected: {e}"));
    let resp: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(resp["status"], "converged", "{resp}");
    resp
}

fn px(resp: &Value) -> f64 {
    resp["primitives"].as_array().unwrap().iter().find(|p| p["id"] == "p").unwrap()["x"]
        .as_f64()
        .unwrap()
}

#[test]
fn a_point_outside_a_circle() {
    let resp = solve(5.0, json!({ "id": "k", "type": "distance", "a": "p", "b": "c", "value": 1.5 }));
    assert!((px(&resp) - 3.5).abs() < 1e-10);
    // Circle first works too.
    let resp = solve(5.0, json!({ "id": "k", "type": "distance", "a": "c", "b": "p", "value": 1.5 }));
    assert!((px(&resp) - 3.5).abs() < 1e-10);
}

#[test]
fn a_point_inside_is_pushed_out_by_an_outside_distance() {
    let resp = solve(0.5, json!({ "id": "k", "type": "distance", "a": "p", "b": "c", "value": 1.0 }));
    assert!((px(&resp) - 3.0).abs() < 1e-10);
}

#[test]
fn an_internal_distance_measures_inside_the_circle() {
    let resp = solve(
        1.0,
        json!({ "id": "k", "type": "distance", "a": "p", "b": "c", "value": 0.5, "internal": true }),
    );
    assert!((px(&resp) - 1.5).abs() < 1e-10);
}
