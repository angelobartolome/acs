//! Every row of both constraint catalogs solves through every entry point
//! that speaks its vocabulary: the native one through `solve_sketch_json`
//! and `acsSolveSketch`, the PlaneGCS dialect through
//! `solve_planegcs_sketch_json`, `acsSolveSketchPlaneGcs` and (with the
//! `c-abi` feature) `P3DSketch_Solve`.

use acs::bindings::sketch_solve::{
    acs_constraint_catalog, acs_planegcs_constraint_catalog, acs_solve_sketch,
    acs_solve_sketch_planegcs,
};
use acs::constraint_catalog::{
    ConstraintSpec, ExtensionFlag, FieldKind, InternalFlag, Vocabulary, field_path,
};
use acs::sketch_solve::{solve_planegcs_sketch_json, solve_sketch_json};
use serde_json::{Value, json};

/// A sketch holding one constraint of `spec`'s row, with fresh geometry for
/// each of its fields: Points spread apart, Lines, radius-2 Circles and arcs
/// (with their endpoints on them), 2.5 × 2 ellipses, scalars suited to the
/// field, the major axis, and value fields referencing a fresh Circle's
/// radius.
fn sketch_for(spec: &ConstraintSpec, vocabulary: Vocabulary) -> Value {
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
    let (entity_key, property_key) = match vocabulary {
        Vocabulary::Native => ("entity", "property"),
        Vocabulary::PlaneGcs => ("o_id", "prop"),
    };
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
                "side" | "N" | "count" => json!(1),
                "angle" => json!(0.5),
                _ => json!(1.5),
            },
            FieldKind::Value => {
                circle(&mut prims, id.clone(), &mut point);
                json!({ entity_key: id, property_key: "radius" })
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
fn every_native_row_solves_through_the_native_entry_points() {
    for spec in Vocabulary::Native.specs() {
        let request = sketch_for(spec, Vocabulary::Native).to_string();
        let what = format!("native {} {:?}", spec.json_type, spec.fields);
        let response = solve_sketch_json(&request).unwrap_or_else(|e| panic!("{what}: {e}"));
        assert_converged(&response, &what);
        assert_eq!(acs_solve_sketch(&request), response, "{what}");
        #[cfg(feature = "c-abi")]
        assert_eq!(
            c_abi_solve(acs::c_abi::P3D_SKETCH_VOCABULARY_NATIVE, &request),
            (0, response),
            "{what}"
        );
    }
}

#[test]
fn every_dialect_row_solves_through_the_dialect_entry_points() {
    for spec in Vocabulary::PlaneGcs.specs() {
        let request = sketch_for(spec, Vocabulary::PlaneGcs).to_string();
        let what = format!("dialect {}", spec.json_type);
        let response =
            solve_planegcs_sketch_json(&request).unwrap_or_else(|e| panic!("{what}: {e}"));
        assert_converged(&response, &what);
        assert_eq!(acs_solve_sketch_planegcs(&request), response, "{what}");
        #[cfg(feature = "c-abi")]
        assert_eq!(
            c_abi_solve(acs::c_abi::P3D_SKETCH_VOCABULARY_PLANEGCS, &request),
            (0, response),
            "{what}"
        );
    }
}

#[cfg(feature = "c-abi")]
fn c_abi_solve(vocabulary: i32, request: &str) -> (i32, String) {
    use acs::c_abi::{P3DSketch_Free, P3DSketch_Solve};
    use std::ffi::{CStr, CString, c_char};
    let request = CString::new(request).unwrap();
    let mut response: *mut c_char = std::ptr::null_mut();
    let code = unsafe { P3DSketch_Solve(request.as_ptr(), vocabulary, &mut response) };
    let text = unsafe { CStr::from_ptr(response) }
        .to_str()
        .unwrap()
        .to_string();
    unsafe { P3DSketch_Free(response) };
    (code, text)
}

/// Each vocabulary is read only by its own entry points.
#[test]
fn a_vocabulary_does_not_accept_the_other_ones_types() {
    let request = |constraint: Value| {
        json!({ "version": 1, "primitives": [
            { "id": "p1", "type": "point", "x": 0.0, "y": 0.0 },
            { "id": "p2", "type": "point", "x": 1.0, "y": 0.0 },
            constraint
        ]})
        .to_string()
    };
    let gcs = request(json!({ "id": "k", "type": "p2p_coincident", "p1_id": "p1", "p2_id": "p2" }));
    let native = request(json!({ "id": "k", "type": "coincident", "a": "p1", "b": "p2" }));
    assert!(solve_planegcs_sketch_json(&gcs).is_ok());
    assert!(solve_sketch_json(&native).is_ok());
    assert!(
        solve_sketch_json(&gcs)
            .unwrap_err()
            .contains("unknown type 'p2p_coincident'")
    );
    assert!(
        solve_planegcs_sketch_json(&native)
            .unwrap_err()
            .contains("unknown type 'coincident'")
    );
}

fn catalog(json: &str) -> Vec<Value> {
    serde_json::from_str::<Value>(json)
        .unwrap()
        .as_array()
        .unwrap()
        .clone()
}

#[test]
fn acs_constraint_catalog_describes_the_native_vocabulary() {
    let rows = catalog(&acs_constraint_catalog());
    assert_eq!(rows.len(), Vocabulary::Native.specs().len());
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
        "midpoint_on",
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
        assert!(types.contains(t), "{t} missing from the native catalog");
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
    // Point–circle and circle–circle distance have an outside row
    // (`internal: false`) and an inside one (`internal: true`).
    let distance_internal: Vec<&Value> = rows
        .iter()
        .filter(|r| r["type"] == "distance" && r.get("internal").is_some())
        .collect();
    assert_eq!(distance_internal.len(), 4);
    for pair in distance_internal.chunks(2) {
        assert_eq!(pair[0]["internal"], false);
        assert_eq!(pair[1]["internal"], true);
        assert_eq!(pair[0]["fields"], pair[1]["fields"]);
        assert_eq!(pair[0]["fields"][1]["kind"], "circle");
    }
}

#[test]
fn the_native_catalog_lists_midpoint_by_entities_and_marks_the_old_forms_deprecated() {
    let rows = catalog(&acs_constraint_catalog());
    let midpoint: Vec<&Value> = rows
        .iter()
        .filter(|r| r["type"] == "midpoint" && r.get("deprecated").is_none())
        .collect();
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

    let deprecated: Vec<&Value> = rows
        .iter()
        .filter(|r| r.get("deprecated").is_some())
        .collect();
    assert_eq!(deprecated.len(), 3);
    for row in &deprecated {
        assert_eq!(row["deprecated"], true);
        assert!(
            row["type"] == "midpoint" || row["type"] == "midpoint_on",
            "{row}"
        );
    }

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

#[test]
fn the_planegcs_catalog_lists_the_dialect_types() {
    let rows = catalog(&acs_planegcs_constraint_catalog());
    assert_eq!(rows.len(), Vocabulary::PlaneGcs.specs().len());
    let p2p = rows.iter().find(|r| r["type"] == "p2p_distance").unwrap();
    assert_eq!(
        p2p,
        &json!({ "type": "p2p_distance", "fields": [
            { "name": "p1_id", "kind": "point" },
            { "name": "p2_id", "kind": "point" },
            { "name": "distance", "kind": "scalar" }
        ]})
    );
}
