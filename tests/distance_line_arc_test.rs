//! Native `distance` between a Line and an arc: the line–circle gap (from
//! the arc's center to the segment, or with `extension: true` to the
//! Extension, minus the radius), with the nearest point of the arc (on the
//! ray from its center through the Line's nearest point) on the arc's span,
//! as for circle–arc distance.

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

/// Line `l` from `la` to `lb`, its endpoints free unless `fixed`.
fn line(a: (f64, f64), b: (f64, f64), fixed: bool) -> Vec<Value> {
    vec![
        json!({ "id": "la", "type": "point", "x": a.0, "y": a.1, "fixed": fixed }),
        json!({ "id": "lb", "type": "point", "x": b.0, "y": b.1, "fixed": fixed }),
        json!({ "id": "l", "type": "line", "p1_id": "la", "p2_id": "lb" }),
    ]
}

fn sketch(parts: Vec<Vec<Value>>) -> Value {
    Value::Array(parts.into_iter().flatten().collect())
}

/// Distance from `p` to segment `ab`, and the segment's nearest point.
fn segment_nearest(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> (f64, (f64, f64)) {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let t = (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
    let q = (a.0 + t * dx, a.1 + t * dy);
    (dist(p, q), q)
}

/// Direction (radians) from `from` to `to`.
fn angle(from: (f64, f64), to: (f64, f64)) -> f64 {
    (to.1 - from.1).atan2(to.0 - from.0)
}

#[test]
fn a_segment_holds_a_gap_from_an_arc_in_either_order() {
    for (a, b) in [("l", "q"), ("q", "l")] {
        let resp = solve(sketch(vec![
            // Radius 2 around the origin, spanning the top.
            arc("q", (0.0, 0.0), 2.0, 0.5, 2.6, true),
            line((-1.0, 3.5), (1.5, 3.8), false),
            vec![json!({ "id": "k", "type": "distance", "a": a, "b": b, "value": 1.0 })],
        ]));
        let (c, la, lb) = (xy(&resp, "q_c"), xy(&resp, "la"), xy(&resp, "lb"));
        let (d, q) = segment_nearest(c, la, lb);
        assert!((d - 3.0).abs() < 1e-8, "d = {d}");
        assert!(on_span(&resp, "q", angle(c, q), 1e-8));
        assert_eq!(resp["redundant"], json!([]));
    }
}

#[test]
fn the_segment_and_the_extension_measure_differently() {
    // A fixed Line off to one side of a fixed center; only the arc's radius
    // and angles are free. The segment's nearest point is its endpoint
    // (3, 5), √34 from the center; the Extension's is the foot (0, 5).
    for (extension, expected) in [(false, 34f64.sqrt() - 1.0), (true, 4.0)] {
        let resp = solve(sketch(vec![
            vec![
                json!({ "id": "q_c", "type": "point", "x": 0.0, "y": 0.0, "fixed": true }),
                json!({ "id": "q_s", "type": "point", "x": 2.0 * 0.8_f64.cos(), "y": 2.0 * 0.8_f64.sin() }),
                json!({ "id": "q_e", "type": "point", "x": 2.0 * 2.3_f64.cos(), "y": 2.0 * 2.3_f64.sin() }),
                json!({ "id": "q", "type": "arc", "c_id": "q_c", "start_id": "q_s", "end_id": "q_e",
                        "radius": 2.0, "start_angle": 0.8, "end_angle": 2.3 }),
            ],
            line((3.0, 5.0), (6.0, 5.0), true),
            vec![json!({ "id": "k", "type": "distance", "a": "l", "b": "q", "value": 1.0,
                         "extension": extension })],
        ]));
        let r = num(&resp, "q", "radius");
        assert!((r - expected).abs() < 1e-8, "extension {extension}: r = {r}");
        let toward = if extension { (0.0, 5.0) } else { (3.0, 5.0) };
        assert!(on_span(&resp, "q", angle((0.0, 0.0), toward), 1e-8), "extension {extension}: {resp}");
    }
}

#[test]
fn a_segment_beyond_the_span_is_brought_onto_it() {
    // A horizontal Line to the right, whose nearest point (its endpoint
    // (4, 1)) lies off the span (which spans the top): it isn't measured to
    // the arc's endpoint; the solve moves it until the ray through its
    // nearest point crosses the span.
    let resp = solve(sketch(vec![
        arc("q", (0.0, 0.0), 2.0, 0.8, 2.3, true),
        line((4.0, 1.0), (6.0, 1.0), false),
        vec![
            json!({ "id": "h", "type": "horizontal", "line": "l" }),
            json!({ "id": "k", "type": "distance", "a": "l", "b": "q", "value": 1.0 }),
        ],
    ]));
    let (c, la, lb) = (xy(&resp, "q_c"), xy(&resp, "la"), xy(&resp, "lb"));
    let (d, q) = segment_nearest(c, la, lb);
    assert!((d - 3.0).abs() < 1e-8, "d = {d}");
    assert!(on_span(&resp, "q", angle(c, q), 1e-8), "{resp}");
}

#[test]
fn the_extension_keeps_the_foot_on_the_span() {
    let resp = solve(sketch(vec![
        arc("q", (0.0, 0.0), 2.0, 0.8, 2.3, true),
        // Nearly vertical, to the right: the foot starts off the span.
        line((4.0, -1.0), (4.5, 3.0), false),
        vec![json!({ "id": "k", "type": "distance", "a": "q", "b": "l", "value": 0.5,
                     "extension": true })],
    ]));
    let (c, la, lb) = (xy(&resp, "q_c"), xy(&resp, "la"), xy(&resp, "lb"));
    let (dx, dy) = (lb.0 - la.0, lb.1 - la.1);
    let t = ((c.0 - la.0) * dx + (c.1 - la.1) * dy) / (dx * dx + dy * dy);
    let foot = (la.0 + t * dx, la.1 + t * dy);
    assert!((dist(c, foot) - 2.5).abs() < 1e-8);
    assert!(on_span(&resp, "q", angle(c, foot), 1e-8), "{resp}");
}

#[test]
fn internal_with_a_line_and_an_arc_is_rejected() {
    let req = request(sketch(vec![
        arc("q", (0.0, 0.0), 2.0, 0.5, 2.6, true),
        line((3.0, 3.0), (5.0, 3.0), false),
        vec![json!({ "id": "k", "type": "distance", "a": "l", "b": "q", "value": 1.0, "internal": true })],
    ]));
    let err = solve_sketch_json(&req).unwrap_err();
    assert!(
        err.contains("field 'internal' applies only between a circle or arc and a point, circle or arc; got a: line, b: arc"),
        "{err}"
    );
}
