//! The `elliptical_arc` primitive: an Ellipse's center, focus and `radmin`
//! plus start and end Points at parametric angles, sweeping counter-
//! clockwise like an Arc. Its implicit rules hold the endpoints on it; `on`
//! measures against its span; `tangent` takes a Line (on the segment and the
//! span, or at a shared endpoint) and an Arc at a shared endpoint.

use acs::sketch_solve::solve_sketch_json;
use serde_json::{Value, json};

// The ellipse used throughout: center (0, 0), focus (3, 0), radmin 4, so
// a = 5 and P(t) = (5 cos t, 4 sin t).
const A: f64 = 5.0;
const B: f64 = 4.0;
const T0: f64 = 0.3;
const T1: f64 = 2.0;

fn at(t: f64) -> (f64, f64) {
    (A * t.cos(), B * t.sin())
}

/// The elliptical arc `E` from T0 to T1, its center `c`, focus `f` and
/// endpoints `s`, `e` at their angles; `fixed` holds all of it.
fn elliptical_arc(fixed: bool) -> Vec<Value> {
    let ((sx, sy), (ex, ey)) = (at(T0), at(T1));
    vec![
        json!({ "id": "c", "type": "point", "x": 0.0, "y": 0.0, "fixed": fixed }),
        json!({ "id": "f", "type": "point", "x": 3.0, "y": 0.0, "fixed": fixed }),
        json!({ "id": "s", "type": "point", "x": sx, "y": sy, "fixed": fixed }),
        json!({ "id": "e", "type": "point", "x": ex, "y": ey, "fixed": fixed }),
        json!({ "id": "E", "type": "elliptical_arc", "c_id": "c", "focus1_id": "f",
                "start_id": "s", "end_id": "e", "radmin": B,
                "start_angle": T0, "end_angle": T1, "fixed": fixed }),
    ]
}

fn request(mut prims: Vec<Value>, rest: Value) -> String {
    prims.extend(rest.as_array().unwrap().clone());
    json!({ "version": 1, "primitives": prims }).to_string()
}

fn solve(request: &str) -> Value {
    let out = solve_sketch_json(request).unwrap_or_else(|e| panic!("rejected: {e}"));
    serde_json::from_str(&out).unwrap()
}

fn converged(request: &str) -> Value {
    let resp = solve(request);
    assert_eq!(resp["status"], "converged", "{resp}");
    resp
}

fn primitive<'a>(resp: &'a Value, id: &str) -> &'a Value {
    resp["primitives"].as_array().unwrap().iter().find(|p| p["id"] == id).unwrap()
}

fn xy(resp: &Value, id: &str) -> (f64, f64) {
    let p = primitive(resp, id);
    (p["x"].as_f64().unwrap(), p["y"].as_f64().unwrap())
}

fn num(resp: &Value, id: &str, key: &str) -> f64 {
    primitive(resp, id)[key].as_f64().unwrap()
}

fn close(p: (f64, f64), q: (f64, f64), what: &str) {
    let d = (p.0 - q.0).hypot(p.1 - q.1);
    assert!(d < 1e-8, "{what}: {p:?} vs {q:?} ({d:e} apart)");
}

/// The point at parametric angle `t` on the solved ellipse of `E`.
fn solved_at(resp: &Value, t: f64) -> (f64, f64) {
    let (c, f) = (xy(resp, "c"), xy(resp, "f"));
    let b = num(resp, "E", "radmin");
    let (dx, dy) = (f.0 - c.0, f.1 - c.1);
    let k = dx.hypot(dy);
    let a = (b * b + k * k).sqrt();
    let (ux, uy) = (dx / k, dy / k);
    let (nx, ny) = (-uy, ux);
    (
        c.0 + a * t.cos() * ux + b * t.sin() * nx,
        c.1 + a * t.cos() * uy + b * t.sin() * ny,
    )
}

#[test]
fn a_free_elliptical_arc_has_seven_degrees_of_freedom() {
    // center 2, focus 2, radmin, two angles; the endpoints follow.
    let resp = converged(&request(elliptical_arc(false), json!([])));
    assert_eq!(resp["dof"], 7);
    assert_eq!(resp["fullyConstrained"], json!([]));
}

#[test]
fn a_fully_fixed_elliptical_arc_is_held_and_has_no_freedom() {
    for flag in ["fixed", "isReference"] {
        let mut prims = elliptical_arc(false);
        for p in &mut prims {
            p[flag] = json!(true);
        }
        let resp = converged(&request(prims, json!([])));
        assert_eq!(resp["dof"], 0, "{flag}");
        assert_eq!(num(&resp, "E", "radmin"), B);
        assert_eq!(num(&resp, "E", "start_angle"), T0);
        assert_eq!(num(&resp, "E", "end_angle"), T1);
        close(xy(&resp, "s"), at(T0), flag);
        let fully: Vec<&str> = resp["fullyConstrained"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert!(fully.contains(&"E"), "{flag}: {fully:?}");
    }
}

#[test]
fn the_endpoints_follow_the_elliptical_arc_and_output_mirrors_input() {
    // Move the start Point onto a fixed point: the free elliptical arc
    // (center fixed) reshapes or turns so that its start is there, and the
    // response carries its solved radmin and angles.
    let resp = converged(&request(
        elliptical_arc(false),
        json!([
            { "id": "q", "type": "point", "x": 4.0, "y": 3.5, "fixed": true },
            { "id": "k1", "type": "coincident", "a": "s", "b": "q" },
            { "id": "k2", "type": "x", "point": "c", "value": 0.0 },
            { "id": "k3", "type": "y", "point": "c", "value": 0.0 }
        ]),
    ));
    close(xy(&resp, "s"), (4.0, 3.5), "start");
    let alpha = num(&resp, "E", "start_angle");
    let beta = num(&resp, "E", "end_angle");
    close(solved_at(&resp, alpha), (4.0, 3.5), "start at its angle");
    close(solved_at(&resp, beta), xy(&resp, "e"), "end at its angle");
}

#[test]
fn endpoints_coincident_with_fixed_points() {
    // A fixed ellipse frame (center, focus, radmin pinned by `equal`), its
    // endpoints coincident with two fixed points on that ellipse: the angles
    // turn to put them there.
    let (p, q) = (at(0.9), at(2.6));
    let resp = converged(&request(
        elliptical_arc(false),
        json!([
            { "id": "P", "type": "point", "x": p.0, "y": p.1, "fixed": true },
            { "id": "Q", "type": "point", "x": q.0, "y": q.1, "fixed": true },
            { "id": "k1", "type": "coincident", "a": "s", "b": "P" },
            { "id": "k2", "type": "coincident", "a": "e", "b": "Q" },
            { "id": "kc", "type": "coincident", "a": "c", "b": "C0" },
            { "id": "C0", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
            { "id": "F0", "type": "point", "x": 3.0, "y": 0.0, "fixed": true },
            { "id": "kf", "type": "coincident", "a": "f", "b": "F0" }
        ]),
    ));
    close(xy(&resp, "s"), p, "start");
    close(xy(&resp, "e"), q, "end");
    assert!((num(&resp, "E", "radmin") - B).abs() < 1e-8);
    assert!((num(&resp, "E", "start_angle") - 0.9).abs() < 1e-8);
    assert!((num(&resp, "E", "end_angle") - 2.6).abs() < 1e-8);
}

#[test]
fn a_point_on_an_elliptical_arc_lands_on_its_span() {
    // A free point starting on the ellipse beyond the end (t = 2.8) is
    // pulled back onto the span (just inside its end: the span check is 0
    // anywhere within it, as for an Arc), not left on the ellipse past it.
    // So is one starting before the start, and one off the ellipse within
    // the span lands on it there.
    for (start, near) in [(at(2.8), Some(T1)), (at(-0.4), Some(T0)), ((2.0, 2.0), None)] {
        let resp = converged(&request(
            elliptical_arc(true),
            json!([
                { "id": "p", "type": "point", "x": start.0, "y": start.1 },
                { "id": "k", "type": "on", "point": "p", "curve": "E" }
            ]),
        ));
        let (x, y) = xy(&resp, "p");
        assert!(((x / A).powi(2) + (y / B).powi(2) - 1.0).abs() < 1e-8, "({x}, {y})");
        let t = (y / B).atan2(x / A);
        assert!((T0 - 1e-9..=T1 + 1e-9).contains(&t), "from {start:?}: t = {t}");
        if let Some(end) = near {
            assert!((t - end).abs() < 0.05, "from {start:?}: t = {t}, not near {end}");
        }
    }
}

/// Distance from (x, y) to the infinite line through `a` and `b`.
fn line_distance(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    ((p.0 - a.0) * dy - (p.1 - a.1) * dx).abs() / dx.hypot(dy)
}

#[test]
fn a_line_tangent_to_an_elliptical_arc_touches_it_on_its_span() {
    // A line from a fixed point, its other end free, turned until it
    // touches the fixed elliptical arc within its span.
    let resp = converged(&request(
        elliptical_arc(true),
        json!([
            { "id": "a", "type": "point", "x": -8.0, "y": 6.0, "fixed": true },
            { "id": "b", "type": "point", "x": 8.0, "y": 4.0 },
            { "id": "l", "type": "line", "p1_id": "a", "p2_id": "b" },
            { "id": "xb", "type": "x", "point": "b", "value": 8.0 },
            { "id": "k", "type": "tangent", "a": "l", "b": "E" }
        ]),
    ));
    let (a, b) = (xy(&resp, "a"), xy(&resp, "b"));
    // Closest approach over the span is 0, at an interior angle.
    let (mut best, mut best_t) = (f64::INFINITY, 0.0);
    for i in 0..=200_000 {
        let t = T0 + (T1 - T0) * i as f64 / 200_000.0;
        let d = line_distance(at(t), a, b);
        if d < best {
            (best, best_t) = (d, t);
        }
    }
    assert!(best < 1e-6, "closest {best} at t = {best_t}");
    assert!(best_t > T0 + 1e-3 && best_t < T1 - 1e-3, "t = {best_t}");
    // And the whole ellipse is on one side: tangent, not crossing.
    let side = |t: f64| {
        let p = at(t);
        (p.0 - a.0) * (b.1 - a.1) - (p.1 - a.1) * (b.0 - a.0)
    };
    let s0 = side(best_t + 0.5).signum();
    for i in 0..=2000 {
        let t = i as f64 * std::f64::consts::TAU / 2000.0;
        assert!(side(t) * s0 > -1e-6, "crosses at t = {t}");
    }
}

#[test]
fn a_line_runs_tangent_off_an_elliptical_arc_endpoint() {
    // The line starts at the elliptical arc's end Point; its other end is
    // free (held at a distance): it leaves along the tangent there.
    let resp = converged(&request(
        elliptical_arc(true),
        json!([
            { "id": "q", "type": "point", "x": -9.0, "y": 0.0 },
            { "id": "l", "type": "line", "p1_id": "e", "p2_id": "q" },
            { "id": "d", "type": "length", "curve": "l", "value": 6.0 },
            { "id": "k", "type": "tangent", "a": "l", "b": "E" }
        ]),
    ));
    let (e, q) = (xy(&resp, "e"), xy(&resp, "q"));
    let tangent = (-A * T1.sin(), B * T1.cos());
    let dir = (q.0 - e.0, q.1 - e.1);
    let cross = (dir.0 * tangent.1 - dir.1 * tangent.0) / (dir.0.hypot(dir.1) * tangent.0.hypot(tangent.1));
    assert!(cross.abs() < 1e-9, "sin of the angle off the tangent: {cross}");
    assert_eq!(resp["redundant"], json!([]));
}

#[test]
fn an_arc_runs_tangent_off_an_elliptical_arc_endpoint() {
    // A circular arc starting at the elliptical arc's end Point: tangent
    // there means its radius at that point is normal to the ellipse.
    let (ex, ey) = at(T1);
    let resp = converged(&request(
        elliptical_arc(true),
        json!([
            { "id": "ac", "type": "point", "x": ex - 1.0, "y": ey - 2.5 },
            { "id": "ae", "type": "point", "x": ex - 3.0, "y": ey - 3.0 },
            { "id": "A1", "type": "arc", "c_id": "ac", "start_id": "e", "end_id": "ae",
              "radius": 2.7, "start_angle": 1.2, "end_angle": 2.8 },
            { "id": "r", "type": "radius", "curve": "A1", "value": 2.0 },
            { "id": "k", "type": "tangent", "a": "A1", "b": "E" }
        ]),
    ));
    let (e, ac) = (xy(&resp, "e"), xy(&resp, "ac"));
    let tangent = (-A * T1.sin(), B * T1.cos());
    let radius = (e.0 - ac.0, e.1 - ac.1);
    let cos = (radius.0 * tangent.0 + radius.1 * tangent.1)
        / (radius.0.hypot(radius.1) * tangent.0.hypot(tangent.1));
    assert!(cos.abs() < 1e-9, "radius not normal to the ellipse: cos {cos}");
}

#[test]
fn an_arc_tangent_to_an_elliptical_arc_needs_a_shared_endpoint() {
    let out = solve_sketch_json(&request(
        elliptical_arc(true),
        json!([
            { "id": "ac", "type": "point", "x": 0.0, "y": 9.0 },
            { "id": "as", "type": "point", "x": 2.0, "y": 9.0 },
            { "id": "ae", "type": "point", "x": -2.0, "y": 9.0 },
            { "id": "A1", "type": "arc", "c_id": "ac", "start_id": "as", "end_id": "ae",
              "radius": 2.0, "start_angle": 0.0, "end_angle": std::f64::consts::PI },
            { "id": "k", "type": "tangent", "a": "A1", "b": "E" }
        ]),
    ))
    .unwrap_err();
    assert!(out.contains("an arc and an elliptical arc are tangent only at an endpoint they share"), "{out}");
    assert!(out.contains("\"constraintId\":\"k\""), "{out}");
}

#[test]
fn ellipse_axis_takes_an_elliptical_arc() {
    // The +major end of the arc's ellipse, (5, 0), though the span doesn't
    // reach it: `ellipse_axis` is about the ellipse.
    let resp = converged(&request(
        elliptical_arc(true),
        json!([
            { "id": "p", "type": "point", "x": 4.0, "y": 0.5 },
            { "id": "k", "type": "ellipse_axis", "ellipse": "E", "point": "p", "which": "major" }
        ]),
    ));
    close(xy(&resp, "p"), (5.0, 0.0), "axis end");
}

#[test]
fn an_elliptical_arc_needs_its_points() {
    let mut prims = elliptical_arc(false);
    prims[4]["end_id"] = json!("ghost");
    let out = solve_sketch_json(&request(prims, json!([]))).unwrap_err();
    assert!(out.contains("elliptical_arc E: 'ghost' is not a point"), "{out}");

    let mut prims = elliptical_arc(false);
    prims[4].as_object_mut().unwrap().remove("start_angle");
    let out = solve_sketch_json(&request(prims, json!([]))).unwrap_err();
    assert!(out.contains("elliptical_arc E: missing field 'start_angle'"), "{out}");
}
