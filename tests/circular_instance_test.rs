use acs::{ConstraintSolver, ConstraintType, Point, SolverResult};

fn instance(angle: f64) -> (f64, f64) {
    let mut solver = ConstraintSolver::new();
    solver.add_point(Point::new("o".into(), 1.0, 1.0, true));
    solver.add_point(Point::new("p0".into(), 3.0, 1.0, true));
    solver.add_point(Point::new("pk".into(), 0.0, 0.0, false));
    solver
        .add_constraint(ConstraintType::CircularInstance(
            "p0".into(),
            "pk".into(),
            "o".into(),
            angle,
        ))
        .unwrap();
    let result = solver.solve().unwrap();
    assert!(
        matches!(result, SolverResult::Converged { .. }),
        "{result:?}"
    );
    let p = solver.get_point("pk".into()).unwrap();
    (p.x, p.y)
}

#[test]
fn rotates_the_source_ccw_about_the_center() {
    let (x, y) = instance(std::f64::consts::FRAC_PI_2);
    assert!(
        (x - 1.0).abs() < 1e-8 && (y - 3.0).abs() < 1e-8,
        "({x}, {y})"
    );
    let (x, y) = instance(std::f64::consts::PI);
    assert!(
        (x + 1.0).abs() < 1e-8 && (y - 1.0).abs() < 1e-8,
        "({x}, {y})"
    );
}

#[test]
fn zero_angle_is_a_coincident_copy() {
    let (x, y) = instance(0.0);
    assert!(
        (x - 3.0).abs() < 1e-8 && (y - 1.0).abs() < 1e-8,
        "({x}, {y})"
    );
}
