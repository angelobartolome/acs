use acs::sketch_solve::solve_sketch_primitives_json;
use serde_json::Value;

fn solve(input: &str) -> Value {
    let out = solve_sketch_primitives_json(input).expect("solve should not error");
    serde_json::from_str(&out).expect("response should be valid JSON")
}

#[test]
fn test_json_solve_converged_includes_stats() {
    let input = r#"{
        "primitives": [
            { "id": "p1", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
            { "id": "p2", "type": "point", "x": 3.0, "y": 4.0, "fixed": false },
            { "id": "c1", "type": "p2p_distance", "p1_id": "p1", "p2_id": "p2", "distance": 10.0 }
        ]
    }"#;

    let resp = solve(input);

    assert_eq!(resp["ok"], Value::Bool(true));
    assert_eq!(resp["status"], "converged");
    assert_eq!(resp["solveStatus"], 1);

    // Additive stats object
    let stats = resp["stats"]
        .as_object()
        .expect("stats should be an object on converged solves");
    assert!(stats["iterations"].as_u64().is_some(), "iterations present");
    let initial_error = stats["initial_error"].as_f64().expect("initial_error");
    let final_error = stats["final_error"].as_f64().expect("final_error");
    assert!(initial_error > 0.0, "initial error should be non-zero");
    assert!(final_error < 1e-6, "final error should be tiny");

    // Solved geometry: p2 must be 10 away from origin
    let prims = resp["primitives"].as_array().expect("primitives array");
    let p2 = prims
        .iter()
        .find(|p| p["id"] == "p2")
        .expect("p2 in response");
    let (x, y) = (p2["x"].as_f64().unwrap(), p2["y"].as_f64().unwrap());
    let dist = (x * x + y * y).sqrt();
    assert!((dist - 10.0).abs() < 1e-6, "distance should be 10, got {dist}");
}

#[test]
fn test_json_fully_constrained_ids_includes_line() {
    // Fixed anchor p1; p2 pinned by a fixed p2p_distance + horizontal; a line
    // between them should be reported fully constrained because both endpoints are.
    let input = r#"{
        "primitives": [
            { "id": "p1", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
            { "id": "p2", "type": "point", "x": 1.0, "y": 1.0, "fixed": false },
            { "id": "l1", "type": "line", "p1_id": "p1", "p2_id": "p2" },
            { "id": "c1", "type": "horizontal_pp", "p1_id": "p1", "p2_id": "p2" },
            { "id": "c2", "type": "p2p_distance", "p1_id": "p1", "p2_id": "p2", "distance": 5.0 }
        ]
    }"#;

    let resp = solve(input);
    assert_eq!(resp["ok"], Value::Bool(true));

    let ids: Vec<String> = resp["fully_constrained_ids"]
        .as_array()
        .expect("fully_constrained_ids array")
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();

    assert!(ids.contains(&"p1".to_string()), "p1 fixed -> constrained: {ids:?}");
    assert!(ids.contains(&"p2".to_string()), "p2 pinned -> constrained: {ids:?}");
    assert!(
        ids.contains(&"l1".to_string()),
        "line l1 should be constrained since both endpoints are: {ids:?}"
    );
}

#[test]
fn test_json_fully_constrained_ids_empty_on_failure() {
    let input = r#"{
        "primitives": [
            { "id": "p1", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
            { "id": "p2", "type": "point", "x": 1.0, "y": 0.0, "fixed": false },
            { "id": "c1", "type": "p2p_distance", "p1_id": "p1", "p2_id": "p2", "distance": 5.0 },
            { "id": "c2", "type": "p2p_distance", "p1_id": "p1", "p2_id": "p2", "distance": 10.0 }
        ],
        "max_iterations": 20
    }"#;

    let resp = solve(input);
    assert_eq!(resp["ok"], Value::Bool(false));
    assert_eq!(
        resp["fully_constrained_ids"].as_array().map(|a| a.len()),
        Some(0),
        "fully_constrained_ids must be empty on a failed solve"
    );
}

#[test]
fn test_json_solve_failed_includes_stats() {
    // Conflicting: p2 cannot be both 5 and 10 away from fixed p1
    let input = r#"{
        "primitives": [
            { "id": "p1", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
            { "id": "p2", "type": "point", "x": 1.0, "y": 0.0, "fixed": false },
            { "id": "c1", "type": "p2p_distance", "p1_id": "p1", "p2_id": "p2", "distance": 5.0 },
            { "id": "c2", "type": "p2p_distance", "p1_id": "p1", "p2_id": "p2", "distance": 10.0 }
        ],
        "max_iterations": 20
    }"#;

    let resp = solve(input);

    assert_eq!(resp["ok"], Value::Bool(false));
    assert_eq!(resp["status"], "failed");

    let stats = resp["stats"]
        .as_object()
        .expect("stats should be present when max iterations reached");
    assert!(stats["iterations"].as_u64().unwrap() > 0);
    assert!(stats["final_error"].as_f64().unwrap() > 1e-10);
}
