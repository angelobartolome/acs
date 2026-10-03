use acs::{Circle, ConstraintSolver, ConstraintType, Point, SolverResult};

/// The circle beside the segment's far end becomes tangent to the Extension
/// where it is; the tangency point is not pulled back over the segment.
#[test]
fn tangent_to_extension_beyond_the_segment() {
    let mut solver = ConstraintSolver::new();
    solver.add_point(Point::new("la".into(), 0.0, 0.0, true));
    solver.add_point(Point::new("lb".into(), 10.0, 0.0, true));
    solver.add_point(Point::new("cc".into(), 15.0, 3.0, false));
    let c = solver.add_circle(Circle::new("c".into(), "cc".into(), 2.0, true));
    solver
        .add_constraint(ConstraintType::TangentExtensionCircle(
            "la".into(),
            "lb".into(),
            "cc".into(),
            c,
        ))
        .unwrap();

    let result = solver.solve().unwrap();
    assert!(matches!(result, SolverResult::Converged { .. }), "{result:?}");

    let cc = solver.get_point("cc".into()).unwrap();
    assert!((cc.y - 2.0).abs() < 1e-8, "center should be 2 from the Extension, got y = {}", cc.y);
    assert!((cc.x - 15.0).abs() < 1e-6, "center should keep its x, got {}", cc.x);
}

/// With a free radius and fixed center, the radius grows to reach the
/// Extension.
#[test]
fn tangent_to_extension_solves_for_radius() {
    let mut solver = ConstraintSolver::new();
    solver.add_point(Point::new("la".into(), 0.0, 0.0, true));
    solver.add_point(Point::new("lb".into(), 1.0, 1.0, true));
    solver.add_point(Point::new("cc".into(), 10.0, 6.0, true));
    let c = solver.add_circle(Circle::new("c".into(), "cc".into(), 1.0, false));
    solver
        .add_constraint(ConstraintType::TangentExtensionCircle(
            "la".into(),
            "lb".into(),
            "cc".into(),
            c.clone(),
        ))
        .unwrap();

    let result = solver.solve().unwrap();
    assert!(matches!(result, SolverResult::Converged { .. }), "{result:?}");

    // Distance from (10, 6) to y = x is 4 / √2.
    let r = solver.get_circle(c).unwrap().radius;
    assert!((r - 4.0 / 2f64.sqrt()).abs() < 1e-8, "radius {r}");
}
