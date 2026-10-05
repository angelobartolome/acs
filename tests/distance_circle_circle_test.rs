//! `distance` between two circles: the outside gap `|c1 − c2| − r1 − r2`,
//! or with `internal: true`, the inside gap `|r1 − r2| − |c1 − c2|` (the
//! smaller circle inside the larger).

use acs::sketch_solve::solve_sketch_json;
use serde_json::{Value, json};

/// A fixed circle at the origin with radius `r1` and a fixed-radius circle
/// of radius `r2` whose center is held on y = 0, starting at `(x, 0)`,
/// plus `constraint`.
fn solve(r1: f64, r2: f64, x: f64, constraint: Value) -> Value {
    let request = json!({ "version": 1, "primitives": [
        { "id": "o1", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "c1", "type": "circle", "c_id": "o1", "radius": r1, "fixed": true },
        { "id": "o2", "type": "point", "x": x, "y": 0.0 },
        { "id": "c2", "type": "circle", "c_id": "o2", "radius": r2, "fixed": true },
        { "id": "oy", "type": "y", "point": "o2", "value": 0.0 },
        constraint
    ]})
    .to_string();
    let out = solve_sketch_json(&request).unwrap_or_else(|e| panic!("rejected: {e}"));
    let resp: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(resp["status"], "converged", "{resp}");
    resp
}

fn x2(resp: &Value) -> f64 {
    resp["primitives"].as_array().unwrap().iter().find(|p| p["id"] == "o2").unwrap()["x"]
        .as_f64()
        .unwrap()
}

#[test]
fn two_circles_apart() {
    let resp = solve(2.0, 1.0, 8.0, json!({ "id": "k", "type": "distance", "a": "c1", "b": "c2", "value": 1.5 }));
    assert!((x2(&resp) - 4.5).abs() < 1e-10);
}

#[test]
fn a_circle_inside_another() {
    // Inside gap 2 between a radius-5 and a radius-1 circle: centers 2 apart.
    for (a, b) in [("c1", "c2"), ("c2", "c1")] {
        let resp = solve(
            5.0,
            1.0,
            1.0,
            json!({ "id": "k", "type": "distance", "a": a, "b": b, "value": 2.0, "internal": true }),
        );
        assert!((x2(&resp) - 2.0).abs() < 1e-10, "{a}, {b}: {}", x2(&resp));
    }
}

#[test]
fn internal_is_only_for_circle_pairs_and_points() {
    let request = json!({ "version": 1, "primitives": [
        { "id": "a", "type": "point", "x": 0.0, "y": 0.0 },
        { "id": "b", "type": "point", "x": 1.0, "y": 0.0 },
        { "id": "k", "type": "distance", "a": "a", "b": "b", "value": 2.0, "internal": true }
    ]})
    .to_string();
    let out = solve_sketch_json(&request).unwrap_err();
    assert!(out.contains("unsupported combination for 'distance'"), "{out}");
    assert!(out.contains("got a: point, b: point, internal"), "{out}");

    let request = json!({ "version": 1, "primitives": [
        { "id": "a", "type": "point", "x": 0.0, "y": 0.0 },
        { "id": "b", "type": "point", "x": 1.0, "y": 0.0 },
        { "id": "k", "type": "distance", "a": "a", "b": "b", "value": 2.0, "internal": "yes" }
    ]})
    .to_string();
    let out = solve_sketch_json(&request).unwrap_err();
    assert!(out.contains("field 'internal' is not true or false"), "{out}");
}
