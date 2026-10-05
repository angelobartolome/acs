//! `angle {arc, value}`: an arc's sweep, `0 < value < 2π`.

use acs::constraints::{ConstraintType, create_constraint, reads};
use acs::sketch_solve::solve_sketch_json;
use nalgebra::DMatrix;
use serde_json::{Value, json};
use std::f64::consts::PI;

/// An arc of radius 2 about the fixed origin from angle `start` (its start
/// Point fixed) to `end`, plus `rest`.
fn request(rest: Value, start: f64, end: f64) -> String {
    let mut prims = json!([
        { "id": "o", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "s", "type": "point", "x": 2.0 * start.cos(), "y": 2.0 * start.sin(), "fixed": true },
        { "id": "e", "type": "point", "x": 2.0 * end.cos(), "y": 2.0 * end.sin() },
        { "id": "a", "type": "arc", "c_id": "o", "start_id": "s", "end_id": "e",
          "radius": 2.0, "start_angle": start, "end_angle": end }
    ]);
    prims.as_array_mut().unwrap().extend(rest.as_array().unwrap().clone());
    json!({ "version": 1, "primitives": prims }).to_string()
}

fn solve(rest: Value, start: f64, end: f64) -> Value {
    let out = solve_sketch_json(&request(rest, start, end))
        .unwrap_or_else(|e| panic!("rejected: {e}"));
    let resp: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(resp["status"], "converged", "{resp}");
    resp
}

fn num(resp: &Value, id: &str, key: &str) -> f64 {
    resp["primitives"].as_array().unwrap().iter().find(|p| p["id"] == id).unwrap()[key]
        .as_f64()
        .unwrap()
}

fn sweep(resp: &Value) -> f64 {
    (num(resp, "a", "end_angle") - num(resp, "a", "start_angle")).rem_euclid(2.0 * PI)
}

fn sweep_of(value: f64) -> Value {
    json!([{ "id": "k", "type": "angle", "arc": "a", "value": value }])
}

#[test]
fn an_arc_sweep() {
    let resp = solve(sweep_of(PI / 2.0), 0.0, 0.4);
    assert!((sweep(&resp) - PI / 2.0).abs() < 1e-9);
    assert!(num(&resp, "e", "x").abs() < 1e-8);
    assert!((num(&resp, "e", "y") - 2.0).abs() < 1e-8);
}

#[test]
fn a_sweep_is_reached_across_the_seam() {
    // A nearly full arc asked for a small sweep, and a small one asked for a
    // nearly full sweep: both cross the 0/2π seam on the way.
    let resp = solve(sweep_of(0.1), 0.0, -0.1);
    assert!((sweep(&resp) - 0.1).abs() < 1e-9, "{}", sweep(&resp));
    let resp = solve(sweep_of(6.0), 0.0, 0.05);
    assert!((sweep(&resp) - 6.0).abs() < 1e-9, "{}", sweep(&resp));
}

/// The residual is the sweep unwrapped near the target, so it changes
/// continuously as the end angle crosses the seam.
#[test]
fn the_residual_is_continuous_across_the_seam() {
    let value = 6.25;
    let c = create_constraint(ConstraintType::ArcSweep("a".into(), value)).unwrap();
    let eval = |start: f64, end: f64| {
        let mut r = vec![0.0; 1];
        let mut j = DMatrix::zeros(1, reads(c.as_ref()).len());
        c.eval(&[start, end], &mut r, &mut j);
        (r[0], j)
    };
    let (below, _) = eval(0.0, 2.0 * PI - 0.01);
    let (above, j) = eval(0.0, 2.0 * PI + 0.01);
    let (wrapped, _) = eval(0.0, 0.01);
    assert!((below - (2.0 * PI - 0.01 - value)).abs() < 1e-12);
    assert!((above - (2.0 * PI + 0.01 - value)).abs() < 1e-12);
    assert!((wrapped - above).abs() < 1e-12, "the same arc, the same residual");
    assert_eq!(j.row(0).iter().copied().collect::<Vec<_>>(), vec![-1.0, 1.0]);
}

#[test]
fn a_sweep_outside_zero_to_two_pi_is_rejected() {
    for value in [0.0, -1.0, 2.0 * PI, 7.0] {
        let out = solve_sketch_json(&request(sweep_of(value), 0.0, 1.0))
            .expect_err("out-of-range sweep must be rejected");
        let resp: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(resp["status"], "invalid");
        assert_eq!(resp["constraintId"], "k");
        assert!(
            resp["error"].as_str().unwrap().contains("between 0 and 2π"),
            "{resp}"
        );
    }
}

#[test]
fn angle_between_lines_still_works_beside_the_arc_variant() {
    let request = json!({ "version": 1, "primitives": [
        { "id": "p1", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "p2", "type": "point", "x": 1.0, "y": 0.0, "fixed": true },
        { "id": "p3", "type": "point", "x": 1.0, "y": 0.5 },
        { "id": "l1", "type": "line", "p1_id": "p1", "p2_id": "p2" },
        { "id": "l2", "type": "line", "p1_id": "p1", "p2_id": "p3" },
        { "id": "k", "type": "angle", "a": "l1", "b": "l2", "value": 0.5 }
    ]})
    .to_string();
    let resp: Value = serde_json::from_str(&solve_sketch_json(&request).unwrap()).unwrap();
    assert_eq!(resp["status"], "converged");
}
