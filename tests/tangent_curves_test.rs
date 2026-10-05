//! Native `tangent` between curves: circle–circle, circle–arc and arc–arc,
//! external (`r1 + r2`) by default and inside (`|r1 − r2|`) with
//! `internal: true`. The tangency point lies on each arc's span, and two arcs
//! sharing an endpoint are tangent at that point (radii collinear there).

use std::f64::consts::{FRAC_PI_2, PI, TAU};

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
fn arc(id: &str, c: (f64, f64), r: f64, a0: f64, a1: f64, center_fixed: bool) -> Vec<Value> {
    let at = |a: f64| (c.0 + r * a.cos(), c.1 + r * a.sin());
    let (s, e) = (at(a0), at(a1));
    vec![
        json!({ "id": format!("{id}_c"), "type": "point", "x": c.0, "y": c.1, "fixed": center_fixed }),
        json!({ "id": format!("{id}_s"), "type": "point", "x": s.0, "y": s.1 }),
        json!({ "id": format!("{id}_e"), "type": "point", "x": e.0, "y": e.1 }),
        json!({ "id": id, "type": "arc", "c_id": format!("{id}_c"), "start_id": format!("{id}_s"),
                "end_id": format!("{id}_e"), "radius": r, "start_angle": a0, "end_angle": a1 }),
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

/// Direction (radians) from point `from` to point `to`.
fn direction(resp: &Value, from: &str, to: &str) -> f64 {
    let (a, b) = (xy(resp, from), xy(resp, to));
    (b.1 - a.1).atan2(b.0 - a.0)
}

// ── Circle–arc ─────────────────────────────────────────────────────────────

#[test]
fn a_circle_and_an_arc_touch_externally_in_either_order() {
    for (a, b) in [("c", "a"), ("a", "c")] {
        let resp = solve(sketch(vec![
            fixed_circle(),
            // Radius 1, span facing the circle (around π), center too close.
            arc("a", (3.5, 0.3), 1.0, 2.5, 3.8, false),
            vec![json!({ "id": "k", "type": "tangent", "a": a, "b": b })],
        ]));
        let d = dist(xy(&resp, "o"), xy(&resp, "a_c"));
        assert!((d - (3.0 + num(&resp, "a", "radius"))).abs() < 1e-8, "d = {d}");
        assert!(on_span(&resp, "a", direction(&resp, "a_c", "o"), 1e-8));
    }
}

#[test]
fn internal_puts_an_arc_inside_a_circle() {
    let resp = solve(sketch(vec![
        fixed_circle(),
        // Radius 1 inside the circle, span facing outwards (around 0).
        arc("a", (1.5, 0.2), 1.0, -0.6, 0.6, false),
        vec![json!({ "id": "k", "type": "tangent", "a": "c", "b": "a", "internal": true })],
    ]));
    let r = num(&resp, "a", "radius");
    let d = dist(xy(&resp, "o"), xy(&resp, "a_c"));
    assert!((d - (3.0 - r)).abs() < 1e-8, "d = {d}, r = {r}");
    // Inside, both radii point the same way at the tangency point.
    assert!(on_span(&resp, "a", direction(&resp, "o", "a_c"), 1e-8));
}

#[test]
fn internal_puts_a_circle_inside_an_arc() {
    let resp = solve(sketch(vec![
        vec![
            json!({ "id": "o", "type": "point", "x": 0.5, "y": 0.0 }),
            json!({ "id": "c", "type": "circle", "c_id": "o", "radius": 1.0, "fixed": true }),
        ],
        // A fixed radius-4 arc around the origin, spanning the top.
        arc("a", (0.0, 0.0), 4.0, 0.5, 2.5, true),
        vec![json!({ "id": "k", "type": "tangent", "a": "a", "b": "c", "internal": true })],
    ]));
    let r = num(&resp, "a", "radius");
    let d = dist(xy(&resp, "o"), xy(&resp, "a_c"));
    assert!((d - (r - 1.0)).abs() < 1e-8, "d = {d}, r = {r}");
    assert!(on_span(&resp, "a", direction(&resp, "a_c", "o"), 1e-8));
}

/// The arc's angles and radius are fixed, its center free: tangency must
/// move the center until the circle lies in the direction the span faces,
/// not just anywhere at distance 4.
#[test]
fn the_tangency_point_is_held_on_the_arcs_span() {
    let mut parts = arc("a", (5.0, 0.0), 1.0, 0.0, FRAC_PI_2, false);
    parts[3]["fixed"] = json!(true);
    let resp = solve(sketch(vec![
        fixed_circle(),
        parts,
        vec![json!({ "id": "k", "type": "tangent", "a": "c", "b": "a" })],
    ]));
    let d = dist(xy(&resp, "o"), xy(&resp, "a_c"));
    assert!((d - 4.0).abs() < 1e-8, "d = {d}");
    let phi = direction(&resp, "a_c", "o");
    assert!(on_span(&resp, "a", phi, 1e-8), "tangency at {phi} is off the span");
}

// ── Arc–arc ────────────────────────────────────────────────────────────────

#[test]
fn two_arcs_touch_externally_and_internally() {
    let resp = solve(sketch(vec![
        arc("a", (0.0, 0.0), 3.0, -0.5, 0.5, true),
        arc("b", (4.5, 0.4), 1.0, PI - 0.6, PI + 0.6, false),
        vec![json!({ "id": "k", "type": "tangent", "a": "b", "b": "a" })],
    ]));
    let d = dist(xy(&resp, "a_c"), xy(&resp, "b_c"));
    assert!((d - num(&resp, "a", "radius") - num(&resp, "b", "radius")).abs() < 1e-8);
    assert!(on_span(&resp, "a", direction(&resp, "a_c", "b_c"), 1e-8));
    assert!(on_span(&resp, "b", direction(&resp, "b_c", "a_c"), 1e-8));

    let resp = solve(sketch(vec![
        arc("a", (0.0, 0.0), 3.0, -0.5, 0.5, true),
        arc("b", (1.5, 0.3), 1.0, -0.6, 0.6, false),
        vec![json!({ "id": "k", "type": "tangent", "a": "a", "b": "b", "internal": true })],
    ]));
    let d = dist(xy(&resp, "a_c"), xy(&resp, "b_c"));
    assert!((d - (num(&resp, "a", "radius") - num(&resp, "b", "radius")).abs()).abs() < 1e-8);
    assert!(on_span(&resp, "a", direction(&resp, "a_c", "b_c"), 1e-8));
    assert!(on_span(&resp, "b", direction(&resp, "a_c", "b_c"), 1e-8));
}

#[test]
fn internal_circles_touch_inside() {
    let resp = solve(sketch(vec![
        fixed_circle(),
        vec![
            json!({ "id": "o2", "type": "point", "x": 1.0, "y": 0.5 }),
            json!({ "id": "c2", "type": "circle", "c_id": "o2", "radius": 1.0, "fixed": true }),
            json!({ "id": "k", "type": "tangent", "a": "c", "b": "c2", "internal": true }),
        ],
    ]));
    assert!((dist(xy(&resp, "o"), xy(&resp, "o2")) - 2.0).abs() < 1e-8);
}

// ── Arcs sharing an endpoint ───────────────────────────────────────────────

/// Arcs `a` (center (0,0), radius 5) and `b`, which share the Point `s`
/// (`a`'s end, `b`'s start) at (0, 5). `b`'s center starts near `bc`.
fn joined_arcs(bc: (f64, f64), b_r: f64, b0: f64, b1: f64, internal: bool) -> Vec<Value> {
    let mut b = arc("b", bc, b_r, b0, b1, false);
    b.remove(1); // `b`'s start is `s`.
    b[2]["start_id"] = json!("s");
    vec![
        json!({ "id": "a_c", "type": "point", "x": 0.0, "y": 0.0 }),
        json!({ "id": "a_s", "type": "point", "x": 5.0, "y": 0.0 }),
        json!({ "id": "s", "type": "point", "x": 0.0, "y": 5.0 }),
        json!({ "id": "a", "type": "arc", "c_id": "a_c", "start_id": "a_s", "end_id": "s",
                "radius": 5.0, "start_angle": 0.0, "end_angle": FRAC_PI_2 }),
        b[0].clone(),
        b[1].clone(),
        b[2].clone(),
        json!({ "id": "k", "type": "tangent", "a": "a", "b": "b", "internal": internal }),
    ]
}

/// (s − a_c)·(s − b_c): negative when the centers are on opposite sides of
/// the shared point, positive on the same side.
fn side(resp: &Value) -> f64 {
    let (s, a, b) = (xy(resp, "s"), xy(resp, "a_c"), xy(resp, "b_c"));
    (s.0 - a.0) * (s.0 - b.0) + (s.1 - a.1) * (s.1 - b.1)
}

/// Sine of the angle between the two radii at the shared point.
fn radii_sine(resp: &Value) -> f64 {
    let (s, a, b) = (xy(resp, "s"), xy(resp, "a_c"), xy(resp, "b_c"));
    let (u, v) = ((s.0 - a.0, s.1 - a.1), (s.0 - b.0, s.1 - b.1));
    (u.0 * v.1 - u.1 * v.0) / (u.0.hypot(u.1) * v.0.hypot(v.1))
}

#[test]
fn arcs_sharing_an_endpoint_are_tangent_there() {
    // An S-bend: `b` curves the other way, its center above `s`.
    let resp = solve(Value::Array(joined_arcs((0.6, 8.2), 3.0, -FRAC_PI_2, 0.5, false)));
    assert!(radii_sine(&resp).abs() < 1e-8);
    assert!(side(&resp) < 0.0);
    // Free arcs joined at a point: 5 + 5 − 2 (shared point) − 1 (tangency).
    assert_eq!(resp["dof"], 7);
    assert_eq!(resp["redundant"], json!([]));
    assert_eq!(resp["conflicting"], json!([]));

    // Inside: `b` continues `a`'s turn with a smaller radius.
    let resp = solve(Value::Array(joined_arcs((0.3, 2.8), 2.0, FRAC_PI_2, 2.5, true)));
    assert!(radii_sine(&resp).abs() < 1e-8);
    assert!(side(&resp) > 0.0);
    assert_eq!(resp["dof"], 7);
    assert_eq!(resp["redundant"], json!([]));
}

/// A chain of three arcs, each sharing an endpoint with the next, the first
/// joint an S-bend and the second inside: correct `dof` and nothing
/// redundant, free and pinned, and at every step of a drag of its end.
#[test]
fn a_fillet_chain_of_arcs_reports_correct_dof_and_no_redundancy() {
    let chain = || {
        let mut prims = joined_arcs((0.6, 8.2), 3.0, -FRAC_PI_2, 0.5, false);
        // `c` starts at `b`'s end and turns inside it.
        prims.extend([
            json!({ "id": "c_c", "type": "point", "x": 1.8, "y": 7.9 }),
            json!({ "id": "c_e", "type": "point", "x": 2.0, "y": 10.0 }),
            json!({ "id": "c", "type": "arc", "c_id": "c_c", "start_id": "b_e", "end_id": "c_e",
                    "radius": 1.0, "start_angle": 0.5, "end_angle": 1.5 }),
            json!({ "id": "k2", "type": "tangent", "a": "b", "b": "c", "internal": true }),
        ]);
        prims
    };
    let resp = solve(Value::Array(chain()));
    // 3 free arcs (15) − 2 shared points (4) − 2 tangencies.
    assert_eq!(resp["dof"], 9);
    assert_eq!(resp["redundant"], json!([]));
    assert_eq!(resp["conflicting"], json!([]));

    // Pin `a` (its Points) and the radii of `b` and `c`: each center is then
    // on the line of the previous radius through the joint, at its radius,
    // leaving only the end angles of `b` and `c`.
    let mut prims = chain();
    for p in prims.iter_mut() {
        if p["id"] == "a_c" || p["id"] == "a_s" || p["id"] == "s" {
            p["fixed"] = json!(true);
        }
    }
    prims.extend([
        json!({ "id": "f1", "type": "radius", "curve": "b", "value": 3.0 }),
        json!({ "id": "f2", "type": "radius", "curve": "c", "value": 1.0 }),
    ]);
    let resp = solve(Value::Array(prims.clone()));
    assert_eq!(resp["redundant"], json!([]), "{resp}");
    assert_eq!(resp["conflicting"], json!([]));
    assert_eq!(resp["dof"], 2);

    let n = prims.len();
    let mut prims = Value::Array(resp["primitives"].as_array().unwrap()[..n].to_vec());
    for step in 0..10 {
        let (tx, ty) = (
            3.0 + 0.4 * (step as f64 * 0.7).sin(),
            9.0 + 0.4 * (step as f64 * 0.9).cos(),
        );
        let mut with_drag = prims.as_array().unwrap().clone();
        with_drag.push(json!({ "id": "dx", "type": "x", "point": "c_e", "value": tx, "temporary": true }));
        with_drag.push(json!({ "id": "dy", "type": "y", "point": "c_e", "value": ty, "temporary": true }));
        let resp = solve(Value::Array(with_drag));
        assert_eq!(resp["redundant"], json!([]), "step {step}: {}", resp["redundant"]);
        assert_eq!(resp["conflicting"], json!([]));
        assert_eq!(resp["dof"], 2);
        prims = Value::Array(resp["primitives"].as_array().unwrap()[..n].to_vec());
    }
}

// ── Rejections ─────────────────────────────────────────────────────────────

#[test]
fn internal_with_a_line_is_rejected() {
    let line = vec![
        json!({ "id": "l1", "type": "point", "x": -5.0, "y": 3.0 }),
        json!({ "id": "l2", "type": "point", "x": 5.0, "y": 3.0 }),
        json!({ "id": "l", "type": "line", "p1_id": "l1", "p2_id": "l2" }),
    ];
    for (a, b) in [("l", "c"), ("c", "l"), ("l", "a")] {
        let (error, id) = reject(sketch(vec![
            fixed_circle(),
            arc("a", (0.0, 8.0), 1.0, 3.5, 6.0, false),
            line.clone(),
            vec![json!({ "id": "k", "type": "tangent", "a": a, "b": b, "internal": true })],
        ]));
        assert_eq!(id, "k");
        assert!(
            error.contains("'internal' applies only to tangency between circles and arcs"),
            "{error}"
        );
    }
}

#[test]
fn internal_must_be_true_or_false_and_only_on_tangent() {
    let (error, _) = reject(sketch(vec![
        fixed_circle(),
        arc("a", (5.0, 0.0), 1.0, 2.5, 3.8, false),
        vec![json!({ "id": "k", "type": "tangent", "a": "c", "b": "a", "internal": "yes" })],
    ]));
    assert_eq!(error, "constraint k: field 'internal' is not true or false");

    let (error, _) = reject(sketch(vec![
        fixed_circle(),
        arc("a", (5.0, 0.0), 1.0, 2.5, 3.8, false),
        vec![json!({ "id": "k", "type": "concentric", "a": "c", "b": "a", "internal": true })],
    ]));
    assert!(error.contains("unsupported combination for 'concentric'"), "{error}");
    assert!(error.contains("internal"), "{error}");
}
