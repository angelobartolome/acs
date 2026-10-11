//! `spline_curve_json` (`acsSplineCurve`, `P3DSpline_Curve`): a Spline's
//! curve from its handles without solving, checked against the `curve` a
//! solve of a one-Spline sketch reports, and its rejections against
//! `spline::check`.

use acs::bindings::sketch_solve::acs_spline_curve;
use acs::sketch_solve::{solve_sketch_json, spline_curve_json};
use acs::spline;
use serde_json::{Value, json};

/// The `curve` a solve reports for one Spline with these (fixed) handles.
fn solved_curve(handles: &[[f64; 2]], interpolated: bool, knots: Option<&[f64]>) -> Value {
    let mut prims: Vec<Value> = handles
        .iter()
        .enumerate()
        .map(|(i, [x, y])| json!({ "id": format!("h{i}"), "type": "point", "x": x, "y": y, "fixed": true }))
        .collect();
    let mut s = json!({ "id": "s", "type": "spline", "interpolated": interpolated,
                        "points": (0..handles.len()).map(|i| format!("h{i}")).collect::<Vec<_>>() });
    if let Some(k) = knots {
        s["knots"] = json!(k);
    }
    prims.push(s);
    let resp = solve_sketch_json(&json!({ "version": 1, "primitives": prims }).to_string()).unwrap();
    let resp: Value = serde_json::from_str(&resp).unwrap();
    resp["primitives"].as_array().unwrap().iter().find(|p| p["id"] == "s").unwrap()["curve"].clone()
}

fn request(handles: &[[f64; 2]], interpolated: bool, knots: Option<&[f64]>) -> String {
    let mut r = json!({ "version": 1, "handles": handles, "interpolated": interpolated });
    if let Some(k) = knots {
        r["knots"] = json!(k);
    }
    r.to_string()
}

fn curve_of(request: &str) -> Value {
    let resp: Value = serde_json::from_str(&spline_curve_json(request).unwrap()).unwrap();
    assert_eq!(resp["version"], 1);
    assert!(resp.get("error").is_none(), "{resp}");
    resp["curve"].clone()
}

fn error_of(request: &str) -> String {
    let resp: Value = serde_json::from_str(&spline_curve_json(request).expect_err("should be rejected")).unwrap();
    assert_eq!(resp["version"], 1);
    assert!(resp.get("curve").is_none(), "{resp}");
    resp["error"].as_str().unwrap().to_string()
}

/// A Spline's handles, `interpolated` and knots.
type Form = (Vec<[f64; 2]>, bool, Option<Vec<f64>>);

const FIT: &[[f64; 2]] = &[[0.0, 0.0], [1.0, 2.0], [3.0, 2.5], [4.0, -1.0], [6.5, 0.3]];

/// Handles and knots of every form: fit points (2 and more), control points
/// on uniform knots, and control points on given knots.
fn forms() -> Vec<Form> {
    vec![
        (FIT.to_vec(), true, None),
        (FIT[..2].to_vec(), true, None),
        (vec![[0.0, 0.0], [1.0, 1.0], [1.0, 1.0], [2.0, 0.0]], true, None),
        (FIT.to_vec(), false, None),
        (FIT[..4].to_vec(), false, None),
        (FIT.to_vec(), false, Some(vec![0.0, 0.0, 0.0, 0.0, 0.3, 2.0, 2.0, 2.0, 2.0])),
        (FIT.to_vec(), false, Some(vec![-1.0, -1.0, -1.0, -1.0, 0.5, 1.0, 1.0, 1.0, 1.0])),
    ]
}

#[test]
fn the_curve_is_the_one_a_solve_reports() {
    for (handles, interpolated, knots) in forms() {
        let req = request(&handles, interpolated, knots.as_deref());
        assert_eq!(
            curve_of(&req),
            solved_curve(&handles, interpolated, knots.as_deref()),
            "{req}"
        );
    }
}

#[test]
fn the_wasm_export_answers_the_same() {
    for (handles, interpolated, knots) in forms() {
        let req = request(&handles, interpolated, knots.as_deref());
        assert_eq!(acs_spline_curve(&req), spline_curve_json(&req).unwrap());
    }
    let bad = request(&FIT[..1], true, None);
    assert_eq!(acs_spline_curve(&bad), spline_curve_json(&bad).unwrap_err());
}

#[test]
fn null_knots_are_uniform_knots() {
    let req = json!({ "version": 1, "handles": FIT, "interpolated": false, "knots": null }).to_string();
    assert_eq!(curve_of(&req), solved_curve(FIT, false, None));
}

#[test]
fn rejections_are_spline_check_s() {
    let cases: Vec<Form> = vec![
        (vec![], true, None),
        (FIT[..1].to_vec(), true, None),
        (FIT[..3].to_vec(), false, None),
        (FIT.to_vec(), true, Some(vec![0.0, 0.0, 0.0, 0.0, 0.5, 1.0, 1.0, 1.0, 1.0])),
        (FIT.to_vec(), false, Some(vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0])),
        (FIT.to_vec(), false, Some(vec![0.0, 0.0, 0.0, 0.0, 0.5, 0.4, 1.0, 1.0, 1.0])),
        (FIT.to_vec(), false, Some(vec![0.0, 0.0, 0.0, 0.1, 0.5, 1.0, 1.0, 1.0, 1.0])),
        (FIT.to_vec(), false, Some(vec![1.0; 9])),
    ];
    for (handles, interpolated, knots) in cases {
        let expected = spline::check(handles.len(), interpolated, knots.as_deref()).unwrap_err();
        assert_eq!(error_of(&request(&handles, interpolated, knots.as_deref())), expected);
    }
}

#[test]
fn a_malformed_request_is_rejected_naming_the_problem() {
    let cases = [
        ("not json".to_string(), "invalid JSON"),
        (json!({ "version": 2, "handles": FIT, "interpolated": true }).to_string(), "version"),
        (json!({ "version": 1, "interpolated": true }).to_string(), "'handles'"),
        (json!({ "version": 1, "handles": [[0, 0], [1]], "interpolated": true }).to_string(), "'handles'"),
        (json!({ "version": 1, "handles": [[0, 0], ["a", 1]], "interpolated": true }).to_string(), "'handles'"),
        (json!({ "version": 1, "handles": FIT }).to_string(), "'interpolated'"),
        (json!({ "version": 1, "handles": FIT, "interpolated": "yes" }).to_string(), "'interpolated'"),
        (json!({ "version": 1, "handles": FIT, "interpolated": false, "knots": [0, "a"] }).to_string(), "'knots'"),
    ];
    for (req, needle) in cases {
        let error = error_of(&req);
        assert!(error.contains(needle), "{req}: {error}");
    }
}
