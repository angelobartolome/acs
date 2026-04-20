use acs::{Circle, ConstraintSolver, ConstraintType, Point, SolverResult};

#[test]
fn test_concentric_constraint() {
    let mut solver = ConstraintSolver::new();

    // Circle 1 — fixed center at origin
    solver.add_point(Point::new("c1_center".into(), 0.0, 0.0, true));
    solver.add_circle(Circle::new("c1".into(), "c1_center".into(), 3.0, true));

    // Circle 2 — center starts offset, should move to coincide with c1's center
    solver.add_point(Point::new("c2_center".into(), 2.0, 3.0, false));
    solver.add_circle(Circle::new("c2".into(), "c2_center".into(), 5.0, true));

    solver
        .add_constraint(ConstraintType::Concentric(
            "c1_center".into(),
            "c2_center".into(),
        ))
        .expect("add concentric");

    let result = solver.solve().expect("solve");
    match result {
        SolverResult::Converged { final_error, .. } => assert!(final_error < 1e-6),
        _ => panic!("did not converge"),
    }

    let c2_center = solver.get_point("c2_center".into()).unwrap();
    assert!(
        (c2_center.x).abs() < 1e-5 && (c2_center.y).abs() < 1e-5,
        "centers should coincide at origin, got ({}, {})",
        c2_center.x,
        c2_center.y
    );
}
