use acs::{ConstraintSolver, ConstraintType, Point, SolverResult};

#[test]
fn test_distance_point_point_constraint() {
    let mut solver = ConstraintSolver::new();

    // Fix p1 at origin
    solver.add_point(Point::new("p1".into(), 0.0, 0.0, true));
    // p2 starts at (1,0), should end up 5 units away
    solver.add_point(Point::new("p2".into(), 1.0, 0.0, false));

    solver
        .add_constraint(ConstraintType::DistancePointPoint(
            "p1".into(),
            "p2".into(),
            5.0,
        ))
        .expect("add distance-point-point");

    let result = solver.solve().expect("solve");
    match result {
        SolverResult::Converged { final_error, .. } => assert!(final_error < 1e-6),
        _ => panic!("did not converge"),
    }

    let p2 = solver.get_point("p2".into()).unwrap();
    let dist = (p2.x * p2.x + p2.y * p2.y).sqrt();
    assert!(
        (dist - 5.0).abs() < 1e-4,
        "distance should be 5.0, got {}",
        dist
    );
}

#[test]
fn test_distance_already_satisfied() {
    let mut solver = ConstraintSolver::new();

    solver.add_point(Point::new("a".into(), 0.0, 0.0, true));
    solver.add_point(Point::new("b".into(), 3.0, 4.0, true)); // dist = 5

    solver
        .add_constraint(ConstraintType::DistancePointPoint("a".into(), "b".into(), 5.0))
        .expect("add constraint");

    let result = solver.solve().expect("solve");
    match result {
        SolverResult::Converged { final_error, .. } => assert!(
            final_error < 1e-6,
            "already satisfied — error should be near zero"
        ),
        _ => panic!("did not converge"),
    }
}
