//! `distance` between two Lines: both of `b`'s endpoints are `value` from
//! `a`'s Extension, on the same side, so the lines are parallel.

use acs::sketch_solve::solve_sketch_json;
use serde_json::{Value, json};

/// A fixed Line `a` along y = 0 and a Line `b` from `(0, y1)` to `(10, y2)`
/// whose endpoints are held at x = 0 and x = 10, plus `rest`.
fn request(y1: f64, y2: f64, rest: Value) -> String {
    let mut prims = json!([
        { "id": "a1", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "a2", "type": "point", "x": 4.0, "y": 0.0, "fixed": true },
        { "id": "a", "type": "line", "p1_id": "a1", "p2_id": "a2" },
        { "id": "b1", "type": "point", "x": 0.0, "y": y1 },
        { "id": "b2", "type": "point", "x": 10.0, "y": y2 },
        { "id": "b", "type": "line", "p1_id": "b1", "p2_id": "b2" },
        { "id": "x1", "type": "x", "point": "b1", "value": 0.0 },
        { "id": "x2", "type": "x", "point": "b2", "value": 10.0 }
    ]);
    prims.as_array_mut().unwrap().extend(rest.as_array().unwrap().clone());
    json!({ "version": 1, "primitives": prims }).to_string()
}

fn solve(y1: f64, y2: f64, rest: Value) -> Value {
    let out = solve_sketch_json(&request(y1, y2, rest)).unwrap_or_else(|e| panic!("rejected: {e}"));
    let resp: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(resp["status"], "converged", "{resp}");
    resp
}

fn y(resp: &Value, id: &str) -> f64 {
    resp["primitives"].as_array().unwrap().iter().find(|p| p["id"] == id).unwrap()["y"]
        .as_f64()
        .unwrap()
}

fn distance() -> Value {
    json!([{ "id": "k", "type": "distance", "a": "a", "b": "b", "value": 4.0 }])
}

#[test]
fn both_endpoints_end_up_the_distance_from_the_extension() {
    // b's far end lies past a's segment: measured to the Extension.
    let resp = solve(3.0, 6.0, distance());
    assert!((y(&resp, "b1") - 4.0).abs() < 1e-10);
    assert!((y(&resp, "b2") - 4.0).abs() < 1e-10);
    assert_eq!(resp["fullyConstrained"].as_array().unwrap().iter().filter(|v| *v == "b").count(), 1);
}

#[test]
fn a_line_below_stays_below() {
    let resp = solve(-3.0, -1.0, distance());
    assert!((y(&resp, "b1") + 4.0).abs() < 1e-10);
    assert!((y(&resp, "b2") + 4.0).abs() < 1e-10);
}

#[test]
fn it_implies_parallel() {
    // An explicit parallel on top is Redundant.
    let resp = solve(
        3.0,
        6.0,
        json!([
            { "id": "k", "type": "distance", "a": "a", "b": "b", "value": 4.0 },
            { "id": "p", "type": "parallel", "a": "a", "b": "b" }
        ]),
    );
    assert_eq!(resp["redundant"], json!(["p"]));
}
