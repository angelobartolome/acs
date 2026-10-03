use acs::{ConstraintSolver, ConstraintType, Point, SolverResult};

/// Segment from (0,0) to (1,0); the point starts beyond its end. It lands on
/// the Extension (y = 0) without being pulled back over the segment.
#[test]
fn point_on_extension_stays_beyond_the_segment() {
    let mut solver = ConstraintSolver::new();
    solver.add_point(Point::new("a".into(), 0.0, 0.0, true));
    solver.add_point(Point::new("b".into(), 1.0, 0.0, true));
    solver.add_point(Point::new("p".into(), 5.0, 2.0, false));
    solver
        .add_constraint(ConstraintType::PointOnExtension(
            "p".into(),
            "a".into(),
            "b".into(),
        ))
        .unwrap();

    let result = solver.solve().unwrap();
    assert!(matches!(result, SolverResult::Converged { .. }), "{result:?}");

    let p = solver.get_point("p".into()).unwrap();
    assert!(p.y.abs() < 1e-8, "p should lie on the Extension, got y = {}", p.y);
    assert!((p.x - 5.0).abs() < 1e-6, "p should keep its x, got {}", p.x);
}

/// Before the start of the segment works the same way.
#[test]
fn point_on_extension_before_the_start() {
    let mut solver = ConstraintSolver::new();
    solver.add_point(Point::new("a".into(), 0.0, 0.0, true));
    solver.add_point(Point::new("b".into(), 0.0, 1.0, true));
    solver.add_point(Point::new("p".into(), 0.5, -4.0, false));
    solver
        .add_constraint(ConstraintType::PointOnExtension(
            "p".into(),
            "a".into(),
            "b".into(),
        ))
        .unwrap();

    let result = solver.solve().unwrap();
    assert!(matches!(result, SolverResult::Converged { .. }), "{result:?}");

    let p = solver.get_point("p".into()).unwrap();
    assert!(p.x.abs() < 1e-8, "p should lie on the Extension, got x = {}", p.x);
    assert!(p.y < -3.0, "p should stay before the start, got y = {}", p.y);
}
