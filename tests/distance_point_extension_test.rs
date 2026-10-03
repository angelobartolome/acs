use acs::{ConstraintSolver, ConstraintType, Point, SolverResult};

fn solve_distance(d: f64, start: (f64, f64)) -> Point {
    let mut solver = ConstraintSolver::new();
    solver.add_point(Point::new("a".into(), 0.0, 0.0, true));
    solver.add_point(Point::new("b".into(), 1.0, 0.0, true));
    solver.add_point(Point::new("p".into(), start.0, start.1, false));
    solver
        .add_constraint(ConstraintType::DistancePointExtension(
            "p".into(),
            "a".into(),
            "b".into(),
            d,
        ))
        .unwrap();

    let result = solver.solve().unwrap();
    assert!(matches!(result, SolverResult::Converged { .. }), "{result:?}");
    solver.get_point("p".into()).unwrap().clone()
}

/// The distance is perpendicular to the Extension, even beyond the segment.
#[test]
fn distance_to_extension_beyond_the_segment() {
    let p = solve_distance(2.0, (5.0, 3.0));
    assert!((p.y - 2.0).abs() < 1e-8, "p should be 2 from the Extension, got y = {}", p.y);
    assert!((p.x - 5.0).abs() < 1e-6, "p should keep its x, got {}", p.x);
}

/// The side the point starts on is kept.
#[test]
fn distance_to_extension_keeps_the_side() {
    let p = solve_distance(2.0, (-4.0, -1.0));
    assert!((p.y + 2.0).abs() < 1e-8, "p should be 2 below the Extension, got y = {}", p.y);
    assert!(p.x < -3.0, "p should stay before the start, got x = {}", p.x);
}

/// Distance 0 puts the point on the Extension.
#[test]
fn zero_distance_to_extension_is_point_on_extension() {
    let p = solve_distance(0.0, (5.0, 3.0));
    assert!(p.y.abs() < 1e-8, "p should lie on the Extension, got y = {}", p.y);
    assert!((p.x - 5.0).abs() < 1e-6, "p should keep its x, got {}", p.x);
}
