//! `angle` between two Lines with `reverseA` / `reverseB`: the angle between
//! two rays from where the Lines meet, with one root. Without the flags the
//! angle is between undirected Lines, with roots at θ and θ + π (see
//! `angle_test.rs`).

use acs::sketch_solve::solve_sketch_json;
use serde_json::{Value, json};

fn solve(primitives: Value) -> Value {
    let request = json!({ "version": 1, "primitives": primitives }).to_string();
    let out = solve_sketch_json(&request).unwrap_or_else(|e| panic!("rejected: {e}"));
    let resp: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(resp["status"], "converged", "{resp}");
    resp
}

fn reject(primitives: Value) -> String {
    let request = json!({ "version": 1, "primitives": primitives }).to_string();
    let out = solve_sketch_json(&request).expect_err("request should be rejected");
    let resp: Value = serde_json::from_str(&out).unwrap();
    resp["error"].as_str().unwrap().to_string()
}

fn xy(resp: &Value, id: &str) -> (f64, f64) {
    let p = resp["primitives"].as_array().unwrap().iter().find(|p| p["id"] == id).unwrap();
    (p["x"].as_f64().unwrap(), p["y"].as_f64().unwrap())
}

/// `a` minus `b`, in degrees, wrapped into [−180, 180).
fn wrapped(a: f64, b: f64) -> f64 {
    (a - b + 180.0).rem_euclid(360.0) - 180.0
}

/// A corner `o` at the origin, Line `l1` fixed along +X from it and a free
/// Line `l2` from it at −30°, each drawn from or to the corner as
/// `a_to_corner` / `b_to_corner` say, with an `angle` typed to `degrees`
/// and the flags (if `flags`) set on each Line whose `p2` is the corner.
/// Returns the angle of `l2`'s far end seen from the corner, in degrees.
fn typed(degrees: f64, a_to_corner: bool, b_to_corner: bool, flags: bool) -> f64 {
    let line = |id: &str, far: &str, to_corner: bool| {
        let (p1, p2) = if to_corner { (far, "o") } else { ("o", far) };
        json!({ "id": id, "type": "line", "p1_id": p1, "p2_id": p2 })
    };
    let start = (-30.0_f64).to_radians();
    let mut angle = json!({ "id": "k1", "type": "angle", "a": "l1", "b": "l2",
                            "value": degrees.to_radians() });
    if flags {
        angle["reverseA"] = json!(a_to_corner);
        angle["reverseB"] = json!(b_to_corner);
    }
    let resp = solve(json!([
        { "id": "o", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "e1", "type": "point", "x": 10.0, "y": 0.0, "fixed": true },
        { "id": "e2", "type": "point", "x": 10.0 * start.cos(), "y": 10.0 * start.sin() },
        line("l1", "e1", a_to_corner),
        line("l2", "e2", b_to_corner),
        angle
    ]));
    let (x, y) = xy(&resp, "e2");
    y.atan2(x).to_degrees()
}

#[test]
fn a_typed_angle_ends_at_the_angle_between_the_rays_for_every_flag() {
    for a_to_corner in [false, true] {
        for b_to_corner in [false, true] {
            for degrees in [175.0, 150.0, 120.0, 90.0] {
                let ends = typed(degrees, a_to_corner, b_to_corner, true);
                assert!(
                    wrapped(ends, degrees).abs() < 1e-6,
                    "typed {degrees}° (reverseA {a_to_corner}, reverseB {b_to_corner}): ends at {ends}°"
                );
            }
        }
    }
}

#[test]
fn without_the_flags_the_angle_keeps_both_roots() {
    // From −30°, 175° lands on its other root, 175° + 180°.
    let ends = typed(175.0, false, false, false);
    assert!(wrapped(ends, 355.0).abs() < 1e-6, "ends at {ends}°");
}

#[test]
fn either_flag_alone_selects_one_root() {
    let request = |flag: &str| {
        let mut angle = json!({ "id": "k1", "type": "angle", "a": "l1", "b": "l2",
                                "value": 175.0_f64.to_radians() });
        angle[flag] = json!(false);
        json!([
            { "id": "o", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
            { "id": "e1", "type": "point", "x": 10.0, "y": 0.0, "fixed": true },
            { "id": "e2", "type": "point", "x": 8.66, "y": -5.0 },
            { "id": "l1", "type": "line", "p1_id": "o", "p2_id": "e1" },
            { "id": "l2", "type": "line", "p1_id": "o", "p2_id": "e2" },
            angle
        ])
    };
    for flag in ["reverseA", "reverseB"] {
        let (x, y) = xy(&solve(request(flag)), "e2");
        let ends = y.atan2(x).to_degrees();
        assert!(wrapped(ends, 175.0).abs() < 1e-6, "{flag}: ends at {ends}°");
    }
}

#[test]
fn a_flag_that_is_not_a_boolean_is_rejected() {
    let err = reject(json!([
        { "id": "o", "type": "point", "x": 0.0, "y": 0.0 },
        { "id": "e1", "type": "point", "x": 1.0, "y": 0.0 },
        { "id": "e2", "type": "point", "x": 0.0, "y": 1.0 },
        { "id": "l1", "type": "line", "p1_id": "o", "p2_id": "e1" },
        { "id": "l2", "type": "line", "p1_id": "o", "p2_id": "e2" },
        { "id": "k1", "type": "angle", "a": "l1", "b": "l2", "value": 1.0, "reverseA": "yes" }
    ]));
    assert!(err.contains("reverseA"), "{err}");
}

#[test]
fn the_catalog_lists_the_flags_on_the_line_angle() {
    let catalog: Value = serde_json::from_str(&acs::constraint_catalog::catalog_json()).unwrap();
    let angles: Vec<&Value> = catalog
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["type"] == "angle")
        .collect();
    let lines = angles.iter().find(|r| r["fields"][0]["kind"] == "line").unwrap();
    assert_eq!(lines["options"], json!(["reverseA", "reverseB"]));
    let arc = angles.iter().find(|r| r["fields"][0]["kind"] == "arc").unwrap();
    assert!(arc.get("options").is_none());
}
