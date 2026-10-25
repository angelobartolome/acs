//! The typed sketch API (`sketch_types`): what `tests/catalog_rows_test.rs`
//! doesn't reach (every row's fields round-trip and solve there). Options and
//! defaults, fields passed through, `analyze: false`, rejections and
//! `spline_curve`.

use acs::sketch_solve::{solve_sketch_json, spline_curve_json};
use acs::sketch_types::*;
use serde_json::{Value, json};

fn point(id: &str, x: f64, y: f64) -> Primitive {
    Primitive::Point { id: id.into(), x, y, fixed: None, is_reference: None, is_construction: None, group: None }
}

fn line(id: &str, p1: &str, p2: &str) -> Primitive {
    Primitive::Line { id: id.into(), p1: p1.into(), p2: p2.into(), is_reference: None, is_construction: None, group: None }
}

fn request(primitives: Vec<Primitive>) -> SolveRequest {
    SolveRequest { version: 1, primitives, max_iterations: None, analyze: None }
}

/// The typed solve, checked against the JSON API on the same request.
fn solve_both(request: &SolveRequest) -> SolveResponse {
    let json = solve_sketch_json(&serde_json::to_string(request).unwrap()).unwrap();
    let typed = solve(request.clone()).unwrap();
    assert_eq!(typed, serde_json::from_str::<SolveResponse>(&json).unwrap());
    typed
}

fn find<'a>(response: &'a SolveResponse, id: &str) -> &'a Primitive {
    response.primitives.iter().find(|p| serde_json::to_value(p).unwrap()["id"] == id).unwrap()
}

#[test]
fn a_typed_request_is_the_json_request() {
    let request = request(vec![
        point("a", 0.0, 0.0),
        line("l", "a", "b"),
        Primitive::Horizontal { id: "h".into(), of: LineOrPoints::Points { a: "a".into(), b: "b".into() }, temporary: None },
        Primitive::Angle {
            id: "k".into(),
            of: AngleOf::Lines { a: "l".into(), b: "m".into(), reverse_a: Some(true), reverse_b: None },
            value: Scalar::Parameter("theta".into()),
            temporary: Some(true),
        },
        Primitive::Equal {
            id: "e".into(),
            a: Operand::Property(PropertyRef { entity: "c".into(), property: Property::Radius }),
            b: Operand::Number(2.0),
            temporary: None,
        },
    ]);
    assert_eq!(
        serde_json::to_value(&request).unwrap(),
        json!({ "version": 1, "primitives": [
            { "type": "point", "id": "a", "x": 0.0, "y": 0.0 },
            { "type": "line", "id": "l", "p1_id": "a", "p2_id": "b" },
            { "type": "horizontal", "id": "h", "a": "a", "b": "b" },
            { "type": "angle", "id": "k", "a": "l", "b": "m", "reverseA": true, "value": "theta", "temporary": true },
            { "type": "equal", "id": "e", "a": { "entity": "c", "property": "radius" }, "b": 2.0 }
        ] })
    );
}

#[test]
fn options_defaults_and_parameters_solve_as_in_json() {
    use std::f64::consts::FRAC_PI_2;
    let fixed = |id: &str, x: f64, y: f64| Primitive::Point {
        id: id.into(), x, y, fixed: Some(true), is_reference: None, is_construction: None, group: None,
    };
    let response = solve_both(&request(vec![
        fixed("o", 0.0, 0.0),
        fixed("p", 4.0, 0.0),
        point("q", 3.0, 3.0),
        point("c1", 0.0, 3.0),
        point("c2", -2.0, 1.0),
        line("op", "o", "p"),
        line("oq", "o", "q"),
        Primitive::Param { id: "theta".into(), value: FRAC_PI_2, name: Some("Theta".into()), group: Some("g".into()) },
        Primitive::Angle {
            id: "ray".into(),
            of: AngleOf::Lines { a: "op".into(), b: "oq".into(), reverse_a: Some(false), reverse_b: Some(false) },
            value: Scalar::Parameter("theta".into()),
            temporary: None,
        },
        Primitive::Distance {
            id: "len".into(), a: "o".into(), b: "q".into(), value: Scalar::Number(2.0),
            extension: None, internal: None, temporary: None,
        },
        // `count` absent: the first copy.
        Primitive::Rotation {
            id: "r1".into(), source: "p".into(), copy: "c1".into(), center: "o".into(),
            angle: Scalar::Number(FRAC_PI_2), count: None, temporary: None,
        },
        Primitive::Rotation {
            id: "r2".into(), source: "p".into(), copy: "c2".into(), center: "o".into(),
            angle: Scalar::Number(FRAC_PI_2), count: Some(Scalar::Number(2.0)), temporary: None,
        },
    ]));
    assert_eq!(response.status, SolveStatus::Converged);
    let at = |id: &str| match find(&response, id) {
        Primitive::Point { x, y, .. } => (*x, *y),
        other => panic!("{other:?}"),
    };
    let close = |(x, y): (f64, f64), (ex, ey): (f64, f64)| (x - ex).abs() < 1e-8 && (y - ey).abs() < 1e-8;
    assert!(close(at("q"), (0.0, 2.0)), "{:?}", at("q"));
    assert!(close(at("c1"), (0.0, 4.0)), "{:?}", at("c1"));
    assert!(close(at("c2"), (-4.0, 0.0)), "{:?}", at("c2"));
}

#[test]
fn passed_through_fields_come_back() {
    let response = solve_both(&request(vec![
        Primitive::Point {
            id: "a".into(), x: 1.0, y: 2.0, fixed: None, is_reference: Some(true),
            is_construction: Some(true), group: Some("g1".into()),
        },
        point("b", 3.0, 3.0),
        Primitive::Line {
            id: "l".into(), p1: "a".into(), p2: "b".into(), is_reference: None,
            is_construction: Some(false), group: Some("g2".into()),
        },
    ]));
    assert_eq!(
        find(&response, "a"),
        &Primitive::Point {
            id: "a".into(), x: 1.0, y: 2.0, fixed: None, is_reference: Some(true),
            is_construction: Some(true), group: Some("g1".into()),
        }
    );
    assert!(matches!(find(&response, "l"), Primitive::Line { group: Some(g), .. } if g == "g2"));
}

#[test]
fn analyze_false_leaves_the_analysis_out() {
    let mut r = request(vec![point("a", 0.0, 0.0), point("b", 1.0, 1.0), line("l", "a", "b")]);
    let analysed = solve_both(&r);
    assert_eq!((analysed.dof, analysed.conflicting.as_deref()), (Some(4), Some(&[][..])));
    r.analyze = Some(false);
    let response = solve_both(&r);
    assert_eq!(
        (response.dof, response.conflicting, response.redundant, response.fully_constrained),
        (None, None, None, None)
    );
}

#[test]
fn a_rejection_is_an_error_with_the_json_message_and_constraint() {
    let r = request(vec![
        point("a", 0.0, 0.0),
        Primitive::Horizontal { id: "k".into(), of: LineOrPoints::Line { line: "ghost".into() }, temporary: None },
    ]);
    let json: Value = serde_json::from_str(&solve_sketch_json(&serde_json::to_string(&r).unwrap()).unwrap_err()).unwrap();
    let SketchError::Invalid { message, constraint_id } = solve(r).unwrap_err();
    assert_eq!(message, json["error"]);
    assert_eq!(constraint_id.as_deref(), Some("k"));

    let SketchError::Invalid { message, .. } = solve(SolveRequest { version: 2, ..request(vec![]) }).unwrap_err();
    assert_eq!(message, "unsupported request version");
}

#[test]
fn spline_curve_is_the_json_curve() {
    let handles = [[0.0, 0.0], [1.0, 2.0], [3.0, 2.5], [4.0, -1.0], [6.0, 0.0]];
    let xy: Vec<Xy> = handles.iter().map(|&[x, y]| Xy { x, y }).collect();
    for (interpolated, knots) in [(true, None), (false, None), (false, Some(vec![0.0, 0.0, 0.0, 0.0, 0.4, 1.0, 1.0, 1.0, 1.0]))] {
        let json = spline_curve_json(&json!({ "version": 1, "handles": handles, "interpolated": interpolated, "knots": knots }).to_string()).unwrap();
        let expected: Curve = serde_json::from_value(serde_json::from_str::<Value>(&json).unwrap()["curve"].clone()).unwrap();
        assert_eq!(spline_curve(xy.clone(), interpolated, knots).unwrap(), expected);
    }
    let SketchError::Invalid { message, constraint_id } = spline_curve(xy[..3].to_vec(), false, None).unwrap_err();
    assert!(message.contains("at least 4 control points"), "{message}");
    assert_eq!(constraint_id, None);
}
