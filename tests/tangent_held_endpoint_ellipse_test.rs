//! A Line tangent to an ellipse (or elliptical arc) with one of its
//! endpoints held on that curve by `on`: the ellipse version of issue #14.
//! Through a point already on the curve, the line–ellipse tangency residual
//! only changes quadratically as the endpoint slides along the Line, so the
//! endpoint would drift from the tangent point and diagnosis would report
//! spurious Redundant constraints. The pair is built as tangency at that
//! endpoint (`TangentLineEllipseAtPoint`), which pins it to the tangent
//! point.

use acs::sketch_solve::solve_sketch_json;
use serde_json::{Value, json};

// Center (0, 0), focus (3, 0), radmin 4: a = 5, P(t) = (5 cos t, 4 sin t).
const A: f64 = 5.0;
const B: f64 = 4.0;
/// Parametric angle of the tangent point.
const T: f64 = 1.0;
/// The elliptical arc's span, around `T`.
const T0: f64 = 0.3;
const T1: f64 = 2.0;

fn at(t: f64) -> (f64, f64) {
    (A * t.cos(), B * t.sin())
}

/// The tangent point, and the fixed far end `q` of the Line: 6 along the
/// tangent from it.
fn tangent_point_and_q() -> ((f64, f64), (f64, f64)) {
    let p = at(T);
    let (tx, ty) = (-A * T.sin(), B * T.cos());
    let n = tx.hypot(ty);
    (p, (p.0 + 6.0 * tx / n, p.1 + 6.0 * ty / n))
}

/// The curve `E`: a fixed ellipse, or a fixed elliptical arc from T0 to T1.
fn curve(arc: bool) -> Vec<Value> {
    let mut prims = vec![
        json!({ "id": "c", "type": "point", "x": 0.0, "y": 0.0, "fixed": true }),
        json!({ "id": "f", "type": "point", "x": 3.0, "y": 0.0, "fixed": true }),
    ];
    if arc {
        let ((sx, sy), (ex, ey)) = (at(T0), at(T1));
        prims.extend([
            json!({ "id": "s", "type": "point", "x": sx, "y": sy, "fixed": true }),
            json!({ "id": "e", "type": "point", "x": ex, "y": ey, "fixed": true }),
            json!({ "id": "E", "type": "elliptical_arc", "c_id": "c", "focus1_id": "f",
                    "start_id": "s", "end_id": "e", "radmin": B,
                    "start_angle": T0, "end_angle": T1, "fixed": true }),
        ]);
    } else {
        prims.push(json!({ "id": "E", "type": "ellipse", "c_id": "c", "focus1_id": "f",
                           "radmin": B, "fixed": true }));
    }
    prims
}

/// A Line `l` from the free `p` (starting at `start`) to the fixed `q`, `p`
/// `on` E and `l` `tangent` to E, plus `extra`.
fn request(arc: bool, start: (f64, f64), extra: Value) -> String {
    let (_, q) = tangent_point_and_q();
    let mut prims = curve(arc);
    prims.extend([
        json!({ "id": "p", "type": "point", "x": start.0, "y": start.1 }),
        json!({ "id": "q", "type": "point", "x": q.0, "y": q.1, "fixed": true }),
        json!({ "id": "l", "type": "line", "p1_id": "p", "p2_id": "q" }),
        json!({ "id": "o", "type": "on", "point": "p", "curve": "E" }),
        json!({ "id": "k", "type": "tangent", "a": "l", "b": "E" }),
    ]);
    prims.extend(extra.as_array().unwrap().clone());
    json!({ "version": 1, "primitives": prims }).to_string()
}

fn solve(request: &str) -> Value {
    let out = solve_sketch_json(request).unwrap_or_else(|e| panic!("rejected: {e}"));
    serde_json::from_str(&out).unwrap()
}

fn xy(resp: &Value, id: &str) -> (f64, f64) {
    let p = resp["primitives"].as_array().unwrap().iter().find(|p| p["id"] == id).unwrap();
    (p["x"].as_f64().unwrap(), p["y"].as_f64().unwrap())
}

/// Starts around the tangent point, on and off the curve, and exactly on it.
fn starts() -> Vec<(f64, f64)> {
    let (t, _) = tangent_point_and_q();
    let mut starts: Vec<(f64, f64)> = [(0.0, 0.5), (0.3, -0.2), (-0.4, 0.1), (0.05, 0.05)]
        .into_iter()
        .map(|(dx, dy)| (t.0 + dx, t.1 + dy))
        .collect();
    starts.extend([at(T + 0.1), at(T - 0.15), t]);
    starts
}

#[test]
fn a_held_endpoint_lands_on_the_tangent_point_from_any_start() {
    let (t, _) = tangent_point_and_q();
    for arc in [false, true] {
        for start in starts() {
            let what = format!("arc {arc}, start {start:?}");
            let resp = solve(&request(arc, start, json!([])));
            assert_eq!(resp["status"], "converged", "{what}: {resp}");
            let p = xy(&resp, "p");
            let err = (p.0 - t.0).hypot(p.1 - t.1);
            assert!(err < 1e-9, "{what}: p is {err:e} from the tangent point");
            assert_eq!(resp["dof"], 0, "{what}: {resp}");
            assert_eq!(resp["redundant"], json!([]), "{what}");
            assert_eq!(resp["conflicting"], json!([]), "{what}");
        }
    }
}

#[test]
fn a_genuinely_redundant_on_is_still_reported() {
    for arc in [false, true] {
        let extra = json!([{ "id": "o2", "type": "on", "point": "p", "curve": "E" }]);
        let resp = solve(&request(arc, starts()[0], extra));
        assert_eq!(resp["status"], "converged", "arc {arc}: {resp}");
        assert_eq!(resp["redundant"], json!(["o2"]), "arc {arc}");
        assert_eq!(resp["dof"], 0, "arc {arc}");
    }
}

#[test]
fn a_tangent_line_with_no_endpoint_on_the_ellipse_is_unchanged() {
    // A horizontal line with its endpoints held at x = ±7 and nothing
    // holding them on the curve settles on the top of the ellipse, y = b.
    for arc in [false, true] {
        let mut prims = curve(arc);
        prims.extend([
            json!({ "id": "a", "type": "point", "x": -7.0, "y": 4.4 }),
            json!({ "id": "b", "type": "point", "x": 7.0, "y": 4.3 }),
            json!({ "id": "l", "type": "line", "p1_id": "a", "p2_id": "b" }),
            json!({ "id": "xa", "type": "x", "point": "a", "value": -7.0 }),
            json!({ "id": "xb", "type": "x", "point": "b", "value": 7.0 }),
            json!({ "id": "h", "type": "horizontal", "line": "l" }),
            json!({ "id": "k", "type": "tangent", "a": "l", "b": "E" }),
        ]);
        let resp = solve(&json!({ "version": 1, "primitives": prims }).to_string());
        assert_eq!(resp["status"], "converged", "arc {arc}: {resp}");
        assert!((xy(&resp, "a").1 - B).abs() < 1e-9, "arc {arc}: {resp}");
        assert_eq!(resp["dof"], 0, "arc {arc}");
    }
}

#[test]
fn a_temporary_on_does_not_change_the_tangent() {
    // A drag goal holding p on E is a soft goal, not a real `on`.
    for arc in [false, true] {
        let (t, q) = tangent_point_and_q();
        let mut prims = curve(arc);
        prims.extend([
            json!({ "id": "p", "type": "point", "x": t.0 + 0.2, "y": t.1 + 0.1 }),
            json!({ "id": "q", "type": "point", "x": q.0, "y": q.1, "fixed": true }),
            json!({ "id": "l", "type": "line", "p1_id": "p", "p2_id": "q" }),
            json!({ "id": "g", "type": "on", "point": "p", "curve": "E", "temporary": true }),
            json!({ "id": "k", "type": "tangent", "a": "l", "b": "E" }),
        ]);
        let resp = solve(&json!({ "version": 1, "primitives": prims }).to_string());
        assert_eq!(resp["status"], "converged", "arc {arc}: {resp}");
    }
}
