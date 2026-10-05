//! `distance` between a Line and a circle: the distance from the center to
//! the segment minus `r` (`extension: true`: to the Extension).

use acs::sketch_solve::solve_sketch_json;
use serde_json::{Value, json};

/// A fixed Line from (0, 0) to (10, 0) and a fixed radius-1 circle whose
/// center is held at x = `x`, starting at `(x, 5)`, plus `constraint`.
fn solve(x: f64, constraint: Value) -> Value {
    let request = json!({ "version": 1, "primitives": [
        { "id": "l1", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "l2", "type": "point", "x": 10.0, "y": 0.0, "fixed": true },
        { "id": "l", "type": "line", "p1_id": "l1", "p2_id": "l2" },
        { "id": "o", "type": "point", "x": x, "y": 5.0 },
        { "id": "c", "type": "circle", "c_id": "o", "radius": 1.0, "fixed": true },
        { "id": "ox", "type": "x", "point": "o", "value": x },
        constraint
    ]})
    .to_string();
    let out = solve_sketch_json(&request).unwrap_or_else(|e| panic!("rejected: {e}"));
    let resp: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(resp["status"], "converged", "{resp}");
    resp
}

fn oy(resp: &Value) -> f64 {
    resp["primitives"].as_array().unwrap().iter().find(|p| p["id"] == "o").unwrap()["y"]
        .as_f64()
        .unwrap()
}

#[test]
fn a_circle_above_the_segment() {
    let resp = solve(4.0, json!({ "id": "k", "type": "distance", "a": "l", "b": "c", "value": 2.0 }));
    assert!((oy(&resp) - 3.0).abs() < 1e-10);
    let resp = solve(4.0, json!({ "id": "k", "type": "distance", "a": "c", "b": "l", "value": 2.0 }));
    assert!((oy(&resp) - 3.0).abs() < 1e-10);
}

#[test]
fn beyond_the_end_it_measures_to_the_segment_or_the_extension() {
    // Center at x = 13: 3 past the end (10, 0). A gap of 4 puts the center
    // 5 from the end: y = 4.
    let resp = solve(13.0, json!({ "id": "k", "type": "distance", "a": "l", "b": "c", "value": 4.0 }));
    assert!((oy(&resp) - 4.0).abs() < 1e-10);
    // To the Extension it is 5 from y = 0.
    let resp = solve(
        13.0,
        json!({ "id": "k", "type": "distance", "a": "l", "b": "c", "value": 4.0, "extension": true }),
    );
    assert!((oy(&resp) - 5.0).abs() < 1e-10);
}
