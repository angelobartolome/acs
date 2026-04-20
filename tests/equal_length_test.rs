use acs::{ConstraintSolver, ConstraintType, Point, SolverResult};

#[test]
fn test_equal_length_constraint() {
    let mut solver = ConstraintSolver::new();

    // Segment 1: from (0,0) to (3,4) — length 5 (fixed)
    solver.add_point(Point::new("p1".into(), 0.0, 0.0, true));
    solver.add_point(Point::new("p2".into(), 3.0, 4.0, true));

    // Segment 2: starts at (5,5) with length 1, p4 free to grow to length 5
    solver.add_point(Point::new("p3".into(), 5.0, 5.0, true));
    solver.add_point(Point::new("p4".into(), 6.0, 5.0, false)); // will move

    solver
        .add_constraint(ConstraintType::EqualLength(
            "p1".into(),
            "p2".into(),
            "p3".into(),
            "p4".into(),
        ))
        .expect("add equal-length");

    solver.set_max_iterations(500);
    solver.solve().expect("solve");

    let p3 = solver.get_point("p3".into()).unwrap();
    let p4 = solver.get_point("p4".into()).unwrap();
    let len2 = ((p4.x - p3.x).powi(2) + (p4.y - p3.y).powi(2)).sqrt();
    assert!(
        (len2 - 5.0).abs() < 1e-4,
        "segment 2 length should be 5, got {}",
        len2
    );
}

#[test]
fn test_equal_length_already_equal() {
    let mut solver = ConstraintSolver::new();

    // Both segments already length 5
    solver.add_point(Point::new("a1".into(), 0.0, 0.0, true));
    solver.add_point(Point::new("a2".into(), 5.0, 0.0, true));
    solver.add_point(Point::new("b1".into(), 10.0, 0.0, true));
    solver.add_point(Point::new("b2".into(), 15.0, 0.0, true));

    solver
        .add_constraint(ConstraintType::EqualLength(
            "a1".into(),
            "a2".into(),
            "b1".into(),
            "b2".into(),
        ))
        .expect("add equal-length");

    let result = solver.solve().expect("solve");
    match result {
        SolverResult::Converged { final_error, .. } => assert!(
            final_error < 1e-6,
            "already equal — should converge immediately"
        ),
        _ => panic!("did not converge"),
    }
}
