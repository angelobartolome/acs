use acs::{Arc, ConstraintSolver, ConstraintType, Point, SolverResult};
use std::f64::consts::FRAC_PI_2;

/// A quarter arc whose endpoints start off it: `arc_rules` moves them onto
/// the arc at its start and end angles (or moves the arc to them).
#[test]
fn arc_rules_puts_the_endpoints_on_the_arc() {
    let mut solver = ConstraintSolver::new();
    solver.add_point(Point::new("c".into(), 0.0, 0.0, true));
    solver.add_point(Point::new("s".into(), 4.0, 1.0, false));
    solver.add_point(Point::new("e".into(), -1.0, 6.0, false));
    solver.add_arc(Arc::new(
        "a".into(),
        "c".into(),
        "s".into(),
        "e".into(),
        5.0,
        0.0,
        FRAC_PI_2,
        true,
    ));
    solver
        .add_constraint(ConstraintType::ArcRules(
            "c".into(),
            "s".into(),
            "e".into(),
            "a".into(),
        ))
        .unwrap();

    let result = solver.solve().unwrap();
    assert!(matches!(result, SolverResult::Converged { .. }), "{result:?}");
    let s = solver.get_point("s".into()).unwrap();
    assert!((s.x - 5.0).abs() < 1e-8 && s.y.abs() < 1e-8, "{s:?}");
    let e = solver.get_point("e".into()).unwrap();
    assert!(e.x.abs() < 1e-8 && (e.y - 5.0).abs() < 1e-8, "{e:?}");
}

/// With fixed endpoints, the arc's radius and angles move to meet them.
#[test]
fn arc_rules_moves_radius_and_angles_to_fixed_endpoints() {
    let mut solver = ConstraintSolver::new();
    solver.add_point(Point::new("c".into(), 0.0, 0.0, true));
    solver.add_point(Point::new("s".into(), 0.0, 3.0, true));
    solver.add_point(Point::new("e".into(), -3.0, 0.0, true));
    solver.add_arc(Arc::new(
        "a".into(),
        "c".into(),
        "s".into(),
        "e".into(),
        2.0,
        1.4,
        3.0,
        false,
    ));
    solver
        .add_constraint(ConstraintType::ArcRules(
            "c".into(),
            "s".into(),
            "e".into(),
            "a".into(),
        ))
        .unwrap();

    let result = solver.solve().unwrap();
    assert!(matches!(result, SolverResult::Converged { .. }), "{result:?}");
    let a = solver.get_arc("a".into()).unwrap();
    assert!((a.radius - 3.0).abs() < 1e-8, "{a:?}");
    assert!((a.start_angle - FRAC_PI_2).abs() < 1e-8, "{a:?}");
    assert!((a.end_angle - std::f64::consts::PI).abs() < 1e-8, "{a:?}");
}

#[test]
fn arc_rules_rejects_a_circle() {
    let mut solver = ConstraintSolver::new();
    solver.add_point(Point::new("c".into(), 0.0, 0.0, true));
    solver.add_circle(acs::Circle::new("k".into(), "c".into(), 1.0, false));
    let err = solver
        .add_constraint(ConstraintType::ArcRules(
            "c".into(),
            "c".into(),
            "c".into(),
            "k".into(),
        ))
        .unwrap_err();
    assert_eq!(err, "'k' is not an arc");
}
