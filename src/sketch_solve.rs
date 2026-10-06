//! JSON sketch API: parses primitives, solves, and builds the response.
//! Constraint types and their fields are declared in `constraint_catalog`.

use std::collections::{HashMap, HashSet};

use serde_json::{json, Value};

use crate::geometry::{Arc as GeoArc, Circle, Ellipse, EllipticalArc, Line, Point};
use crate::constraint_catalog::{self, EllipticalArcPoints, References};
use crate::{ConstraintSolver, ConstraintType, SolverResult};

fn as_str<'a>(v: &'a Value, key: &str) -> Option<&'a str> {
    v.get(key)?.as_str()
}

fn as_string(v: &Value, key: &str) -> Option<String> {
    as_str(v, key).map(String::from)
}

const GEOMETRY_TYPES: &[&str] = &["point", "line", "circle", "arc", "ellipse", "elliptical_arc"];

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

/// Circle and Arc IDs mapped to their center Point IDs.
fn build_centers(primitives: &[Value]) -> HashMap<String, String> {
    primitives
        .iter()
        .filter(|p| matches!(as_str(p, "type"), Some("circle" | "arc")))
        .filter_map(|p| Some((as_string(p, "id")?, as_string(p, "c_id")?)))
        .collect()
}

/// Arc IDs mapped to their (start, end) Point IDs.
fn build_arc_endpoints(primitives: &[Value]) -> HashMap<String, (String, String)> {
    primitives
        .iter()
        .filter(|p| as_str(p, "type") == Some("arc"))
        .filter_map(|p| {
            Some((
                as_string(p, "id")?,
                (as_string(p, "start_id")?, as_string(p, "end_id")?),
            ))
        })
        .collect()
}

/// Ellipse IDs mapped to their (center, focus) Point IDs.
fn build_ellipses(primitives: &[Value]) -> HashMap<String, (String, String)> {
    primitives
        .iter()
        .filter(|p| as_str(p, "type") == Some("ellipse"))
        .filter_map(|p| {
            Some((
                as_string(p, "id")?,
                (as_string(p, "c_id")?, as_string(p, "focus1_id")?),
            ))
        })
        .collect()
}

/// EllipticalArc IDs mapped to their Points.
fn build_elliptical_arcs(primitives: &[Value]) -> HashMap<String, EllipticalArcPoints> {
    primitives
        .iter()
        .filter(|p| as_str(p, "type") == Some("elliptical_arc"))
        .filter_map(|p| {
            let points = EllipticalArcPoints {
                center: as_string(p, "c_id")?,
                focus: as_string(p, "focus1_id")?,
                start: as_string(p, "start_id")?,
                end: as_string(p, "end_id")?,
            };
            Some((as_string(p, "id")?, points))
        })
        .collect()
}

fn required_str(p: &Value, key: &str, what: &str) -> Result<String, String> {
    as_string(p, key).ok_or_else(|| format!("{what}: missing field '{key}'"))
}

fn required_f64(p: &Value, key: &str, what: &str) -> Result<f64, String> {
    p.get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| format!("{what}: missing field '{key}'"))
}

/// Whether the solver must hold an entity's own values (a Point's
/// coordinates, a Circle's radius, an Arc's radius and angles, an Ellipse's
/// `radmin`, an EllipticalArc's `radmin` and angles): `fixed`, or Reference Geometry (`isReference`), which never
/// moves.
fn fixed(p: &Value) -> bool {
    let flag = |key| p.get(key).and_then(Value::as_bool).unwrap_or(false);
    flag("fixed") || flag("isReference")
}

/// Adds one geometry primitive to the solver. Errors name the primitive and
/// the missing field.
fn register_geometry(cs: &mut ConstraintSolver, t: &str, id: &str, p: &Value) -> Result<(), String> {
    let what = format!("{t} {id}");
    let id = id.to_string();
    match t {
        "point" => {
            let x = required_f64(p, "x", &what)?;
            let y = required_f64(p, "y", &what)?;
            cs.add_point(Point::new(id, x, y, fixed(p)));
        }
        "line" => {
            let p1 = required_str(p, "p1_id", &what)?;
            let p2 = required_str(p, "p2_id", &what)?;
            cs.add_line(Line::new(id, p1, p2));
        }
        "circle" => {
            let center = required_str(p, "c_id", &what)?;
            let radius = required_f64(p, "radius", &what)?;
            cs.add_circle(Circle::new(id, center, radius, fixed(p)));
        }
        "arc" => {
            let center = required_str(p, "c_id", &what)?;
            let start = required_str(p, "start_id", &what)?;
            let end = required_str(p, "end_id", &what)?;
            let radius = required_f64(p, "radius", &what)?;
            let start_angle = required_f64(p, "start_angle", &what)?;
            let end_angle = required_f64(p, "end_angle", &what)?;
            cs.add_arc(GeoArc::new(
                id,
                center,
                start,
                end,
                radius,
                start_angle,
                end_angle,
                fixed(p),
            ));
        }
        "ellipse" => {
            let center = required_str(p, "c_id", &what)?;
            let focus = required_str(p, "focus1_id", &what)?;
            let radmin = required_f64(p, "radmin", &what)?;
            cs.add_ellipse(Ellipse::new(id, center, focus, radmin, fixed(p)));
        }
        "elliptical_arc" => {
            let center = required_str(p, "c_id", &what)?;
            let focus = required_str(p, "focus1_id", &what)?;
            let start = required_str(p, "start_id", &what)?;
            let end = required_str(p, "end_id", &what)?;
            let radmin = required_f64(p, "radmin", &what)?;
            let start_angle = required_f64(p, "start_angle", &what)?;
            let end_angle = required_f64(p, "end_angle", &what)?;
            cs.add_elliptical_arc(EllipticalArc::new(
                id,
                center,
                focus,
                start,
                end,
                radmin,
                start_angle,
                end_angle,
                fixed(p),
            ));
        }
        _ => unreachable!("not a geometry type: {t}"),
    }
    Ok(())
}

fn apply_solution_to_primitives(out: &mut [Value], cs: &ConstraintSolver) {
    for p in out.iter_mut() {
        let Some(t) = p.get("type").and_then(|x| x.as_str()).map(String::from) else {
            continue;
        };
        let Some(id) = p.get("id").and_then(|x| x.as_str()).map(String::from) else {
            continue;
        };
        let (t, id) = (t.as_str(), id.as_str());
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
        } else if t == "arc" {
            if let Some(a) = cs.get_arc(id.to_string())
                && let Some(obj) = p.as_object_mut()
            {
                obj.insert("radius".to_string(), json!(a.radius));
                obj.insert("start_angle".to_string(), json!(a.start_angle));
                obj.insert("end_angle".to_string(), json!(a.end_angle));
            }
        } else if t == "ellipse"
            && let Some(e) = cs.get_ellipse(id.to_string())
            && let Some(obj) = p.as_object_mut()
        {
            obj.insert("radmin".to_string(), json!(e.radmin));
        } else if t == "elliptical_arc"
            && let Some(e) = cs.get_elliptical_arc(id.to_string())
            && let Some(obj) = p.as_object_mut()
        {
            obj.insert("radmin".to_string(), json!(e.radmin));
            obj.insert("start_angle".to_string(), json!(e.start_angle));
            obj.insert("end_angle".to_string(), json!(e.end_angle));
        }
    }
}

/// Computes the set of fully-constrained (DOF = 0) entity IDs, including lines.
///
/// The solver only knows about parameter-bearing entities (points, circles, arcs,
/// ellipses); a `Line` owns no parameters, so it is reported fully constrained when
/// both of its endpoints (`p1_id`, `p2_id`) are fully constrained. An ellipse's
/// shape and place also depend on its center and focus Points, so it is reported
/// only when its `radmin` and both of those Points are; an elliptical arc only
/// when its `radmin`, angles, center, focus and endpoints are.
fn fully_constrained_ids(cs: &ConstraintSolver, primitives: &[Value]) -> Vec<String> {
    let mut constrained: std::collections::BTreeSet<String> =
        cs.fully_constrained_entity_ids().into_iter().collect();

    for p in primitives {
        let Some(t) = p.get("type").and_then(|x| x.as_str()) else {
            continue;
        };
        let (keys, own_vars): (&[&str], bool) = match t {
            "line" => (&["p1_id", "p2_id"], false),
            "ellipse" => (&["c_id", "focus1_id"], true),
            "elliptical_arc" => (&["c_id", "focus1_id", "start_id", "end_id"], true),
            _ => continue,
        };
        let Some(id) = as_string(p, "id") else {
            continue;
        };
        let points_locked = keys.iter().all(|k| {
            as_string(p, k).is_some_and(|point| constrained.contains(&point))
        });
        if !points_locked {
            constrained.remove(&id);
        } else if !own_vars {
            constrained.insert(id);
        }
    }

    constrained.into_iter().collect()
}

/// The wire type of a Parameter: a named value constraints share by id. The
/// solver holds it fixed and echoes it back unchanged.
const PARAM: &str = "param";

/// Parameter id → value. Errors name a Parameter without a numeric `value`.
fn build_params(typed: &[(&str, &str, &Value)]) -> Result<HashMap<String, f64>, String> {
    typed
        .iter()
        .filter(|(t, ..)| *t == PARAM)
        .map(|&(t, id, p)| Ok((id.to_string(), required_f64(p, "value", &format!("{t} {id}"))?)))
        .collect()
}

/// Why a request couldn't be read, and the constraint it's about, if any.
struct Rejection {
    error: String,
    constraint_id: Option<String>,
}

impl From<String> for Rejection {
    fn from(error: String) -> Self {
        Rejection {
            error,
            constraint_id: None,
        }
    }
}

impl From<&str> for Rejection {
    fn from(error: &str) -> Self {
        error.to_string().into()
    }
}

fn constraint_error(id: &str, error: impl std::fmt::Display) -> Rejection {
    Rejection {
        error: format!("constraint {id}: {error}"),
        constraint_id: Some(id.to_string()),
    }
}

/// The `{ version: 1, status: "invalid", error, constraintId? }` response for
/// a request that couldn't be read.
fn error_response(r: Rejection) -> String {
    let mut out = json!({ "version": 1, "status": "invalid", "error": r.error });
    if let Some(id) = r.constraint_id {
        out["constraintId"] = json!(id);
    }
    out.to_string()
}

/// Solves one sketch (`acsSolveSketch`, `P3DSketch_Solve`), with this
/// contract:
///
/// - Request: `{ "version": 1, "primitives": [...], "maxIterations"?: n }`.
/// - Response: `{ "version": 1, "status": "converged" | "failed",
///   "primitives": [...], "conflicting": [...], "redundant": [...], "dof": n }`
///   with the request's primitives in the same order and their solved values.
///   ACS also adds `stats` (`iterations`, `initialError`, `finalError`) and
///   `fullyConstrained` (entity IDs with zero degrees of freedom, only on a
///   converged solve).
///
/// Returns `Ok(response)` when the request was understood, whatever the solve
/// status, and `Err(response)` with `status: "invalid"` and an `error` naming
/// the problem otherwise: malformed JSON, a wrong version or `maxIterations`,
/// a duplicate id, an unknown or unsupported type, a missing field, a
/// reference to a missing entity or a combination of entity kinds
/// no variant of the type takes. When the problem is a constraint, the
/// response also carries its `constraintId`.
pub fn solve_sketch_json(request: &str) -> Result<String, String> {
    solve_request(request).map_err(error_response)
}

fn solve_request(request: &str) -> Result<String, Rejection> {
    let root: Value = serde_json::from_str(request).map_err(|e| format!("invalid JSON: {e}"))?;
    if root.get("version").and_then(Value::as_f64) != Some(1.0) {
        return Err("unsupported request version".into());
    }
    let primitives = root
        .get("primitives")
        .and_then(Value::as_array)
        .ok_or("request has no primitives array")?;

    let mut cs = ConstraintSolver::new();
    if let Some(v) = root.get("maxIterations") {
        let n = v
            .as_u64()
            .ok_or("maxIterations must be a non-negative whole number")?;
        cs.set_max_iterations(n as usize);
    }

    let mut seen = HashSet::new();
    let mut typed: Vec<(&str, &str, &Value)> = Vec::with_capacity(primitives.len());
    for (i, p) in primitives.iter().enumerate() {
        let t = as_str(p, "type").ok_or_else(|| format!("primitive #{i} has no type"))?;
        let id = as_str(p, "id").ok_or_else(|| format!("primitive #{i} has no id"))?;
        if !seen.insert(id) {
            return Err(format!("duplicate id '{id}'").into());
        }
        typed.push((t, id, p));
    }

    let refs = References {
        line_endpoints: build_line_endpoints(primitives),
        centers: build_centers(primitives),
        params: build_params(&typed)?,
        entity_types: typed
            .iter()
            .filter(|(t, ..)| is_geometry(t))
            .map(|&(t, id, _)| (id.to_string(), t.to_string()))
            .collect(),
        arc_endpoints: build_arc_endpoints(primitives),
        ellipses: build_ellipses(primitives),
        elliptical_arcs: build_elliptical_arcs(primitives),
    };

    for &(t, id, p) in typed.iter().filter(|(t, ..)| is_geometry(t)) {
        register_geometry(&mut cs, t, id, p)?;
    }
    check_geometry_references(&cs, &typed)?;
    // JSON id of each constraint, by `ConstraintSolver` index.
    let mut constraint_ids: Vec<&str> = Vec::new();
    let mut constraints: Vec<(ConstraintType, bool)> = Vec::new();
    for &(t, id, p) in typed.iter().filter(|(t, ..)| !is_geometry(t) && *t != PARAM) {
        let temporary = p.get("temporary").and_then(Value::as_bool).unwrap_or(false);
        let ct = constraint_catalog::parse(t, p, &refs).map_err(|e| constraint_error(id, e))?;
        constraints.push((ct, temporary));
        constraint_ids.push(id);
    }
    constraint_catalog::tangent_at_held_endpoints(&mut constraints);
    for ((ct, temporary), id) in constraints.into_iter().zip(&constraint_ids) {
        if temporary {
            cs.add_temporary_constraint(ct)
        } else {
            cs.add_constraint(ct)
        }
        .map_err(|e| constraint_error(id, e))?;
    }

    let (converged, iterations, initial_error, final_error) = match cs.solve()? {
        SolverResult::Converged {
            iterations,
            initial_error,
            final_error,
        } => (true, iterations, initial_error, final_error),
        SolverResult::MaxIterationsReached {
            iterations,
            initial_error,
            final_error,
        } => (false, iterations, initial_error, final_error),
    };

    let mut out = primitives.clone();
    apply_solution_to_primitives(&mut out, &cs);

    // Zero-DOF analysis is only meaningful at a solved configuration.
    let fully_constrained = if converged {
        fully_constrained_ids(&cs, primitives)
    } else {
        Vec::new()
    };

    // Diagnosed at the solved position, after the solve.
    let diagnosis = cs.diagnose();
    let to_ids = |indices: &[usize]| -> Vec<&str> { indices.iter().map(|&i| constraint_ids[i]).collect() };

    Ok(json!({
        "version": 1,
        "status": if converged { "converged" } else { "failed" },
        "primitives": out,
        "conflicting": to_ids(&diagnosis.conflicting),
        "redundant": to_ids(&diagnosis.redundant),
        "dof": cs.dof(),
        "fullyConstrained": fully_constrained,
        "stats": {
            "iterations": iterations,
            "initialError": initial_error,
            "finalError": final_error,
        },
    })
    .to_string())
}

/// Lines' endpoints, circles' centers, arcs' centers and endpoints,
/// ellipses' centers and foci and elliptical arcs' centers, foci and
/// endpoints must be Points.
fn check_geometry_references(cs: &ConstraintSolver, typed: &[(&str, &str, &Value)]) -> Result<(), String> {
    for &(t, id, p) in typed {
        let keys: &[&str] = match t {
            "line" => &["p1_id", "p2_id"],
            "circle" => &["c_id"],
            "arc" => &["c_id", "start_id", "end_id"],
            "ellipse" => &["c_id", "focus1_id"],
            "elliptical_arc" => &["c_id", "focus1_id", "start_id", "end_id"],
            _ => continue,
        };
        for key in keys {
            let target = as_str(p, key).unwrap_or_default();
            if cs.get_point(target.to_string()).is_none() {
                return Err(format!("{t} {id}: '{target}' is not a point"));
            }
        }
    }
    Ok(())
}
