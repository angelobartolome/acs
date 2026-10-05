//! An Arc's implicit rules (`ArcRulesConstraint`, added for every arc) move
//! its endpoints onto it, or the arc to its endpoints.

use acs::{Arc, ConstraintSolver, Point, SolverResult};
use std::f64::consts::FRAC_PI_2;

/// A fixed quarter arc whose endpoints start off it: its rules move them
/// onto the arc at its start and end angles.
#[test]
fn the_arc_rules_put_the_endpoints_on_the_arc() {
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

    let result = solver.solve().unwrap();
    assert!(matches!(result, SolverResult::Converged { .. }), "{result:?}");
    let s = solver.get_point("s".into()).unwrap();
    assert!((s.x - 5.0).abs() < 1e-8 && s.y.abs() < 1e-8, "{s:?}");
    let e = solver.get_point("e".into()).unwrap();
    assert!(e.x.abs() < 1e-8 && (e.y - 5.0).abs() < 1e-8, "{e:?}");
}

/// With fixed endpoints, the arc's radius and angles move to meet them.
#[test]
fn the_arc_rules_move_radius_and_angles_to_fixed_endpoints() {
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

    let result = solver.solve().unwrap();
    assert!(matches!(result, SolverResult::Converged { .. }), "{result:?}");
    let a = solver.get_arc("a".into()).unwrap();
    assert!((a.radius - 3.0).abs() < 1e-8, "{a:?}");
    assert!((a.start_angle - FRAC_PI_2).abs() < 1e-8, "{a:?}");
    assert!((a.end_angle - std::f64::consts::PI).abs() < 1e-8, "{a:?}");
}
