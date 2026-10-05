//! `normal {line, curve}`: a Line's Extension passes through a Circle's or
//! Arc's center. Direction only: the line need not reach the curve, nor an
//! arc's span.

use acs::bindings::sketch_solve::acs_solve_sketch;
use acs::sketch_solve::solve_sketch_json;
use serde_json::{Value, json};

fn solve(primitives: Value) -> Value {
    let request = json!({ "version": 1, "primitives": primitives }).to_string();
    let out = solve_sketch_json(&request).unwrap_or_else(|e| panic!("rejected: {e}"));
    assert_eq!(acs_solve_sketch(&request), out);
    let resp: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(resp["status"], "converged", "{resp}");
    resp
}

fn xy(resp: &Value, id: &str) -> (f64, f64) {
    let p = resp["primitives"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == id)
        .unwrap();
    (p["x"].as_f64().unwrap(), p["y"].as_f64().unwrap())
}

/// Distance from (px, py) to the infinite line through (ax, ay), (bx, by).
fn off_line(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    (dx * (p.1 - a.1) - dy * (p.0 - a.0)).abs() / dx.hypot(dy)
}

#[test]
fn normal_to_a_circle_points_at_its_center() {
    // A short line far outside the circle, turned off its center.
    let resp = solve(json!([
        { "id": "o", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "c", "type": "circle", "c_id": "o", "radius": 1.0, "fixed": true },
        { "id": "p", "type": "point", "x": 5.0, "y": 0.0, "fixed": true },
        { "id": "q", "type": "point", "x": 7.0, "y": 2.0 },
        { "id": "l", "type": "line", "p1_id": "p", "p2_id": "q" },
        { "id": "k", "type": "normal", "line": "l", "curve": "c" }
    ]));
    let q = xy(&resp, "q");
    assert!(off_line((0.0, 0.0), (5.0, 0.0), q) < 1e-8, "q = {q:?}");
    assert!(q.1.abs() < 1e-8, "q = {q:?}");
    assert_eq!(resp["dof"], 1, "{resp}");
}

#[test]
fn normal_to_an_arc_ignores_its_span() {
    // A fixed quarter arc in the first quadrant; the line sits in the third
    // quadrant, nowhere near the span, and still turns onto the center.
    let resp = solve(json!([
        { "id": "o", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "s", "type": "point", "x": 2.0, "y": 0.0, "fixed": true },
        { "id": "e", "type": "point", "x": 0.0, "y": 2.0, "fixed": true },
        { "id": "a", "type": "arc", "c_id": "o", "start_id": "s", "end_id": "e",
          "radius": 2.0, "start_angle": 0.0, "end_angle": std::f64::consts::FRAC_PI_2, "fixed": true },
        { "id": "p", "type": "point", "x": -5.0, "y": -5.0, "fixed": true },
        { "id": "q", "type": "point", "x": -6.0, "y": -9.0 },
        { "id": "l", "type": "line", "p1_id": "p", "p2_id": "q" },
        { "id": "k", "type": "normal", "line": "l", "curve": "a" }
    ]));
    let q = xy(&resp, "q");
    assert!(off_line((0.0, 0.0), (-5.0, -5.0), q) < 1e-8, "q = {q:?}");
    assert!(q.0 < -5.0, "q should stay beyond p, away from the arc: {q:?}");
}

#[test]
fn normal_moves_a_free_circle_onto_the_line() {
    let resp = solve(json!([
        { "id": "p", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "q", "type": "point", "x": 1.0, "y": 0.0, "fixed": true },
        { "id": "l", "type": "line", "p1_id": "p", "p2_id": "q" },
        { "id": "o", "type": "point", "x": 10.0, "y": 3.0 },
        { "id": "c", "type": "circle", "c_id": "o", "radius": 1.0 },
        { "id": "k", "type": "normal", "line": "l", "curve": "c" }
    ]));
    assert!(xy(&resp, "o").1.abs() < 1e-8, "{resp}");
}

#[test]
fn normal_takes_a_circle_or_an_arc_not_a_line() {
    let request = json!({ "version": 1, "primitives": [
        { "id": "p", "type": "point", "x": 0.0, "y": 0.0 },
        { "id": "q", "type": "point", "x": 1.0, "y": 0.0 },
        { "id": "l", "type": "line", "p1_id": "p", "p2_id": "q" },
        { "id": "k", "type": "normal", "line": "l", "curve": "l" }
    ]})
    .to_string();
    let out = solve_sketch_json(&request).expect_err("a line is not a curve here");
    let resp: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(resp["status"], "invalid");
    assert_eq!(resp["constraintId"], "k");
    assert!(resp["error"].as_str().unwrap().contains("normal"), "{resp}");
}
