//! `collinear {a, b}`: both of Line `b`'s endpoints lie on Line `a`'s
//! Extension. The segments need not overlap, and the constraint is one
//! constraint (`ConstraintType::Collinear`), Redundant or Conflicting as a
//! unit.

use acs::bindings::sketch_solve::acs_solve_sketch;
use acs::sketch_solve::solve_sketch_json;
use acs::{ConstraintSolver, ConstraintType, Point, SolverResult};
use serde_json::{Value, json};

fn solve(primitives: Value) -> Value {
    let request = json!({ "version": 1, "primitives": primitives }).to_string();
    let out = solve_sketch_json(&request).unwrap_or_else(|e| panic!("rejected: {e}"));
    assert_eq!(acs_solve_sketch(&request), out);
    serde_json::from_str(&out).unwrap()
}

fn xy(resp: &Value, id: &str) -> (f64, f64) {
    let p = resp["primitives"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == id)
        .unwrap();
    (p["x"].as_f64().unwrap(), p["y"].as_f64().unwrap())
}

/// A fixed Line `a` from (0,0) to (4,0) and a free Line `b` well past its
/// end, off the axis: `b` lands on y = 0 and keeps its gap from `a`.
fn gapped(collinear: Value) -> Value {
    json!([
        { "id": "a1", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "a2", "type": "point", "x": 4.0, "y": 0.0, "fixed": true },
        { "id": "a", "type": "line", "p1_id": "a1", "p2_id": "a2" },
        { "id": "b1", "type": "point", "x": 8.0, "y": 1.0 },
        { "id": "b2", "type": "point", "x": 12.0, "y": 2.0 },
        { "id": "b", "type": "line", "p1_id": "b1", "p2_id": "b2" },
        collinear
    ])
}

#[test]
fn collinear_lines_may_have_a_gap_between_them() {
    let resp = solve(gapped(json!({ "id": "k", "type": "collinear", "a": "a", "b": "b" })));
    assert_eq!(resp["status"], "converged", "{resp}");
    for id in ["b1", "b2"] {
        let (x, y) = xy(&resp, id);
        assert!(y.abs() < 1e-8, "{id} should lie on the Extension, got y = {y}");
        assert!(x > 6.0, "{id} should stay past a's end, got x = {x}");
    }
    assert_eq!(resp["conflicting"], json!([]));
    assert_eq!(resp["redundant"], json!([]));
}

#[test]
fn collinear_takes_its_lines_in_either_order() {
    // With `a` fixed, `b` must move onto it whichever field names which.
    let resp = solve(gapped(json!({ "id": "k", "type": "collinear", "a": "b", "b": "a" })));
    assert_eq!(resp["status"], "converged", "{resp}");
    for id in ["b1", "b2"] {
        assert!(xy(&resp, id).1.abs() < 1e-8, "{resp}");
    }
}

#[test]
fn two_free_collinear_lines_have_six_degrees_of_freedom() {
    let resp = solve(json!([
        { "id": "a1", "type": "point", "x": 0.0, "y": 0.0 },
        { "id": "a2", "type": "point", "x": 4.0, "y": 1.0 },
        { "id": "a", "type": "line", "p1_id": "a1", "p2_id": "a2" },
        { "id": "b1", "type": "point", "x": 7.0, "y": 3.0 },
        { "id": "b2", "type": "point", "x": 11.0, "y": 2.0 },
        { "id": "b", "type": "line", "p1_id": "b1", "p2_id": "b2" },
        { "id": "k", "type": "collinear", "a": "a", "b": "b" }
    ]));
    assert_eq!(resp["status"], "converged", "{resp}");
    assert_eq!(resp["dof"], 6, "{resp}");
}

#[test]
fn collinear_lines_sharing_an_endpoint_line_up() {
    let resp = solve(json!([
        { "id": "p", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "q", "type": "point", "x": 3.0, "y": 0.0, "fixed": true },
        { "id": "r", "type": "point", "x": 6.0, "y": 2.0 },
        { "id": "a", "type": "line", "p1_id": "p", "p2_id": "q" },
        { "id": "b", "type": "line", "p1_id": "q", "p2_id": "r" },
        { "id": "k", "type": "collinear", "a": "a", "b": "b" }
    ]));
    assert_eq!(resp["status"], "converged", "{resp}");
    assert!(xy(&resp, "r").1.abs() < 1e-8, "{resp}");
    // b's shared endpoint already lies on a: one residual is redundant
    // structurally but the constraint is still independent (1 DOF removed).
    assert_eq!(resp["redundant"], json!([]));
    assert_eq!(resp["dof"], 1, "{resp}");
}

#[test]
fn free_collinear_lines_sharing_an_endpoint_keep_five_degrees_of_freedom() {
    let resp = solve(json!([
        { "id": "p", "type": "point", "x": 0.0, "y": 0.0 },
        { "id": "q", "type": "point", "x": 3.0, "y": 1.0 },
        { "id": "r", "type": "point", "x": 6.0, "y": 0.0 },
        { "id": "a", "type": "line", "p1_id": "p", "p2_id": "q" },
        { "id": "b", "type": "line", "p1_id": "q", "p2_id": "r" },
        { "id": "k", "type": "collinear", "a": "a", "b": "b" }
    ]));
    assert_eq!(resp["status"], "converged", "{resp}");
    assert_eq!(resp["redundant"], json!([]), "{resp}");
    assert_eq!(resp["dof"], 5, "{resp}");
}

#[test]
fn a_repeated_collinear_is_redundant_as_one_constraint() {
    let mut prims = gapped(json!({ "id": "k1", "type": "collinear", "a": "a", "b": "b" }));
    prims
        .as_array_mut()
        .unwrap()
        .push(json!({ "id": "k2", "type": "collinear", "a": "b", "b": "a" }));
    let resp = solve(prims);
    assert_eq!(resp["status"], "converged", "{resp}");
    assert_eq!(resp["redundant"], json!(["k2"]), "{resp}");
}

#[test]
fn collinear_through_the_rust_api() {
    let mut solver = ConstraintSolver::new();
    solver.add_point(Point::new("a1".into(), 0.0, 0.0, true));
    solver.add_point(Point::new("a2".into(), 1.0, 1.0, true));
    solver.add_point(Point::new("b1".into(), 5.0, 3.0, false));
    solver.add_point(Point::new("b2".into(), 6.0, 9.0, false));
    solver
        .add_constraint(ConstraintType::Collinear(
            "a1".into(),
            "a2".into(),
            "b1".into(),
            "b2".into(),
        ))
        .unwrap();

    let result = solver.solve().unwrap();
    assert!(matches!(result, SolverResult::Converged { .. }), "{result:?}");
    for id in ["b1", "b2"] {
        let p = solver.get_point(id.into()).unwrap();
        assert!((p.x - p.y).abs() < 1e-8, "{id} should lie on y = x: {p:?}");
    }
}

#[test]
fn collinear_needs_two_lines() {
    let request = json!({ "version": 1, "primitives": [
        { "id": "a1", "type": "point", "x": 0.0, "y": 0.0 },
        { "id": "a2", "type": "point", "x": 4.0, "y": 0.0 },
        { "id": "a", "type": "line", "p1_id": "a1", "p2_id": "a2" },
        { "id": "k", "type": "collinear", "a": "a", "b": "a1" }
    ]})
    .to_string();
    let out = solve_sketch_json(&request).expect_err("a point is not a line");
    let resp: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(resp["status"], "invalid");
    assert_eq!(resp["constraintId"], "k");
}
