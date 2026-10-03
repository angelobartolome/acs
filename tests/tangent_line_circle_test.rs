use acs::{Circle, ConstraintSolver, ConstraintType, Point};

/// A horizontal line y=0 should be tangent to a circle with center on y-axis.
/// The centre must end up at distance r from the line.
#[test]
fn test_tangent_line_circle() {
    let mut solver = ConstraintSolver::new();

    // Horizontal line along x-axis (fixed)
    solver.add_point(Point::new("la".into(), 0.0, 0.0, true));
    solver.add_point(Point::new("lb".into(), 10.0, 0.0, true));

    // Circle: center starts at (5, 3), radius 2 (fixed) — needs to move so dist(center, line) = 2
    solver.add_point(Point::new("cc".into(), 5.0, 3.0, false));
    let c = solver.add_circle(Circle::new("c".into(), "cc".into(), 2.0, true));

    solver
        .add_constraint(ConstraintType::TangentLineCircle(
            "la".into(),
            "lb".into(),
            "cc".into(),
            c,
        ))
        .expect("add tangent-line-circle");

    solver.set_max_iterations(500);
    solver.solve().expect("solve");

    // The perpendicular distance from center to the horizontal line (y=0) must equal r=2
    let cc = solver.get_point("cc".into()).unwrap();
    assert!(
        (cc.y.abs() - 2.0).abs() < 5e-2,
        "dist(center, line) should be 2, got |y|={}",
        cc.y.abs()
    );
}

/// The line is a segment: the tangency point must lie between its endpoints.
/// A circle starting beside the segment's far end is pulled back over it.
#[test]
fn test_tangent_point_stays_on_segment() {
    use acs::SolverResult;

    let mut solver = ConstraintSolver::new();
    solver.add_point(Point::new("la".into(), 0.0, 0.0, true));
    solver.add_point(Point::new("lb".into(), 10.0, 0.0, true));
    solver.add_point(Point::new("cc".into(), 15.0, 3.0, false));
    let c = solver.add_circle(Circle::new("c".into(), "cc".into(), 2.0, true));
    solver
        .add_constraint(ConstraintType::TangentLineCircle(
            "la".into(),
            "lb".into(),
            "cc".into(),
            c,
        ))
        .unwrap();

    let result = solver.solve().unwrap();
    assert!(matches!(result, SolverResult::Converged { .. }), "{result:?}");

    let cc = solver.get_point("cc".into()).unwrap();
    assert!((cc.y.abs() - 2.0).abs() < 1e-8, "|y| should be 2, got {}", cc.y);
    assert!(
        (0.0..=10.0 + 1e-8).contains(&cc.x),
        "tangency point (x = {}) should be within the segment",
        cc.x
    );
}
