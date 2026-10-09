//! Splines through the JSON API: the entity (fit-point and control-point
//! forms, round trip, drags, degrees of freedom) and the constraints against
//! one (`on`, `tangent` with a line, circle, arc, ellipse or spline),
//! including diagnosis.

use acs::sketch_solve::solve_sketch_json;
use acs::spline::BSpline;
use serde_json::{Value, json};

fn solve(prims: &[Value]) -> Value {
    let req = json!({ "version": 1, "primitives": prims }).to_string();
    let resp = solve_sketch_json(&req).unwrap_or_else(|e| panic!("rejected: {e}"));
    serde_json::from_str(&resp).unwrap()
}

fn reject(prims: &[Value]) -> String {
    let req = json!({ "version": 1, "primitives": prims }).to_string();
    let err = solve_sketch_json(&req).expect_err("should be rejected");
    let v: Value = serde_json::from_str(&err).unwrap();
    v["error"].as_str().unwrap().to_string()
}

fn pt(id: &str, x: f64, y: f64) -> Value {
    json!({ "id": id, "type": "point", "x": x, "y": y })
}

fn fixed(id: &str, x: f64, y: f64) -> Value {
    json!({ "id": id, "type": "point", "x": x, "y": y, "fixed": true })
}

fn xy(resp: &Value, id: &str) -> [f64; 2] {
    let p = resp["primitives"].as_array().unwrap().iter().find(|p| p["id"] == id).unwrap();
    [p["x"].as_f64().unwrap(), p["y"].as_f64().unwrap()]
}

fn prim<'a>(resp: &'a Value, id: &str) -> &'a Value {
    resp["primitives"].as_array().unwrap().iter().find(|p| p["id"] == id).unwrap()
}

/// The solved curve of spline `id`, from the response's `curve`.
fn curve(resp: &Value, id: &str) -> BSpline {
    let c = &prim(resp, id)["curve"];
    BSpline {
        degree: c["degree"].as_u64().unwrap() as usize,
        knots: c["knots"].as_array().unwrap().iter().map(|k| k.as_f64().unwrap()).collect(),
        control_points: c["control_points"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| [p[0].as_f64().unwrap(), p[1].as_f64().unwrap()])
            .collect(),
    }
}

/// Distance from `p` to the curve (dense sampling plus refinement) and the
/// parameter of the nearest point.
fn nearest(c: &BSpline, p: [f64; 2]) -> (f64, f64) {
    let (lo, hi) = c.domain();
    let d = |t: f64| {
        let q = c.point(t);
        (q[0] - p[0]).hypot(q[1] - p[1])
    };
    let mut t = (0..=400).map(|i| lo + (hi - lo) * i as f64 / 400.0).min_by(|a, b| d(*a).total_cmp(&d(*b))).unwrap();
    let mut step = (hi - lo) / 400.0;
    while step > 1e-15 {
        for cand in [t - step, t + step] {
            if (lo..=hi).contains(&cand) && d(cand) < d(t) {
                t = cand;
            }
        }
        step /= 2.0;
    }
    (d(t), t)
}

/// The smallest `score(q, τ̂)` over the curve's points q and unit tangents
/// τ̂ (dense sampling plus local refinement), and its parameter.
fn touch(c: &BSpline, score: impl Fn([f64; 2], [f64; 2]) -> f64) -> (f64, f64) {
    let (lo, hi) = c.domain();
    let f = |t: f64| score(c.point(t), unit(c.derivative(t)));
    let mut t = (0..=400).map(|i| lo + (hi - lo) * i as f64 / 400.0).min_by(|a, b| f(*a).total_cmp(&f(*b))).unwrap();
    let mut step = (hi - lo) / 400.0;
    while step > 1e-15 {
        for cand in [t - step, t + step] {
            if (lo..=hi).contains(&cand) && f(cand) < f(t) {
                t = cand;
            }
        }
        step /= 2.0;
    }
    (f(t), t)
}

fn unit(v: [f64; 2]) -> [f64; 2] {
    let n = v[0].hypot(v[1]);
    [v[0] / n, v[1] / n]
}

fn cross(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}

const FIT: [(f64, f64); 5] = [(0.0, 0.0), (2.0, 2.0), (4.0, 1.0), (6.0, 3.0), (8.0, 0.5)];

/// A spline `s` through (or controlled by) points `s0…s4` at `FIT`, the
/// handles fixed when `pinned`.
fn spline_prims(interpolated: bool, pinned: bool) -> Vec<Value> {
    let mut prims: Vec<Value> = FIT
        .iter()
        .enumerate()
        .map(|(i, &(x, y))| if pinned { fixed(&format!("s{i}"), x, y) } else { pt(&format!("s{i}"), x, y) })
        .collect();
    let ids: Vec<String> = (0..FIT.len()).map(|i| format!("s{i}")).collect();
    prims.push(json!({ "id": "s", "type": "spline", "points": ids, "interpolated": interpolated }));
    prims
}

// ── Entity ────────────────────────────────────────────────────────────────

#[test]
fn a_fit_point_spline_solves_and_round_trips() {
    let mut prims = spline_prims(true, false);
    prims.push(json!({ "id": "k", "type": "horizontal", "a": "s0", "b": "s4" }));
    let resp = solve(&prims);
    assert_eq!(resp["status"], "converged", "{resp}");
    let c = curve(&resp, "s");
    assert_eq!(c.degree, 3);
    assert_eq!(c.control_points.len(), FIT.len() + 2);
    // The curve passes through every fit point, at its solved position.
    for i in 0..FIT.len() {
        let (d, _) = nearest(&c, xy(&resp, &format!("s{i}")));
        assert!(d < 1e-9, "fit point s{i} is {d} off the curve");
    }
    // The response is a valid request with the same result.
    let again = solve(resp["primitives"].as_array().unwrap());
    assert_eq!(again["status"], "converged");
    assert_eq!(again["primitives"], resp["primitives"]);
    assert_eq!(again["stats"]["iterations"], 0);
}

#[test]
fn a_solved_sketch_with_spline_constraints_needs_no_iterations_again() {
    let mut prims = spline_prims(true, false);
    prims[0]["fixed"] = json!(true);
    prims.extend([pt("p", 3.0, 3.0), pt("cc", 4.0, 5.0)]);
    prims.push(json!({ "id": "circ", "type": "circle", "c_id": "cc", "radius": 1.0 }));
    prims.push(json!({ "id": "on", "type": "on", "point": "p", "curve": "s" }));
    prims.push(json!({ "id": "k", "type": "tangent", "a": "circ", "b": "s" }));
    let resp = solve(&prims);
    assert_eq!(resp["status"], "converged", "{resp}");
    // The curve parameters start where the solve left the contacts, so the
    // pre-solver finds everything already holding.
    let again = solve(resp["primitives"].as_array().unwrap());
    assert_eq!(again["stats"]["iterations"], 0, "{again}");
    assert_eq!(again["primitives"], resp["primitives"]);
}

#[test]
fn a_temporary_on_leaves_the_degrees_of_freedom_alone() {
    let mut prims = spline_prims(true, true);
    prims.push(pt("p", 4.0, 3.0));
    prims.push(json!({ "id": "on", "type": "on", "point": "p", "curve": "s", "temporary": true }));
    let resp = solve(&prims);
    assert_eq!(resp["status"], "converged");
    assert_eq!(resp["dof"], 2, "a free point, whatever the soft goal");
    let (d, _) = nearest(&curve(&resp, "s"), xy(&resp, "p"));
    assert!(d < 1e-9, "the goal is met: {d}");
}

#[test]
fn a_curve_parameter_needs_an_id_no_entity_has() {
    let mut prims = spline_prims(true, true);
    prims.push(pt("k#0", 4.0, 3.0));
    prims.push(json!({ "id": "k", "type": "on", "point": "k#0", "curve": "s" }));
    assert!(reject(&prims).contains("curve parameter 'k#0'"));
}

#[test]
fn fit_and_control_forms_give_the_same_curve() {
    let fit: Vec<[f64; 2]> = FIT.iter().map(|&(x, y)| [x, y]).collect();
    let a = BSpline::from_fit_points(&fit).unwrap();
    let b = BSpline::from_control_points(&a.control_points, Some(&a.knots)).unwrap();
    assert_eq!(a, b);

    // Through the JSON API: a fit-point spline, then a control-point spline
    // on the control points and knots its response reports.
    let resp = solve(&spline_prims(true, false));
    let fit_curve = curve(&resp, "s");
    let mut prims: Vec<Value> = fit_curve
        .control_points
        .iter()
        .enumerate()
        .map(|(i, &[x, y])| pt(&format!("c{i}"), x, y))
        .collect();
    let ids: Vec<String> = (0..prims.len()).map(|i| format!("c{i}")).collect();
    prims.push(json!({ "id": "c", "type": "spline", "points": ids, "interpolated": false,
                       "knots": fit_curve.knots }));
    let control_curve = curve(&solve(&prims), "c");
    for i in 0..=50 {
        let t = i as f64 / 50.0;
        let (p, q) = (fit_curve.point(t), control_curve.point(t));
        assert!((p[0] - q[0]).abs() < 1e-12 && (p[1] - q[1]).abs() < 1e-12, "t = {t}");
    }
}

#[test]
fn control_points_default_to_clamped_uniform_knots() {
    let resp = solve(&spline_prims(false, false));
    let c = curve(&resp, "s");
    assert_eq!(c.degree, 3);
    assert_eq!(c.knots, vec![0.0, 0.0, 0.0, 0.0, 0.5, 1.0, 1.0, 1.0, 1.0]);
    // Clamped: the curve starts and ends at the end control points.
    assert_eq!(c.point(0.0), [0.0, 0.0]);
    let end = c.point(1.0);
    assert!((end[0] - 8.0).abs() < 1e-12 && (end[1] - 0.5).abs() < 1e-12);
}

#[test]
fn a_free_spline_has_two_degrees_of_freedom_per_handle() {
    for interpolated in [true, false] {
        let resp = solve(&spline_prims(interpolated, false));
        assert_eq!(resp["dof"], 2 * FIT.len() as u64);
        assert!(!resp["fullyConstrained"].as_array().unwrap().contains(&json!("s")));

        let pinned = solve(&spline_prims(interpolated, true));
        assert_eq!(pinned["dof"], 0);
        assert!(pinned["fullyConstrained"].as_array().unwrap().contains(&json!("s")));
    }
    // Two fit points: a straight segment, still 4 degrees of freedom.
    let resp = solve(&[
        pt("a", 0.0, 0.0),
        pt("b", 3.0, 1.0),
        json!({ "id": "s", "type": "spline", "points": ["a", "b"], "interpolated": true }),
    ]);
    assert_eq!(resp["dof"], 4);
}

#[test]
fn a_drag_on_a_fit_point_moves_the_curve() {
    let mut prims = spline_prims(true, false);
    prims[0]["fixed"] = json!(true);
    prims[4]["fixed"] = json!(true);
    // A point held on the curve, and a drag of the middle fit point.
    prims.push(pt("p", 3.0, 1.6));
    prims.push(json!({ "id": "on", "type": "on", "point": "p", "curve": "s" }));
    prims.push(fixed("cursor", 4.0, 4.0));
    prims.push(json!({ "id": "drag", "type": "coincident", "a": "s2", "b": "cursor", "temporary": true }));
    let before = curve(&solve(&spline_prims(true, false)), "s");
    let resp = solve(&prims);
    assert_eq!(resp["status"], "converged", "{resp}");
    let s2 = xy(&resp, "s2");
    assert!((s2[0] - 4.0).abs() < 1e-8 && (s2[1] - 4.0).abs() < 1e-8, "{s2:?}");
    let after = curve(&resp, "s");
    assert!((after.point(0.5)[1] - before.point(0.5)[1]).abs() > 0.5);
    let (d, _) = nearest(&after, xy(&resp, "p"));
    assert!(d < 1e-9, "the point held on the curve is {d} off it");
}

#[test]
fn malformed_splines_are_rejected() {
    let one = [pt("a", 0.0, 0.0), json!({ "id": "s", "type": "spline", "points": ["a"], "interpolated": true })];
    assert!(reject(&one).contains("at least 2 points"));

    let mut knots_on_fit = spline_prims(true, false);
    knots_on_fit[5]["knots"] = json!([0, 0, 0, 0, 0.5, 1, 1, 1, 1]);
    assert!(reject(&knots_on_fit).contains("control-point spline"));

    let mut unclamped = spline_prims(false, false);
    unclamped[5]["knots"] = json!([0, 0.1, 0.2, 0.3, 0.5, 0.7, 0.8, 0.9, 1]);
    assert!(reject(&unclamped).contains("clamped"));

    let mut wrong_length = spline_prims(false, false);
    wrong_length[5]["knots"] = json!([0, 0, 0, 0, 1, 1, 1, 1]);
    assert!(reject(&wrong_length).contains("needs 9 knots"));

    let mut not_a_point = spline_prims(true, false);
    not_a_point.push(json!({ "id": "c", "type": "circle", "c_id": "s0", "radius": 1 }));
    not_a_point[5]["points"] = json!(["s0", "c"]);
    assert!(reject(&not_a_point).contains("'c' is not a point"));

    let missing_flag = [pt("a", 0.0, 0.0), pt("b", 1.0, 0.0), json!({ "id": "s", "type": "spline", "points": ["a", "b"] })];
    assert!(reject(&missing_flag).contains("'interpolated'"));
}

// ── on ──────────────────────────────────────────────────────────────────────

#[test]
fn a_point_on_a_spline() {
    for interpolated in [true, false] {
        let mut prims = spline_prims(interpolated, true);
        prims.push(pt("p", 5.0, 5.0));
        prims.push(json!({ "id": "on", "type": "on", "point": "p", "curve": "s" }));
        let resp = solve(&prims);
        assert_eq!(resp["status"], "converged", "{resp}");
        let (d, _) = nearest(&curve(&resp, "s"), xy(&resp, "p"));
        assert!(d < 1e-9, "{d}");
        // A point on a pinned curve slides along it: one degree of freedom.
        assert_eq!(resp["dof"], 1);
        assert_eq!(resp["redundant"], json!([]));
    }
}

#[test]
fn a_point_on_a_spline_stays_between_its_ends() {
    let mut prims = spline_prims(true, true);
    // Nearest the curve's continuation past its end.
    prims.push(pt("p", 10.0, -2.0));
    prims.push(json!({ "id": "on", "type": "on", "point": "p", "curve": "s" }));
    let resp = solve(&prims);
    assert_eq!(resp["status"], "converged", "{resp}");
    let c = curve(&resp, "s");
    let (d, t) = nearest(&c, xy(&resp, "p"));
    assert!(d < 1e-9 && (0.0..=1.0).contains(&t), "{d} at {t}");
}

#[test]
fn a_spline_on_held_twice_is_redundant() {
    let mut prims = spline_prims(true, true);
    prims.push(pt("p", 3.0, 2.5));
    prims.push(json!({ "id": "on1", "type": "on", "point": "p", "curve": "s" }));
    prims.push(json!({ "id": "on2", "type": "on", "point": "p", "curve": "s" }));
    let resp = solve(&prims);
    assert_eq!(resp["status"], "converged", "{resp}");
    assert_eq!(resp["redundant"], json!(["on2"]));
    assert_eq!(resp["conflicting"], json!([]));
    assert_eq!(resp["dof"], 1);
}

#[test]
fn a_spline_on_against_a_pinned_point_off_it_is_conflicting() {
    let mut prims = spline_prims(true, true);
    prims.push(fixed("p", 3.0, 5.0));
    prims.push(json!({ "id": "on", "type": "on", "point": "p", "curve": "s" }));
    let resp = solve(&prims);
    assert_eq!(resp["status"], "failed");
    assert_eq!(resp["conflicting"], json!(["on"]));
}

#[test]
fn a_point_on_a_free_spline_moves_the_curve_too() {
    let mut prims = spline_prims(true, false);
    prims.push(fixed("p", 4.0, 3.0));
    prims.push(json!({ "id": "on", "type": "on", "point": "p", "curve": "s" }));
    let resp = solve(&prims);
    assert_eq!(resp["status"], "converged", "{resp}");
    let (d, _) = nearest(&curve(&resp, "s"), [4.0, 3.0]);
    assert!(d < 1e-9);
    assert_eq!(resp["dof"], 2 * FIT.len() as u64 - 1);
}

// ── tangent ─────────────────────────────────────────────────────────────────

/// The nearest point of the curve to `p` and the unit tangent there.
fn foot(c: &BSpline, p: [f64; 2]) -> ([f64; 2], [f64; 2], f64) {
    let (_, t) = nearest(c, p);
    (c.point(t), unit(c.derivative(t)), t)
}

#[test]
fn a_line_tangent_to_a_spline() {
    for interpolated in [true, false] {
        let mut prims = spline_prims(interpolated, true);
        prims.extend([pt("a", 1.0, 3.5), pt("b", 4.0, 3.0)]);
        prims.push(json!({ "id": "l", "type": "line", "p1_id": "a", "p2_id": "b" }));
        prims.push(json!({ "id": "k", "type": "tangent", "a": "l", "b": "s" }));
        let resp = solve(&prims);
        assert_eq!(resp["status"], "converged", "{resp}");
        let (a, b) = (xy(&resp, "a"), xy(&resp, "b"));
        let g = unit([b[0] - a[0], b[1] - a[1]]);
        let c = curve(&resp, "s");
        // Some point of the curve lies on the line, with the curve along it.
        let (off, t) = touch(&c, |q, tau| cross(g, [q[0] - a[0], q[1] - a[1]]).abs() + cross(g, tau).abs());
        assert!(off < 1e-8, "{off}");
        // And on the segment.
        let q = c.point(t);
        let along = g[0] * (q[0] - a[0]) + g[1] * (q[1] - a[1]);
        assert!(along >= -1e-9 && along <= (b[0] - a[0]).hypot(b[1] - a[1]) + 1e-9);
        assert_eq!(resp["dof"], 3, "a free line tangent to a pinned curve");
        assert_eq!(resp["redundant"], json!([]));
    }
}

#[test]
fn a_circle_tangent_to_a_spline_from_a_poor_start() {
    // The circle starts far from the curve and much too small.
    let mut prims = spline_prims(true, true);
    prims.push(pt("cc", 4.0, 9.0));
    prims.push(json!({ "id": "circ", "type": "circle", "c_id": "cc", "radius": 0.3 }));
    prims.push(json!({ "id": "r", "type": "radius", "curve": "circ", "value": 1.5 }));
    prims.push(json!({ "id": "k", "type": "tangent", "a": "s", "b": "circ" }));
    let resp = solve(&prims);
    assert_eq!(resp["status"], "converged", "{resp}");
    let c = xy(&resp, "cc");
    let (d, _) = nearest(&curve(&resp, "s"), c);
    assert!((d - 1.5).abs() < 1e-8, "center {d} from the curve");
    assert_eq!(resp["dof"], 1);
}

#[test]
fn an_arc_tangent_to_a_spline_touches_on_its_span() {
    let mut prims = spline_prims(true, true);
    let (cx, cy, r, a0, a1) = (4.0, 4.0, 2.0, -2.5_f64, -0.8_f64);
    prims.extend([
        pt("ac", cx, cy),
        pt("as", cx + r * a0.cos(), cy + r * a0.sin()),
        pt("ae", cx + r * a1.cos(), cy + r * a1.sin()),
    ]);
    prims.push(json!({ "id": "arc", "type": "arc", "c_id": "ac", "start_id": "as", "end_id": "ae",
                       "radius": r, "start_angle": a0, "end_angle": a1 }));
    prims.push(json!({ "id": "k", "type": "tangent", "a": "arc", "b": "s" }));
    let resp = solve(&prims);
    assert_eq!(resp["status"], "converged", "{resp}");
    let arc = prim(&resp, "arc");
    let (r, a0, a1) = (
        arc["radius"].as_f64().unwrap(),
        arc["start_angle"].as_f64().unwrap(),
        arc["end_angle"].as_f64().unwrap(),
    );
    let c = xy(&resp, "ac");
    // A point of the curve at the radius, its radius normal to the curve.
    let (off, t) = touch(&curve(&resp, "s"), |q, tau| {
        let v = [q[0] - c[0], q[1] - c[1]];
        (v[0].hypot(v[1]) - r).abs() + (v[0] * tau[0] + v[1] * tau[1]).abs()
    });
    assert!(off < 1e-8, "{off}");
    let q = curve(&resp, "s").point(t);
    let phi = (q[1] - c[1]).atan2(q[0] - c[0]);
    let u = (phi - a0).rem_euclid(std::f64::consts::TAU);
    assert!(u <= (a1 - a0).rem_euclid(std::f64::consts::TAU) + 1e-9, "contact off the arc's span");
}

#[test]
fn an_ellipse_tangent_to_a_spline() {
    let mut prims = spline_prims(true, true);
    prims.extend([pt("ec", 4.0, 5.0), pt("ef", 5.0, 5.0)]);
    prims.push(json!({ "id": "e", "type": "ellipse", "c_id": "ec", "focus1_id": "ef", "radmin": 1.0 }));
    prims.push(json!({ "id": "k", "type": "tangent", "a": "e", "b": "s" }));
    let resp = solve(&prims);
    assert_eq!(resp["status"], "converged", "{resp}");
    let (c, f) = (xy(&resp, "ec"), xy(&resp, "ef"));
    let b = prim(&resp, "e")["radmin"].as_f64().unwrap();
    let major = (b * b + (f[0] - c[0]).powi(2) + (f[1] - c[1]).powi(2)).sqrt();
    let f2 = [2.0 * c[0] - f[0], 2.0 * c[1] - f[1]];
    // The curve touches the ellipse: the focal-distance sum dips to 2a and
    // no lower.
    let sum = |q: [f64; 2]| (q[0] - f[0]).hypot(q[1] - f[1]) + (q[0] - f2[0]).hypot(q[1] - f2[1]);
    let s = curve(&resp, "s");
    let min = (0..=4000).map(|i| sum(s.point(i as f64 / 4000.0))).fold(f64::INFINITY, f64::min);
    assert!((min - 2.0 * major).abs() < 1e-6, "{min} vs {}", 2.0 * major);
    assert_eq!(resp["dof"], 4);
}

#[test]
fn two_splines_tangent() {
    let mut prims = spline_prims(true, true);
    for (i, (x, y)) in [(1.0, 5.0), (4.0, 2.5), (7.0, 5.0)].into_iter().enumerate() {
        prims.push(pt(&format!("u{i}"), x, y));
    }
    prims.push(json!({ "id": "u", "type": "spline", "points": ["u0", "u1", "u2"], "interpolated": true }));
    prims.push(json!({ "id": "k", "type": "tangent", "a": "s", "b": "u" }));
    let resp = solve(&prims);
    assert_eq!(resp["status"], "converged", "{resp}");
    let (s, u) = (curve(&resp, "s"), curve(&resp, "u"));
    // A point of u lies on s with the same tangent.
    let (gap, t) = touch(&u, |q, _| nearest(&s, q).0);
    assert!(gap < 1e-7, "{gap}");
    let (_, tau_s, _) = foot(&s, u.point(t));
    assert!(cross(tau_s, unit(u.derivative(t))).abs() < 1e-6);
    assert_eq!(resp["dof"], 5);
}

#[test]
fn a_line_from_a_spline_end_along_its_tangent() {
    // The line starts at the spline's last fit point: tangent there.
    let mut prims = spline_prims(true, true);
    prims.push(pt("b", 10.0, 1.0));
    prims.push(json!({ "id": "l", "type": "line", "p1_id": "s4", "p2_id": "b" }));
    prims.push(json!({ "id": "k", "type": "tangent", "a": "l", "b": "s" }));
    let resp = solve(&prims);
    assert_eq!(resp["status"], "converged", "{resp}");
    let b = xy(&resp, "b");
    let c = curve(&resp, "s");
    let g = unit([b[0] - 8.0, b[1] - 0.5]);
    assert!(cross(g, unit(c.derivative(1.0))).abs() < 1e-10);
    assert_eq!(resp["dof"], 1);
    assert_eq!(resp["redundant"], json!([]));
}

#[test]
fn a_line_whose_end_is_held_on_a_spline_is_tangent_there() {
    let mut prims = spline_prims(true, true);
    prims.extend([pt("a", 3.0, 2.0), pt("b", 5.0, 3.0)]);
    prims.push(json!({ "id": "l", "type": "line", "p1_id": "a", "p2_id": "b" }));
    prims.push(json!({ "id": "on", "type": "on", "point": "a", "curve": "s" }));
    prims.push(json!({ "id": "k", "type": "tangent", "a": "l", "b": "s" }));
    prims.push(json!({ "id": "len", "type": "length", "curve": "l", "value": 2.0 }));
    let resp = solve(&prims);
    assert_eq!(resp["status"], "converged", "{resp}");
    let (a, b) = (xy(&resp, "a"), xy(&resp, "b"));
    let (q, tau, _) = foot(&curve(&resp, "s"), a);
    assert!((q[0] - a[0]).hypot(q[1] - a[1]) < 1e-9);
    assert!(cross(unit([b[0] - a[0], b[1] - a[1]]), tau).abs() < 1e-9);
    // a slides along the curve; the line follows: one degree of freedom.
    assert_eq!(resp["dof"], 1);
    assert_eq!(resp["redundant"], json!([]));
}

#[test]
fn an_arc_and_a_spline_sharing_an_end_are_tangent_there() {
    let mut prims = spline_prims(true, true);
    // An arc ending at the spline's first fit point (0, 0).
    let (r, a0) = (2.0, 0.3_f64);
    prims.extend([pt("ac", -1.0, 1.5), pt("as", -1.0 + r * a0.cos(), 1.5 + r * a0.sin())]);
    prims.push(json!({ "id": "arc", "type": "arc", "c_id": "ac", "start_id": "as", "end_id": "s0",
                       "radius": r, "start_angle": a0, "end_angle": -1.0 }));
    prims.push(json!({ "id": "k", "type": "tangent", "a": "arc", "b": "s" }));
    let resp = solve(&prims);
    assert_eq!(resp["status"], "converged", "{resp}");
    let c = xy(&resp, "ac");
    let radius = unit([0.0 - c[0], 0.0 - c[1]]);
    let tau = unit(curve(&resp, "s").derivative(0.0));
    assert!((radius[0] * tau[0] + radius[1] * tau[1]).abs() < 1e-10);
    assert_eq!(resp["redundant"], json!([]));
}

#[test]
fn two_splines_sharing_an_end_are_tangent_there() {
    let mut prims = spline_prims(true, true);
    prims.extend([pt("u1", 10.0, 3.0), pt("u2", 12.0, 0.0)]);
    prims.push(json!({ "id": "u", "type": "spline", "points": ["s4", "u1", "u2"], "interpolated": true }));
    prims.push(json!({ "id": "k", "type": "tangent", "a": "s", "b": "u" }));
    let resp = solve(&prims);
    assert_eq!(resp["status"], "converged", "{resp}");
    let (s, u) = (curve(&resp, "s"), curve(&resp, "u"));
    assert!(cross(unit(s.derivative(1.0)), unit(u.derivative(0.0))).abs() < 1e-10);
    assert_eq!(resp["dof"], 3);
}

#[test]
fn spline_constraints_are_rejected_against_the_wrong_kinds() {
    let mut prims = spline_prims(true, true);
    prims.push(json!({ "id": "k", "type": "tangent", "a": "s0", "b": "s" }));
    assert!(reject(&prims).contains("unsupported combination for 'tangent'"));

    let mut prims = spline_prims(true, true);
    prims.extend([pt("cc", 4.0, 4.0)]);
    prims.push(json!({ "id": "circ", "type": "circle", "c_id": "cc", "radius": 1 }));
    prims.push(json!({ "id": "k", "type": "tangent", "a": "circ", "b": "s", "internal": true }));
    assert!(solve_sketch_json(&json!({ "version": 1, "primitives": prims }).to_string()).is_err());
}

#[test]
fn tangents_solve_from_poor_starts() {
    // Far from the curve, at the wrong angle: a short segment off its end,
    // ellipses well clear of it, a second spline above and below it.
    let mut cases: Vec<(String, Vec<Value>)> = Vec::new();
    for (ax, ay, bx, by) in [(-3.0, 8.0, 0.0, 9.0), (2.0, -5.0, 6.0, -4.0), (-3.0, -5.0, 12.0, -4.0)] {
        cases.push((
            format!("line {ax},{ay}"),
            vec![
                pt("a", ax, ay),
                pt("b", bx, by),
                json!({ "id": "l", "type": "line", "p1_id": "a", "p2_id": "b" }),
                json!({ "id": "k", "type": "tangent", "a": "l", "b": "s" }),
            ],
        ));
    }
    for (cx, cy, fx, b) in [(4.0, 10.0, 6.0, 0.5), (-4.0, -3.0, -3.0, 2.0)] {
        cases.push((
            format!("ellipse {cx},{cy}"),
            vec![
                pt("ec", cx, cy),
                pt("ef", fx, cy),
                json!({ "id": "e", "type": "ellipse", "c_id": "ec", "focus1_id": "ef", "radmin": b }),
                json!({ "id": "k", "type": "tangent", "a": "e", "b": "s" }),
            ],
        ));
    }
    for (dy, mid) in [(10.0, 8.0), (-6.0, -2.0)] {
        cases.push((
            format!("spline {dy}"),
            vec![
                pt("u0", 1.0, dy),
                pt("u1", 4.0, mid),
                pt("u2", 7.0, dy),
                json!({ "id": "u", "type": "spline", "points": ["u0", "u1", "u2"], "interpolated": true }),
                json!({ "id": "k", "type": "tangent", "a": "s", "b": "u" }),
            ],
        ));
    }
    for (name, extra) in cases {
        let mut prims = spline_prims(true, true);
        prims.extend(extra);
        let resp = solve(&prims);
        assert_eq!(resp["status"], "converged", "{name}: {resp}");
        assert!(resp["stats"]["iterations"].as_u64().unwrap() <= 30, "{name}: {}", resp["stats"]);
    }
}
