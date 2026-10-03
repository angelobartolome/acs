use acs::{ConstraintSolver, ConstraintType, Point, SolverResult};

/// Axis from (0, 0) to (2, 0), both fixed; a fixed source `a` and a free
/// image `b` starting at the origin.
fn mirror(a: (f64, f64)) -> (f64, f64) {
    let mut solver = ConstraintSolver::new();
    solver.add_point(Point::new("m1".into(), 0.0, 0.0, true));
    solver.add_point(Point::new("m2".into(), 2.0, 0.0, true));
    solver.add_point(Point::new("a".into(), a.0, a.1, true));
    solver.add_point(Point::new("b".into(), 0.0, 0.0, false));
    solver
        .add_constraint(ConstraintType::MirrorPointExtension(
            "a".into(),
            "b".into(),
            "m1".into(),
            "m2".into(),
        ))
        .unwrap();
    let result = solver.solve().unwrap();
    assert!(
        matches!(result, SolverResult::Converged { .. }),
        "{result:?}"
    );
    let b = solver.get_point("b".into()).unwrap();
    (b.x, b.y)
}

#[test]
fn mirrors_across_the_axis() {
    let (x, y) = mirror((1.0, 3.0));
    assert!(
        (x - 1.0).abs() < 1e-8 && (y + 3.0).abs() < 1e-8,
        "({x}, {y})"
    );
}

/// The axis is the Line's Extension: a point beside the segment's span is
/// reflected across the infinite line, not about the nearest endpoint.
#[test]
fn mirrors_beyond_the_axis_segment() {
    let (x, y) = mirror((7.0, 1.5));
    assert!(
        (x - 7.0).abs() < 1e-8 && (y + 1.5).abs() < 1e-8,
        "({x}, {y})"
    );
}

#[test]
fn a_point_on_the_axis_is_its_own_image() {
    let (x, y) = mirror((5.0, 0.0));
    assert!((x - 5.0).abs() < 1e-8 && y.abs() < 1e-8, "({x}, {y})");
}
