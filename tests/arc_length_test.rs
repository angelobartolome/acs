//! `length {curve, value}`: a Line's length, or an Arc's length `r · sweep`.

use acs::constraints::{ConstraintType, create_constraint, reads};
use acs::sketch_solve::solve_sketch_json;
use nalgebra::DMatrix;
use serde_json::{Value, json};
use std::f64::consts::PI;

fn solve(primitives: Value) -> Value {
    let request = json!({ "version": 1, "primitives": primitives }).to_string();
    let out = solve_sketch_json(&request).unwrap_or_else(|e| panic!("rejected: {e}"));
    let resp: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(resp["status"], "converged", "{resp}");
    resp
}

fn num(resp: &Value, id: &str, key: &str) -> f64 {
    resp["primitives"].as_array().unwrap().iter().find(|p| p["id"] == id).unwrap()[key]
        .as_f64()
        .unwrap()
}

#[test]
fn a_line_length() {
    let resp = solve(json!([
        { "id": "a", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "b", "type": "point", "x": 3.0, "y": 4.0 },
        { "id": "l", "type": "line", "p1_id": "a", "p2_id": "b" },
        { "id": "k", "type": "length", "curve": "l", "value": 7.0 }
    ]));
    assert!((num(&resp, "b", "x").hypot(num(&resp, "b", "y")) - 7.0).abs() < 1e-10);
}

/// A quarter arc of radius 2 centered at the origin, starting at (2, 0)
/// (held there when `start_fixed`), plus `rest`.
fn quarter_arc(start_fixed: bool, rest: Value) -> Value {
    let mut prims = json!([
        { "id": "o", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "s", "type": "point", "x": 2.0, "y": 0.0, "fixed": start_fixed },
        { "id": "e", "type": "point", "x": 0.0, "y": 2.0 },
        { "id": "a", "type": "arc", "c_id": "o", "start_id": "s", "end_id": "e",
          "radius": 2.0, "start_angle": 0.0, "end_angle": PI / 2.0 }
    ]);
    prims.as_array_mut().unwrap().extend(rest.as_array().unwrap().clone());
    prims
}

#[test]
fn an_arc_length_is_its_radius_times_its_sweep() {
    // Radius 2, start at angle 0: a length of 3 ends at angle 1.5.
    let resp = solve(quarter_arc(true, json!([
        { "id": "k", "type": "length", "curve": "a", "value": 3.0 }
    ])));
    let sweep = num(&resp, "a", "end_angle") - num(&resp, "a", "start_angle");
    assert!((sweep.rem_euclid(2.0 * PI) - 1.5).abs() < 1e-9, "sweep {sweep}");
    assert!((num(&resp, "e", "x") - 2.0 * 1.5_f64.cos()).abs() < 1e-8);
    assert!((num(&resp, "e", "y") - 2.0 * 1.5_f64.sin()).abs() < 1e-8);
}

#[test]
fn an_arc_length_and_sweep_set_its_radius() {
    let resp = solve(quarter_arc(false, json!([
        { "id": "k1", "type": "angle", "arc": "a", "value": 1.0 },
        { "id": "k2", "type": "length", "curve": "a", "value": 3.0 }
    ])));
    assert!((num(&resp, "a", "radius") - 3.0).abs() < 1e-9);
}

/// The sweep is `(end − start) mod 2π`, with 0 a full turn.
#[test]
fn an_arc_length_measures_the_counter_clockwise_sweep() {
    let c = create_constraint(ConstraintType::ArcLength("a".into(), 1.0)).unwrap();
    let eval = |x: &[f64]| {
        let mut r = vec![0.0; c.num_residuals()];
        let mut j = DMatrix::zeros(1, reads(c.as_ref()).len());
        c.eval(x, &mut r, &mut j);
        (r[0], j)
    };
    // x = [radius, start, end]
    assert!((eval(&[2.0, 0.0, 1.0]).0 - 1.0).abs() < 1e-12);
    // Clockwise-looking angles still sweep counter-clockwise: 2π − 1.
    assert!((eval(&[2.0, 1.0, 0.0]).0 - (2.0 * (2.0 * PI - 1.0) - 1.0)).abs() < 1e-12);
    // A full turn.
    assert!((eval(&[1.0, 0.5, 0.5]).0 - (2.0 * PI - 1.0)).abs() < 1e-12);
    let (_, j) = eval(&[2.0, 0.0, 1.0]);
    assert_eq!(j.row(0).iter().copied().collect::<Vec<_>>(), vec![1.0, -2.0, 2.0]);
}
