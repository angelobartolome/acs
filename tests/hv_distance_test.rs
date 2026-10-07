//! Native `horizontal_distance` and `vertical_distance`: signed
//! dimensions along one axis, `b.x − a.x = value` (`y` for vertical), or a
//! Line's `p2 − p1`. A negative value puts `b` left of / below `a`.

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

/// The error of a request that must be rejected, and the constraint it names.
fn reject(primitives: Value) -> (String, Value) {
    let out = solve_sketch_json(&request(primitives)).expect_err("should be rejected");
    let resp: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(resp["status"], "invalid");
    (resp["error"].as_str().unwrap().to_string(), resp["constraintId"].clone())
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

fn close(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-8, "{a} != {b}");
}

/// A fixed point `a` at (1, 2) and a free point `b` at (4, 6).
fn points() -> Vec<Value> {
    vec![
        json!({ "id": "a", "type": "point", "x": 1.0, "y": 2.0, "fixed": true }),
        json!({ "id": "b", "type": "point", "x": 4.0, "y": 6.0 }),
    ]
}

fn with(mut prims: Vec<Value>, constraints: &[Value]) -> Value {
    prims.extend(constraints.iter().cloned());
    Value::Array(prims)
}

#[test]
fn horizontal_distance_sets_only_the_x_offset() {
    for value in [5.0, -5.0, 0.0] {
        let resp = solve(with(
            points(),
            &[json!({ "id": "k", "type": "horizontal_distance", "a": "a", "b": "b", "value": value })],
        ));
        let (bx, by) = xy(&resp, "b");
        close(bx - 1.0, value);
        close(by, 6.0);
        assert_eq!(resp["dof"], 1, "{resp}");
    }
}

#[test]
fn vertical_distance_sets_only_the_y_offset() {
    for value in [3.0, -7.5] {
        let resp = solve(with(
            points(),
            &[json!({ "id": "k", "type": "vertical_distance", "a": "a", "b": "b", "value": value })],
        ));
        let (bx, by) = xy(&resp, "b");
        close(by - 2.0, value);
        close(bx, 4.0);
        assert_eq!(resp["dof"], 1, "{resp}");
    }
}

/// Order matters: swapping `a` and `b` flips the sign.
#[test]
fn a_and_b_are_ordered() {
    let resp = solve(with(
        points(),
        &[json!({ "id": "k", "type": "horizontal_distance", "a": "b", "b": "a", "value": 2.0 })],
    ));
    close(xy(&resp, "b").0, -1.0);
}

#[test]
fn line_form_measures_p2_minus_p1() {
    for (ty, value) in [
        ("horizontal_distance", 6.0),
        ("horizontal_distance", -2.5),
        ("vertical_distance", -4.0),
    ] {
        let resp = solve(with(
            points(),
            &[
                json!({ "id": "l", "type": "line", "p1_id": "a", "p2_id": "b" }),
                json!({ "id": "k", "type": ty, "line": "l", "value": value }),
            ],
        ));
        let (bx, by) = xy(&resp, "b");
        if ty == "horizontal_distance" {
            close(bx - 1.0, value);
            close(by, 6.0);
        } else {
            close(by - 2.0, value);
            close(bx, 4.0);
        }
        assert_eq!(resp["dof"], 1, "{resp}");
    }
}

/// With a point–point `distance` the second point is fully placed on the
/// side the solve started from: the dimension fixes one axis only.
#[test]
fn with_a_distance_both_coordinates_are_fixed() {
    let resp = solve(with(
        points(),
        &[
            json!({ "id": "h", "type": "horizontal_distance", "a": "a", "b": "b", "value": 3.0 }),
            json!({ "id": "d", "type": "distance", "a": "a", "b": "b", "value": 5.0 }),
        ],
    ));
    let (bx, by) = xy(&resp, "b");
    close(bx, 4.0);
    close(by, 6.0);
    assert_eq!(resp["dof"], 0, "{resp}");
    assert_eq!(resp["redundant"], json!([]));
    assert_eq!(resp["conflicting"], json!([]));
    assert!(
        resp["fullyConstrained"].as_array().unwrap().contains(&json!("b")),
        "{resp}"
    );
}

/// Both dimensions together pin `b`; a third one along x is Redundant when
/// consistent and Conflicting when not.
#[test]
fn diagnosis_counts_each_dimension_once() {
    let both = [
        json!({ "id": "h", "type": "horizontal_distance", "a": "a", "b": "b", "value": -2.0 }),
        json!({ "id": "v", "type": "vertical_distance", "a": "a", "b": "b", "value": -3.0 }),
    ];
    let resp = solve(with(points(), &both));
    let (bx, by) = xy(&resp, "b");
    close(bx, -1.0);
    close(by, -1.0);
    assert_eq!(resp["dof"], 0);

    let resp = solve(with(
        points(),
        &[
            both[0].clone(),
            both[1].clone(),
            json!({ "id": "x", "type": "x", "point": "b", "value": -1.0 }),
        ],
    ));
    assert_eq!(resp["redundant"], json!(["x"]), "{resp}");

    let out = solve_sketch_json(&request(with(
        points(),
        &[
            both[0].clone(),
            json!({ "id": "x", "type": "x", "point": "b", "value": 5.0 }),
        ],
    )))
    .unwrap();
    let resp: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(resp["status"], "failed", "{resp}");
    assert_eq!(resp["conflicting"], json!(["h", "x"]), "{resp}");
}

#[test]
fn value_takes_a_parameter() {
    let resp = solve(with(
        points(),
        &[
            json!({ "id": "w", "type": "param", "value": -4.0 }),
            json!({ "id": "k", "type": "horizontal_distance", "a": "a", "b": "b", "value": "w" }),
        ],
    ));
    close(xy(&resp, "b").0, -3.0);
}

#[test]
fn wrong_kinds_are_rejected() {
    let prims = || {
        let mut p = points();
        p.push(json!({ "id": "c", "type": "circle", "c_id": "b", "radius": 1.0 }));
        p.push(json!({ "id": "l", "type": "line", "p1_id": "a", "p2_id": "b" }));
        p
    };
    for ty in ["horizontal_distance", "vertical_distance"] {
        let (err, id) = reject(with(
            prims(),
            &[json!({ "id": "k", "type": ty, "a": "a", "b": "c", "value": 1.0 })],
        ));
        assert_eq!(id, "k");
        assert!(err.contains("unsupported combination"), "{err}");
        assert!(err.contains("b: circle"), "{err}");

        let (err, _) = reject(with(
            prims(),
            &[json!({ "id": "k", "type": ty, "line": "c", "value": 1.0 })],
        ));
        assert!(err.contains("line: circle"), "{err}");

        let (err, _) = reject(with(
            prims(),
            &[json!({ "id": "k", "type": ty, "a": "l", "b": "b", "value": 1.0 })],
        ));
        assert!(err.contains("a: line"), "{err}");
    }
}
