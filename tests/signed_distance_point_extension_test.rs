use acs::{ConstraintSolver, ConstraintType, Point, SolverResult};

/// Solves a free point `p` against the fixed line (0,0) → (10,0).
fn solve_signed(d: f64, side: f64, start: (f64, f64)) -> Point {
    let mut solver = ConstraintSolver::new();
    solver.add_point(Point::new("a".into(), 0.0, 0.0, true));
    solver.add_point(Point::new("b".into(), 10.0, 0.0, true));
    solver.add_point(Point::new("p".into(), start.0, start.1, false));
    solver
        .add_constraint(ConstraintType::SignedDistancePointExtension(
            "p".into(),
            "a".into(),
            "b".into(),
            d,
            side,
        ))
        .unwrap();

    let result = solver.solve().unwrap();
    assert!(
        matches!(result, SolverResult::Converged { .. }),
        "{result:?}"
    );
    solver.get_point("p".into()).unwrap().clone()
}

/// Side +1 is left of a → b (above this line), even starting below it.
#[test]
fn positive_side_is_left_of_the_line() {
    let p = solve_signed(2.0, 1.0, (5.0, -1.0));
    assert!(
        (p.y - 2.0).abs() < 1e-8,
        "p should be 2 above, got y = {}",
        p.y
    );
}

/// Side −1 is right of a → b (below this line), even starting above it.
#[test]
fn negative_side_is_right_of_the_line() {
    let p = solve_signed(2.0, -1.0, (5.0, 3.0));
    assert!(
        (p.y + 2.0).abs() < 1e-8,
        "p should be 2 below, got y = {}",
        p.y
    );
}

/// Measured against the Extension: a point beyond the segment stays there.
#[test]
fn holds_beyond_the_segment() {
    let p = solve_signed(1.5, 1.0, (-6.0, 0.5));
    assert!((p.y - 1.5).abs() < 1e-8, "got y = {}", p.y);
    assert!(
        p.x < -5.0,
        "p should stay before the start, got x = {}",
        p.x
    );
}
