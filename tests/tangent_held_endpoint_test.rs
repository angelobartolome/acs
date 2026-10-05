//! A Line tangent to a circle (or arc) with one of its endpoints held on that
//! curve by `on` (issue #14). Through a point already on the curve, the
//! distance form of tangency only changes quadratically, so the endpoint
//! drifts micrometres from the tangent point and diagnosis reports spurious
//! Redundant constraints. The pair is built as tangency at that endpoint
//! (`TangentAtPoint`), which pins the endpoint to the tangent point.

use acs::sketch_solve::solve_sketch_json;
use serde_json::{Value, json};

const R1: f64 = 71.12530597767243;
const R2: f64 = 124.6;
const C2: (f64, f64) = (385.180931461, 53.474694022);

/// The exact tangent points of the upper external tangent of the two circles.
fn tangent_points() -> ((f64, f64), (f64, f64)) {
    let d = C2.0.hypot(C2.1);
    let (ux, uy) = (C2.0 / d, C2.1 / d);
    let cos_a = (R1 - R2) / d;
    let sin_a = (1.0 - cos_a * cos_a).sqrt();
    let (nx, ny) = (ux * cos_a - uy * sin_a, uy * cos_a + ux * sin_a);
    ((R1 * nx, R1 * ny), (C2.0 + R2 * nx, C2.1 + R2 * ny))
}

/// The issue's sketch: a line `top` from `t1` to `t2`, each endpoint `on` its
/// circle and the line `tangent` to both. `tangent` carries `extension`.
fn belt(t1: (f64, f64), t2: (f64, f64), extension: bool, extra: Value) -> String {
    let mut prims = json!([
        { "type": "point", "id": "c1", "x": 0.0, "y": 0.0, "fixed": true },
        { "type": "circle", "id": "C1", "c_id": "c1", "radius": R1 },
        { "type": "point", "id": "c2", "x": C2.0, "y": C2.1, "fixed": true },
        { "type": "circle", "id": "C2", "c_id": "c2", "radius": R2 },
        { "type": "point", "id": "t1", "x": t1.0, "y": t1.1 },
        { "type": "point", "id": "t2", "x": t2.0, "y": t2.1 },
        { "type": "line", "id": "top", "p1_id": "t1", "p2_id": "t2" },
        { "type": "on", "id": "o1", "point": "t1", "curve": "C1" },
        { "type": "on", "id": "o2", "point": "t2", "curve": "C2" },
        { "type": "tangent", "id": "k1", "a": "top", "b": "C1", "extension": extension },
        { "type": "tangent", "id": "k2", "a": "top", "b": "C2", "extension": extension },
        { "type": "radius", "id": "r1", "curve": "C1", "value": R1 },
        { "type": "radius", "id": "r2", "curve": "C2", "value": R2 }
    ]);
    prims.as_array_mut().unwrap().extend(extra.as_array().unwrap().clone());
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

fn starts() -> Vec<((f64, f64), (f64, f64))> {
    let (t1, t2) = tangent_points();
    let mut starts = vec![(
        (-18.875227787783853, 68.43544183088915),
        (351.23867491568944, 173.86248831571584),
    )];
    for (dx, dy) in [(0.0, 0.5), (2.0, -1.0), (5.0, 5.0)] {
        starts.push(((t1.0 + dx, t1.1 + dy), (t2.0 - dy, t2.1 + dx)));
    }
    starts.push((t1, t2));
    starts
}

fn assert_at_tangent_points(resp: &Value, what: &str) {
    let (t1, t2) = tangent_points();
    for (id, t) in [("t1", t1), ("t2", t2)] {
        let p = xy(resp, id);
        let err = (p.0 - t.0).hypot(p.1 - t.1);
        assert!(err < 1e-9, "{what}: {id} is {err:e} from its tangent point");
    }
}

#[test]
fn held_endpoints_land_on_the_tangent_points_from_any_start() {
    for extension in [false, true] {
        for (t1, t2) in starts() {
            let what = format!("start {t1:?} {t2:?}, extension {extension}");
            let resp = solve(&belt(t1, t2, extension, json!([])));
            assert_eq!(resp["status"], "converged", "{what}: {resp}");
            assert_at_tangent_points(&resp, &what);
            assert_eq!(resp["dof"], 0, "{what}");
            assert_eq!(resp["redundant"], json!([]), "{what}");
            assert_eq!(resp["conflicting"], json!([]), "{what}");
        }
    }
}

#[test]
fn a_genuinely_redundant_constraint_is_still_reported() {
    // A third `on` holding t1 on C1 again duplicates o1.
    let (t1, t2) = starts()[0];
    let extra = json!([{ "type": "on", "id": "o3", "point": "t1", "curve": "C1" }]);
    let resp = solve(&belt(t1, t2, false, extra));
    assert_eq!(resp["status"], "converged", "{resp}");
    assert_eq!(resp["redundant"], json!(["o3"]));
    assert_eq!(resp["dof"], 0);
}

#[test]
fn a_tangent_line_with_no_endpoint_on_the_circle_is_unchanged() {
    // One free line tangent to a fixed circle, its endpoints held at x = ±5
    // and nothing holding them on the circle: the line settles at y = r.
    let request = json!({ "version": 1, "primitives": [
        { "type": "point", "id": "c", "x": 0.0, "y": 0.0, "fixed": true },
        { "type": "circle", "id": "C", "c_id": "c", "radius": 2.0, "fixed": true },
        { "type": "point", "id": "a", "x": -5.0, "y": 2.4 },
        { "type": "point", "id": "b", "x": 5.0, "y": 2.1 },
        { "type": "line", "id": "l", "p1_id": "a", "p2_id": "b" },
        { "type": "x", "id": "xa", "point": "a", "value": -5.0 },
        { "type": "x", "id": "xb", "point": "b", "value": 5.0 },
        { "type": "horizontal", "id": "h", "line": "l" },
        { "type": "tangent", "id": "k", "a": "l", "b": "C" }
    ]})
    .to_string();
    let resp = solve(&request);
    assert_eq!(resp["status"], "converged", "{resp}");
    assert!((xy(&resp, "a").1 - 2.0).abs() < 1e-9, "{resp}");
    assert_eq!(resp["dof"], 0);
}

#[test]
fn an_endpoint_on_an_arc_is_held_at_the_tangent_point_too() {
    // The line ends at a point `on` the arc (not the arc's own endpoint).
    let r: f64 = 3.0;
    let request = |px: f64, py: f64| {
        json!({ "version": 1, "primitives": [
            { "type": "point", "id": "c", "x": 0.0, "y": 0.0, "fixed": true },
            { "type": "point", "id": "s", "x": r, "y": 0.0 },
            { "type": "point", "id": "e", "x": -r, "y": 0.0 },
            { "type": "arc", "id": "A", "c_id": "c", "start_id": "s", "end_id": "e",
              "radius": r, "start_angle": 0.0, "end_angle": std::f64::consts::PI, "fixed": true },
            { "type": "point", "id": "p", "x": px, "y": py },
            { "type": "point", "id": "q", "x": 9.0, "y": 7.0, "fixed": true },
            { "type": "line", "id": "l", "p1_id": "p", "p2_id": "q" },
            { "type": "on", "id": "o", "point": "p", "curve": "A" },
            { "type": "tangent", "id": "k", "a": "l", "b": "A" }
        ]})
        .to_string()
    };
    // From the fixed q = (9, 7), the tangent point on the upper half circle.
    let d2: f64 = 9.0 * 9.0 + 7.0 * 7.0;
    let (qx, qy) = (9.0, 7.0);
    let k = r * r / d2;
    let h = r * (d2 - r * r).sqrt() / d2;
    let candidates = [(k * qx - h * qy, k * qy + h * qx), (k * qx + h * qy, k * qy - h * qx)];
    let t = candidates.into_iter().find(|t| t.1 > 0.0).unwrap();
    for (px, py) in [(t.0 + 0.3, t.1 - 0.2), (t.0 - 0.4, t.1 + 0.1)] {
        let resp = solve(&request(px, py));
        assert_eq!(resp["status"], "converged", "{resp}");
        let p = xy(&resp, "p");
        assert!((p.0 - t.0).hypot(p.1 - t.1) < 1e-9, "p = {p:?}, tangent point {t:?}");
        assert_eq!(resp["redundant"], json!([]));
        assert_eq!(resp["dof"], 0);
    }
}

#[test]
fn a_temporary_on_does_not_change_the_tangent() {
    // A drag goal holding t1 on C1 is a soft goal, not a real `on`: the
    // tangent keeps its distance form and the sketch still converges.
    let (t1, t2) = starts()[0];
    let request = json!({ "version": 1, "primitives": [
        { "type": "point", "id": "c1", "x": 0.0, "y": 0.0, "fixed": true },
        { "type": "circle", "id": "C1", "c_id": "c1", "radius": R1, "fixed": true },
        { "type": "point", "id": "t1", "x": t1.0, "y": t1.1 },
        { "type": "point", "id": "t2", "x": t2.0, "y": t2.1, "fixed": true },
        { "type": "line", "id": "top", "p1_id": "t1", "p2_id": "t2" },
        { "type": "on", "id": "g", "point": "t1", "curve": "C1", "temporary": true },
        { "type": "tangent", "id": "k1", "a": "top", "b": "C1" }
    ]})
    .to_string();
    let resp = solve(&request);
    assert_eq!(resp["status"], "converged", "{resp}");
}
