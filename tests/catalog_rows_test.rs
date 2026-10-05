//! Every row of the constraint catalog solves through every entry point:
//! `solve_sketch_json`, `acsSolveSketch` and (with the `c-abi` feature)
//! `P3DSketch_Solve`.

use acs::bindings::sketch_solve::{acs_constraint_catalog, acs_solve_sketch};
use acs::constraint_catalog::{ConstraintSpec, ExtensionFlag, FieldKind, InternalFlag, field_path, specs};
use acs::sketch_solve::solve_sketch_json;
use serde_json::{Value, json};

/// A sketch holding one constraint of `spec`'s row, with fresh geometry for
/// each of its fields: Points spread apart, Lines, radius-2 Circles and arcs
/// (with their endpoints on them), 2.5 × 2 ellipses, scalars suited to the
/// field, the major axis, and value fields referencing a fresh Circle's
/// radius.
fn sketch_for(spec: &ConstraintSpec) -> Value {
    let mut prims: Vec<Value> = Vec::new();
    let mut n = 0;
    let mut point = |prims: &mut Vec<Value>| {
        n += 1;
        let id = format!("p{n}");
        let (x, y) = (
            2.7 * n as f64,
            1.1 * n as f64 + if n % 2 == 0 { 1.9 } else { -0.8 },
        );
        prims.push(json!({ "id": id, "type": "point", "x": x, "y": y }));
        id
    };
    let circle =
        |prims: &mut Vec<Value>, id: String, point: &mut dyn FnMut(&mut Vec<Value>) -> String| {
            let c = point(prims);
            prims.push(json!({ "id": id, "type": "circle", "c_id": c, "radius": 2.0 }));
        };
    let mut constraint = json!({ "id": "k", "type": spec.json_type });
    if spec.extension == ExtensionFlag::Extension {
        constraint["extension"] = json!(true);
    }
    if spec.internal == InternalFlag::Internal {
        constraint["internal"] = json!(true);
    }
    for (i, &(name, kind)) in spec.fields.iter().enumerate() {
        let id = format!("{name}_{i}");
        let value = match kind {
            FieldKind::Point => json!(point(&mut prims)),
            FieldKind::Line => {
                let (a, b) = (point(&mut prims), point(&mut prims));
                prims.push(json!({ "id": id, "type": "line", "p1_id": a, "p2_id": b }));
                json!(id)
            }
            FieldKind::Circle => {
                circle(&mut prims, id.clone(), &mut point);
                json!(id)
            }
            FieldKind::Arc => {
                let c = point(&mut prims);
                let (cx, cy) = {
                    let p = prims.last().unwrap();
                    (p["x"].as_f64().unwrap(), p["y"].as_f64().unwrap())
                };
                let (a0, a1, r) = (0.2_f64, 1.4_f64, 2.0);
                let s = format!("{id}_start");
                let e = format!("{id}_end");
                prims.push(json!({ "id": s, "type": "point", "x": cx + r * a0.cos(), "y": cy + r * a0.sin() }));
                prims.push(json!({ "id": e, "type": "point", "x": cx + r * a1.cos(), "y": cy + r * a1.sin() }));
                prims.push(
                    json!({ "id": id, "type": "arc", "c_id": c, "start_id": s, "end_id": e,
                                   "radius": r, "start_angle": a0, "end_angle": a1 }),
                );
                json!(id)
            }
            FieldKind::Ellipse => {
                let c = point(&mut prims);
                let (cx, cy) = {
                    let p = prims.last().unwrap();
                    (p["x"].as_f64().unwrap(), p["y"].as_f64().unwrap())
                };
                let f = format!("{id}_focus");
                prims.push(json!({ "id": f, "type": "point", "x": cx + 1.5, "y": cy }));
                prims.push(json!({ "id": id, "type": "ellipse", "c_id": c, "focus1_id": f, "radmin": 2.0 }));
                json!(id)
            }
            FieldKind::Axis => json!("major"),
            FieldKind::Scalar => match name {
                "side" | "count" => json!(1),
                "angle" => json!(0.5),
                _ => json!(1.5),
            },
            FieldKind::Value => {
                circle(&mut prims, id.clone(), &mut point);
                json!({ "entity": id, "property": "radius" })
            }
        };
        match field_path(name) {
            (name, None) => constraint[name] = value,
            (name, Some(i)) => {
                if constraint.get(name).is_none() {
                    constraint[name] = json!([]);
                }
                let array = constraint[name].as_array_mut().unwrap();
                array.resize(array.len().max(i + 1), Value::Null);
                array[i] = value;
            }
        }
    }
    prims.push(constraint);
    json!({ "version": 1, "primitives": prims })
}

fn assert_converged(response: &str, what: &str) {
    let resp: Value = serde_json::from_str(response).unwrap();
    assert_eq!(resp["status"], "converged", "{what}: {response}");
}

#[test]
fn every_row_solves_through_every_entry_point() {
    for spec in specs() {
        let request = sketch_for(spec).to_string();
        let what = format!("{} {:?}", spec.json_type, spec.fields);
        let response = solve_sketch_json(&request).unwrap_or_else(|e| panic!("{what}: {e}"));
        assert_converged(&response, &what);
        assert_eq!(acs_solve_sketch(&request), response, "{what}");
        #[cfg(feature = "c-abi")]
        assert_eq!(
            c_abi_solve(&request),
            (0, response),
            "{what}"
        );
    }
}

#[cfg(feature = "c-abi")]
fn c_abi_solve(request: &str) -> (i32, String) {
    use acs::c_abi::{P3DSketch_Free, P3DSketch_Solve};
    use std::ffi::{CStr, CString, c_char};
    let request = CString::new(request).unwrap();
    let mut response: *mut c_char = std::ptr::null_mut();
    let code = unsafe { P3DSketch_Solve(request.as_ptr(), &mut response) };
    let text = unsafe { CStr::from_ptr(response) }
        .to_str()
        .unwrap()
        .to_string();
    unsafe { P3DSketch_Free(response) };
    (code, text)
}

fn catalog(json: &str) -> Vec<Value> {
    serde_json::from_str::<Value>(json)
        .unwrap()
        .as_array()
        .unwrap()
        .clone()
}

#[test]
fn acs_constraint_catalog_describes_every_row() {
    let rows = catalog(&acs_constraint_catalog());
    assert_eq!(rows.len(), specs().len());
    let types: std::collections::BTreeSet<&str> =
        rows.iter().map(|r| r["type"].as_str().unwrap()).collect();
    for t in [
        "coincident",
        "horizontal",
        "vertical",
        "parallel",
        "perpendicular",
        "angle",
        "direction",
        "distance",
        "offset",
        "on",
        "midpoint",
        "tangent",
        "concentric",
        "equal",
        "radius",
        "difference",
        "x",
        "y",
        "mirror",
        "rotation",
        "translation",
        "ellipse_axis",
        "diameter",
        "length",
    ] {
        assert!(types.contains(t), "{t} missing from the catalog");
    }
    // `distance` from a point to a Line: a segment row and an Extension row.
    let point_line: Vec<&Value> = rows
        .iter()
        .filter(|r| {
            r["type"] == "distance"
                && r["fields"][0]["kind"] == "point"
                && r["fields"][1]["kind"] == "line"
        })
        .collect();
    assert_eq!(point_line.len(), 2);
    assert_eq!(point_line[0]["extension"], false);
    assert_eq!(point_line[1]["extension"], true);
    assert_eq!(
        point_line[0]["fields"],
        json!([
            { "name": "a", "kind": "point" },
            { "name": "b", "kind": "line" },
            { "name": "value", "kind": "scalar" }
        ])
    );
    // Rows without an Extension variant carry no flag.
    let coincident = rows.iter().find(|r| r["type"] == "coincident").unwrap();
    assert!(coincident.get("extension").is_none());
    // Point–circle, circle–circle, circle–arc and arc–arc distance have an
    // outside row (`internal: false`) and an inside one (`internal: true`).
    let distance_internal: Vec<&Value> = rows
        .iter()
        .filter(|r| r["type"] == "distance" && r.get("internal").is_some())
        .collect();
    assert_eq!(distance_internal.len(), 8);
    for pair in distance_internal.chunks(2) {
        assert_eq!(pair[0]["internal"], false);
        assert_eq!(pair[1]["internal"], true);
        assert_eq!(pair[0]["fields"], pair[1]["fields"]);
        assert!(["circle", "arc"].contains(&pair[0]["fields"][1]["kind"].as_str().unwrap()));
    }
}

#[test]
fn the_catalog_lists_midpoint_by_entities() {
    let rows = catalog(&acs_constraint_catalog());
    let midpoint: Vec<&Value> = rows.iter().filter(|r| r["type"] == "midpoint").collect();
    let entities = |first: &str| {
        json!([
            { "name": "entities", "index": 0, "kind": first },
            { "name": "entities", "index": 1, "kind": "line" }
        ])
    };
    assert_eq!(midpoint.len(), 3);
    assert_eq!(midpoint[0]["fields"], entities("point"));
    assert!(midpoint[0].get("extension").is_none());
    assert_eq!(midpoint[1]["fields"], entities("line"));
    assert_eq!(midpoint[1]["extension"], false);
    assert_eq!(midpoint[2]["fields"], entities("line"));
    assert_eq!(midpoint[2]["extension"], true);

    assert!(!types_of(&rows).contains("midpoint_on"));
    assert!(rows.iter().all(|r| r.get("deprecated").is_none()));

    let axis: Vec<&Value> = rows
        .iter()
        .filter(|r| r["type"] == "ellipse_axis")
        .collect();
    assert_eq!(axis.len(), 2);
    assert_eq!(
        axis[1]["fields"],
        json!([
            { "name": "ellipse", "kind": "ellipse" },
            { "name": "a", "kind": "point" },
            { "name": "b", "kind": "point" },
            { "name": "which", "kind": "axis" }
        ])
    );
}

/// `tangent` between circles and arcs has an external row (`internal:
/// false`) and an inside one (`internal: true`) per pair of kinds; rows with
/// a Line carry no flag.
#[test]
fn the_native_catalog_marks_external_and_inside_tangency() {
    let rows = catalog(&acs_constraint_catalog());
    let tangent: Vec<&Value> = rows.iter().filter(|r| r["type"] == "tangent").collect();
    let kinds = |r: &Value| -> Vec<String> {
        r["fields"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f["kind"].as_str().unwrap().to_string())
            .collect()
    };
    for pair in [["circle", "circle"], ["circle", "arc"], ["arc", "arc"]] {
        let flags: Vec<&Value> = tangent
            .iter()
            .filter(|r| kinds(r) == pair)
            .map(|r| &r["internal"])
            .collect();
        assert_eq!(flags, [&json!(false), &json!(true)], "{pair:?}");
    }
    for row in tangent.iter().filter(|r| kinds(r).contains(&"line".to_string())) {
        assert!(row.get("internal").is_none(), "{row}");
    }
    let coincident = rows.iter().find(|r| r["type"] == "coincident").unwrap();
    assert!(coincident.get("internal").is_none());
}

fn types_of(rows: &[Value]) -> std::collections::BTreeSet<&str> {
    rows.iter().map(|r| r["type"].as_str().unwrap()).collect()
}
