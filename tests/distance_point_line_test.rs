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
