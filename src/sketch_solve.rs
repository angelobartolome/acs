//! Solve Part3D

use std::collections::HashMap;

use serde_json::{json, Value};

use crate::geometry::{Arc as GeoArc, Circle, Line, Point};
use crate::{ConstraintSolver, ConstraintType, SolverResult};

fn as_str<'a>(v: &'a Value, key: &str) -> Option<&'a str> {
    v.get(key)?.as_str()
}

fn as_string(v: &Value, key: &str) -> Option<String> {
    as_str(v, key).map(String::from)
}

/// `distance`, `angle`, etc. may be number, numeric string, or param ref object.
fn extract_scalar_f64(v: &Value) -> Option<f64> {
    if let Some(n) = v.as_f64() {
        return Some(n);
    }
    if let Some(i) = v.as_i64() {
        return Some(i as f64);
    }
    if let Some(u) = v.as_u64() {
        return Some(u as f64);
    }
    if let Some(s) = v.as_str() {
        return s.parse().ok();
    }
    None
}

const GEOMETRY_TYPES: &[&str] = &[
    "point",
    "line",
    "circle",
    "arc",
    "ellipse",
    "arc_of_ellipse",
    "hyperbola",
    "arc_of_hyperbola",
    "parabola",
    "arc_of_parabola",
];

fn is_geometry(t: &str) -> bool {
    GEOMETRY_TYPES.contains(&t)
}

fn build_line_endpoints(primitives: &[Value]) -> HashMap<String, (String, String)> {
    let mut m = HashMap::new();
    for p in primitives {
        let Some(t) = p.get("type").and_then(|x| x.as_str()) else {
            continue;
        };
        if t != "line" {
            continue;
        }
        let Some(id) = as_str(p, "id") else {
            continue;
        };
        let Some(p1) = as_string(p, "p1_id") else {
            continue;
        };
        let Some(p2) = as_string(p, "p2_id") else {
            continue;
        };
        m.insert(id.to_string(), (p1, p2));
    }
    m
}

fn constraint_to_acs(
    c: &Value,
    lines: &HashMap<String, (String, String)>,
    skipped: &mut Vec<String>,
) -> Option<ConstraintType> {
    let ctype = as_str(c, "type")?;
    let cid = as_string(c, "id").unwrap_or_default();

    let line_pts = |lid: &str| -> Option<(String, String)> {
        lines.get(lid).cloned()
    };

    match ctype {
        "horizontal_pp" => {
            let p1 = as_string(c, "p1_id")?;
            let p2 = as_string(c, "p2_id")?;
            Some(ConstraintType::Horizontal(p1, p2))
        }
        "vertical_pp" => {
            let p1 = as_string(c, "p1_id")?;
            let p2 = as_string(c, "p2_id")?;
            Some(ConstraintType::Vertical(p1, p2))
        }
        "horizontal_l" => {
            let lid = as_string(c, "l_id")?;
            let (p1, p2) = line_pts(&lid)?;
            Some(ConstraintType::Horizontal(p1, p2))
        }
        "vertical_l" => {
            let lid = as_string(c, "l_id")?;
            let (p1, p2) = line_pts(&lid)?;
            Some(ConstraintType::Vertical(p1, p2))
        }
        "parallel" => {
            let l1 = as_string(c, "l1_id")?;
            let l2 = as_string(c, "l2_id")?;
            let (a, b) = line_pts(&l1)?;
            let (c2, d) = line_pts(&l2)?;
            Some(ConstraintType::Parallel(a, b, c2, d))
        }
        "perpendicular_ll" => {
            let l1 = as_string(c, "l1_id")?;
            let l2 = as_string(c, "l2_id")?;
            let (a, b) = line_pts(&l1)?;
            let (c2, d) = line_pts(&l2)?;
            Some(ConstraintType::Perpendicular(a, b, c2, d))
        }
        "perpendicular_pppp" => {
            let a = as_string(c, "l1p1_id")?;
            let b = as_string(c, "l1p2_id")?;
            let c2 = as_string(c, "l2p1_id")?;
            let d = as_string(c, "l2p2_id")?;
            Some(ConstraintType::Perpendicular(a, b, c2, d))
        }
        "p2p_coincident" => {
            let p1 = as_string(c, "p1_id")?;
            let p2 = as_string(c, "p2_id")?;
            Some(ConstraintType::Coincident(p1, p2))
        }
        "point_on_line_pl" => {
            let p = as_string(c, "p_id")?;
            let lid = as_string(c, "l_id")?;
            let (a, b) = line_pts(&lid)?;
            Some(ConstraintType::PointOnLine(p, a, b))
        }
        "point_on_line_ppp" => {
            let p = as_string(c, "p_id")?;
            let a = as_string(c, "lp1_id")?;
            let b = as_string(c, "lp2_id")?;
            Some(ConstraintType::PointOnLine(p, a, b))
        }
        "point_on_circle" => {
            let p = as_string(c, "p_id")?;
            let circle = as_string(c, "c_id")?;
            let center = as_string(c, "center_id")?;
            Some(ConstraintType::PointOnCircle(p, center, circle))
        }
        "p2p_distance" => {
            let p1 = as_string(c, "p1_id")?;
            let p2 = as_string(c, "p2_id")?;
            let dist = c.get("distance").and_then(extract_scalar_f64)?;
            Some(ConstraintType::DistancePointPoint(p1, p2, dist))
        }
        "p2l_distance" => {
            let p = as_string(c, "p_id")?;
            let lid = as_string(c, "l_id")?;
            let (a, b) = line_pts(&lid)?;
            let dist = c.get("distance").and_then(extract_scalar_f64)?;
            Some(ConstraintType::DistancePointLine(p, a, b, dist))
        }
        "l2l_angle_pppp" => {
            let p1 = as_string(c, "l1p1_id")?;
            let p2 = as_string(c, "l1p2_id")?;
            let p3 = as_string(c, "l2p1_id")?;
            let p4 = as_string(c, "l2p2_id")?;
            let ang = c.get("angle").and_then(extract_scalar_f64)?;
            Some(ConstraintType::Angle(p1, p2, p3, p4, ang))
        }
        "l2l_angle_ll" => {
            let l1 = as_string(c, "l1_id")?;
            let l2 = as_string(c, "l2_id")?;
            let (a, b) = line_pts(&l1)?;
            let (c2, d) = line_pts(&l2)?;
            let ang = c.get("angle").and_then(extract_scalar_f64)?;
            Some(ConstraintType::Angle(a, b, c2, d, ang))
        }
        "equal_length" => {
            let l1 = as_string(c, "l1_id")?;
            let l2 = as_string(c, "l2_id")?;
            let (a, b) = line_pts(&l1)?;
            let (c2, d) = line_pts(&l2)?;
            Some(ConstraintType::EqualLength(a, b, c2, d))
        }
        "equal_radius_cc" => {
            let c1 = as_string(c, "c1_id")?;
            let c2 = as_string(c, "c2_id")?;
            Some(ConstraintType::EqualRadius(c1, c2))
        }
        "equal_radius_aa" => {
            let a1 = as_string(c, "a1_id")?;
            let a2 = as_string(c, "a2_id")?;
            Some(ConstraintType::EqualRadius(a1, a2))
        }
        "circle_radius" => {
            let circle = as_string(c, "c_id")?;
            let r = c.get("radius").and_then(extract_scalar_f64)?;
            Some(ConstraintType::FixedRadius(circle, r))
        }
        "arc_radius" => {
            let arc = as_string(c, "a_id")?;
            let r = c.get("radius").and_then(extract_scalar_f64)?;
            Some(ConstraintType::FixedRadius(arc, r))
        }
        "tangent_lc" => {
            let lid = as_string(c, "l_id")?;
            let (pa, pb) = line_pts(&lid)?;
            let circle = as_string(c, "c_id")?;
            let center = as_string(c, "center_id")?;
            Some(ConstraintType::TangentLineCircle(pa, pb, center, circle))
        }
        "midpoint_on_line_ll" => {
            let l1 = as_string(c, "l1_id")?;
            let l2 = as_string(c, "l2_id")?;
            let (a, b) = line_pts(&l1)?;
            let (c2, d) = line_pts(&l2)?;
            Some(ConstraintType::MidpointOfLineOnLine(a, b, c2, d))
        }
        "midpoint_on_line_pppp" => {
            let a = as_string(c, "l1p1_id")?;
            let b = as_string(c, "l1p2_id")?;
            let c2 = as_string(c, "l2p1_id")?;
            let d = as_string(c, "l2p2_id")?;
            Some(ConstraintType::MidpointOfLineOnLine(a, b, c2, d))
        }
        "p2p_symmetric_ppp" => {
            let p1 = as_string(c, "p1_id")?;
            let p2 = as_string(c, "p2_id")?;
            let pm = as_string(c, "p_id")?;
            Some(ConstraintType::Midpoint(pm, p1, p2))
        }
        "p2p_symmetric_ppl" => {
            let p1 = as_string(c, "p1_id")?;
            let p2 = as_string(c, "p2_id")?;
            let lid = as_string(c, "l_id")?;
            let (a, b) = line_pts(&lid)?;
            Some(ConstraintType::Symmetric(p1, p2, a, b))
        }
        "coordinate_x" => {
            let p = as_string(c, "p_id")?;
            let x = c.get("x").and_then(extract_scalar_f64)?;
            Some(ConstraintType::EqualX(p, x))
        }
        "coordinate_y" => {
            let p = as_string(c, "p_id")?;
            let y = c.get("y").and_then(extract_scalar_f64)?;
            Some(ConstraintType::EqualY(p, y))
        }
        _ => {
            skipped.push(format!("{}:{}", cid, ctype));
            None
        }
    }
}

/// Resolve circle center point id for constraints that only reference `c_id`.
fn enrich_circle_center_refs(primitives: &[Value]) -> Vec<Value> {
    let mut circle_center: HashMap<String, String> = HashMap::new();
    for p in primitives {
        let Some(t) = p.get("type").and_then(|x| x.as_str()) else {
            continue;
        };
        if t != "circle" {
            continue;
        }
        let Some(id) = as_str(p, "id") else {
            continue;
        };
        if let Some(cid) = as_string(p, "c_id") {
            circle_center.insert(id.to_string(), cid);
        }
    }

    primitives
        .iter()
        .map(|p| {
            let Some(t) = p.get("type").and_then(|x| x.as_str()) else {
                return p.clone();
            };
            let needs_center = t == "point_on_circle" || t == "tangent_lc";
            if !needs_center || p.get("center_id").is_some() {
                return p.clone();
            }
            let Some(c_id) = as_str(p, "c_id") else {
                return p.clone();
            };
            let Some(center) = circle_center.get(c_id).cloned() else {
                return p.clone();
            };
            let mut o = p.clone();
            if let Some(obj) = o.as_object_mut() {
                obj.insert("center_id".to_string(), json!(center));
            }
            o
        })
        .collect()
}

fn register_geometry(cs: &mut ConstraintSolver, p: &Value) -> Result<(), String> {
    let t = p
        .get("type")
        .and_then(|x| x.as_str())
        .ok_or_else(|| "primitive missing type".to_string())?;
    if !is_geometry(t) {
        return Ok(());
    }

    match t {
        "point" => {
            let id = as_string(p, "id").ok_or_else(|| "point missing id".to_string())?;
            let x = p.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let y = p.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let fixed = p.get("fixed").and_then(|v| v.as_bool()).unwrap_or(false);
            cs.add_point(Point::new(id, x, y, fixed));
        }
        "line" => {
            let id = as_string(p, "id").ok_or_else(|| "line missing id".to_string())?;
            let p1 = as_string(p, "p1_id").ok_or_else(|| "line missing p1_id".to_string())?;
            let p2 = as_string(p, "p2_id").ok_or_else(|| "line missing p2_id".to_string())?;
            cs.add_line(Line::new(id, p1, p2));
        }
        "circle" => {
            let id = as_string(p, "id").ok_or_else(|| "circle missing id".to_string())?;
            let center = as_string(p, "c_id").ok_or_else(|| "circle missing c_id".to_string())?;
            let radius = p.get("radius").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let fixed = p.get("fixed").and_then(|v| v.as_bool()).unwrap_or(false);
            cs.add_circle(Circle::new(id, center, radius, fixed));
        }
        "arc" => {
            let id = as_string(p, "id").ok_or_else(|| "arc missing id".to_string())?;
            let center = as_string(p, "c_id").ok_or_else(|| "arc missing c_id".to_string())?;
            let radius = p.get("radius").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let start_angle = p
                .get("start_angle")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let end_angle = p.get("end_angle").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let fixed = p.get("fixed").and_then(|v| v.as_bool()).unwrap_or(false);
            cs.add_arc(GeoArc::new(
                id,
                center,
                radius,
                start_angle,
                end_angle,
                fixed,
            ));
        }
        _ => { /* ellipse etc. — not in ACS system */ }
    }
    Ok(())
}

fn apply_solution_to_primitives(out: &mut [Value], cs: &ConstraintSolver) {
    for p in out.iter_mut() {
        let Some(t) = p.get("type").and_then(|x| x.as_str()) else {
            continue;
        };
        let Some(id) = p.get("id").and_then(|x| x.as_str()) else {
            continue;
        };
        if t == "point" {
            if let Some(pt) = cs.get_point(id.to_string())
                && let Some(obj) = p.as_object_mut()
            {
                obj.insert("x".to_string(), json!(pt.x));
                obj.insert("y".to_string(), json!(pt.y));
            }
        } else if t == "circle" {
            if let Some(c) = cs.get_circle(id.to_string())
                && let Some(obj) = p.as_object_mut()
            {
                obj.insert("radius".to_string(), json!(c.radius));
            }
        } else if t == "arc"
            && let Some(a) = cs.get_arc(id.to_string())
            && let Some(obj) = p.as_object_mut()
        {
            obj.insert("radius".to_string(), json!(a.radius));
            obj.insert("start_angle".to_string(), json!(a.start_angle));
            obj.insert("end_angle".to_string(), json!(a.end_angle));
        }
    }
}

/// Computes the set of fully-constrained (DOF = 0) entity IDs, including lines.
///
/// The solver only knows about parameter-bearing entities (points, circles, arcs);
/// a `Line` owns no parameters, so it is reported fully constrained when both of its
/// endpoints (`p1_id`, `p2_id`) are fully constrained.
fn fully_constrained_ids(cs: &ConstraintSolver, primitives: &[Value]) -> Vec<String> {
    let mut constrained: std::collections::BTreeSet<String> =
        cs.fully_constrained_entity_ids().into_iter().collect();

    for p in primitives {
        let Some(t) = p.get("type").and_then(|x| x.as_str()) else {
            continue;
        };
        if t != "line" {
            continue;
        }
        let (Some(id), Some(p1), Some(p2)) = (
            as_string(p, "id"),
            as_string(p, "p1_id"),
            as_string(p, "p2_id"),
        ) else {
            continue;
        };
        if constrained.contains(&p1) && constrained.contains(&p2) {
            constrained.insert(id);
        }
    }

    constrained.into_iter().collect()
}

/// Input: `{ "primitives": [ ... ], "max_iterations"?: number }` or a bare JSON array (primitives only).
pub fn solve_sketch_primitives_json(input: &str) -> Result<String, String> {
    let root: Value = serde_json::from_str(input).map_err(|e| e.to_string())?;

    let (primitives_val, max_iter) = if root.is_array() {
        (root.clone(), None)
    } else {
        let arr = root
            .get("primitives")
            .cloned()
            .filter(|v| v.is_array())
            .ok_or_else(|| "expected object with primitives array or a bare array".to_string())?;
        let mi = root
            .get("max_iterations")
            .and_then(|v| v.as_u64())
            .map(|u| u as usize);
        (arr, mi)
    };

    let primitives_arr = primitives_val
        .as_array()
        .ok_or_else(|| "primitives must be an array".to_string())?;

    let enriched = enrich_circle_center_refs(primitives_arr);
    let lines = build_line_endpoints(&enriched);

    let mut cs = ConstraintSolver::new();
    if let Some(n) = max_iter {
        cs.set_max_iterations(n);
    }

    for p in &enriched {
        register_geometry(&mut cs, p).map_err(|e| format!("geometry: {e}"))?;
    }

    let mut skipped_constraint_ids: Vec<String> = Vec::new();

    for p in &enriched {
        let Some(t) = p.get("type").and_then(|x| x.as_str()) else {
            continue;
        };
        if is_geometry(t) {
            continue;
        }

        let Some(ct) = constraint_to_acs(p, &lines, &mut skipped_constraint_ids) else {
            continue;
        };

        if let Err(e) = cs.add_constraint(ct) {
            let id = as_string(p, "id").unwrap_or_default();
            skipped_constraint_ids.push(format!("{}:add_error:{e}", id));
        }
    }

    let solve_result = cs.solve();

    let mut out: Vec<Value> = primitives_arr.clone();

    let (status, solve_status_num, error_msg, stats): (&str, i32, Option<String>, Option<Value>) =
        match &solve_result {
            Ok(SolverResult::Converged {
                iterations,
                final_error,
                initial_error,
            }) => (
                "converged",
                1,
                None,
                Some(json!({
                    "iterations": iterations,
                    "initial_error": initial_error,
                    "final_error": final_error,
                })),
            ),
            Ok(SolverResult::MaxIterationsReached {
                iterations,
                final_error,
                initial_error,
            }) => (
                "failed",
                2,
                Some("Solver did not converge".to_string()),
                Some(json!({
                    "iterations": iterations,
                    "initial_error": initial_error,
                    "final_error": final_error,
                })),
            ),
            Err(e) => ("failed", 2, Some(e.clone()), None),
        };

    if solve_result.is_ok() {
        apply_solution_to_primitives(&mut out, &cs);
    }

    let ok_converged = matches!(solve_result, Ok(SolverResult::Converged { .. }));

    // Fully-constrained (DOF = 0) analysis is only meaningful at a converged
    // configuration; report an empty list otherwise.
    let fully_constrained: Vec<String> = if ok_converged {
        fully_constrained_ids(&cs, primitives_arr)
    } else {
        Vec::new()
    };

    let response = json!({
        "ok": ok_converged,
        "status": status,
        "solveStatus": solve_status_num,
        "primitives": out,
        "skipped_constraint_ids": skipped_constraint_ids,
        "conflicting_constraint_ids": Vec::<String>::new(),
        "fully_constrained_ids": fully_constrained,
        "error": error_msg,
        "stats": stats,
    });

    serde_json::to_string(&response).map_err(|e| e.to_string())
}
