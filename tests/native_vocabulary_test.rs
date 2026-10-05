//! ACS's native constraint vocabulary through `solve_sketch_json` (and
//! `acsSolveSketch`, which must answer identically): one type per
//! relationship, the variant inferred from the referenced entities' kinds,
//! `extension: true` selecting the Extension variants, and a clear error for
//! a combination no variant takes.

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

/// Solves a request that must be rejected, returning its error and the
/// constraint it names.
fn reject(primitives: Value) -> (String, Value) {
    let request = request(primitives);
    let out = solve_sketch_json(&request).expect_err("request should be rejected");
    assert_eq!(acs_solve_sketch(&request), out);
    let resp: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(resp["status"], "invalid");
    (
        resp["error"].as_str().unwrap().to_string(),
        resp["constraintId"].clone(),
    )
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

fn close(resp: &Value, id: &str, x: f64, y: f64) {
    let (px, py) = xy(resp, id);
    assert!(
        (px - x).abs() < 1e-6 && (py - y).abs() < 1e-6,
        "{id} = ({px}, {py}), expected ({x}, {y})"
    );
}

fn radius(resp: &Value, id: &str) -> f64 {
    primitive(resp, id)["radius"].as_f64().unwrap()
}

/// A fixed horizontal Line `l` from (0,0) to (10,0), plus `rest`.
fn with_fixed_line(rest: Value) -> Value {
    let mut prims = json!([
        { "id": "l1", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "l2", "type": "point", "x": 10.0, "y": 0.0, "fixed": true },
        { "id": "l", "type": "line", "p1_id": "l1", "p2_id": "l2" }
    ]);
    prims
        .as_array_mut()
        .unwrap()
        .extend(rest.as_array().unwrap().clone());
    prims
}

// ── Variant inference ──────────────────────────────────────────────────────

#[test]
fn distance_between_two_points() {
    let resp = solve(json!([
        { "id": "a", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "b", "type": "point", "x": 3.0, "y": 4.0 },
        { "id": "k", "type": "distance", "a": "a", "b": "b", "value": 10.0 }
    ]));
    let (x, y) = xy(&resp, "b");
    assert!((x.hypot(y) - 10.0).abs() < 1e-8);
}

/// A point beyond the Line's far end, at (13, 4): to the segment it is 5
/// from the endpoint (10, 0); to the Extension it is 4 from y = 0.
fn beyond_the_end(constraint: Value) -> Value {
    with_fixed_line(json!([
        { "id": "p", "type": "point", "x": 13.0, "y": 4.0 },
        { "id": "px", "type": "x", "point": "p", "value": 13.0 },
        constraint
    ]))
}

#[test]
fn distance_from_a_point_to_a_line_is_measured_to_the_segment() {
    let resp = solve(beyond_the_end(
        json!({ "id": "k", "type": "distance", "a": "p", "b": "l", "value": 5.0 }),
    ));
    close(&resp, "p", 13.0, 4.0);
    // Segment distance 3.5 from (10, 0), staying at x = 13: y = √3.25
    // (the Extension variant would give y = 3.5).
    let resp = solve(beyond_the_end(
        json!({ "id": "k", "type": "distance", "a": "p", "b": "l", "value": 3.5 }),
    ));
    close(&resp, "p", 13.0, 3.25_f64.sqrt());
}

#[test]
fn distance_with_extension_is_measured_to_the_extension() {
    let resp = solve(beyond_the_end(
        json!({ "id": "k", "type": "distance", "a": "p", "b": "l", "value": 3.0, "extension": true }),
    ));
    close(&resp, "p", 13.0, 3.0);
}

#[test]
fn distance_takes_its_point_and_line_in_either_order() {
    let resp = solve(beyond_the_end(
        json!({ "id": "k", "type": "distance", "a": "l", "b": "p", "value": 3.0, "extension": true }),
    ));
    close(&resp, "p", 13.0, 3.0);
}

#[test]
fn on_a_line_with_extension_holds_beyond_the_segment() {
    let resp = solve(beyond_the_end(
        json!({ "id": "k", "type": "on", "point": "p", "curve": "l", "extension": true }),
    ));
    close(&resp, "p", 13.0, 0.0);
}

#[test]
fn on_a_circle_and_on_an_arc() {
    let resp = solve(json!([
        { "id": "o", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "c", "type": "circle", "c_id": "o", "radius": 2.0, "fixed": true },
        { "id": "p", "type": "point", "x": 3.0, "y": 1.0 },
        { "id": "k", "type": "on", "point": "p", "curve": "c" }
    ]));
    let (x, y) = xy(&resp, "p");
    assert!((x.hypot(y) - 2.0).abs() < 1e-8);

    // A fixed quarter arc of radius 2 from 0 to π/2: (−3, −1) lands on its span.
    let resp = solve(json!([
        { "id": "o", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "s", "type": "point", "x": 2.0, "y": 0.0, "fixed": true },
        { "id": "e", "type": "point", "x": 0.0, "y": 2.0, "fixed": true },
        { "id": "a", "type": "arc", "c_id": "o", "start_id": "s", "end_id": "e",
          "radius": 2.0, "start_angle": 0.0, "end_angle": std::f64::consts::FRAC_PI_2, "fixed": true },
        { "id": "p", "type": "point", "x": 1.5, "y": 1.0 },
        { "id": "k", "type": "on", "point": "p", "curve": "a" }
    ]));
    let (x, y) = xy(&resp, "p");
    assert!((x.hypot(y) - 2.0).abs() < 1e-8);
    assert!(x >= -1e-9 && y >= -1e-9, "on the span: ({x}, {y})");
}

#[test]
fn on_a_line_point_cannot_leave_the_segment() {
    // A free point on the segment with x pinned past the end: the segment
    // variant can't hold both, so the solve fails.
    let request = request(beyond_the_end(
        json!({ "id": "k", "type": "on", "point": "p", "curve": "l" }),
    ));
    let resp: Value = serde_json::from_str(&solve_sketch_json(&request).unwrap()).unwrap();
    assert_eq!(resp["status"], "failed");
}

#[test]
fn horizontal_and_direction_take_a_line_or_two_points() {
    let resp = solve(json!([
        { "id": "a", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "b", "type": "point", "x": 5.0, "y": 2.0 },
        { "id": "c", "type": "point", "x": 1.0, "y": 4.0 },
        { "id": "ab", "type": "line", "p1_id": "a", "p2_id": "b" },
        { "id": "k1", "type": "horizontal", "line": "ab" },
        { "id": "k2", "type": "vertical", "a": "a", "b": "c" }
    ]));
    assert!(xy(&resp, "b").1.abs() < 1e-8);
    assert!(xy(&resp, "c").0.abs() < 1e-8);

    let resp = solve(json!([
        { "id": "a", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "b", "type": "point", "x": 5.0, "y": 2.0 },
        { "id": "ab", "type": "line", "p1_id": "a", "p2_id": "b" },
        { "id": "k1", "type": "direction", "line": "ab", "value": std::f64::consts::FRAC_PI_4 },
        { "id": "k2", "type": "distance", "a": "a", "b": "b", "value": 2.0_f64.sqrt() }
    ]));
    close(&resp, "b", 1.0, 1.0);
}

#[test]
fn tangent_infers_line_circle_line_arc_and_circle_circle() {
    // Line y = 0 from (0,0) to (10,0); circle centered at (5, 3).
    let line_circle = |tangent: Value| {
        with_fixed_line(json!([
            { "id": "o", "type": "point", "x": 5.0, "y": 3.0, "fixed": true },
            { "id": "c", "type": "circle", "c_id": "o", "radius": 1.0 },
            tangent
        ]))
    };
    let resp = solve(line_circle(
        json!({ "id": "k", "type": "tangent", "a": "l", "b": "c" }),
    ));
    assert!((radius(&resp, "c") - 3.0).abs() < 1e-8);
    // Circle first works too.
    let resp = solve(line_circle(
        json!({ "id": "k", "type": "tangent", "a": "c", "b": "l" }),
    ));
    assert!((radius(&resp, "c") - 3.0).abs() < 1e-8);

    // Circle beyond the segment: the Extension variant touches y = 0 at x = 14.
    let resp = solve(with_fixed_line(json!([
        { "id": "o", "type": "point", "x": 14.0, "y": 3.0, "fixed": true },
        { "id": "c", "type": "circle", "c_id": "o", "radius": 1.0 },
        { "id": "k", "type": "tangent", "a": "l", "b": "c", "extension": true }
    ])));
    assert!((radius(&resp, "c") - 3.0).abs() < 1e-8);

    let resp = solve(json!([
        { "id": "o1", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "o2", "type": "point", "x": 5.0, "y": 0.0, "fixed": true },
        { "id": "c1", "type": "circle", "c_id": "o1", "radius": 2.0, "fixed": true },
        { "id": "c2", "type": "circle", "c_id": "o2", "radius": 1.0 },
        { "id": "k", "type": "tangent", "a": "c1", "b": "c2" }
    ]));
    assert!((radius(&resp, "c2") - 3.0).abs() < 1e-8);
}

#[test]
fn concentric_and_equal_over_circles_and_arcs() {
    let resp = solve(json!([
        { "id": "o1", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "o2", "type": "point", "x": 1.0, "y": 2.0 },
        { "id": "s", "type": "point", "x": 4.0, "y": 2.0 },
        { "id": "e", "type": "point", "x": 1.0, "y": 5.0 },
        { "id": "c", "type": "circle", "c_id": "o1", "radius": 2.0, "fixed": true },
        { "id": "a", "type": "arc", "c_id": "o2", "start_id": "s", "end_id": "e",
          "radius": 3.0, "start_angle": 0.0, "end_angle": std::f64::consts::FRAC_PI_2 },
        { "id": "k1", "type": "concentric", "a": "a", "b": "c" },
        { "id": "k2", "type": "equal", "a": "c", "b": "a" }
    ]));
    close(&resp, "o2", 0.0, 0.0);
    assert!((radius(&resp, "a") - 2.0).abs() < 1e-8);
}

#[test]
fn equal_over_lines_and_over_values() {
    let resp = solve(json!([
        { "id": "a", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "b", "type": "point", "x": 4.0, "y": 0.0, "fixed": true },
        { "id": "c", "type": "point", "x": 0.0, "y": 1.0, "fixed": true },
        { "id": "d", "type": "point", "x": 2.0, "y": 1.0 },
        { "id": "ab", "type": "line", "p1_id": "a", "p2_id": "b" },
        { "id": "cd", "type": "line", "p1_id": "c", "p2_id": "d" },
        { "id": "k1", "type": "equal", "a": "ab", "b": "cd" },
        { "id": "k2", "type": "horizontal", "line": "cd" }
    ]));
    close(&resp, "d", 4.0, 1.0);

    let resp = solve(json!([
        { "id": "r", "type": "param", "value": 2.5 },
        { "id": "o", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "c", "type": "circle", "c_id": "o", "radius": 1.0 },
        { "id": "p", "type": "point", "x": 1.0, "y": 1.0 },
        { "id": "k1", "type": "equal", "a": { "entity": "c", "property": "radius" }, "b": "r" },
        { "id": "k2", "type": "equal", "a": { "entity": "p", "property": "x" }, "b": 7 },
        { "id": "k3", "type": "difference", "a": { "entity": "p", "property": "x" },
          "b": { "entity": "p", "property": "y" }, "value": "r" }
    ]));
    assert!((radius(&resp, "c") - 2.5).abs() < 1e-8);
    close(&resp, "p", 7.0, 9.5);
}

// ── One type per relationship ──────────────────────────────────────────────

#[test]
fn coincident_parallel_perpendicular_angle() {
    let resp = solve(json!([
        { "id": "a", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "b", "type": "point", "x": 4.0, "y": 0.0, "fixed": true },
        { "id": "c", "type": "point", "x": 0.1, "y": 0.2 },
        { "id": "d", "type": "point", "x": 3.0, "y": 1.0 },
        { "id": "e", "type": "point", "x": 1.0, "y": 3.0 },
        { "id": "ab", "type": "line", "p1_id": "a", "p2_id": "b" },
        { "id": "cd", "type": "line", "p1_id": "c", "p2_id": "d" },
        { "id": "ce", "type": "line", "p1_id": "c", "p2_id": "e" },
        { "id": "k1", "type": "coincident", "a": "a", "b": "c" },
        { "id": "k2", "type": "angle", "a": "ab", "b": "cd", "value": std::f64::consts::FRAC_PI_4 },
        { "id": "k3", "type": "perpendicular", "a": "cd", "b": "ce" },
        { "id": "k4", "type": "distance", "a": "c", "b": "d", "value": 2.0_f64.sqrt() },
        { "id": "k5", "type": "distance", "a": "c", "b": "e", "value": 2.0_f64.sqrt() }
    ]));
    close(&resp, "c", 0.0, 0.0);
    close(&resp, "d", 1.0, 1.0);
    close(&resp, "e", -1.0, 1.0);

    let resp = solve(json!([
        { "id": "a", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "b", "type": "point", "x": 4.0, "y": 0.0, "fixed": true },
        { "id": "c", "type": "point", "x": 0.0, "y": 2.0, "fixed": true },
        { "id": "d", "type": "point", "x": 3.0, "y": 3.0 },
        { "id": "ab", "type": "line", "p1_id": "a", "p2_id": "b" },
        { "id": "cd", "type": "line", "p1_id": "c", "p2_id": "d" },
        { "id": "k1", "type": "parallel", "a": "ab", "b": "cd" },
        { "id": "k2", "type": "x", "point": "d", "value": 3.0 }
    ]));
    close(&resp, "d", 3.0, 2.0);
}

#[test]
fn offset_x_y_radius() {
    let resp = solve(with_fixed_line(json!([
        { "id": "p", "type": "point", "x": 4.0, "y": 1.0 },
        { "id": "k1", "type": "offset", "point": "p", "line": "l", "value": 2.0, "side": -1 },
        { "id": "k2", "type": "x", "point": "p", "value": 4.0 }
    ])));
    close(&resp, "p", 4.0, -2.0);

    let resp = solve(json!([
        { "id": "o", "type": "point", "x": 1.0, "y": 1.0 },
        { "id": "c", "type": "circle", "c_id": "o", "radius": 1.0 },
        { "id": "k1", "type": "x", "point": "o", "value": 3.0 },
        { "id": "k2", "type": "y", "point": "o", "value": -2.0 },
        { "id": "k3", "type": "radius", "curve": "c", "value": 4.0 }
    ]));
    close(&resp, "o", 3.0, -2.0);
    assert!((radius(&resp, "c") - 4.0).abs() < 1e-8);
}

#[test]
fn midpoint_of_a_line() {
    let resp = solve(with_fixed_line(json!([
        { "id": "m", "type": "point", "x": 2.0, "y": 3.0 },
        { "id": "k", "type": "midpoint", "entities": ["m", "l"] }
    ])));
    close(&resp, "m", 5.0, 0.0);
}

#[test]
fn midpoint_of_a_line_on_a_line() {
    // q's midpoint, held at x = 15 past l's end: on l's segment it can't
    // be, on l's Extension it is.
    let sketch = |extension: bool| {
        with_fixed_line(json!([
            { "id": "q1", "type": "point", "x": 14.0, "y": 2.0, "fixed": true },
            { "id": "q2", "type": "point", "x": 16.0, "y": 1.0 },
            { "id": "q", "type": "line", "p1_id": "q1", "p2_id": "q2" },
            { "id": "k1", "type": "midpoint", "entities": ["q", "l"], "extension": extension },
            { "id": "k2", "type": "x", "point": "q2", "value": 16.0 }
        ]))
    };
    let resp = solve(sketch(true));
    close(&resp, "q2", 16.0, -2.0);

    let out = solve_sketch_json(&request(sketch(false))).unwrap();
    let resp: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(resp["status"], "failed", "{resp}");
}

#[test]
fn midpoint_takes_a_point_or_a_line_then_a_line() {
    let (error, id) = reject(with_fixed_line(json!([
        { "id": "m", "type": "point", "x": 2.0, "y": 3.0 },
        { "id": "k", "type": "midpoint", "entities": ["l", "m"] }
    ])));
    assert_eq!(id, "k");
    assert!(
        error.contains("got entities[0]: line, entities[1]: point"),
        "{error}"
    );
    assert!(
        error.contains("(entities[0]: point, entities[1]: line)"),
        "{error}"
    );

    let (error, _) = reject(with_fixed_line(json!([
        { "id": "k", "type": "midpoint", "entities": "l" }
    ])));
    assert!(
        error.contains("unsupported combination for 'midpoint'"),
        "{error}"
    );

    let (error, _) = reject(with_fixed_line(json!([
        { "id": "m", "type": "point", "x": 2.0, "y": 3.0 },
        { "id": "k", "type": "midpoint", "entities": ["m", "l", "m"] }
    ])));
    assert!(
        error.contains("unsupported combination for 'midpoint'"),
        "{error}"
    );
}

/// The 0.1.5 forms, removed in 0.1.7: `midpoint {point, line}` and
/// `midpoint_on`.
#[test]
fn removed_midpoint_forms_are_rejected() {
    let (error, _) = reject(with_fixed_line(json!([
        { "id": "m", "type": "point", "x": 2.0, "y": 3.0 },
        { "id": "k", "type": "midpoint", "point": "m", "line": "l" }
    ])));
    assert!(error.contains("unsupported combination for 'midpoint'"), "{error}");

    let (error, _) = reject(with_fixed_line(json!([
        { "id": "q1", "type": "point", "x": 14.0, "y": 2.0 },
        { "id": "q2", "type": "point", "x": 16.0, "y": 1.0 },
        { "id": "q", "type": "line", "p1_id": "q1", "p2_id": "q2" },
        { "id": "k", "type": "midpoint_on", "line": "q", "on": "l" }
    ])));
    assert_eq!(error, "constraint k: unknown type 'midpoint_on'");
}

#[test]
fn mirror_rotation_translation() {
    let resp = solve(json!([
        { "id": "m1", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "m2", "type": "point", "x": 0.0, "y": 1.0, "fixed": true },
        { "id": "axis", "type": "line", "p1_id": "m1", "p2_id": "m2" },
        { "id": "s", "type": "point", "x": 3.0, "y": 5.0, "fixed": true },
        { "id": "i", "type": "point", "x": -1.0, "y": 2.0 },
        { "id": "k", "type": "mirror", "source": "s", "image": "i", "axis": "axis" }
    ]));
    close(&resp, "i", -3.0, 5.0);

    let resp = solve(json!([
        { "id": "c", "type": "point", "x": 1.0, "y": 1.0, "fixed": true },
        { "id": "s", "type": "point", "x": 3.0, "y": 1.0, "fixed": true },
        { "id": "r", "type": "point", "x": 0.0, "y": 2.0 },
        { "id": "k", "type": "rotation", "source": "s", "copy": "r", "center": "c",
          "angle": std::f64::consts::FRAC_PI_2 }
    ]));
    close(&resp, "r", 1.0, 3.0);

    let resp = solve(json!([
        { "id": "f", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "t", "type": "point", "x": 0.0, "y": 5.0, "fixed": true },
        { "id": "s", "type": "point", "x": 2.0, "y": 1.0, "fixed": true },
        { "id": "q", "type": "point", "x": 0.0, "y": 0.0 },
        { "id": "k", "type": "translation", "source": "s", "copy": "q", "from": "f", "to": "t",
          "distance": 1.5, "count": 2 }
    ]));
    close(&resp, "q", 2.0, 4.0);
}

// ── Rejections ─────────────────────────────────────────────────────────────

#[test]
fn an_unsupported_combination_is_rejected_naming_the_constraint_and_kinds() {
    let (error, id) = reject(json!([
        { "id": "a", "type": "point", "x": 0.0, "y": 0.0 },
        { "id": "o", "type": "point", "x": 1.0, "y": 0.0 },
        { "id": "s", "type": "point", "x": 2.0, "y": 0.0 },
        { "id": "e", "type": "point", "x": 1.0, "y": 1.0 },
        { "id": "c", "type": "arc", "c_id": "o", "start_id": "s", "end_id": "e",
          "radius": 1.0, "start_angle": 0.0, "end_angle": std::f64::consts::FRAC_PI_2 },
        { "id": "k1", "type": "distance", "a": "a", "b": "c", "value": 2.0 }
    ]));
    assert_eq!(id, "k1");
    assert_eq!(
        error,
        "constraint k1: unsupported combination for 'distance': got a: point, b: arc; \
         expected (a: point, b: point) or (a: point, b: line) or (a: point, b: line, extension) \
         or (a: point, b: circle) or (a: point, b: circle, internal) or (a: line, b: circle) \
         or (a: line, b: circle, extension) or (a: circle, b: circle) \
         or (a: circle, b: circle, internal) or (a: circle, b: arc) \
         or (a: circle, b: arc, internal) or (a: arc, b: arc) or (a: arc, b: arc, internal) \
         or (a: line, b: line)"
    );
}

#[test]
fn extension_on_a_variant_without_one_is_rejected() {
    let (error, id) = reject(json!([
        { "id": "a", "type": "point", "x": 0.0, "y": 0.0 },
        { "id": "o", "type": "point", "x": 1.0, "y": 0.0 },
        { "id": "c", "type": "circle", "c_id": "o", "radius": 1.0 },
        { "id": "k1", "type": "on", "point": "a", "curve": "c", "extension": true }
    ]));
    assert_eq!(id, "k1");
    assert!(
        error.contains("unsupported combination for 'on'"),
        "{error}"
    );
    assert!(
        error.contains("got point: point, curve: circle, extension"),
        "{error}"
    );

    let (error, _) = reject(json!([
        { "id": "a", "type": "point", "x": 0.0, "y": 0.0 },
        { "id": "b", "type": "point", "x": 1.0, "y": 0.0 },
        { "id": "k1", "type": "coincident", "a": "a", "b": "b", "extension": true }
    ]));
    assert!(
        error.contains("unsupported combination for 'coincident'"),
        "{error}"
    );

    let (error, _) = reject(json!([
        { "id": "a", "type": "point", "x": 0.0, "y": 0.0 },
        { "id": "b", "type": "point", "x": 1.0, "y": 0.0 },
        { "id": "k1", "type": "coincident", "a": "a", "b": "b", "extension": "yes" }
    ]));
    assert_eq!(
        error,
        "constraint k1: field 'extension' is not true or false"
    );
}

#[test]
fn a_single_variant_type_names_its_missing_field_or_wrong_entity() {
    let (error, _) = reject(json!([
        { "id": "a", "type": "point", "x": 0.0, "y": 0.0 },
        { "id": "k1", "type": "coincident", "a": "a" }
    ]));
    assert_eq!(error, "constraint k1: missing field 'b'");

    let (error, _) = reject(json!([
        { "id": "a", "type": "point", "x": 0.0, "y": 0.0 },
        { "id": "k1", "type": "coincident", "a": "a", "b": "ghost" }
    ]));
    assert_eq!(error, "constraint k1: 'ghost' is not a point");

    let (error, _) = reject(json!([
        { "id": "a", "type": "point", "x": 0.0, "y": 0.0 },
        { "id": "b", "type": "point", "x": 1.0, "y": 0.0 },
        { "id": "k1", "type": "distance", "a": "a", "b": "b" }
    ]));
    assert_eq!(error, "constraint k1: missing field 'value'");
}

#[test]
fn a_missing_entity_in_a_multi_variant_type_is_named() {
    let (error, _) = reject(json!([
        { "id": "a", "type": "point", "x": 0.0, "y": 0.0 },
        { "id": "k1", "type": "horizontal", "a": "a", "b": "ghost" }
    ]));
    assert!(error.contains("b: 'ghost' (not an entity)"), "{error}");
    assert!(
        error.contains("expected (line: line) or (a: point, b: point)"),
        "{error}"
    );
}

#[test]
fn symmetric_is_not_a_native_type() {
    let (error, _) = reject(json!([
        { "id": "a", "type": "point", "x": 0.0, "y": 0.0 },
        { "id": "k1", "type": "symmetric", "a": "a" }
    ]));
    assert_eq!(error, "constraint k1: unknown type 'symmetric'");
}

// ── Ellipses ───────────────────────────────────────────────────────────────

/// The fixed 5 × 4 ellipse `e` at the origin (center `c`, focus `f` at
/// (3, 0), radmin 4), plus `rest`.
fn with_fixed_ellipse(rest: Value) -> Value {
    let mut prims = json!([
        { "id": "c", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "f", "type": "point", "x": 3.0, "y": 0.0, "fixed": true },
        { "id": "e", "type": "ellipse", "c_id": "c", "focus1_id": "f", "radmin": 4.0, "fixed": true }
    ]);
    prims
        .as_array_mut()
        .unwrap()
        .extend(rest.as_array().unwrap().clone());
    prims
}

#[test]
fn on_an_ellipse() {
    let resp = solve(with_fixed_ellipse(json!([
        { "id": "p", "type": "point", "x": 2.0, "y": 5.0 },
        { "id": "k", "type": "on", "point": "p", "curve": "e" }
    ])));
    let (x, y) = xy(&resp, "p");
    assert!(
        (x * x / 25.0 + y * y / 16.0 - 1.0).abs() < 1e-8,
        "({x}, {y})"
    );
}

#[test]
fn tangent_line_ellipse_in_either_order() {
    for (a, b) in [("l", "e"), ("e", "l")] {
        let resp = solve(with_fixed_ellipse(json!([
            { "id": "p1", "type": "point", "x": -2.0, "y": 6.0 },
            { "id": "p2", "type": "point", "x": 2.0, "y": 6.5 },
            { "id": "l", "type": "line", "p1_id": "p1", "p2_id": "p2" },
            { "id": "h", "type": "horizontal", "line": "l" },
            { "id": "x1", "type": "x", "point": "p1", "value": -2.0 },
            { "id": "x2", "type": "x", "point": "p2", "value": 2.0 },
            { "id": "k", "type": "tangent", "a": a, "b": b }
        ])));
        // Touches the top co-vertex (0, 4).
        close(&resp, "p1", -2.0, 4.0);
        close(&resp, "p2", 2.0, 4.0);
    }
}

#[test]
fn tangent_line_ellipse_has_no_extension_variant() {
    let (error, id) = reject(with_fixed_ellipse(json!([
        { "id": "p1", "type": "point", "x": -2.0, "y": 6.0 },
        { "id": "p2", "type": "point", "x": 2.0, "y": 6.0 },
        { "id": "l", "type": "line", "p1_id": "p1", "p2_id": "p2" },
        { "id": "k", "type": "tangent", "a": "l", "b": "e", "extension": true }
    ])));
    assert_eq!(id, "k");
    assert!(
        error.contains("got a: line, b: ellipse, extension"),
        "{error}"
    );
    assert!(error.contains("(a: line, b: ellipse)"), "{error}");
}

#[test]
fn ellipse_axis_places_points_at_the_ends_of_either_axis() {
    let resp = solve(with_fixed_ellipse(json!([
        { "id": "a", "type": "point", "x": 4.5, "y": 0.4 },
        { "id": "b", "type": "point", "x": -5.3, "y": -0.2 },
        { "id": "m", "type": "point", "x": 0.4, "y": 3.6 },
        { "id": "k1", "type": "ellipse_axis", "ellipse": "e", "point": "a", "which": "major" },
        { "id": "k2", "type": "ellipse_axis", "ellipse": "e", "point": "b", "which": "major" },
        { "id": "k3", "type": "ellipse_axis", "ellipse": "e", "point": "m", "which": "minor" }
    ])));
    close(&resp, "a", 5.0, 0.0);
    close(&resp, "b", -5.0, 0.0);
    close(&resp, "m", 0.0, 4.0);
}

#[test]
fn two_point_ellipse_axis_keeps_its_points_on_opposite_ends() {
    // Both points start nearer the +major end: one `ellipse_axis` each puts
    // both there; the two-point form puts one at each end.
    let points = json!([
        { "id": "a", "type": "point", "x": 4.5, "y": 0.4 },
        { "id": "b", "type": "point", "x": 3.9, "y": -0.6 }
    ]);
    let with = |constraints: Value| {
        let mut prims = points.clone();
        prims
            .as_array_mut()
            .unwrap()
            .extend(constraints.as_array().unwrap().clone());
        with_fixed_ellipse(prims)
    };
    let resp = solve(with(json!([
        { "id": "k1", "type": "ellipse_axis", "ellipse": "e", "point": "a", "which": "major" },
        { "id": "k2", "type": "ellipse_axis", "ellipse": "e", "point": "b", "which": "major" }
    ])));
    close(&resp, "a", 5.0, 0.0);
    close(&resp, "b", 5.0, 0.0);

    let resp = solve(with(json!([
        { "id": "k", "type": "ellipse_axis", "ellipse": "e", "a": "a", "b": "b", "which": "major" }
    ])));
    let ((ax, ay), (bx, by)) = (xy(&resp, "a"), xy(&resp, "b"));
    assert!(
        (ax + bx).abs() < 1e-8 && (ay + by).abs() < 1e-8,
        "a = ({ax}, {ay}), b = ({bx}, {by})"
    );
    assert!(
        (ax.abs() - 5.0).abs() < 1e-8 && ay.abs() < 1e-8,
        "a = ({ax}, {ay})"
    );

    let resp = solve(with(json!([
        { "id": "k", "type": "ellipse_axis", "ellipse": "e", "a": "a", "b": "b", "which": "minor" }
    ])));
    let ((ax, ay), (bx, by)) = (xy(&resp, "a"), xy(&resp, "b"));
    assert!(
        (ax + bx).abs() < 1e-8 && (ay + by).abs() < 1e-8,
        "a = ({ax}, {ay}), b = ({bx}, {by})"
    );
    assert!(
        ax.abs() < 1e-8 && (ay.abs() - 4.0).abs() < 1e-8,
        "a = ({ax}, {ay})"
    );
}

#[test]
fn ellipse_axis_moves_a_free_ellipse() {
    // A free ellipse whose major axis end is pinned at (7, 0): the ellipse
    // grows (its center and minor radius held) until it reaches it.
    let resp = solve(json!([
        { "id": "c", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "f", "type": "point", "x": 3.0, "y": 0.0 },
        { "id": "e", "type": "ellipse", "c_id": "c", "focus1_id": "f", "radmin": 4.0 },
        { "id": "p", "type": "point", "x": 7.0, "y": 0.0, "fixed": true },
        { "id": "r", "type": "equal", "a": { "entity": "e", "property": "radmin" }, "b": 4.0 },
        { "id": "k", "type": "ellipse_axis", "ellipse": "e", "point": "p", "which": "major" }
    ]));
    let b = primitive(&resp, "e")["radmin"].as_f64().unwrap();
    let (fx, fy) = xy(&resp, "f");
    assert!((b - 4.0).abs() < 1e-8);
    assert!(
        (b.hypot(fx.hypot(fy)) - 7.0).abs() < 1e-8,
        "f = ({fx}, {fy})"
    );
    assert!(fy.abs() < 1e-8);
}

#[test]
fn ellipse_axis_needs_major_or_minor() {
    let (error, id) = reject(with_fixed_ellipse(json!([
        { "id": "p", "type": "point", "x": 5.0, "y": 0.0 },
        { "id": "k", "type": "ellipse_axis", "ellipse": "e", "point": "p", "which": "focal" }
    ])));
    assert_eq!(id, "k");
    assert_eq!(
        error,
        "constraint k: field 'which' is not \"major\" or \"minor\""
    );

    let (error, _) = reject(with_fixed_ellipse(json!([
        { "id": "p", "type": "point", "x": 5.0, "y": 0.0 },
        { "id": "k", "type": "ellipse_axis", "ellipse": "p", "point": "p", "which": "major" }
    ])));
    assert!(
        error.contains("got ellipse: point, point: point"),
        "{error}"
    );
}

#[test]
fn an_ellipse_is_not_a_circle() {
    let (error, _) = reject(with_fixed_ellipse(json!([
        { "id": "k", "type": "radius", "curve": "e", "value": 2.0 }
    ])));
    assert!(error.contains("got curve: ellipse"), "{error}");
}

#[test]
fn equal_over_radmin_property_references() {
    let resp = solve(with_fixed_ellipse(json!([
        { "id": "c2", "type": "point", "x": 20.0, "y": 0.0 },
        { "id": "f2", "type": "point", "x": 23.0, "y": 0.0 },
        { "id": "e2", "type": "ellipse", "c_id": "c2", "focus1_id": "f2", "radmin": 2.0 },
        { "id": "k", "type": "equal",
          "a": { "entity": "e", "property": "radmin" }, "b": { "entity": "e2", "property": "radmin" } }
    ])));
    assert!((primitive(&resp, "e2")["radmin"].as_f64().unwrap() - 4.0).abs() < 1e-8);
}

// ── Tangency at a shared point (ticket 22) ─────────────────────────────────

/// The demo's slot: fixed centers, two radius-15 arcs, and two lines each
/// tangent to both arcs at the arcs' own endpoints. `c1` and the endpoints
/// are as in a user-reported request after a drag.
fn slot(t2: (f64, f64)) -> Value {
    json!([
        { "id": "c1", "type": "point", "x": -21.11288626762544, "y": 7.309455402309044, "fixed": true },
        { "id": "c2", "type": "point", "x": 25.0, "y": 0.0, "fixed": true },
        { "id": "t1", "type": "point", "x": -18.764522755504945, "y": 22.12448793062099 },
        { "id": "t2", "type": "point", "x": t2.0, "y": t2.1 },
        { "id": "b1", "type": "point", "x": -23.461249779745945, "y": -7.505577126002914 },
        { "id": "b2", "type": "point", "x": 22.651636487879497, "y": -14.815032528311958 },
        { "id": "a1", "type": "arc", "c_id": "c1", "start_id": "t1", "end_id": "b1", "radius": 15.0,
          "start_angle": 1.41359205596306, "end_angle": 4.555184709552853 },
        { "id": "a2", "type": "arc", "c_id": "c2", "start_id": "b2", "end_id": "t2", "radius": 15.0,
          "start_angle": -1.7280005976267336, "end_angle": 1.4135920559630595 },
        { "id": "lt", "type": "line", "p1_id": "t1", "p2_id": "t2" },
        { "id": "lb", "type": "line", "p1_id": "b1", "p2_id": "b2" },
        { "id": "k1", "type": "radius", "curve": "a1", "value": 15.0 },
        { "id": "k2", "type": "equal", "a": "a1", "b": "a2" },
        { "id": "k3", "type": "tangent", "a": "lt", "b": "a1" },
        { "id": "k4", "type": "tangent", "a": "lt", "b": "a2" },
        { "id": "k5", "type": "tangent", "a": "lb", "b": "a1" },
        { "id": "k6", "type": "tangent", "a": "lb", "b": "a2" }
    ])
}

fn assert_slot_fully_determined(resp: &Value) {
    assert_eq!(resp["redundant"], json!([]), "{}", resp["redundant"]);
    assert_eq!(resp["conflicting"], json!([]));
    assert_eq!(resp["dof"], 0);
    let fully: Vec<&str> = resp["fullyConstrained"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    for id in ["t1", "t2", "b1", "b2", "a1", "a2", "lt", "lb"] {
        assert!(fully.contains(&id), "{id} not fully constrained: {fully:?}");
    }
}

/// The user-reported request: converged, but with `redundant: [k3, k6]`,
/// `dof: 2` and only some endpoints fully constrained, because the distance
/// form of tangency is degenerate when the line already ends on the arc.
#[test]
fn a_slot_tangent_at_its_shared_endpoints_is_fully_determined() {
    let resp = solve(slot((27.348363512120503, 14.815032528311944)));
    assert_slot_fully_determined(&resp);
}

/// Dragging a slot endpoint (soft goals at the cursor, each result fed back)
/// reports the same clean diagnosis at every step, and releasing it settles.
#[test]
fn dragging_a_slot_endpoint_never_reports_redundancy() {
    let mut prims = solve(slot((27.348363512120503, 14.815032528311944)))["primitives"].clone();
    for step in 0..15 {
        let (tx, ty) = (
            30.0 + 4.0 * (step as f64 * 0.7).sin(),
            12.0 + 5.0 * (step as f64 * 0.9).cos(),
        );
        let mut with_drag = prims.as_array().unwrap().clone();
        with_drag.push(
            json!({ "id": "dx", "type": "x", "point": "t2", "value": tx, "temporary": true }),
        );
        with_drag.push(
            json!({ "id": "dy", "type": "y", "point": "t2", "value": ty, "temporary": true }),
        );
        let resp = solve(Value::Array(with_drag));
        assert_slot_fully_determined(&resp);
        let n = prims.as_array().unwrap().len();
        prims = Value::Array(resp["primitives"].as_array().unwrap()[..n].to_vec());
    }
    assert_slot_fully_determined(&solve(prims));
}
