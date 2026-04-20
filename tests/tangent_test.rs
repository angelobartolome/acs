use acs::{Circle, ConstraintSolver, ConstraintType, Point, SolverResult};

/// External tangency: distance between centres should equal r1 + r2.
#[test]
fn test_tangent_circle_circle() {
    let mut solver = ConstraintSolver::new();

    // Circle 1: center at origin, radius 3 (fixed)
    solver.add_point(Point::new("c1_center".into(), 0.0, 0.0, true));
    let c1 = solver.add_circle(Circle::new("c1".into(), "c1_center".into(), 3.0, true));

    // Circle 2: center along x-axis, radius 2 (fixed) — starts too close
    solver.add_point(Point::new("c2_center".into(), 4.0, 0.0, false));
    let c2 = solver.add_circle(Circle::new("c2".into(), "c2_center".into(), 2.0, true));

    solver
        .add_constraint(ConstraintType::Tangent(
            "c1_center".into(),
            c1,
            "c2_center".into(),
            c2,
        ))
        .expect("add tangent");

    let result = solver.solve().expect("solve");
    match result {
        SolverResult::Converged { final_error, .. } => assert!(final_error < 1e-6),
        _ => panic!("did not converge"),
    }

    let c2_center = solver.get_point("c2_center".into()).unwrap();
    let dist = (c2_center.x * c2_center.x + c2_center.y * c2_center.y).sqrt();
    assert!(
        (dist - 5.0).abs() < 1e-4,
        "distance between centres should be 5 (3+2), got {}",
        dist
    );
}
