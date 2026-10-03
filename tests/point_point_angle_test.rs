use acs::{ConstraintSolver, ConstraintType, Point, SolverResult};

fn solve_direction(angle: f64, start: (f64, f64)) -> (f64, f64) {
    let mut solver = ConstraintSolver::new();
    solver.add_point(Point::new("c".into(), 0.0, 0.0, true));
    solver.add_point(Point::new("p".into(), start.0, start.1, false));
    solver
        .add_constraint(ConstraintType::PointPointAngle(
            "c".into(),
            "p".into(),
            angle,
        ))
        .unwrap();
    solver
        .add_constraint(ConstraintType::DistancePointPoint(
            "c".into(),
            "p".into(),
            2.0,
        ))
        .unwrap();
    let result = solver.solve().unwrap();
    assert!(
        matches!(result, SolverResult::Converged { .. }),
        "{result:?}"
    );
    let p = solver.get_point("p".into()).unwrap();
    (p.x, p.y)
}

#[test]
fn direction_is_measured_ccw_from_the_x_axis() {
    let (x, y) = solve_direction(1.0, (2.0, 0.0));
    assert!((x - 2.0 * 1f64.cos()).abs() < 1e-8);
    assert!((y - 2.0 * 1f64.sin()).abs() < 1e-8);
}

/// Starting almost opposite the target, the point still ends up pointing
/// along θ, not θ + π (the residual is an angle, not a sin/cos product).
#[test]
fn no_spurious_solution_at_the_opposite_direction() {
    let (x, y) = solve_direction(0.0, (-2.0, 0.3));
    assert!((x - 2.0).abs() < 1e-8 && y.abs() < 1e-8, "({x}, {y})");
}
