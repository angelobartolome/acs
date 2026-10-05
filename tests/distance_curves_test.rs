//! Native `distance` between a circle and an arc, or two arcs: the gap
//! between them along their line of centers, outside each other by default
//! and inside one another with `internal: true`, as for two circles. Like
//! their tangency (a gap of 0), the nearest points lie on each arc's span.

use std::f64::consts::TAU;

use acs::bindings::sketch_solve::acs_solve_sketch;
use acs::sketch_solve::solve_sketch_json;
use serde_json::{Value, json};

fn request(primitives: Value) -> String {
    json!({ "version": 1, "primitives": primitives }).to_string()
}

/// Solves a request that must converge, returning the parsed response.
fn solve(primitives: Value) -> Value {
    let request = request(primitives);
    let out = solve_sketch_json(&request).unwrap_or_else(|e| panic!("rejected: {e}"));
    assert_eq!(acs_solve_sketch(&request), out);
    let resp: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(resp["status"], "converged", "{resp}");
    resp
}

fn primitive<'a>(resp: &'a Value, id: &str) -> &'a Value {
    resp["primitives"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == id)
        .unwrap()
}

fn xy(resp: &Value, id: &str) -> (f64, f64) {
    let p = primitive(resp, id);
    (p["x"].as_f64().unwrap(), p["y"].as_f64().unwrap())
}

fn num(resp: &Value, id: &str, field: &str) -> f64 {
    primitive(resp, id)[field].as_f64().unwrap()
}

fn dist(a: (f64, f64), b: (f64, f64)) -> f64 {
    (b.0 - a.0).hypot(b.1 - a.1)
}

/// Direction (radians) from point `from` to point `to`.
fn direction(resp: &Value, from: &str, to: &str) -> f64 {
    let (a, b) = (xy(resp, from), xy(resp, to));
    (b.1 - a.1).atan2(b.0 - a.0)
}

/// Whether direction `phi` lies on the span of arc `id` (CCW from its start
/// angle to its end angle), within `tol` radians.
fn on_span(resp: &Value, id: &str, phi: f64, tol: f64) -> bool {
    let (start, end) = (num(resp, id, "start_angle"), num(resp, id, "end_angle"));
    let mut sweep = (end - start).rem_euclid(TAU);
    if sweep == 0.0 {
        sweep = TAU;
    }
    let u = (phi - start).rem_euclid(TAU);
    u <= sweep + tol || u >= TAU - tol
}

/// Center Point `{id}_c`, endpoints `{id}_s`/`{id}_e` (on it) and the Arc
/// `id` itself, centered at `c` with radius `r` from angle `a0` to `a1`.
fn arc(id: &str, c: (f64, f64), r: f64, a0: f64, a1: f64, fixed: bool) -> Vec<Value> {
    let at = |a: f64| (c.0 + r * a.cos(), c.1 + r * a.sin());
    let (s, e) = (at(a0), at(a1));
    vec![
        json!({ "id": format!("{id}_c"), "type": "point", "x": c.0, "y": c.1, "fixed": fixed }),
        json!({ "id": format!("{id}_s"), "type": "point", "x": s.0, "y": s.1, "fixed": fixed }),
        json!({ "id": format!("{id}_e"), "type": "point", "x": e.0, "y": e.1, "fixed": fixed }),
        json!({ "id": id, "type": "arc", "c_id": format!("{id}_c"), "start_id": format!("{id}_s"),
                "end_id": format!("{id}_e"), "radius": r, "start_angle": a0, "end_angle": a1,
                "fixed": fixed }),
    ]
}

/// A fixed radius-3 Circle `c` at the origin.
fn fixed_circle() -> Vec<Value> {
    vec![
        json!({ "id": "o", "type": "point", "x": 0.0, "y": 0.0, "fixed": true }),
        json!({ "id": "c", "type": "circle", "c_id": "o", "radius": 3.0, "fixed": true }),
    ]
}

fn sketch(parts: Vec<Vec<Value>>) -> Value {
    Value::Array(parts.into_iter().flatten().collect())
}

#[test]
fn two_arcs_hold_a_gap_outside_each_other_in_either_order() {
    for (a, b) in [("p", "q"), ("q", "p")] {
        let resp = solve(sketch(vec![
            // Radius 2 around the origin, span facing right (around 0).
            arc("p", (0.0, 0.0), 2.0, -0.8, 0.8, true),
            // Radius 1, span facing left (around π).
            arc("q", (6.0, 0.5), 1.0, 2.4, 3.9, false),
            vec![json!({ "id": "k", "type": "distance", "a": a, "b": b, "value": 1.5 })],
        ]));
        let rq = num(&resp, "q", "radius");
        let d = dist(xy(&resp, "p_c"), xy(&resp, "q_c"));
        assert!((d - (2.0 + rq + 1.5)).abs() < 1e-8, "d = {d}, rq = {rq}");
        assert!(on_span(&resp, "p", direction(&resp, "p_c", "q_c"), 1e-8));
        assert!(on_span(&resp, "q", direction(&resp, "q_c", "p_c"), 1e-8));
    }
}

#[test]
fn internal_holds_a_gap_between_an_arc_and_the_inside_of_another() {
    let resp = solve(sketch(vec![
        // A fixed radius-5 arc around the origin, spanning the top.
        arc("p", (0.0, 0.0), 5.0, 0.5, 2.6, true),
        // Radius 1 inside it, span facing up.
        arc("q", (0.3, 1.5), 1.0, 1.0, 2.2, false),
        vec![json!({ "id": "k", "type": "distance", "a": "p", "b": "q", "value": 0.5, "internal": true })],
    ]));
    let rq = num(&resp, "q", "radius");
    let d = dist(xy(&resp, "p_c"), xy(&resp, "q_c"));
    assert!((d - (5.0 - rq - 0.5)).abs() < 1e-8, "d = {d}, rq = {rq}");
    // Inside, both nearest points lie the same way from their centers.
    let phi = direction(&resp, "p_c", "q_c");
    assert!(on_span(&resp, "p", phi, 1e-8));
    assert!(on_span(&resp, "q", phi, 1e-8));
}

#[test]
fn a_circle_and_an_arc_hold_a_gap_outside_each_other_in_either_order() {
    for (a, b) in [("c", "q"), ("q", "c")] {
        let resp = solve(sketch(vec![
            fixed_circle(),
            // Radius 1, span facing the circle (around π).
            arc("q", (5.0, 0.4), 1.0, 2.5, 3.8, false),
            vec![json!({ "id": "k", "type": "distance", "a": a, "b": b, "value": 2.0 })],
        ]));
        let rq = num(&resp, "q", "radius");
        let d = dist(xy(&resp, "o"), xy(&resp, "q_c"));
        assert!((d - (3.0 + rq + 2.0)).abs() < 1e-8, "d = {d}, rq = {rq}");
        assert!(on_span(&resp, "q", direction(&resp, "q_c", "o"), 1e-8));
    }
}

#[test]
fn internal_holds_a_gap_between_an_arc_and_the_inside_of_a_circle() {
    let resp = solve(sketch(vec![
        fixed_circle(),
        // Radius 1 inside the circle, span facing outwards (around 0).
        arc("q", (1.0, 0.2), 1.0, -0.6, 0.6, false),
        vec![json!({ "id": "k", "type": "distance", "a": "c", "b": "q", "value": 0.5, "internal": true })],
    ]));
    let rq = num(&resp, "q", "radius");
    let d = dist(xy(&resp, "o"), xy(&resp, "q_c"));
    assert!((d - (3.0 - rq - 0.5)).abs() < 1e-8, "d = {d}, rq = {rq}");
    assert!(on_span(&resp, "q", direction(&resp, "o", "q_c"), 1e-8));
}

#[test]
fn an_arc_turns_its_span_to_face_the_curve_it_keeps_a_gap_from() {
    let resp = solve(sketch(vec![
        fixed_circle(),
        // Span facing away from the circle (around 0); only the angles are free.
        vec![
            json!({ "id": "q_c", "type": "point", "x": 6.0, "y": 0.0, "fixed": true }),
            json!({ "id": "q_s", "type": "point", "x": 6.0 + 0.6_f64.cos() * 2.0, "y": -(0.6_f64.sin()) * 2.0 }),
            json!({ "id": "q_e", "type": "point", "x": 6.0 + 0.6_f64.cos() * 2.0, "y": 0.6_f64.sin() * 2.0 }),
            json!({ "id": "q", "type": "arc", "c_id": "q_c", "start_id": "q_s", "end_id": "q_e",
                    "radius": 2.0, "start_angle": -0.6, "end_angle": 0.6 }),
        ],
        vec![json!({ "id": "k", "type": "distance", "a": "c", "b": "q", "value": 1.0 })],
    ]));
    assert!(on_span(&resp, "q", direction(&resp, "q_c", "o"), 1e-8));
}
