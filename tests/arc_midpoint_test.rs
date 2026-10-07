//! `midpoint {entities: [point, arc]}`: the point is the middle of the arc's
//! span, `c + r·(cos m, sin m)` with `m = α + sweep/2`.

use acs::constraints::{ConstraintType, create_constraint, reads};
use acs::sketch_solve::solve_sketch_json;
use nalgebra::DMatrix;
use serde_json::{Value, json};
use std::f64::consts::PI;

/// An arc of radius `r` about (`cx`, `cy`) from angle `start` to `end`, all
/// of it fixed when `fixed`, a point `m` at (3, 4), and `rest`.
fn request(fixed: bool, (cx, cy, r): (f64, f64, f64), start: f64, end: f64, rest: Value) -> String {
    let mut prims = json!([
        { "id": "o", "type": "point", "x": cx, "y": cy, "fixed": fixed },
        { "id": "s", "type": "point", "x": cx + r * start.cos(), "y": cy + r * start.sin(), "fixed": fixed },
        { "id": "e", "type": "point", "x": cx + r * end.cos(), "y": cy + r * end.sin(), "fixed": fixed },
        { "id": "a", "type": "arc", "c_id": "o", "start_id": "s", "end_id": "e",
          "radius": r, "start_angle": start, "end_angle": end, "fixed": fixed },
        { "id": "m", "type": "point", "x": 3.0, "y": 4.0 },
        { "id": "k", "type": "midpoint", "entities": ["m", "a"] }
    ]);
    prims.as_array_mut().unwrap().extend(rest.as_array().unwrap().clone());
    json!({ "version": 1, "primitives": prims }).to_string()
}

fn solve(request: String) -> Value {
    let out = solve_sketch_json(&request).unwrap_or_else(|e| panic!("rejected: {e}"));
    let resp: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(resp["status"], "converged", "{resp}");
    assert_eq!(resp["conflicting"], json!([]), "{resp}");
    assert_eq!(resp["redundant"], json!([]), "{resp}");
    resp
}

fn num(resp: &Value, id: &str, key: &str) -> f64 {
    resp["primitives"].as_array().unwrap().iter().find(|p| p["id"] == id).unwrap()[key]
        .as_f64()
        .unwrap()
}

fn close(resp: &Value, id: &str, x: f64, y: f64) {
    let (px, py) = (num(resp, id, "x"), num(resp, id, "y"));
    assert!(
        (px - x).abs() < 1e-8 && (py - y).abs() < 1e-8,
        "{id} at ({px}, {py}), expected ({x}, {y})"
    );
}

/// The span's middle as the response has the arc.
fn span_middle(resp: &Value) -> (f64, f64) {
    let (start, end) = (num(resp, "a", "start_angle"), num(resp, "a", "end_angle"));
    let mut sweep = (end - start).rem_euclid(2.0 * PI);
    if sweep == 0.0 {
        sweep = 2.0 * PI;
    }
    let m = start + sweep / 2.0;
    let r = num(resp, "a", "radius");
    (num(resp, "o", "x") + r * m.cos(), num(resp, "o", "y") + r * m.sin())
}

#[test]
fn the_point_lands_at_the_middle_of_a_short_span() {
    let resp = solve(request(true, (1.0, 2.0, 2.0), 0.2, 1.2, json!([])));
    close(&resp, "m", 1.0 + 2.0 * 0.7f64.cos(), 2.0 + 2.0 * 0.7f64.sin());
}

#[test]
fn a_span_across_the_seam_has_its_middle_across_it() {
    // From 3 to −3: a short span through ±π, its middle at π, not at
    // (3 + −3)/2 = 0 on the other side.
    let resp = solve(request(true, (0.0, 0.0, 2.0), 3.0, -3.0, json!([])));
    close(&resp, "m", -2.0, 0.0);
}

#[test]
fn a_span_past_a_half_turn_across_the_seam() {
    // From 1 to −1: a sweep of 2π − 2 > π through ±π, its middle at π.
    let resp = solve(request(true, (0.0, 0.0, 2.0), 1.0, -1.0, json!([])));
    close(&resp, "m", -2.0, 0.0);
    // From −1 to 1: the complementary span, its middle at 0.
    let resp = solve(request(true, (0.0, 0.0, 2.0), -1.0, 1.0, json!([])));
    close(&resp, "m", 2.0, 0.0);
    // From 0.5 to 4.5: a sweep of 4 > π, its middle at 2.5.
    let resp = solve(request(true, (0.0, 0.0, 2.0), 0.5, 4.5, json!([])));
    close(&resp, "m", 2.0 * 2.5f64.cos(), 2.0 * 2.5f64.sin());
}

#[test]
fn moving_the_arc_carries_the_point() {
    // The center moved to (5, −1), the radius set to 3: the point follows.
    let resp = solve(request(
        false,
        (0.0, 0.0, 2.0),
        0.3,
        2.0,
        json!([
            { "id": "kx", "type": "x", "point": "o", "value": 5.0 },
            { "id": "ky", "type": "y", "point": "o", "value": -1.0 },
            { "id": "kr", "type": "radius", "curve": "a", "value": 3.0 }
        ]),
    ));
    close(&resp, "o", 5.0, -1.0);
    assert!((num(&resp, "a", "radius") - 3.0).abs() < 1e-8);
    let (x, y) = span_middle(&resp);
    close(&resp, "m", x, y);
}

#[test]
fn a_free_point_at_the_middle_of_a_free_arc_loses_two_dof() {
    // A free arc (5 DOF) and a free point (2): 7, less 2 for the midpoint.
    let resp = solve(request(false, (0.0, 0.0, 2.0), 0.3, 2.0, json!([])));
    assert_eq!(resp["dof"], 5, "{resp}");
    let (x, y) = span_middle(&resp);
    close(&resp, "m", x, y);
}

#[test]
fn the_middle_of_a_fixed_arc_is_fully_constrained() {
    let resp = solve(request(true, (0.0, 0.0, 2.0), 0.3, 2.0, json!([])));
    assert_eq!(resp["dof"], 0, "{resp}");
    let ids: Vec<&str> = resp["fullyConstrained"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(ids.contains(&"m"), "{resp}");
}

#[test]
fn the_arc_comes_second() {
    let mut req: Value = serde_json::from_str(&request(true, (0.0, 0.0, 2.0), 0.3, 2.0, json!([])))
        .unwrap();
    let k = req["primitives"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|p| p["id"] == "k")
        .unwrap();
    k["entities"] = json!(["a", "m"]);
    let out = solve_sketch_json(&req.to_string()).expect_err("[arc, point] is rejected");
    let resp: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(resp["constraintId"], "k");
    assert!(
        resp["error"].as_str().unwrap().contains("(entities[0]: point, entities[1]: arc)"),
        "{resp}"
    );
}

/// The wrap: as the end angle passes the start, the sweep jumps between 2π
/// and 0 and the middle flips to the opposite side; on either side of it the
/// partials of m are ½.
#[test]
fn the_middle_flips_where_the_sweep_wraps() {
    let c = create_constraint(ConstraintType::MidpointOfArc("p".into(), "c".into(), "a".into()))
        .unwrap();
    let eval = |start: f64, end: f64| {
        let mut r = vec![0.0; 2];
        let mut j = DMatrix::zeros(2, reads(c.as_ref()).len());
        // The point at the origin and the center there too: R = −r·(cos m, sin m).
        c.eval(&[0.0, 0.0, 0.0, 0.0, 1.0, start, end], &mut r, &mut j);
        (r, j)
    };
    // Nearly a full turn: the middle near π.
    let (full, j) = eval(0.0, -0.01);
    assert!((full[0] - 1.0).abs() < 1e-3, "{full:?}");
    // Nearly none: the middle near 0.
    let (none, _) = eval(0.0, 0.01);
    assert!((none[0] + 1.0).abs() < 1e-3, "{none:?}");
    // ∂R₁/∂α = ∂R₁/∂β = −r cos(m)/2.
    let m = PI - 0.005;
    assert!((j[(1, 5)] + 0.5 * m.cos()).abs() < 1e-12);
    assert!((j[(1, 6)] + 0.5 * m.cos()).abs() < 1e-12);
}
