//! The JSON solve contract (shared with the `P3DSketch_Solve` C ABI) and
//! the solver's behaviour through it: the envelope, rejected requests,
//! diagnosis, soft goals, drags, Parameters, arcs, Reference Geometry,
//! mirrors, arrays and ellipses. Every request also goes through
//! `acsSolveSketch`, which must answer identically.

use acs::bindings::sketch_solve::acs_solve_sketch;
use acs::sketch_solve::solve_sketch_json;
use serde_json::{Value, json};

/// Solves a request that must be understood, returning the parsed response.
fn solve(request: Value) -> Value {
    let request = request.to_string();
    let out = solve_sketch_json(&request).expect("request should be understood");
    assert_eq!(acs_solve_sketch(&request), out);
    serde_json::from_str(&out).expect("response should be valid JSON")
}

/// Solves a request that must be rejected, returning the parsed error response.
fn reject(request: &str) -> Value {
    let out = solve_sketch_json(request).expect_err("request should be rejected");
    assert_eq!(acs_solve_sketch(request), out);
    let resp: Value = serde_json::from_str(&out).expect("error response should be valid JSON");
    assert_eq!(resp["version"], 1);
    assert_eq!(resp["status"], "invalid");
    resp
}

fn request(primitives: Value) -> Value {
    json!({ "version": 1, "primitives": primitives })
}

fn point(resp: &Value, id: &str) -> (f64, f64) {
    let p = resp["primitives"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == id)
        .unwrap();
    (p["x"].as_f64().unwrap(), p["y"].as_f64().unwrap())
}

// ── The response envelope ──────────────────────────────────────────────────

#[test]
fn converged_response_has_the_contract_envelope() {
    let resp = solve(request(json!([
        { "id": "p1", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "p2", "type": "point", "x": 3.0, "y": 4.0, "fixed": false },
        { "id": "k1", "type": "distance", "a": "p1", "b": "p2", "value": 10.0 }
    ])));

    assert_eq!(resp["version"], 1);
    assert_eq!(resp["status"], "converged");
    assert_eq!(resp["conflicting"], json!([]));
    assert_eq!(resp["redundant"], json!([]));
    assert!(resp["dof"].is_u64());
    assert!(resp.get("error").is_none());

    // Same primitives, same order, solved values.
    let ids: Vec<&str> = resp["primitives"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["p1", "p2", "k1"]);
    let (x, y) = point(&resp, "p2");
    assert!(((x * x + y * y).sqrt() - 10.0).abs() < 1e-8);

    // ACS extension: solve statistics.
    let stats = &resp["stats"];
    assert!(stats["iterations"].as_u64().is_some());
    assert!(stats["finalError"].as_f64().unwrap() < 1e-6);
    assert!(stats["initialError"].as_f64().is_some());
}

/// A sketch shaped like an editor's: Reference Geometry, construction flags,
/// groups and temporary constraints are all accepted.
#[test]
fn editor_sketch_with_existing_constraints_solves() {
    let resp = solve(request(json!([
        { "id": "origin", "type": "point", "x": 0.0, "y": 0.0, "fixed": true,
          "isConstruction": true, "isReference": true },
        { "id": "a", "type": "point", "x": 0.2, "y": 0.1, "fixed": false },
        { "id": "b", "type": "point", "x": 9.0, "y": 1.0, "fixed": false },
        { "id": "l1", "type": "line", "p1_id": "a", "p2_id": "b", "isConstruction": false },
        { "id": "c", "type": "point", "x": 20.0, "y": 0.0, "fixed": false },
        { "id": "circle1", "type": "circle", "c_id": "c", "radius": 2.0, "group": "g1" },
        { "id": "k1", "type": "coincident", "a": "a", "b": "origin" },
        { "id": "k2", "type": "horizontal", "line": "l1" },
        { "id": "k3", "type": "distance", "a": "a", "b": "b", "value": 10.0 },
        { "id": "k4", "type": "radius", "curve": "circle1", "value": 4.0 },
        { "id": "k5", "type": "x", "point": "c", "value": 25.0, "temporary": true }
    ])));

    assert_eq!(resp["status"], "converged", "{resp}");
    let (ax, ay) = point(&resp, "a");
    assert!(ax.abs() < 1e-8 && ay.abs() < 1e-8, "a coincides with the origin");
    let (bx, by) = point(&resp, "b");
    assert!((bx.abs() - 10.0).abs() < 1e-8 && by.abs() < 1e-8);
    let circle = resp["primitives"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == "circle1")
        .unwrap();
    assert!((circle["radius"].as_f64().unwrap() - 4.0).abs() < 1e-8);
    assert_eq!(circle["group"], "g1", "unknown fields pass through");
    assert!((point(&resp, "c").0 - 25.0).abs() < 1e-8);
}

#[test]
fn failed_solve_is_a_response_not_an_error() {
    // p2 can't be both 5 and 10 from fixed p1.
    let resp = solve(request(json!([
        { "id": "p1", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "p2", "type": "point", "x": 3.0, "y": 4.0 },
        { "id": "k1", "type": "distance", "a": "p1", "b": "p2", "value": 5.0 },
        { "id": "k2", "type": "distance", "a": "p1", "b": "p2", "value": 10.0 }
    ])));

    assert_eq!(resp["status"], "failed");
    assert_eq!(resp["primitives"].as_array().unwrap().len(), 4);
    assert!(resp["stats"]["finalError"].as_f64().unwrap() > 1e-6);
    assert_eq!(resp["fullyConstrained"], json!([]));
}

#[test]
fn max_iterations_is_honoured() {
    // A 5-unit move can't happen in one Dog-Leg step from the initial
    // 0.1 trust radius.
    let mut req = request(json!([
        { "id": "p1", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "p2", "type": "point", "x": 3.0, "y": 4.0 },
        { "id": "k1", "type": "distance", "a": "p1", "b": "p2", "value": 10.0 }
    ]));
    req["maxIterations"] = json!(1);
    let resp = solve(req.clone());
    assert_eq!(resp["status"], "failed");
    assert_eq!(resp["stats"]["iterations"], 1);

    req["maxIterations"] = json!(100);
    assert_eq!(solve(req)["status"], "converged");
}

// ── Degrees of freedom ─────────────────────────────────────────────────────

#[test]
fn dof_counts_what_the_constraints_leave_free() {
    let free_point = json!({ "id": "p", "type": "point", "x": 1.0, "y": 2.0 });
    let fixed_point = json!({ "id": "o", "type": "point", "x": 0.0, "y": 0.0, "fixed": true });

    assert_eq!(solve(request(json!([free_point])))["dof"], 2);
    assert_eq!(solve(request(json!([fixed_point])))["dof"], 0);
    assert_eq!(
        solve(request(json!([
            free_point,
            { "id": "k1", "type": "x", "point": "p", "value": 3.0 }
        ])))["dof"],
        1
    );
    // Rectangle anchored at a fixed corner, with width and height: 0 DOF.
    let rect = solve(request(json!([
        { "id": "a", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "b", "type": "point", "x": 4.0, "y": 0.5 },
        { "id": "c", "type": "point", "x": 4.5, "y": 3.0 },
        { "id": "d", "type": "point", "x": 0.5, "y": 3.5 },
        { "id": "ab", "type": "line", "p1_id": "a", "p2_id": "b" },
        { "id": "bc", "type": "line", "p1_id": "b", "p2_id": "c" },
        { "id": "cd", "type": "line", "p1_id": "c", "p2_id": "d" },
        { "id": "da", "type": "line", "p1_id": "d", "p2_id": "a" },
        { "id": "k1", "type": "horizontal", "line": "ab" },
        { "id": "k2", "type": "horizontal", "line": "cd" },
        { "id": "k3", "type": "vertical", "line": "bc" },
        { "id": "k4", "type": "vertical", "line": "da" },
        { "id": "k5", "type": "distance", "a": "a", "b": "b", "value": 4.0 },
        { "id": "k6", "type": "distance", "a": "b", "b": "c", "value": 3.0 }
    ])));
    assert_eq!(rect["status"], "converged");
    assert_eq!(rect["dof"], 0);
}

#[test]
fn fully_constrained_extension_includes_lines() {
    let resp = solve(request(json!([
        { "id": "p1", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "p2", "type": "point", "x": 3.0, "y": 1.0 },
        { "id": "p3", "type": "point", "x": 5.0, "y": 5.0 },
        { "id": "line1", "type": "line", "p1_id": "p1", "p2_id": "p2" },
        { "id": "c1", "type": "horizontal", "a": "p1", "b": "p2" },
        { "id": "c2", "type": "distance", "a": "p1", "b": "p2", "value": 5.0 }
    ])));

    let ids: Vec<&str> = resp["fullyConstrained"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(ids.contains(&"p1") && ids.contains(&"p2") && ids.contains(&"line1"));
    assert!(!ids.contains(&"p3"));
}

// ── Rejected requests ──────────────────────────────────────────────────────

#[test]
fn request_without_version_1_is_rejected() {
    let prims = r#"[{ "id": "p", "type": "point", "x": 0, "y": 0 }]"#;
    let resp = reject(&format!(r#"{{ "primitives": {prims} }}"#));
    assert_eq!(resp["error"], "unsupported request version");
    reject(&format!(r#"{{ "version": 2, "primitives": {prims} }}"#));
    reject(prims);
}

#[test]
fn malformed_requests_are_rejected() {
    assert!(reject("not json")["error"].as_str().unwrap().contains("JSON"));
    assert_eq!(
        reject(r#"{ "version": 1 }"#)["error"],
        "request has no primitives array"
    );
}

#[test]
fn unknown_constraint_type_is_rejected_by_name() {
    let resp = reject(
        &request(json!([
            { "id": "p1", "type": "point", "x": 0.0, "y": 0.0 },
            { "id": "k1", "type": "made_up", "p1_id": "p1" }
        ]))
        .to_string(),
    );
    let error = resp["error"].as_str().unwrap();
    assert!(error.contains("made_up") && error.contains("k1"), "{error}");
    assert_eq!(resp["constraintId"], "k1");
}

#[test]
fn constraint_missing_a_field_is_rejected_naming_it() {
    let resp = reject(
        &request(json!([
            { "id": "p1", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
            { "id": "p2", "type": "point", "x": 3.0, "y": 4.0 },
            { "id": "k1", "type": "distance", "a": "p1", "b": "p2" }
        ]))
        .to_string(),
    );
    let error = resp["error"].as_str().unwrap();
    assert!(error.contains("'value'") && error.contains("k1"), "{error}");
}

#[test]
fn geometry_missing_a_field_is_rejected_naming_it() {
    let resp = reject(&request(json!([{ "id": "p1", "type": "point", "x": 0.0 }])).to_string());
    let error = resp["error"].as_str().unwrap();
    assert!(error.contains("'y'") && error.contains("p1"), "{error}");
}

#[test]
fn constraint_on_a_missing_entity_is_rejected() {
    let resp = reject(
        &request(json!([
            { "id": "p1", "type": "point", "x": 0.0, "y": 0.0 },
            { "id": "k1", "type": "coincident", "a": "p1", "b": "ghost" }
        ]))
        .to_string(),
    );
    assert_eq!(resp["error"], "constraint k1: 'ghost' is not a point");
    assert_eq!(resp["constraintId"], "k1");
}

// ── Constraint coverage through the contract ───────────────────────────────

#[test]
fn tangent_and_concentric_circles() {
    let resp = solve(request(json!([
        { "id": "o1", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "o2", "type": "point", "x": 9.0, "y": 1.0 },
        { "id": "o3", "type": "point", "x": 2.0, "y": 3.0 },
        { "id": "c1", "type": "circle", "c_id": "o1", "radius": 3.0, "fixed": true },
        { "id": "c2", "type": "circle", "c_id": "o2", "radius": 2.0, "fixed": true },
        { "id": "c3", "type": "circle", "c_id": "o3", "radius": 1.0 },
        { "id": "k1", "type": "tangent", "a": "c1", "b": "c2" },
        { "id": "k2", "type": "concentric", "a": "c1", "b": "c3" }
    ])));

    assert_eq!(resp["status"], "converged", "{resp}");
    let (x2, y2) = point(&resp, "o2");
    assert!(((x2 * x2 + y2 * y2).sqrt() - 5.0).abs() < 1e-8, "centers 3 + 2 apart");
    let (x3, y3) = point(&resp, "o3");
    assert!(x3.abs() < 1e-8 && y3.abs() < 1e-8, "c3 shares c1's center");
}

/// Arcs resolve their center from `c_id`, just like circles.
#[test]
fn point_on_arc_resolves_center() {
    let resp = solve(request(json!([
        { "id": "o", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "s", "type": "point", "x": 5.0, "y": 0.0 },
        { "id": "e", "type": "point", "x": 0.0, "y": 5.0 },
        { "id": "a1", "type": "arc", "c_id": "o", "start_id": "s", "end_id": "e", "radius": 5.0,
          "start_angle": 0.0, "end_angle": 1.5, "fixed": true },
        { "id": "p", "type": "point", "x": 1.0, "y": 1.0 },
        { "id": "k1", "type": "on", "point": "p", "curve": "a1" }
    ])));

    assert_eq!(resp["status"], "converged", "{resp}");
    let (x, y) = point(&resp, "p");
    assert!(((x * x + y * y).sqrt() - 5.0).abs() < 1e-8);
}

#[test]
fn scalars_may_be_numeric_strings() {
    let resp = solve(request(json!([
        { "id": "p", "type": "point", "x": 1.0, "y": 2.0 },
        { "id": "k1", "type": "x", "point": "p", "value": "7.5" }
    ])));
    assert!((point(&resp, "p").0 - 7.5).abs() < 1e-8);
}

#[test]
fn max_iterations_must_be_a_whole_number() {
    for bad in [json!(-1), json!(10.5), json!("5")] {
        let mut req = request(json!([{ "id": "p", "type": "point", "x": 0.0, "y": 0.0 }]));
        req["maxIterations"] = bad;
        let resp = reject(&req.to_string());
        assert!(resp["error"].as_str().unwrap().contains("maxIterations"));
    }
}

#[test]
fn duplicate_ids_are_rejected() {
    let resp = reject(
        &request(json!([
            { "id": "p", "type": "point", "x": 0.0, "y": 0.0 },
            { "id": "p", "type": "point", "x": 1.0, "y": 1.0 }
        ]))
        .to_string(),
    );
    assert_eq!(resp["error"], "duplicate id 'p'");
}

#[test]
fn geometry_referencing_a_missing_point_is_rejected() {
    let resp = reject(
        &request(json!([
            { "id": "a", "type": "point", "x": 0.0, "y": 0.0 },
            { "id": "l1", "type": "line", "p1_id": "a", "p2_id": "nowhere" }
        ]))
        .to_string(),
    );
    assert_eq!(resp["error"], "line l1: 'nowhere' is not a point");
    assert!(resp.get("constraintId").is_none());

    let resp = reject(
        &request(json!([{ "id": "c1", "type": "circle", "c_id": "nowhere", "radius": 1.0 }]))
            .to_string(),
    );
    assert_eq!(resp["error"], "circle c1: 'nowhere' is not a point");
}

/// An ellipse needs its center and focus, and both must be Points.
#[test]
fn ellipse_needs_its_center_and_focus_points() {
    let resp = reject(
        &request(json!([
            { "id": "o", "type": "point", "x": 0.0, "y": 0.0 },
            { "id": "e", "type": "ellipse", "c_id": "o", "radmin": 1.0 }
        ]))
        .to_string(),
    );
    assert_eq!(resp["error"], "ellipse e: missing field 'focus1_id'");
    assert!(resp.get("constraintId").is_none());

    let resp = reject(
        &request(json!([
            { "id": "o", "type": "point", "x": 0.0, "y": 0.0 },
            { "id": "e", "type": "ellipse", "c_id": "o", "focus1_id": "nowhere", "radmin": 1.0 }
        ]))
        .to_string(),
    );
    assert_eq!(resp["error"], "ellipse e: 'nowhere' is not a point");
}

// ── Extension variants ─────────────────────────────────────────────────────
//
// Each sketch has a fixed horizontal Line l from (0,0) to (10,0) and geometry
// that starts beyond its far end. The plain constraint pulls the geometry back
// over the segment; the Extension variant leaves it beyond, on the Extension.

/// The fixed Line l plus `rest`.
fn with_fixed_line(rest: Value) -> Value {
    let mut primitives = json!([
        { "id": "a", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "b", "type": "point", "x": 10.0, "y": 0.0, "fixed": true },
        { "id": "l", "type": "line", "p1_id": "a", "p2_id": "b" }
    ]);
    primitives
        .as_array_mut()
        .unwrap()
        .extend(rest.as_array().unwrap().iter().cloned());
    request(primitives)
}

#[test]
fn point_on_extension_holds_beyond_the_segment() {
    let sketch = |extension: bool| {
        with_fixed_line(json!([
            { "id": "p", "type": "point", "x": 15.0, "y": 2.0 },
            { "id": "k", "type": "on", "point": "p", "curve": "l", "extension": extension }
        ]))
    };

    let resp = solve(sketch(false));
    assert_eq!(resp["status"], "converged", "{resp}");
    let (x, y) = point(&resp, "p");
    assert!(
        y.abs() < 1e-8 && x <= 10.0 + 1e-8,
        "pulled onto the segment: ({x}, {y})"
    );

    let resp = solve(sketch(true));
    assert_eq!(resp["status"], "converged", "{resp}");
    let (x, y) = point(&resp, "p");
    assert!(y.abs() < 1e-8, "on the Extension, got y = {y}");
    assert!(x > 14.0, "stays beyond the segment, got x = {x}");
}

#[test]
fn distance_to_extension_holds_beyond_the_segment() {
    let sketch = |extension: bool, d: f64| {
        with_fixed_line(json!([
            { "id": "p", "type": "point", "x": 15.0, "y": 3.0 },
            { "id": "k", "type": "distance", "a": "p", "b": "l", "value": d,
              "extension": extension }
        ]))
    };

    // Plain: 1 from the segment means 1 from its endpoint b out here.
    let resp = solve(sketch(false, 1.0));
    assert_eq!(resp["status"], "converged", "{resp}");
    let (x, y) = point(&resp, "p");
    assert!(
        (((x - 10.0).powi(2) + y * y).sqrt() - 1.0).abs() < 1e-8,
        "({x}, {y})"
    );

    // Extension: 1 from the infinite line, where it started along it.
    let resp = solve(sketch(true, 1.0));
    assert_eq!(resp["status"], "converged", "{resp}");
    let (x, y) = point(&resp, "p");
    assert!((y - 1.0).abs() < 1e-8, "1 above the Extension, got y = {y}");
    assert!(x > 14.0, "stays beyond the segment, got x = {x}");

    // Distance 0 puts the point on the Extension.
    let resp = solve(sketch(true, 0.0));
    assert_eq!(resp["status"], "converged", "{resp}");
    let (x, y) = point(&resp, "p");
    assert!(y.abs() < 1e-8 && x > 14.0, "({x}, {y})");
}

#[test]
fn tangent_to_extension_holds_beyond_the_segment() {
    let sketch = |extension: bool| {
        with_fixed_line(json!([
            { "id": "o", "type": "point", "x": 15.0, "y": 3.0 },
            { "id": "c", "type": "circle", "c_id": "o", "radius": 2.0, "fixed": true },
            { "id": "k", "type": "tangent", "a": "l", "b": "c", "extension": extension }
        ]))
    };

    let resp = solve(sketch(false));
    assert_eq!(resp["status"], "converged", "{resp}");
    let (x, y) = point(&resp, "o");
    assert!(
        (y - 2.0).abs() < 1e-8 && x <= 10.0 + 1e-8,
        "pulled over the segment: ({x}, {y})"
    );

    let resp = solve(sketch(true));
    assert_eq!(resp["status"], "converged", "{resp}");
    let (x, y) = point(&resp, "o");
    assert!(
        (y - 2.0).abs() < 1e-8,
        "tangent to the Extension, got y = {y}"
    );
    assert!(
        x > 14.0,
        "tangency point stays beyond the segment, got x = {x}"
    );
}

#[test]
fn midpoint_on_extension_holds_beyond_the_segment() {
    let sketch = |extension: bool| {
        with_fixed_line(json!([
            { "id": "m1", "type": "point", "x": 14.0, "y": 1.0 },
            { "id": "m2", "type": "point", "x": 16.0, "y": 3.0 },
            { "id": "l2", "type": "line", "p1_id": "m1", "p2_id": "m2" },
            { "id": "k", "type": "midpoint", "entities": ["l2", "l"], "extension": extension }
        ]))
    };
    let midpoint = |resp: &Value| {
        let (x1, y1) = point(resp, "m1");
        let (x2, y2) = point(resp, "m2");
        ((x1 + x2) / 2.0, (y1 + y2) / 2.0)
    };

    let resp = solve(sketch(false));
    assert_eq!(resp["status"], "converged", "{resp}");
    let (x, y) = midpoint(&resp);
    assert!(
        y.abs() < 1e-8 && x <= 10.0 + 1e-8,
        "pulled onto the segment: ({x}, {y})"
    );

    let resp = solve(sketch(true));
    assert_eq!(resp["status"], "converged", "{resp}");
    let (x, y) = midpoint(&resp);
    assert!(y.abs() < 1e-8, "midpoint on the Extension, got y = {y}");
    assert!(x > 14.0, "stays beyond the segment, got x = {x}");
}

// ── Conflicting and Redundant diagnosis ────────────────────────────────────

fn ids(v: &Value) -> Vec<&str> {
    v.as_array().unwrap().iter().map(|x| x.as_str().unwrap()).collect()
}

/// A rectangle anchored at a fixed corner with width 4 and height 3: 0 DOF.
fn rectangle() -> Vec<Value> {
    json!([
        { "id": "a", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "b", "type": "point", "x": 4.0, "y": 0.5 },
        { "id": "c", "type": "point", "x": 4.5, "y": 3.0 },
        { "id": "d", "type": "point", "x": 0.5, "y": 3.5 },
        { "id": "ab", "type": "line", "p1_id": "a", "p2_id": "b" },
        { "id": "bc", "type": "line", "p1_id": "b", "p2_id": "c" },
        { "id": "cd", "type": "line", "p1_id": "c", "p2_id": "d" },
        { "id": "da", "type": "line", "p1_id": "d", "p2_id": "a" },
        { "id": "k1", "type": "horizontal", "line": "ab" },
        { "id": "k2", "type": "horizontal", "line": "cd" },
        { "id": "k3", "type": "vertical", "line": "bc" },
        { "id": "k4", "type": "vertical", "line": "da" },
        { "id": "k5", "type": "distance", "a": "a", "b": "b", "value": 4.0 },
        { "id": "k6", "type": "distance", "a": "b", "b": "c", "value": 3.0 }
    ])
    .as_array()
    .unwrap()
    .clone()
}

fn rectangle_with(extra: Value) -> Value {
    let mut prims = rectangle();
    prims.extend(extra.as_array().unwrap().iter().cloned());
    request(Value::Array(prims))
}

#[test]
fn incompatible_distances_are_both_conflicting() {
    let resp = solve(request(json!([
        { "id": "p1", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "p2", "type": "point", "x": 3.0, "y": 4.0 },
        { "id": "p3", "type": "point", "x": 9.0, "y": 9.0 },
        { "id": "k1", "type": "distance", "a": "p1", "b": "p2", "value": 5.0 },
        { "id": "k2", "type": "distance", "a": "p1", "b": "p2", "value": 10.0 },
        { "id": "k3", "type": "x", "point": "p3", "value": 1.0 }
    ])));

    assert_eq!(resp["status"], "failed");
    assert_eq!(ids(&resp["conflicting"]), ["k1", "k2"]);
    assert_eq!(resp["redundant"], json!([]));
}

#[test]
fn constraint_between_fixed_points_that_does_not_hold_is_conflicting() {
    let resp = solve(request(json!([
        { "id": "p1", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "p2", "type": "point", "x": 3.0, "y": 4.0, "fixed": true },
        { "id": "k1", "type": "distance", "a": "p1", "b": "p2", "value": 7.0 }
    ])));
    assert_eq!(resp["status"], "failed");
    assert_eq!(ids(&resp["conflicting"]), ["k1"]);
}

#[test]
fn a_well_constrained_sketch_has_nothing_to_report() {
    let resp = solve(request(Value::Array(rectangle())));
    assert_eq!(resp["status"], "converged");
    assert_eq!(resp["conflicting"], json!([]));
    assert_eq!(resp["redundant"], json!([]));
}

#[test]
fn opposite_side_of_a_constrained_rectangle_is_redundant() {
    let resp = solve(rectangle_with(json!([
        { "id": "k7", "type": "distance", "a": "c", "b": "d", "value": 4.0 }
    ])));
    assert_eq!(resp["status"], "converged", "{resp}");
    assert_eq!(ids(&resp["redundant"]), ["k7"]);
    assert_eq!(resp["conflicting"], json!([]));
    assert_eq!(resp["dof"], 0);
}

#[test]
fn incompatible_opposite_side_of_a_rectangle_conflicts_with_what_implies_it() {
    let resp = solve(rectangle_with(json!([
        { "id": "k7", "type": "distance", "a": "c", "b": "d", "value": 6.0 }
    ])));
    assert_eq!(resp["status"], "failed");
    let conflicting = ids(&resp["conflicting"]);
    assert!(conflicting.contains(&"k5") && conflicting.contains(&"k7"), "{conflicting:?}");
    assert!(!conflicting.contains(&"k6"), "the height plays no part: {conflicting:?}");
    assert_eq!(resp["redundant"], json!([]));
}

#[test]
fn duplicated_horizontal_reports_the_later_one_as_redundant() {
    let resp = solve(request(json!([
        { "id": "a", "type": "point", "x": 0.0, "y": 0.0 },
        { "id": "b", "type": "point", "x": 4.0, "y": 1.0 },
        { "id": "l", "type": "line", "p1_id": "a", "p2_id": "b" },
        { "id": "h1", "type": "horizontal", "line": "l" },
        { "id": "h2", "type": "horizontal", "a": "a", "b": "b" }
    ])));
    assert_eq!(resp["status"], "converged");
    assert_eq!(ids(&resp["redundant"]), ["h2"]);
    assert_eq!(resp["conflicting"], json!([]));
    assert_eq!(resp["dof"], 3);
}

/// x = 3, y = 4 and |p| = 5 are three equations in two unknowns. At the
/// start (1, 1) they disagree, so a solve stopped after one iteration reports
/// all three as conflicting; once solved, the distance is merely redundant.
#[test]
fn diagnosis_happens_at_the_solved_position() {
    let mut req = request(json!([
        { "id": "o", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "p", "type": "point", "x": 1.0, "y": 1.0 },
        { "id": "k1", "type": "x", "point": "p", "value": 3.0 },
        { "id": "k2", "type": "y", "point": "p", "value": 4.0 },
        { "id": "k3", "type": "distance", "a": "o", "b": "p", "value": 5.0 }
    ]));

    let solved = solve(req.clone());
    assert_eq!(solved["status"], "converged");
    assert_eq!(solved["conflicting"], json!([]));
    assert_eq!(ids(&solved["redundant"]), ["k3"]);

    req["maxIterations"] = json!(1);
    let stopped = solve(req);
    assert_eq!(stopped["status"], "failed");
    assert_eq!(ids(&stopped["conflicting"]), ["k1", "k2", "k3"]);
}

#[test]
fn temporary_constraints_are_never_reported() {
    // A temporary duplicate of a permanent constraint is not redundant…
    let resp = solve(request(json!([
        { "id": "a", "type": "point", "x": 0.0, "y": 0.0 },
        { "id": "b", "type": "point", "x": 4.0, "y": 1.0 },
        { "id": "h1", "type": "horizontal", "a": "a", "b": "b" },
        { "id": "t1", "type": "horizontal", "a": "a", "b": "b", "temporary": true }
    ])));
    assert_eq!(resp["status"], "converged");
    assert_eq!(resp["redundant"], json!([]));

    // …and a drag that can't be honoured is a soft goal: the rectangle (0
    // DOF) solves and stays put, and nothing is blamed.
    let resp = solve(rectangle_with(json!([
        { "id": "t1", "type": "x", "point": "c", "value": 9.0, "temporary": true },
        { "id": "t2", "type": "y", "point": "c", "value": 9.0, "temporary": true }
    ])));
    assert_eq!(resp["status"], "converged", "{resp}");
    let (x, y) = point(&resp, "c");
    assert!((x - 4.0).abs() < 1e-8 && (y - 3.0).abs() < 1e-8, "({x}, {y})");
    assert_eq!(resp["conflicting"], json!([]));
    assert_eq!(resp["redundant"], json!([]));
    assert_eq!(resp["dof"], 0);
}

#[test]
fn temporary_constraints_are_soft_goals_within_the_real_ones() {
    // A point on a fixed Line dragged off it slides to the closest point.
    let resp = solve(request(json!([
        { "id": "a", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "b", "type": "point", "x": 10.0, "y": 0.0, "fixed": true },
        { "id": "l", "type": "line", "p1_id": "a", "p2_id": "b" },
        { "id": "p", "type": "point", "x": 2.0, "y": 0.0 },
        { "id": "k", "type": "on", "point": "p", "curve": "l" },
        { "id": "t1", "type": "x", "point": "p", "value": 6.0, "temporary": true },
        { "id": "t2", "type": "y", "point": "p", "value": 4.0, "temporary": true }
    ])));
    assert_eq!(resp["status"], "converged", "{resp}");
    let (x, y) = point(&resp, "p");
    assert!((x - 6.0).abs() < 1e-8 && y.abs() < 1e-8, "({x}, {y})");
    assert_eq!(resp["conflicting"], json!([]));
    assert_eq!(resp["redundant"], json!([]));
    assert_eq!(resp["dof"], 1);
}

#[test]
fn temporary_constraints_do_not_count_toward_dof() {
    let resp = solve(request(json!([
        { "id": "p", "type": "point", "x": 1.0, "y": 2.0 },
        { "id": "t1", "type": "x", "point": "p", "value": 3.0, "temporary": true },
        { "id": "t2", "type": "y", "point": "p", "value": 5.0, "temporary": true }
    ])));
    assert_eq!(resp["status"], "converged");
    let (x, y) = point(&resp, "p");
    assert!((x - 3.0).abs() < 1e-8 && (y - 5.0).abs() < 1e-8);
    assert_eq!(resp["dof"], 2);
    assert_eq!(resp["fullyConstrained"], json!([]));
}

// ── Drag goals at a fold ───────────────────────────────────────────────────
//
// An editor's sketch: a rectangle symmetric about a fixed origin with a
// horizontal bottom, a vertical left side and a top edge `cd` of length 80,
// plus an arc whose end lies on `cd`. `cd` starts exactly horizontal, so
// moving corner `a` sideways needs `cd` to tilt, which changes its length
// only quadratically: a fold, where the real constraints' Jacobian allows no
// sideways motion to first order.

const FOLD_POINTS: [(&str, (f64, f64)); 7] = [
    ("a", (-40.0, -50.0)),
    ("b", (40.0, -50.0)),
    ("c", (40.0, 50.0)),
    ("d", (-40.0, 50.0)),
    ("as", (40.0, 35.0)),
    ("ae", (25.0, 50.0)),
    ("ac", (25.0, 35.0)),
];

/// The fold sketch with its free points at `at` and a drag goal on `a`.
fn fold_request(at: &[(&str, (f64, f64))], goal: (f64, f64)) -> Value {
    let mut primitives = vec![json!({ "type": "point", "id": "o", "x": 0, "y": 0, "fixed": true })];
    for (id, (x, y)) in at {
        primitives.push(json!({ "type": "point", "id": id, "x": x, "y": y, "fixed": false }));
    }
    primitives.extend([
        json!({ "type": "line", "id": "ab", "p1_id": "a", "p2_id": "b" }),
        json!({ "type": "line", "id": "cd", "p1_id": "c", "p2_id": "d" }),
        json!({ "type": "line", "id": "da", "p1_id": "d", "p2_id": "a" }),
        json!({ "type": "horizontal", "id": "h", "line": "ab" }),
        json!({ "type": "vertical", "id": "v", "line": "da" }),
        json!({ "type": "line", "id": "diag", "p1_id": "a", "p2_id": "c" }),
        json!({ "type": "midpoint", "id": "s", "entities": ["o", "diag"] }),
        json!({ "type": "distance", "id": "w", "a": "c", "b": "d", "value": 80 }),
        json!({ "type": "arc", "id": "arc", "c_id": "ac", "start_id": "as", "end_id": "ae",
                "radius": 15, "start_angle": 0, "end_angle": std::f64::consts::FRAC_PI_2 }),
        json!({ "type": "on", "id": "on", "point": "ae", "curve": "cd" }),
        json!({ "type": "x", "id": "gx", "point": "a", "value": goal.0, "temporary": true }),
        json!({ "type": "y", "id": "gy", "point": "a", "value": goal.1, "temporary": true }),
    ]);
    request(Value::Array(primitives))
}

/// The free points' positions in `resp`, to feed back as the next request.
fn fold_positions(resp: &Value) -> Vec<(&'static str, (f64, f64))> {
    FOLD_POINTS.iter().map(|&(id, _)| (id, point(resp, id))).collect()
}

/// Every real constraint of the fold sketch holds in `resp`.
fn assert_fold_constraints_hold(resp: &Value) {
    let p = |id| point(resp, id);
    let (a, b, c, d, ae, ac, as_) = (p("a"), p("b"), p("c"), p("d"), p("ae"), p("ac"), p("as"));
    let tol = 1e-9;
    assert!((a.1 - b.1).abs() < tol, "ab horizontal");
    assert!((d.0 - a.0).abs() < tol, "da vertical");
    assert!((a.0 + c.0).abs() < tol && (a.1 + c.1).abs() < tol, "symmetric about o");
    assert!((((c.0 - d.0).powi(2) + (c.1 - d.1).powi(2)).sqrt() - 80.0).abs() < tol, "|cd| = 80");
    let (ux, uy) = (d.0 - c.0, d.1 - c.1);
    let t = ((ae.0 - c.0) * ux + (ae.1 - c.1) * uy) / (ux * ux + uy * uy);
    let off = ((ae.0 - c.0 - t * ux).powi(2) + (ae.1 - c.1 - t * uy).powi(2)).sqrt();
    assert!(off < tol && (-tol..=1.0 + tol).contains(&t), "ae on segment cd");
    let arc = primitive(resp, "arc");
    let r = arc["radius"].as_f64().unwrap();
    let (s, e) = (arc["start_angle"].as_f64().unwrap(), arc["end_angle"].as_f64().unwrap());
    assert!((as_.0 - ac.0 - r * s.cos()).abs() < tol && (as_.1 - ac.1 - r * s.sin()).abs() < tol, "arc start");
    assert!((ae.0 - ac.0 - r * e.cos()).abs() < tol && (ae.1 - ac.1 - r * e.sin()).abs() < tol, "arc end");
}

/// Ticket 23's repro: before, `a` didn't move and the solve still reported
/// `converged`, after ~1700 iterations.
#[test]
fn a_drag_at_a_fold_reaches_the_cursor() {
    let resp = solve(fold_request(&FOLD_POINTS, (-39.9, -49.9)));
    assert_eq!(resp["status"], "converged", "{resp}");
    assert_fold_constraints_hold(&resp);
    let (x, y) = point(&resp, "a");
    assert!((x + 39.9).abs() < 1e-8 && (y + 49.9).abs() < 1e-8, "a at ({x}, {y})");
    assert_eq!(resp["conflicting"], json!([]));
    assert_eq!(resp["redundant"], json!([]));
    let iterations = resp["stats"]["iterations"].as_u64().unwrap();
    eprintln!("fold repro: {iterations} iterations");
    assert!(iterations <= 40, "{iterations} iterations");
}

/// A drag away from the fold, back and on past it, each result fed back as
/// the next request: `a` follows the cursor in small steps, each converging
/// quickly. Past the fold (x < -40) the cursor is out of reach: `a` stops at
/// x = -40 (with `cd` horizontal again), level with the cursor.
#[test]
fn a_drag_sequence_across_the_fold_moves_smoothly() {
    let mut at = FOLD_POINTS.to_vec();
    let mut prev = (-40.0, -50.0);
    let mut worst = 0;
    let mut per_step = Vec::new();
    let steps = 44;
    for k in 1..=steps {
        let t = (k as f64 - 0.5) / 40.0;
        // Sideways in to x = -34 and back out past -40 while rising.
        let cursor = (-40.0 + 6.0 * (std::f64::consts::PI * t).sin(), -50.0 + 4.0 * t);
        let resp = solve(fold_request(&at, cursor));
        assert_eq!(resp["status"], "converged", "step {k}: {resp}");
        assert_fold_constraints_hold(&resp);
        let a = point(&resp, "a");
        let expected = (cursor.0.max(-40.0), cursor.1);
        assert!(
            (a.0 - expected.0).abs() < 1e-8 && (a.1 - expected.1).abs() < 1e-8,
            "step {k}: a at {a:?}, expected {expected:?}"
        );
        let jump = ((a.0 - prev.0).powi(2) + (a.1 - prev.1).powi(2)).sqrt();
        assert!(jump < 1.0, "step {k} jumped {jump}");
        let iterations = resp["stats"]["iterations"].as_u64().unwrap();
        worst = worst.max(iterations);
        per_step.push(iterations);
        prev = a;
        at = fold_positions(&resp);
    }
    eprintln!("fold drag: iterations per step {per_step:?}");
    assert!(worst <= 40, "{worst} iterations in one step");
}

// ── Parameters and property references ─────────────────────────────────────

fn primitive<'a>(resp: &'a Value, id: &str) -> &'a Value {
    resp["primitives"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == id)
        .unwrap()
}

fn radius(resp: &Value, id: &str) -> f64 {
    primitive(resp, id)["radius"].as_f64().unwrap()
}

/// One Parameter drives a point-to-point distance, a point-to-line distance
/// and a signed distance; changing it moves all three.
fn parameter_sketch(offset: f64) -> Value {
    with_fixed_line(json!([
        { "id": "offset", "type": "param", "name": "Offset1", "value": offset, "group": "g" },
        { "id": "p", "type": "point", "x": 3.0, "y": 1.0 },
        { "id": "q", "type": "point", "x": 5.0, "y": -1.0 },
        { "id": "r", "type": "point", "x": 6.0, "y": 4.0 },
        { "id": "k1", "type": "offset", "point": "p", "line": "l",
          "value": "offset", "side": 1 },
        { "id": "k2", "type": "distance", "a": "q", "b": "l", "value": "offset" },
        { "id": "k3", "type": "distance", "a": "p", "b": "r", "value": "offset" }
    ]))
}

#[test]
fn a_parameter_drives_every_constraint_that_names_it() {
    for offset in [2.0, 3.5] {
        let resp = solve(parameter_sketch(offset));
        assert_eq!(resp["status"], "converged", "{resp}");
        let (_, py) = point(&resp, "p");
        let (_, qy) = point(&resp, "q");
        let (px, _) = point(&resp, "p");
        let (rx, ry) = point(&resp, "r");
        assert!((py - offset).abs() < 1e-8, "p {offset} above l, got {py}");
        assert!((qy + offset).abs() < 1e-8, "q {offset} below l, got {qy}");
        let pr = ((rx - px).powi(2) + (ry - py).powi(2)).sqrt();
        assert!((pr - offset).abs() < 1e-8, "r {offset} from p, got {pr}");
    }
}

#[test]
fn parameters_are_echoed_back_unchanged() {
    let req = parameter_sketch(2.0);
    let resp = solve(req.clone());
    let param = req["primitives"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == "offset")
        .unwrap();
    assert_eq!(primitive(&resp, "offset"), param);
}

#[test]
fn signed_distance_holds_the_point_on_its_side_of_the_extension() {
    // p starts below l (on its right) and beyond its far end.
    let sketch = |side: i32| {
        with_fixed_line(json!([
            { "id": "p", "type": "point", "x": 15.0, "y": -1.0 },
            { "id": "k", "type": "offset", "point": "p", "line": "l",
              "value": 2.0, "side": side }
        ]))
    };

    // side 1 is left of a → b: above the line, so p crosses over.
    let resp = solve(sketch(1));
    assert_eq!(resp["status"], "converged", "{resp}");
    let (x, y) = point(&resp, "p");
    assert!((y - 2.0).abs() < 1e-8, "2 above the Extension, got y = {y}");
    assert!(x > 14.0, "stays beyond the segment, got x = {x}");

    let resp = solve(sketch(-1));
    assert_eq!(resp["status"], "converged", "{resp}");
    let (x, y) = point(&resp, "p");
    assert!((y + 2.0).abs() < 1e-8, "2 below the Extension, got y = {y}");
    assert!(x > 14.0, "stays beyond the segment, got x = {x}");
}

/// A Linked Offset of a circle: the offset's radius is the fixed source's
/// minus the Parameter (b − a = value).
#[test]
fn difference_over_radius_references_solves() {
    let sketch = |offset: f64| {
        request(json!([
            { "id": "o", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
            { "id": "src", "type": "circle", "c_id": "o", "radius": 15.0, "fixed": true },
            { "id": "off", "type": "circle", "c_id": "o", "radius": 10.0 },
            { "id": "d", "type": "param", "name": "Offset1", "value": offset },
            { "id": "k", "type": "difference",
              "a": { "entity": "off", "property": "radius" },
              "b": { "entity": "src", "property": "radius" },
              "value": "d" }
        ]))
    };
    for offset in [2.0, 4.0] {
        let resp = solve(sketch(offset));
        assert_eq!(resp["status"], "converged", "{resp}");
        assert!((radius(&resp, "src") - 15.0).abs() < 1e-12, "the fixed source keeps its radius");
        assert!((radius(&resp, "off") - (15.0 - offset)).abs() < 1e-8, "{resp}");
        assert_eq!(resp["dof"], 0);
    }
}

#[test]
fn equal_over_radius_references_solves() {
    let resp = solve(request(json!([
        { "id": "o1", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "o2", "type": "point", "x": 9.0, "y": 0.0, "fixed": true },
        { "id": "c1", "type": "circle", "c_id": "o1", "radius": 3.0 },
        { "id": "c2", "type": "circle", "c_id": "o2", "radius": 1.0 },
        { "id": "k1", "type": "equal",
          "a": { "entity": "c1", "property": "radius" },
          "b": { "entity": "c2", "property": "radius" } },
        { "id": "k2", "type": "equal", "a": { "entity": "c2", "property": "radius" }, "b": 2.5 }
    ])));
    assert_eq!(resp["status"], "converged", "{resp}");
    assert!((radius(&resp, "c1") - 2.5).abs() < 1e-8, "{resp}");
    assert!((radius(&resp, "c2") - 2.5).abs() < 1e-8, "{resp}");
}

/// Points expose `x` and `y`.
#[test]
fn equal_over_point_coordinates_solves() {
    let resp = solve(request(json!([
        { "id": "p", "type": "point", "x": 1.0, "y": 2.0 },
        { "id": "q", "type": "point", "x": 4.0, "y": 6.0, "fixed": true },
        { "id": "k1", "type": "equal", "a": { "entity": "p", "property": "x" }, "b": { "entity": "q", "property": "y" } },
        { "id": "k2", "type": "equal", "a": { "entity": "p", "property": "y" }, "b": -3 }
    ])));
    assert_eq!(resp["status"], "converged", "{resp}");
    let (x, y) = point(&resp, "p");
    assert!((x - 6.0).abs() < 1e-8 && (y + 3.0).abs() < 1e-8, "({x}, {y})");
}

fn rejected_constraint(primitives: Value) -> (String, Value) {
    let resp = reject(&request(primitives).to_string());
    (resp["error"].as_str().unwrap().to_string(), resp["constraintId"].clone())
}

#[test]
fn unknown_parameter_id_is_rejected() {
    let (error, id) = rejected_constraint(json!([
        { "id": "a", "type": "point", "x": 0.0, "y": 0.0 },
        { "id": "b", "type": "point", "x": 1.0, "y": 0.0 },
        { "id": "k", "type": "distance", "a": "a", "b": "b", "value": "nope" }
    ]));
    assert_eq!(error, "constraint k: field 'value': unknown Parameter 'nope'");
    assert_eq!(id, "k");
}

#[test]
fn bad_property_references_are_rejected() {
    let base = || {
        json!([
            { "id": "o", "type": "point", "x": 0.0, "y": 0.0 },
            { "id": "c", "type": "circle", "c_id": "o", "radius": 1.0 },
            { "id": "d", "type": "param", "value": 1.0 }
        ])
    };
    let with = |a: Value| {
        let mut prims = base();
        prims.as_array_mut().unwrap().push(json!(
            { "id": "k", "type": "equal", "a": a, "b": 2.0 }
        ));
        rejected_constraint(prims)
    };

    let (error, id) = with(json!({ "entity": "c", "property": "diameter" }));
    assert_eq!(error, "constraint k: field 'a': circle 'c' has no property 'diameter'");
    assert_eq!(id, "k");

    let (error, _) = with(json!({ "entity": "ghost", "property": "radius" }));
    assert_eq!(error, "constraint k: field 'a': 'ghost' is not an entity");

    let (error, _) = with(json!({ "entity": "d", "property": "value" }));
    assert_eq!(error, "constraint k: field 'a': 'd' is not an entity");

    let (error, _) = with(json!({ "entity": "c" }));
    assert_eq!(error, "constraint k: field 'a': property reference has no 'property'");

    let (error, _) = with(json!("missing"));
    assert_eq!(error, "constraint k: field 'a': unknown Parameter 'missing'");
}

/// Property references are for `difference` and `equal`; other scalar fields
/// take a number or a Parameter id.
#[test]
fn property_reference_in_a_scalar_field_is_rejected() {
    let (error, _) = rejected_constraint(json!([
        { "id": "o", "type": "point", "x": 0.0, "y": 0.0 },
        { "id": "c", "type": "circle", "c_id": "o", "radius": 1.0 },
        { "id": "k", "type": "radius", "curve": "c", "value": { "entity": "c", "property": "radius" } }
    ]));
    assert_eq!(error, "constraint k: field 'value' is not a number or a Parameter id");
}

#[test]
fn parameter_without_a_value_is_rejected() {
    let resp = reject(&request(json!([{ "id": "d", "type": "param", "name": "d" }])).to_string());
    assert_eq!(resp["error"], "param d: missing field 'value'");
}

/// A relation between two constants moves nothing: it holds or it conflicts.
#[test]
fn relation_between_constants_holds_or_conflicts() {
    let sketch = |value: f64| {
        request(json!([
            { "id": "d", "type": "param", "value": value },
            { "id": "k", "type": "equal", "a": "d", "b": 2.0 }
        ]))
    };
    let resp = solve(sketch(2.0));
    assert_eq!(resp["status"], "converged", "{resp}");
    assert_eq!(resp["conflicting"], json!([]));

    let resp = solve(sketch(3.0));
    assert_eq!(resp["status"], "failed", "{resp}");
    assert_eq!(resp["conflicting"], json!(["k"]));
}

// ── Arcs ───────────────────────────────────────────────────────────────────
//
// An arc references its center, start and end Points, and its endpoints
// always lie on it. Arcs sweep counter-clockwise
// from `start_angle` to `end_angle`.

use std::f64::consts::{FRAC_PI_2, PI};

fn num(resp: &Value, id: &str, field: &str) -> f64 {
    primitive(resp, id)[field].as_f64().unwrap()
}

/// Appends `rest` to `primitives` and wraps them in a request.
fn with(mut primitives: Value, rest: Value) -> Value {
    primitives
        .as_array_mut()
        .unwrap()
        .extend(rest.as_array().unwrap().iter().cloned());
    request(primitives)
}

/// Asserts arc `id`'s rules hold in a response: its start and end
/// Points sit at its radius from its center, at its start and end angles.
fn assert_arc_rules_hold(resp: &Value, id: &str) {
    let a = primitive(resp, id);
    let (cx, cy) = point(resp, a["c_id"].as_str().unwrap());
    let r = a["radius"].as_f64().unwrap();
    for (end, angle) in [("start_id", "start_angle"), ("end_id", "end_angle")] {
        let (x, y) = point(resp, a[end].as_str().unwrap());
        let t = a[angle].as_f64().unwrap();
        assert!(
            (x - (cx + r * t.cos())).abs() < 1e-8 && (y - (cy + r * t.sin())).abs() < 1e-8,
            "{id}'s {end} ({x}, {y}) is off the arc: center ({cx}, {cy}), r {r}, angle {t}"
        );
    }
}

/// How far (in radians) the direction of `p` from arc `arc`'s center falls
/// outside the arc's span, which sweeps counter-clockwise from its start
/// angle to its end angle: 0 within it.
fn off_span(resp: &Value, arc: &str, (px, py): (f64, f64)) -> f64 {
    let (cx, cy) = point(resp, primitive(resp, arc)["c_id"].as_str().unwrap());
    let start = num(resp, arc, "start_angle");
    let end = num(resp, arc, "end_angle");
    let u = ((py - cy).atan2(px - cx) - start).rem_euclid(2.0 * PI);
    let mut sweep = (end - start).rem_euclid(2.0 * PI);
    if sweep == 0.0 {
        sweep = 2.0 * PI;
    }
    if u <= sweep { 0.0 } else { (u - sweep).min(2.0 * PI - u) }
}

/// A free quarter arc around `o` from (10, 0) to (0, 10).
fn quarter_arc(rest: Value) -> Value {
    with(
        json!([
            { "id": "o", "type": "point", "x": 0.0, "y": 0.0 },
            { "id": "s", "type": "point", "x": 10.0, "y": 0.0 },
            { "id": "e", "type": "point", "x": 0.0, "y": 10.0 },
            { "id": "a", "type": "arc", "c_id": "o", "start_id": "s", "end_id": "e",
              "radius": 10.0, "start_angle": 0.0, "end_angle": FRAC_PI_2 }
        ]),
        rest,
    )
}

#[test]
fn line_sharing_an_arc_endpoint_stays_attached_as_radius_and_center_change() {
    let resp = solve(quarter_arc(json!([
        { "id": "q", "type": "point", "x": -10.0, "y": 10.0, "fixed": true },
        { "id": "l", "type": "line", "p1_id": "e", "p2_id": "q" },
        { "id": "k1", "type": "radius", "curve": "a", "value": 15.0 },
        { "id": "k2", "type": "x", "point": "o", "value": 2.0 },
        { "id": "k3", "type": "y", "point": "o", "value": -3.0 }
    ])));

    assert_eq!(resp["status"], "converged", "{resp}");
    assert!((num(&resp, "a", "radius") - 15.0).abs() < 1e-8);
    let (ox, oy) = point(&resp, "o");
    assert!((ox - 2.0).abs() < 1e-8 && (oy + 3.0).abs() < 1e-8);
    // The line's endpoint is the arc's end Point, which moved with the arc.
    assert_arc_rules_hold(&resp, "a");
    let (ex, ey) = point(&resp, "e");
    assert!((((ex - 2.0).powi(2) + (ey + 3.0).powi(2)).sqrt() - 15.0).abs() < 1e-8);
}

#[test]
fn dragging_an_arc_endpoint_moves_the_arc_with_it() {
    let resp = solve(quarter_arc(json!([
        { "id": "t1", "type": "x", "point": "e", "value": 1.0, "temporary": true },
        { "id": "t2", "type": "y", "point": "e", "value": 12.0, "temporary": true }
    ])));
    assert_eq!(resp["status"], "converged", "{resp}");
    let (ex, ey) = point(&resp, "e");
    assert!((ex - 1.0).abs() < 1e-8 && (ey - 12.0).abs() < 1e-8);
    assert_arc_rules_hold(&resp, "a");
}

#[test]
fn a_free_arc_has_five_degrees_of_freedom() {
    // Center (2), radius and two angles; the endpoints follow.
    let resp = solve(quarter_arc(json!([])));
    assert_eq!(resp["status"], "converged", "{resp}");
    assert_eq!(resp["dof"], 5);
}

/// The quarter arc's primitives, its endpoints off it.
fn quarter_arc_off_its_endpoints() -> Value {
    json!([
        { "id": "o", "type": "point", "x": 0.0, "y": 0.0 },
        { "id": "s", "type": "point", "x": 12.0, "y": 1.0 },
        { "id": "e", "type": "point", "x": -1.0, "y": 9.0 },
        { "id": "a", "type": "arc", "c_id": "o", "start_id": "s", "end_id": "e",
          "radius": 10.0, "start_angle": 0.0, "end_angle": FRAC_PI_2 }
    ])
}

#[test]
fn an_arc_keeps_its_endpoints_on_itself() {
    let resp = solve(request(quarter_arc_off_its_endpoints()));
    assert_eq!(resp["status"], "converged", "{resp}");
    assert_arc_rules_hold(&resp, "a");
    assert_eq!(resp["dof"], 5);
    assert_eq!(resp["redundant"], json!([]));
}

#[test]
fn a_reference_arc_and_its_endpoints_never_move() {
    let reference = json!([
        { "id": "o", "type": "point", "x": 0.0, "y": 0.0, "isReference": true },
        { "id": "s", "type": "point", "x": 5.0, "y": 0.0, "isReference": true },
        { "id": "e", "type": "point", "x": 0.0, "y": 5.0, "isReference": true },
        { "id": "a", "type": "arc", "c_id": "o", "start_id": "s", "end_id": "e",
          "radius": 5.0, "start_angle": 0.0, "end_angle": FRAC_PI_2, "isReference": true },
        { "id": "q", "type": "point", "x": 3.0, "y": 7.0 },
        { "id": "l", "type": "line", "p1_id": "e", "p2_id": "q" },
        { "id": "k", "type": "distance", "a": "e", "b": "q", "value": 8.0 },
        { "id": "t1", "type": "x", "point": "s", "value": 9.0, "temporary": true },
        { "id": "t2", "type": "y", "point": "s", "value": 9.0, "temporary": true }
    ]);
    let resp = solve(request(reference.clone()));
    assert_eq!(resp["status"], "converged", "{resp}");
    for id in ["o", "s", "e", "a"] {
        assert_eq!(primitive(&resp, id), primitive(&request(reference.clone()), id), "{id}");
    }
}

#[test]
fn arc_needs_its_endpoint_points() {
    let resp = reject(
        &request(json!([
            { "id": "o", "type": "point", "x": 0.0, "y": 0.0 },
            { "id": "a", "type": "arc", "c_id": "o", "radius": 1.0,
              "start_angle": 0.0, "end_angle": 1.0 }
        ]))
        .to_string(),
    );
    assert_eq!(resp["error"], "arc a: missing field 'start_id'");

    let resp = reject(
        &request(json!([
            { "id": "o", "type": "point", "x": 0.0, "y": 0.0 },
            { "id": "s", "type": "point", "x": 1.0, "y": 0.0 },
            { "id": "a", "type": "arc", "c_id": "o", "start_id": "s", "end_id": "nowhere",
              "radius": 1.0, "start_angle": 0.0, "end_angle": 1.0 }
        ]))
        .to_string(),
    );
    assert_eq!(resp["error"], "arc a: 'nowhere' is not a point");
}

/// A fixed quarter arc of radius 5 around the origin, from angle 0 to π/2.
fn fixed_quarter_arc(rest: Value) -> Value {
    with(
        json!([
            { "id": "o", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
            { "id": "s", "type": "point", "x": 5.0, "y": 0.0, "fixed": true },
            { "id": "e", "type": "point", "x": 0.0, "y": 5.0, "fixed": true },
            { "id": "a", "type": "arc", "c_id": "o", "start_id": "s", "end_id": "e",
              "radius": 5.0, "start_angle": 0.0, "end_angle": FRAC_PI_2, "fixed": true }
        ]),
        rest,
    )
}

#[test]
fn point_on_arc_lands_on_the_arcs_span() {
    // On the arc's circle but off its span, then off the circle on either side.
    for (x, y) in [(-3.0, -4.0), (-7.0, 1.0), (8.0, -3.0), (2.0, 2.0)] {
        let resp = solve(fixed_quarter_arc(json!([
            { "id": "p", "type": "point", "x": x, "y": y },
            { "id": "k", "type": "on", "point": "p", "curve": "a" }
        ])));
        assert_eq!(resp["status"], "converged", "{resp}");
        let p = point(&resp, "p");
        assert!(((p.0 * p.0 + p.1 * p.1).sqrt() - 5.0).abs() < 1e-8, "{p:?}");
        assert!(off_span(&resp, "a", p) < 1e-8, "from ({x}, {y}) to {p:?}, off the span");
    }
}

#[test]
fn point_on_arc_extends_a_free_arc_to_reach_a_fixed_point() {
    let resp = solve(quarter_arc(json!([
        { "id": "p", "type": "point", "x": -10.0, "y": 0.5, "fixed": true },
        { "id": "k", "type": "on", "point": "p", "curve": "a" }
    ])));
    assert_eq!(resp["status"], "converged", "{resp}");
    assert_arc_rules_hold(&resp, "a");
    assert!(off_span(&resp, "a", (-10.0, 0.5)) < 1e-8);
}

/// The foot of the perpendicular from `c` onto line `l`, and its parameter
/// along the line (0 at p1, 1 at p2).
fn foot(resp: &Value, l: &str, (cx, cy): (f64, f64)) -> ((f64, f64), f64) {
    let line = primitive(resp, l);
    let (ax, ay) = point(resp, line["p1_id"].as_str().unwrap());
    let (bx, by) = point(resp, line["p2_id"].as_str().unwrap());
    let (dx, dy) = (bx - ax, by - ay);
    let t = ((cx - ax) * dx + (cy - ay) * dy) / (dx * dx + dy * dy);
    ((ax + t * dx, ay + t * dy), t)
}

#[test]
fn tangent_line_arc_touches_the_arc_on_its_span_and_the_segment() {
    // From a line crossing the arc, from one tangent to the arc's circle at
    // (0, -5), off the arc's span, and from one whose segment stops short.
    for ((x1, y1), (x2, y2)) in [
        ((1.0, 7.0), (7.0, 1.0)),
        ((-3.0, -5.0), (3.0, -5.0)),
        ((9.0, 9.0), (12.0, 6.0)),
    ] {
        let resp = solve(fixed_quarter_arc(json!([
            { "id": "p1", "type": "point", "x": x1, "y": y1 },
            { "id": "p2", "type": "point", "x": x2, "y": y2 },
            { "id": "l", "type": "line", "p1_id": "p1", "p2_id": "p2" },
            { "id": "k", "type": "tangent", "a": "l", "b": "a" }
        ])));
        assert_eq!(resp["status"], "converged", "{resp}");
        let (f, t) = foot(&resp, "l", (0.0, 0.0));
        assert!(((f.0 * f.0 + f.1 * f.1).sqrt() - 5.0).abs() < 1e-8, "tangent: {f:?}");
        assert!((-1e-8..=1.0 + 1e-8).contains(&t), "touches the segment, t = {t}");
        assert!(off_span(&resp, "a", f) < 1e-8, "touches the arc's span, at {f:?}");
    }
}

#[test]
fn slot_of_two_arcs_and_two_tangent_lines_solves() {
    let resp = solve(request(json!([
        { "id": "c1", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "c2", "type": "point", "x": 20.5, "y": 0.4 },
        { "id": "s1", "type": "point", "x": 0.3, "y": 5.2 },
        { "id": "e1", "type": "point", "x": -0.2, "y": -4.9 },
        { "id": "s2", "type": "point", "x": 20.0, "y": -5.3 },
        { "id": "e2", "type": "point", "x": 19.6, "y": 5.1 },
        { "id": "a1", "type": "arc", "c_id": "c1", "start_id": "s1", "end_id": "e1",
          "radius": 5.0, "start_angle": FRAC_PI_2, "end_angle": 3.0 * FRAC_PI_2 },
        { "id": "a2", "type": "arc", "c_id": "c2", "start_id": "s2", "end_id": "e2",
          "radius": 5.2, "start_angle": -FRAC_PI_2, "end_angle": FRAC_PI_2 },
        { "id": "top", "type": "line", "p1_id": "e2", "p2_id": "s1" },
        { "id": "bottom", "type": "line", "p1_id": "e1", "p2_id": "s2" },
        { "id": "k1", "type": "tangent", "a": "top", "b": "a1" },
        { "id": "k2", "type": "tangent", "a": "top", "b": "a2" },
        { "id": "k3", "type": "tangent", "a": "bottom", "b": "a1" },
        { "id": "k4", "type": "tangent", "a": "bottom", "b": "a2" },
        { "id": "k5", "type": "radius", "curve": "a1", "value": 5.0 },
        { "id": "k6", "type": "horizontal", "line": "top" },
        { "id": "k7", "type": "distance", "a": "c1", "b": "c2", "value": 20.0 },
        { "id": "k8", "type": "equal", "a": "a1", "b": "a2" }
    ])));
    assert_eq!(resp["status"], "converged", "{resp}");
    assert_arc_rules_hold(&resp, "a1");
    assert_arc_rules_hold(&resp, "a2");
    assert!((num(&resp, "a2", "radius") - 5.0).abs() < 1e-6);
    let (_, ty) = point(&resp, "s1");
    assert!((ty.abs() - 5.0).abs() < 1e-6, "top line tangent at height ±5, got {ty}");
}

#[test]
fn every_arc_constraint_holds_with_shared_points() {
    // The line runs from the arc's end Point, and a point on the arc is
    // also the line's far end.
    let resp = solve(quarter_arc(json!([
        { "id": "q", "type": "point", "x": -6.0, "y": 9.0 },
        { "id": "l", "type": "line", "p1_id": "e", "p2_id": "q" },
        { "id": "k1", "type": "tangent", "a": "l", "b": "a" },
        { "id": "k2", "type": "x", "point": "o", "value": 0.0 },
        { "id": "k3", "type": "y", "point": "o", "value": 0.0 }
    ])));
    assert_eq!(resp["status"], "converged", "{resp}");
    assert_arc_rules_hold(&resp, "a");
}

// ── Reference Geometry ─────────────────────────────────────────────────────

/// Reference Geometry: a point, a circle and an arc (with its Points), all
/// `isReference` and none `fixed`.
fn reference_geometry() -> Value {
    json!([
        { "id": "ro", "type": "point", "x": 0.0, "y": 0.0, "isReference": true },
        { "id": "rs", "type": "point", "x": 5.0, "y": 0.0, "isReference": true },
        { "id": "re", "type": "point", "x": 0.0, "y": 5.0, "isReference": true },
        { "id": "rc", "type": "circle", "c_id": "ro", "radius": 3.0, "isReference": true },
        { "id": "ra", "type": "arc", "c_id": "ro", "start_id": "rs", "end_id": "re",
          "radius": 5.0, "start_angle": 0.0, "end_angle": FRAC_PI_2, "isReference": true }
    ])
}

fn assert_reference_geometry_unmoved(resp: &Value) {
    for (id, (x, y)) in [("ro", (0.0, 0.0)), ("rs", (5.0, 0.0)), ("re", (0.0, 5.0))] {
        assert_eq!(point(resp, id), (x, y), "{id} moved");
    }
    assert_eq!(num(resp, "rc", "radius"), 3.0);
    assert_eq!(num(resp, "ra", "radius"), 5.0);
    assert_eq!(num(resp, "ra", "start_angle"), 0.0);
    assert_eq!(num(resp, "ra", "end_angle"), FRAC_PI_2);
}

#[test]
fn reference_geometry_holds_while_free_geometry_follows_it() {
    let resp = solve(with(
        reference_geometry(),
        json!([
            { "id": "o2", "type": "point", "x": 10.0, "y": 0.0 },
            { "id": "kc", "type": "circle", "c_id": "o2", "radius": 1.0 },
            { "id": "q", "type": "point", "x": 9.0, "y": 1.0 },
            { "id": "q2", "type": "point", "x": 1.0, "y": 9.0 },
            { "id": "l", "type": "line", "p1_id": "q", "p2_id": "q2" },
            { "id": "p", "type": "point", "x": -2.0, "y": -7.0 },
            { "id": "k1", "type": "equal", "a": "rc", "b": "kc" },
            { "id": "k2", "type": "tangent", "a": "rc", "b": "kc" },
            { "id": "k3", "type": "on", "point": "p", "curve": "ra" },
            { "id": "k4", "type": "tangent", "a": "l", "b": "ra" },
            { "id": "k5", "type": "equal", "a": "ra", "b": "ra" }
        ]),
    ));

    assert_eq!(resp["status"], "converged", "{resp}");
    assert_reference_geometry_unmoved(&resp);
    assert!((num(&resp, "kc", "radius") - 3.0).abs() < 1e-8, "free circle took the reference radius");
}

#[test]
fn reference_geometry_holds_even_when_constrained_against_it() {
    let resp = solve(with(
        reference_geometry(),
        json!([
            { "id": "k1", "type": "radius", "curve": "rc", "value": 4.0 },
            { "id": "k2", "type": "radius", "curve": "ra", "value": 6.0 },
            { "id": "k3", "type": "x", "point": "rs", "value": 9.0 }
        ]),
    ));

    assert_eq!(resp["status"], "failed", "{resp}");
    assert_reference_geometry_unmoved(&resp);
    let mut conflicting = ids(&resp["conflicting"]);
    conflicting.sort();
    assert_eq!(conflicting, ["k1", "k2", "k3"]);
}

// ── Direction, mirror and arrays ───────────────────────────────────────────

fn close(resp: &Value, id: &str, x: f64, y: f64) {
    let (px, py) = point(resp, id);
    assert!(
        (px - x).abs() < 1e-7 && (py - y).abs() < 1e-7,
        "{id}: expected ({x}, {y}), got ({px}, {py})"
    );
}

/// Appends temporary coordinate constraints that drag `id` to (x, y), the
/// way an editor drags a point.
fn drag(mut req: Value, id: &str, x: f64, y: f64) -> Value {
    let prims = req["primitives"].as_array_mut().unwrap();
    prims.push(json!({ "id": "drag_x", "type": "x", "point": id, "value": x,
                       "temporary": true }));
    prims.push(json!({ "id": "drag_y", "type": "y", "point": id, "value": y,
                       "temporary": true }));
    req
}

#[test]
fn direction_solves_to_the_requested_angle() {
    for angle in [0.4, 2.5, -2.0, std::f64::consts::PI] {
        let resp = solve(request(json!([
            { "id": "c", "type": "point", "x": 1.0, "y": 1.0, "fixed": true },
            { "id": "p", "type": "point", "x": 4.0, "y": 1.5 },
            { "id": "k", "type": "direction", "a": "c", "b": "p", "value": angle },
            { "id": "d", "type": "distance", "a": "c", "b": "p", "value": 2.0 }
        ])));
        assert_eq!(resp["status"], "converged", "{angle}: {resp}");
        close(&resp, "p", 1.0 + 2.0 * angle.cos(), 1.0 + 2.0 * angle.sin());
    }
}

#[test]
fn direction_takes_a_parameter() {
    let resp = solve(request(json!([
        { "id": "a", "type": "param", "value": 1.0 },
        { "id": "c", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "p", "type": "point", "x": 3.0, "y": 0.0 },
        { "id": "k", "type": "direction", "a": "c", "b": "p", "value": "a" },
        { "id": "d", "type": "distance", "a": "c", "b": "p", "value": 3.0 }
    ])));
    assert_eq!(resp["status"], "converged", "{resp}");
    close(&resp, "p", 3.0 * 1f64.cos(), 3.0 * 1f64.sin());
}

/// Rotates (x, y) about (cx, cy) by `a`.
fn rotate(x: f64, y: f64, cx: f64, cy: f64, a: f64) -> (f64, f64) {
    let (s, c) = a.sin_cos();
    (cx + c * (x - cx) - s * (y - cy), cy + s * (x - cx) + c * (y - cy))
}

/// A circular array of a line and a circle with 3 copies about a fixed
/// center: per copy, a rotated point per source point with a `rotation`
/// (angle 2π/(n+1)·k), the copied line, and an `equal` for the copied
/// circle. Like an array tool, copies are
/// placed where they belong.
fn circular_array_sketch() -> Value {
    let sources = [("a", 5.0, 2.0), ("b", 7.0, 3.0), ("cc", 4.0, 5.0)];
    let mut prims = vec![
        json!({ "id": "o", "type": "point", "x": 1.0, "y": 2.0, "fixed": true }),
        json!({ "id": "a", "type": "point", "x": 5.0, "y": 2.0 }),
        json!({ "id": "b", "type": "point", "x": 7.0, "y": 3.0 }),
        json!({ "id": "l", "type": "line", "p1_id": "a", "p2_id": "b" }),
        json!({ "id": "cc", "type": "point", "x": 4.0, "y": 5.0 }),
        json!({ "id": "c", "type": "circle", "c_id": "cc", "radius": 0.5 }),
    ];
    let n = 3;
    for k in 1..=n {
        let angle = std::f64::consts::TAU / (n as f64 + 1.0) * k as f64;
        for (src, x, y) in sources {
            let (x, y) = rotate(x, y, 1.0, 2.0, angle);
            prims.push(json!({ "id": format!("{src}{k}"), "type": "point", "x": x, "y": y }));
            prims.push(json!({ "id": format!("ci_{src}{k}"), "type": "rotation",
                               "source": src, "copy": format!("{src}{k}"),
                               "center": "o", "angle": angle }));
        }
        prims.push(json!({ "id": format!("l{k}"), "type": "line",
                           "p1_id": format!("a{k}"), "p2_id": format!("b{k}") }));
        prims.push(json!({ "id": format!("c{k}"), "type": "circle",
                           "c_id": format!("cc{k}"), "radius": 0.5 }));
        prims.push(json!({ "id": format!("er{k}"), "type": "equal",
                           "a": "c", "b": format!("c{k}") }));
    }
    request(Value::Array(prims))
}

fn assert_circular_copies(resp: &Value) {
    for k in 1..=3 {
        let angle = std::f64::consts::TAU / 4.0 * k as f64;
        for id in ["a", "b", "cc"] {
            let (x, y) = point(resp, id);
            let (ex, ey) = rotate(x, y, 1.0, 2.0, angle);
            close(resp, &format!("{id}{k}"), ex, ey);
        }
        assert!((radius(resp, &format!("c{k}")) - radius(resp, "c")).abs() < 1e-8);
    }
}

#[test]
fn circular_array_solves_and_follows_its_source() {
    let req = circular_array_sketch();
    let resp = solve(req.clone());
    assert_eq!(resp["status"], "converged", "{resp}");
    close(&resp, "a", 5.0, 2.0); // the source doesn't move
    assert_circular_copies(&resp);

    let resp = solve(drag(req, "a", 3.0, -1.0));
    assert_eq!(resp["status"], "converged", "{resp}");
    close(&resp, "a", 3.0, -1.0);
    assert_circular_copies(&resp);
    // a's first copy (90° about (1, 2)) moved with it.
    close(&resp, "a1", 4.0, 4.0);
}

/// A linear array of a line along a fixed axis line (direction ax1 → ax2,
/// or ax2 → ax1 with `flip`): per copy and source point a `translation`
/// with `distance` and `count` = copy index. Like an array tool, copies
/// are placed where they belong.
fn linear_array_sketch(flip: bool) -> Value {
    let sign = if flip { -1.0 } else { 1.0 };
    let sources = [("a", 5.0, 0.0), ("b", 6.0, 1.0)];
    let (d1, d2) = if flip { ("ax2", "ax1") } else { ("ax1", "ax2") };
    let mut prims = vec![
        json!({ "id": "ax1", "type": "point", "x": 0.0, "y": 0.0, "fixed": true }),
        json!({ "id": "ax2", "type": "point", "x": 3.0, "y": 4.0, "fixed": true }),
        json!({ "id": "axis", "type": "line", "p1_id": "ax1", "p2_id": "ax2" }),
        json!({ "id": "a", "type": "point", "x": 5.0, "y": 0.0 }),
        json!({ "id": "b", "type": "point", "x": 6.0, "y": 1.0 }),
        json!({ "id": "l", "type": "line", "p1_id": "a", "p2_id": "b" }),
    ];
    for k in 1..=2 {
        for (src, x, y) in sources {
            let d = sign * 2.5 * k as f64;
            prims.push(json!({ "id": format!("{src}{k}"), "type": "point",
                               "x": x + 0.6 * d, "y": y + 0.8 * d }));
            prims.push(json!({ "id": format!("li_{src}{k}"), "type": "translation",
                               "source": src, "copy": format!("{src}{k}"),
                               "from": d1, "to": d2,
                               "distance": 2.5, "count": k }));
        }
        prims.push(json!({ "id": format!("l{k}"), "type": "line",
                           "p1_id": format!("a{k}"), "p2_id": format!("b{k}") }));
    }
    request(Value::Array(prims))
}

fn assert_linear_copies(resp: &Value, flip: bool) {
    let dir = if flip { -1.0 } else { 1.0 };
    for k in 1..=2 {
        for id in ["a", "b"] {
            let (x, y) = point(resp, id);
            let s = dir * 2.5 * k as f64;
            close(resp, &format!("{id}{k}"), x + 0.6 * s, y + 0.8 * s);
        }
    }
}

#[test]
fn linear_array_solves_and_follows_its_source() {
    for flip in [false, true] {
        let req = linear_array_sketch(flip);
        let resp = solve(req.clone());
        assert_eq!(resp["status"], "converged", "{resp}");
        close(&resp, "a", 5.0, 0.0);
        assert_linear_copies(&resp, flip);

        let resp = solve(drag(req, "b", 8.0, -2.0));
        assert_eq!(resp["status"], "converged", "{resp}");
        close(&resp, "b", 8.0, -2.0);
        assert_linear_copies(&resp, flip);
    }
}

#[test]
fn translation_spacing_takes_a_parameter() {
    let resp = solve(request(json!([
        { "id": "gap", "type": "param", "value": 4.0 },
        { "id": "ax1", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "ax2", "type": "point", "x": 1.0, "y": 0.0, "fixed": true },
        { "id": "p", "type": "point", "x": 2.0, "y": 3.0, "fixed": true },
        { "id": "q", "type": "point", "x": 0.0, "y": 0.0 },
        { "id": "k", "type": "translation", "source": "p", "copy": "q",
          "from": "ax1", "to": "ax2", "distance": "gap", "count": 3 }
    ])));
    assert_eq!(resp["status"], "converged", "{resp}");
    close(&resp, "q", 14.0, 3.0);
}

/// A mirrored line: mirrored endpoints, each tied to its source by
/// `mirror` across the axis line, and the copied
/// line. Point `b` lies beyond the axis segment's end: the mirror axis is the
/// axis line's Extension, so it still gets its true image.
fn mirror_sketch() -> Value {
    request(json!([
        { "id": "m1", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "m2", "type": "point", "x": 0.0, "y": 2.0, "fixed": true },
        { "id": "axis", "type": "line", "p1_id": "m1", "p2_id": "m2" },
        { "id": "a", "type": "point", "x": 2.0, "y": 1.0 },
        { "id": "b", "type": "point", "x": 3.0, "y": 6.0 },
        { "id": "l", "type": "line", "p1_id": "a", "p2_id": "b" },
        { "id": "a2", "type": "point", "x": -2.0, "y": 1.0 },
        { "id": "b2", "type": "point", "x": -3.0, "y": 6.0 },
        { "id": "l2", "type": "line", "p1_id": "a2", "p2_id": "b2" },
        { "id": "k1", "type": "mirror", "source": "a", "image": "a2", "axis": "axis" },
        { "id": "k2", "type": "mirror", "source": "b", "image": "b2", "axis": "axis" }
    ]))
}

#[test]
fn mirror_solves_across_the_axis_extension_and_follows_its_source() {
    let req = mirror_sketch();
    let resp = solve(req.clone());
    assert_eq!(resp["status"], "converged", "{resp}");
    close(&resp, "a", 2.0, 1.0);
    close(&resp, "a2", -2.0, 1.0);
    close(&resp, "b", 3.0, 6.0);
    close(&resp, "b2", -3.0, 6.0);

    let resp = solve(drag(req, "b", 4.0, -3.0));
    assert_eq!(resp["status"], "converged", "{resp}");
    close(&resp, "b", 4.0, -3.0);
    close(&resp, "b2", -4.0, -3.0);
}

#[test]
fn mirror_across_a_slanted_axis() {
    // Axis y = x: the image of (3, 1) is (1, 3).
    let resp = solve(request(json!([
        { "id": "m1", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "m2", "type": "point", "x": 1.0, "y": 1.0, "fixed": true },
        { "id": "axis", "type": "line", "p1_id": "m1", "p2_id": "m2" },
        { "id": "a", "type": "point", "x": 3.0, "y": 1.0, "fixed": true },
        { "id": "b", "type": "point", "x": 0.0, "y": 0.0 },
        { "id": "k", "type": "mirror", "source": "a", "image": "b", "axis": "axis" }
    ])));
    assert_eq!(resp["status"], "converged", "{resp}");
    close(&resp, "b", 1.0, 3.0);
}

#[test]
fn mirror_across_a_short_axis_uses_its_extension() {
    // Axis x = 0 from (0,0) to (0,1); the source sits beyond the axis
    // segment's end, so only a mirror across the Extension puts its image
    // at (-3, 5).
    let resp = solve(request(json!([
        { "id": "m1", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
        { "id": "m2", "type": "point", "x": 0.0, "y": 1.0, "fixed": true },
        { "id": "axis", "type": "line", "p1_id": "m1", "p2_id": "m2" },
        { "id": "a", "type": "point", "x": 3.0, "y": 5.0, "fixed": true },
        { "id": "b", "type": "point", "x": -1.0, "y": 2.0 },
        { "id": "k", "type": "mirror", "source": "a", "image": "b", "axis": "axis" }
    ])));
    assert_eq!(resp["status"], "converged", "{resp}");
    close(&resp, "b", -3.0, 5.0);
}

// ── Ellipses ───────────────────────────────────────────────────────────────
//
// The ellipse: `{ type: "ellipse", c_id, focus1_id, radmin }`, a center
// Point, a focus Point (on the major axis) and the minor radius.

/// The sketch an ellipse tool writes for a 5 × 3 ellipse at (2, 1) with a
/// horizontal major axis: center, a construction focus, the ellipse, its
/// four axis points held by two-point `ellipse_axis` constraints, and
/// distances on the two diameters.
/// `major` is the major diameter the distance asks for.
fn ellipse_tool_sketch(major: f64) -> Vec<Value> {
    vec![
        json!({ "id": "c", "type": "point", "x": 2.0, "y": 1.0, "fixed": false }),
        json!({ "id": "f", "type": "point", "x": 6.0, "y": 1.0, "fixed": false, "isConstruction": true }),
        json!({ "id": "e", "type": "ellipse", "c_id": "c", "focus1_id": "f", "radmin": 3.0 }),
        json!({ "id": "maj1", "type": "point", "x": 7.0, "y": 1.0, "fixed": false }),
        json!({ "id": "maj2", "type": "point", "x": -3.0, "y": 1.0, "fixed": false }),
        json!({ "id": "min1", "type": "point", "x": 2.0, "y": 4.0, "fixed": false }),
        json!({ "id": "min2", "type": "point", "x": 2.0, "y": -2.0, "fixed": false }),
        json!({ "id": "ia1", "type": "ellipse_axis", "which": "major",
                "ellipse": "e", "a": "maj1", "b": "maj2" }),
        json!({ "id": "ia2", "type": "ellipse_axis", "which": "minor",
                "ellipse": "e", "a": "min1", "b": "min2" }),
        json!({ "id": "d1", "type": "distance", "a": "maj1", "b": "maj2", "value": major }),
        json!({ "id": "d2", "type": "distance", "a": "min1", "b": "min2", "value": 6.0 }),
    ]
}

/// The ellipse `e`'s center, unit major direction, major and minor radius.
fn ellipse_frame(resp: &Value, e: &str) -> ((f64, f64), (f64, f64), f64, f64) {
    let p = primitive(resp, e);
    let c = point(resp, p["c_id"].as_str().unwrap());
    let f = point(resp, p["focus1_id"].as_str().unwrap());
    let b = p["radmin"].as_f64().unwrap();
    let k = (f.0 - c.0).hypot(f.1 - c.1);
    (c, ((f.0 - c.0) / k, (f.1 - c.1) / k), b.hypot(k), b)
}

#[test]
fn an_ellipse_tool_sketch_solves_with_its_axis_points() {
    // As drawn: already solved.
    let resp = solve(request(json!(ellipse_tool_sketch(10.0))));
    assert_eq!(resp["status"], "converged", "{resp}");
    assert_eq!(resp["conflicting"], json!([]));
    assert_eq!(resp["redundant"], json!([]));
    // The ellipse can still move and turn.
    assert_eq!(resp["dof"], 3);

    // The major diameter dimension edited to 12: the ellipse grows along its
    // major axis, the minor radius stays 3, the axis points follow.
    let resp = solve(request(json!(ellipse_tool_sketch(12.0))));
    assert_eq!(resp["status"], "converged", "{resp}");
    let (c, u, a, b) = ellipse_frame(&resp, "e");
    assert!((a - 6.0).abs() < 1e-8 && (b - 3.0).abs() < 1e-8, "a {a}, b {b}");
    let n = (-u.1, u.0);
    for (id, dir, r) in [("maj1", u, a), ("maj2", u, -a), ("min1", n, b), ("min2", n, -b)] {
        let (x, y) = point(&resp, id);
        assert!(
            (x - (c.0 + r * dir.0)).abs() < 1e-8 && (y - (c.1 + r * dir.1)).abs() < 1e-8,
            "{id} = ({x}, {y})"
        );
    }
}

#[test]
fn a_fully_dimensioned_ellipse_is_fully_constrained() {
    let mut prims = ellipse_tool_sketch(10.0);
    prims.extend([
        json!({ "id": "x", "type": "x", "point": "c", "value": 2.0 }),
        json!({ "id": "y", "type": "y", "point": "c", "value": 1.0 }),
        json!({ "id": "h", "type": "horizontal", "a": "c", "b": "f" }),
    ]);
    let resp = solve(request(json!(prims)));
    assert_eq!(resp["status"], "converged", "{resp}");
    assert_eq!(resp["dof"], 0);
    let fully = resp["fullyConstrained"].as_array().unwrap();
    for id in ["c", "f", "e", "maj1", "maj2", "min1", "min2"] {
        assert!(fully.contains(&json!(id)), "{id} not in {fully:?}");
    }

    // Without the horizontal the ellipse can turn about its fixed center:
    // its radmin is set but it isn't fully constrained.
    let resp = solve(request(json!(prims[..prims.len() - 1])));
    let fully = resp["fullyConstrained"].as_array().unwrap();
    assert!(fully.contains(&json!("c")));
    assert!(!fully.contains(&json!("e")), "{fully:?}");
}

/// The fixed 5 × 4 ellipse at the origin (foci (±3, 0)), plus `rest`.
fn fixed_ellipse(rest: Value) -> Value {
    with(
        json!([
            { "id": "c", "type": "point", "x": 0.0, "y": 0.0, "fixed": true },
            { "id": "f", "type": "point", "x": 3.0, "y": 0.0, "fixed": true },
            { "id": "e", "type": "ellipse", "c_id": "c", "focus1_id": "f", "radmin": 4.0, "fixed": true }
        ]),
        rest,
    )
}

#[test]
fn point_on_ellipse_solves() {
    let resp = solve(fixed_ellipse(json!([
        { "id": "p", "type": "point", "x": 2.0, "y": 5.0 },
        { "id": "k", "type": "on", "point": "p", "curve": "e" }
    ])));
    assert_eq!(resp["status"], "converged", "{resp}");
    let (x, y) = point(&resp, "p");
    assert!((x * x / 25.0 + y * y / 16.0 - 1.0).abs() < 1e-8, "({x}, {y})");
}

#[test]
fn tangent_line_ellipse_solves_and_touches_on_the_segment() {
    // A Line of fixed length 1, upright, beside the ellipse's right vertex
    // (5, 0) but above it: it slides over until it touches at the vertex.
    let resp = solve(fixed_ellipse(json!([
        { "id": "a", "type": "point", "x": 6.0, "y": 2.0 },
        { "id": "b", "type": "point", "x": 6.0, "y": 3.0 },
        { "id": "l", "type": "line", "p1_id": "a", "p2_id": "b" },
        { "id": "v", "type": "vertical", "line": "l" },
        { "id": "len", "type": "distance", "a": "a", "b": "b", "value": 1.0 },
        { "id": "k", "type": "tangent", "a": "l", "b": "e" }
    ])));
    assert_eq!(resp["status"], "converged", "{resp}");
    let (a, b) = (point(&resp, "a"), point(&resp, "b"));
    assert!((a.0 - 5.0).abs() < 1e-8 && (b.0 - 5.0).abs() < 1e-8, "{a:?} {b:?}");
    assert!(a.1.min(b.1) <= 1e-8 && a.1.max(b.1) >= -1e-8, "{a:?} {b:?}");
}

/// Mirror and array tools tie a copy's minor radius to the
/// source's with `equal` over `{ entity, property: "radmin" }`.
#[test]
fn equal_over_radmin_references_solves() {
    let resp = solve(fixed_ellipse(json!([
        { "id": "c2", "type": "point", "x": 20.0, "y": 0.0 },
        { "id": "f2", "type": "point", "x": 23.0, "y": 0.0 },
        { "id": "e2", "type": "ellipse", "c_id": "c2", "focus1_id": "f2", "radmin": 2.0 },
        { "id": "k", "type": "equal",
          "a": { "entity": "e", "property": "radmin" }, "b": { "entity": "e2", "property": "radmin" } }
    ])));
    assert_eq!(resp["status"], "converged", "{resp}");
    assert!((num(&resp, "e2", "radmin") - 4.0).abs() < 1e-8);
    assert_eq!(num(&resp, "e", "radmin"), 4.0);

    let resp = reject(
        &fixed_ellipse(json!([
            { "id": "k", "type": "equal",
              "a": { "entity": "e", "property": "radius" }, "b": 1.0 }
        ]))
        .to_string(),
    );
    assert_eq!(
        resp["error"],
        "constraint k: field 'a': ellipse 'e' has no property 'radius'"
    );
    assert_eq!(resp["constraintId"], "k");
}

#[test]
fn a_reference_ellipse_never_moves() {
    // A Reference ellipse (its center and focus Reference Points too), with
    // its axis points and a point on it dragged away.
    let reference = json!([
        { "id": "c", "type": "point", "x": 2.0, "y": 1.0, "isReference": true },
        { "id": "f", "type": "point", "x": 6.0, "y": 1.0, "isReference": true },
        { "id": "e", "type": "ellipse", "c_id": "c", "focus1_id": "f", "radmin": 3.0, "isReference": true },
        { "id": "maj1", "type": "point", "x": 7.0, "y": 1.0 },
        { "id": "maj2", "type": "point", "x": -3.0, "y": 1.0 },
        { "id": "ia", "type": "ellipse_axis", "which": "major",
          "ellipse": "e", "a": "maj1", "b": "maj2" },
        { "id": "p", "type": "point", "x": 2.0, "y": 4.0 },
        { "id": "on", "type": "on", "point": "p", "curve": "e" },
        { "id": "t1", "type": "x", "point": "maj1", "value": 12.0, "temporary": true },
        { "id": "t2", "type": "y", "point": "p", "value": 9.0, "temporary": true }
    ]);
    let resp = solve(request(reference.clone()));
    assert_eq!(resp["status"], "converged", "{resp}");
    for id in ["c", "f", "e", "maj1", "maj2"] {
        assert_eq!(primitive(&resp, id), primitive(&request(reference.clone()), id), "{id}");
    }
    // The point stays on the ellipse, as near (·, 9) as it can: its top.
    let (x, y) = point(&resp, "p");
    assert!((x - 2.0).abs() < 1e-6 && (y - 4.0).abs() < 1e-6, "({x}, {y})");

    // A constraint the reference ellipse can't meet fails; it still doesn't move.
    let mut conflicting = reference.as_array().unwrap().clone();
    conflicting.push(json!({ "id": "d", "type": "distance", "a": "maj1", "b": "maj2", "value": 4.0 }));
    let resp = solve(request(json!(conflicting)));
    assert_eq!(resp["status"], "failed", "{resp}");
    for id in ["c", "f", "e"] {
        assert_eq!(primitive(&resp, id), primitive(&request(reference.clone()), id), "{id}");
    }
}

/// Line–arc `tangent` at an arc's own endpoint (a slot) uses the
/// angle-at-point form: no spurious Redundant, and the slot's `dof` is 0.
#[test]
fn tangent_line_arc_at_shared_endpoints_is_fully_determined() {
    let resp = solve(request(json!([
        { "id": "c1", "type": "point", "x": -25.0, "y": 0.0, "fixed": true },
        { "id": "c2", "type": "point", "x": 25.0, "y": 0.0, "fixed": true },
        { "id": "t1", "type": "point", "x": -24.0, "y": 13.0 },
        { "id": "t2", "type": "point", "x": 26.0, "y": 16.0 },
        { "id": "b1", "type": "point", "x": -26.0, "y": -14.0 },
        { "id": "b2", "type": "point", "x": 24.0, "y": -16.0 },
        { "id": "a1", "type": "arc", "c_id": "c1", "start_id": "t1", "end_id": "b1", "radius": 14.0,
          "start_angle": std::f64::consts::FRAC_PI_2, "end_angle": 3.0 * std::f64::consts::FRAC_PI_2 },
        { "id": "a2", "type": "arc", "c_id": "c2", "start_id": "b2", "end_id": "t2", "radius": 16.0,
          "start_angle": -std::f64::consts::FRAC_PI_2, "end_angle": std::f64::consts::FRAC_PI_2 },
        { "id": "lt", "type": "line", "p1_id": "t1", "p2_id": "t2" },
        { "id": "lb", "type": "line", "p1_id": "b1", "p2_id": "b2" },
        { "id": "k1", "type": "radius", "curve": "a1", "value": 15.0 },
        { "id": "k2", "type": "equal", "a": "a1", "b": "a2" },
        { "id": "k3", "type": "tangent", "a": "lt", "b": "a1" },
        { "id": "k4", "type": "tangent", "a": "lt", "b": "a2" },
        { "id": "k5", "type": "tangent", "a": "lb", "b": "a1" },
        { "id": "k6", "type": "tangent", "a": "lb", "b": "a2" }
    ])));
    assert_eq!(resp["status"], "converged", "{resp}");
    assert_eq!(resp["redundant"], json!([]));
    assert_eq!(resp["dof"], 0);
    let (_, y) = point(&resp, "t1");
    assert!((y - 15.0).abs() < 1e-8, "top line at y = 15, got {y}");
}

/// Every primitive needs an id.
#[test]
fn primitives_need_an_id() {
    let resp = reject(
        &request(json!([{ "type": "point", "name": "p", "x": 0.0, "y": 0.0 }])).to_string(),
    );
    assert_eq!(resp["error"], "primitive #0 has no id");
}

/// Fixed values (and Reference Geometry) come back exactly as they were sent: a client
/// compares the geometry a re-solve produces bit for bit (its mesh cache keys hash it), so a
/// last-bit change from parsing the request is a different result. The request is raw text,
/// and the response is checked as text, so no JSON parsing here can round the values.
#[test]
fn fixed_values_come_back_bit_for_bit() {
    let request = r#"{"version":1,"primitives":[
        {"type":"point","id":"a","x":-13.194212831353965,"y":-7.6999999999999975,"isReference":true},
        {"type":"point","id":"b","x":10.470031817585495,"y":2.0043856560173614e-15,"fixed":true},
        {"type":"point","id":"c","x":1.9129833546113182,"y":7.6999999999999975,"isReference":true},
        {"type":"point","id":"s","x":11,"y":0,"isReference":true},
        {"type":"point","id":"e","x":0,"y":11,"isReference":true},
        {"type":"arc","id":"arc","c_id":"c","start_id":"s","end_id":"e","radius":64.326080862224131,
         "start_angle":-1.4773857794649061,"end_angle":1.5707963267948966,"isReference":true}
    ]}"#;
    let out = solve_sketch_json(request).expect("request should be understood");
    for literal in [
        "-13.194212831353965",
        "-7.6999999999999975",
        "10.470031817585495",
        "2.0043856560173614e-15",
        "1.9129833546113182",
        "64.32608086222413",
        "-1.4773857794649061",
    ] {
        assert!(out.contains(literal), "{literal} should come back unchanged in {out}");
    }
}
