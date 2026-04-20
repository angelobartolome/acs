use std::f64::consts::{FRAC_PI_2, FRAC_PI_4};

use acs::{ConstraintSolver, ConstraintType, Point, SolverResult};

/// Force the angle from L1 to L2 to be 90°.
/// L1 is fixed horizontal; L2 should rotate to be perpendicular.
#[test]
fn test_angle_90_degrees() {
    let mut solver = ConstraintSolver::new();

    // L1: horizontal (fixed)
    solver.add_point(Point::new("p1".into(), 0.0, 0.0, true));
    solver.add_point(Point::new("p2".into(), 1.0, 0.0, true));

    // L2: starts diagonal, should become vertical (90° from horizontal)
    solver.add_point(Point::new("p3".into(), 0.0, 0.0, true));
    solver.add_point(Point::new("p4".into(), 0.5, 1.0, false));

    solver
        .add_constraint(ConstraintType::Angle(
            "p1".into(),
            "p2".into(),
            "p3".into(),
            "p4".into(),
            FRAC_PI_2,
        ))
        .expect("add angle");

    let result = solver.solve().expect("solve");
    match result {
        SolverResult::Converged { final_error, .. } => assert!(final_error < 1e-6),
        _ => panic!("did not converge"),
    }

    let p3 = solver.get_point("p3".into()).unwrap();
    let p4 = solver.get_point("p4".into()).unwrap();
    // L1 direction: (1,0), L2 direction: (p4-p3)
    // For 90°: dot must be ~0
    let dot = (1.0_f64) * (p4.x - p3.x) + (0.0_f64) * (p4.y - p3.y);
    assert!(
        dot.abs() < 1e-4,
        "angle should be 90°, dot with L1 = {}",
        dot
    );
}

/// Force the angle to be 45°.
#[test]
fn test_angle_45_degrees() {
    let mut solver = ConstraintSolver::new();

    // L1: horizontal (fixed)
    solver.add_point(Point::new("p1".into(), 0.0, 0.0, true));
    solver.add_point(Point::new("p2".into(), 1.0, 0.0, true));

    // L2: starts vertical, should rotate to 45°
    solver.add_point(Point::new("p3".into(), 0.0, 0.0, true));
    solver.add_point(Point::new("p4".into(), 0.0, 1.0, false));

    solver
        .add_constraint(ConstraintType::Angle(
            "p1".into(),
            "p2".into(),
            "p3".into(),
            "p4".into(),
            FRAC_PI_4,
        ))
        .expect("add angle 45°");

    let result = solver.solve().expect("solve");
    match result {
        SolverResult::Converged { final_error, .. } => assert!(final_error < 1e-6),
        _ => panic!("did not converge"),
    }

    let p3 = solver.get_point("p3".into()).unwrap();
    let p4 = solver.get_point("p4".into()).unwrap();
    let dx = p4.x - p3.x;
    let dy = p4.y - p3.y;
    let measured = dy.atan2(dx);
    assert!(
        (measured - FRAC_PI_4).abs() < 1e-4,
        "angle should be 45°, got {}°",
        measured.to_degrees()
    );
}
