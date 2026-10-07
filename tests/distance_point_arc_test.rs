//! Native `distance` between a point and an arc: the point–circle gap,
//! signed (`|p − c| − r` outside, `r − |p − c|` with `internal: true`), with
//! the nearest point of the arc (on the ray from its center through the
//! point) on the arc's span, as for circle–arc distance.

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

fn point(id: &str, x: f64, y: f64, fixed: bool) -> Vec<Value> {
    vec![json!({ "id": id, "type": "point", "x": x, "y": y, "fixed": fixed })]
}

fn sketch(parts: Vec<Vec<Value>>) -> Value {
    Value::Array(parts.into_iter().flatten().collect())
}

#[test]
fn a_point_holds_a_gap_outside_an_arc_in_either_order() {
    for (a, b) in [("p", "q"), ("q", "p")] {
        let resp = solve(sketch(vec![
            // Radius 2 around the origin, spanning the top.
            arc("q", (0.0, 0.0), 2.0, 0.5, 2.6, true),
            point("p", 0.5, 3.0, false),
            vec![json!({ "id": "k", "type": "distance", "a": a, "b": b, "value": 1.5 })],
        ]));
        let d = dist(xy(&resp, "q_c"), xy(&resp, "p"));
        assert!((d - (2.0 + 1.5)).abs() < 1e-8, "d = {d}");
        assert!(on_span(&resp, "q", direction(&resp, "q_c", "p"), 1e-8));
        // The point keeps one degree of freedom: it slides along the offset arc.
        assert_eq!(resp["dof"], 1, "{resp}");
        assert_eq!(resp["redundant"], json!([]));
    }
}

#[test]
fn internal_holds_a_gap_between_a_point_and_the_inside_of_an_arc() {
    let resp = solve(sketch(vec![
        arc("q", (0.0, 0.0), 3.0, 0.5, 2.6, true),
        point("p", 0.2, 1.0, false),
        vec![json!({ "id": "k", "type": "distance", "a": "p", "b": "q", "value": 0.5, "internal": true })],
    ]));
    let d = dist(xy(&resp, "q_c"), xy(&resp, "p"));
    assert!((d - (3.0 - 0.5)).abs() < 1e-8, "d = {d}");
    assert!(on_span(&resp, "q", direction(&resp, "q_c", "p"), 1e-8));
}

#[test]
fn the_gap_is_signed_so_a_point_can_cross_the_arc() {
    // Outside by −0.5 is 0.5 inside; inside by −0.5 is 0.5 outside.
    for (internal, expected) in [(false, 2.5), (true, 3.5)] {
        let resp = solve(sketch(vec![
            arc("q", (0.0, 0.0), 3.0, 0.5, 2.6, true),
            point("p", 0.0, 4.0, false),
            vec![json!({ "id": "k", "type": "distance", "a": "p", "b": "q", "value": -0.5,
                         "internal": internal })],
        ]));
        let d = dist(xy(&resp, "q_c"), xy(&resp, "p"));
        assert!((d - expected).abs() < 1e-8, "internal {internal}: d = {d}");
    }
}

#[test]
fn a_point_beyond_the_span_is_brought_onto_it() {
    // The point starts below the x axis, off the span (which spans the top):
    // it isn't measured to an endpoint; the solve moves it until its nearest
    // point on the arc's circle lies on the span.
    let resp = solve(sketch(vec![
        arc("q", (0.0, 0.0), 2.0, 0.5, 2.6, true),
        point("p", 3.0, -1.0, false),
        vec![json!({ "id": "k", "type": "distance", "a": "p", "b": "q", "value": 1.0 })],
    ]));
    let d = dist(xy(&resp, "q_c"), xy(&resp, "p"));
    assert!((d - 3.0).abs() < 1e-8, "d = {d}");
    assert!(on_span(&resp, "q", direction(&resp, "q_c", "p"), 1e-8));
}

#[test]
fn an_arc_turns_its_span_to_face_a_fixed_point() {
    // Only the arc's angles (and endpoints) are free: its span, facing
    // away from the point, turns toward it.
    let resp = solve(sketch(vec![
        point("p", -5.0, 0.0, true),
        vec![
            json!({ "id": "q_c", "type": "point", "x": 0.0, "y": 0.0, "fixed": true }),
            json!({ "id": "q_s", "type": "point", "x": 2.0 * 0.6_f64.cos(), "y": -2.0 * 0.6_f64.sin() }),
            json!({ "id": "q_e", "type": "point", "x": 2.0 * 0.6_f64.cos(), "y": 2.0 * 0.6_f64.sin() }),
            json!({ "id": "q", "type": "arc", "c_id": "q_c", "start_id": "q_s", "end_id": "q_e",
                    "radius": 2.0, "start_angle": -0.6, "end_angle": 0.6 }),
        ],
        vec![json!({ "id": "k", "type": "distance", "a": "p", "b": "q", "value": 1.0 })],
    ]));
    assert!((num(&resp, "q", "radius") - 4.0).abs() < 1e-8);
    assert!(on_span(&resp, "q", direction(&resp, "q_c", "p"), 1e-8));
}
