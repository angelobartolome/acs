use acs::{ConstraintSolver, ConstraintType, Point};

#[test]
fn test_distance_point_line_constraint() {
    let mut solver = ConstraintSolver::new();

    // Horizontal line along x-axis (fixed)
    solver.add_point(Point::new("la".into(), 0.0, 0.0, true));
    solver.add_point(Point::new("lb".into(), 10.0, 0.0, true));

    // Point starts at (5, 1), should end up exactly 3 units from the line
    solver.add_point(Point::new("p".into(), 5.0, 1.0, false));

    solver
        .add_constraint(ConstraintType::DistancePointLine(
            "p".into(),
            "la".into(),
            "lb".into(),
            3.0,
        ))
        .expect("add distance-point-line");

    solver.set_max_iterations(500);
    solver.solve().expect("solve");

    let p = solver.get_point("p".into()).unwrap();
    // Line is y=0, so distance = |py|
    assert!(
        (p.y.abs() - 3.0).abs() < 1e-4,
        "distance to line should be 3, got |y|={}",
        p.y.abs()
    );
}

/// The line is a segment: past its end, the distance is measured to the
/// nearest endpoint, not to the line's extension.
#[test]
fn test_distance_past_segment_end_is_to_endpoint() {
    use acs::SolverResult;

    let mut solver = ConstraintSolver::new();
    solver.add_point(Point::new("a".into(), 0.0, 0.0, true));
    solver.add_point(Point::new("b".into(), 1.0, 0.0, true));
    solver.add_point(Point::new("p".into(), 5.0, 0.5, false));
    solver
        .add_constraint(ConstraintType::DistancePointLine(
            "p".into(),
            "a".into(),
            "b".into(),
            1.0,
        ))
        .unwrap();

    let result = solver.solve().unwrap();
    assert!(matches!(result, SolverResult::Converged { .. }), "{result:?}");

    let p = solver.get_point("p".into()).unwrap();
    // Distance from p to segment [(0,0), (1,0)].
    let t = p.x.clamp(0.0, 1.0);
    let dist = ((p.x - t).powi(2) + p.y.powi(2)).sqrt();
    assert!((dist - 1.0).abs() < 1e-8, "distance to segment should be 1, got {dist}");
}
