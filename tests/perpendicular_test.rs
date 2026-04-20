use acs::{ConstraintSolver, ConstraintType, Point, SolverResult};

#[test]
fn test_perpendicular_constraint() {
    let mut solver = ConstraintSolver::new();

    // L1: horizontal line along x-axis (fixed)
    solver.add_point(Point::new("p1".into(), 0.0, 0.0, true));
    solver.add_point(Point::new("p2".into(), 1.0, 0.0, true));

    // L2: starts at origin, initially diagonal — should become vertical
    solver.add_point(Point::new("p3".into(), 0.0, 0.0, true));
    solver.add_point(Point::new("p4".into(), 1.0, 1.0, false));

    solver
        .add_constraint(ConstraintType::Perpendicular(
            "p1".into(),
            "p2".into(),
            "p3".into(),
            "p4".into(),
        ))
        .expect("add perpendicular");

    let result = solver.solve().expect("solve");
    match result {
        SolverResult::Converged { final_error, .. } => assert!(final_error < 1e-6),
        _ => panic!("did not converge"),
    }

    // After constraint, direction of L2 must be perpendicular to L1 = (1,0)
    // So L2 direction must have zero x-component (p4.x ≈ p3.x)
    let p3 = solver.get_point("p3".into()).unwrap();
    let p4 = solver.get_point("p4".into()).unwrap();
    let dot = (1.0 - 0.0) * (p4.x - p3.x) + (0.0 - 0.0) * (p4.y - p3.y);
    assert!(
        dot.abs() < 1e-4,
        "lines not perpendicular, dot = {}",
        dot
    );
}
