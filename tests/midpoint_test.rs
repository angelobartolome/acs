use acs::{ConstraintSolver, ConstraintType, Point, SolverResult};

#[test]
fn test_midpoint_constraint() {
    let mut solver = ConstraintSolver::new();

    // Endpoints fixed at (0,0) and (10,4)
    solver.add_point(Point::new("a".into(), 0.0, 0.0, true));
    solver.add_point(Point::new("b".into(), 10.0, 4.0, true));

    // Midpoint starts at a wrong position, should converge to (5, 2)
    solver.add_point(Point::new("m".into(), 3.0, 1.0, false));

    solver
        .add_constraint(ConstraintType::Midpoint("m".into(), "a".into(), "b".into()))
        .expect("add midpoint");

    let result = solver.solve().expect("solve");
    match result {
        SolverResult::Converged { final_error, .. } => assert!(final_error < 1e-6),
        _ => panic!("did not converge"),
    }

    let m = solver.get_point("m".into()).unwrap();
    assert!(
        (m.x - 5.0).abs() < 1e-5 && (m.y - 2.0).abs() < 1e-5,
        "midpoint should be (5, 2), got ({}, {})",
        m.x,
        m.y
    );
}

#[test]
fn test_midpoint_moves_endpoint() {
    let mut solver = ConstraintSolver::new();

    // Fix a and the midpoint; let b float
    solver.add_point(Point::new("a".into(), 0.0, 0.0, true));
    solver.add_point(Point::new("m".into(), 5.0, 0.0, true));
    solver.add_point(Point::new("b".into(), 6.0, 0.0, false));

    solver
        .add_constraint(ConstraintType::Midpoint("m".into(), "a".into(), "b".into()))
        .expect("add midpoint");

    solver.set_max_iterations(500);
    solver.solve().expect("solve");

    let b = solver.get_point("b".into()).unwrap();
    assert!(
        (b.x - 10.0).abs() < 1e-5,
        "b.x should be 10, got {}",
        b.x
    );
}
