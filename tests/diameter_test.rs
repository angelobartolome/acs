//! `diameter {curve, value}`: a circle's or arc's diameter, `2r = value`.

use acs::sketch_solve::solve_sketch_json;
use acs::{Circle, ConstraintSolver, ConstraintType, Point, SolverResult};
use serde_json::{Value, json};

fn solve(primitives: Value) -> Value {
    let request = json!({ "version": 1, "primitives": primitives }).to_string();
    let out = solve_sketch_json(&request).unwrap_or_else(|e| panic!("rejected: {e}"));
    let resp: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(resp["status"], "converged", "{resp}");
    resp
}

fn primitive<'a>(resp: &'a Value, id: &str) -> &'a Value {
    resp["primitives"].as_array().unwrap().iter().find(|p| p["id"] == id).unwrap()
}

#[test]
fn a_circle_diameter_sets_twice_its_radius() {
    let mut solver = ConstraintSolver::new();
    solver.add_point(Point::new("o".into(), 0.0, 0.0, true));
    let c = solver.add_circle(Circle::new("c".into(), "o".into(), 1.0, false));
    solver.add_constraint(ConstraintType::Diameter(c, 6.0)).unwrap();
    assert!(matches!(solver.solve().unwrap(), SolverResult::Converged { .. }));
    assert!((solver.get_circle("c".into()).unwrap().radius - 3.0).abs() < 1e-10);
}

#[test]
fn diameter_takes_a_circle_or_an_arc() {
    let resp = solve(json!([
        { "id": "o", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "c", "type": "circle", "c_id": "o", "radius": 1.0 },
        { "id": "k", "type": "diameter", "curve": "c", "value": 5.0 }
    ]));
    assert!((primitive(&resp, "c")["radius"].as_f64().unwrap() - 2.5).abs() < 1e-10);

    let resp = solve(json!([
        { "id": "o", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "s", "type": "point", "x": 1.0, "y": 0.0 },
        { "id": "e", "type": "point", "x": 0.0, "y": 1.0 },
        { "id": "a", "type": "arc", "c_id": "o", "start_id": "s", "end_id": "e",
          "radius": 1.0, "start_angle": 0.0, "end_angle": std::f64::consts::FRAC_PI_2 },
        { "id": "k", "type": "diameter", "curve": "a", "value": 8.0 }
    ]));
    assert!((primitive(&resp, "a")["radius"].as_f64().unwrap() - 4.0).abs() < 1e-10);
    // The arc's endpoints follow it.
    let s = primitive(&resp, "s");
    let (x, y) = (s["x"].as_f64().unwrap(), s["y"].as_f64().unwrap());
    assert!((x.hypot(y) - 4.0).abs() < 1e-8);
}

#[test]
fn a_diameter_and_an_agreeing_radius_are_redundant() {
    let resp = solve(json!([
        { "id": "o", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "c", "type": "circle", "c_id": "o", "radius": 1.0 },
        { "id": "k1", "type": "diameter", "curve": "c", "value": 4.0 },
        { "id": "k2", "type": "radius", "curve": "c", "value": 2.0 }
    ]));
    assert_eq!(resp["redundant"], json!(["k2"]));
}
