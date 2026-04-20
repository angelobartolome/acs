use acs::{Circle, ConstraintSolver, ConstraintType, Point, SolverResult};

#[test]
fn test_fixed_radius_constraint() {
    let mut solver = ConstraintSolver::new();

    solver.add_point(Point::new("center".into(), 0.0, 0.0, true));
    let c_id = solver.add_circle(Circle::new("c".into(), "center".into(), 5.0, false));

    solver
        .add_constraint(ConstraintType::FixedRadius(c_id.clone(), 3.0))
        .expect("add fixed radius");

    let result = solver.solve().expect("solve");
    match result {
        SolverResult::Converged { final_error, .. } => assert!(final_error < 1e-6),
        _ => panic!("did not converge"),
    }

    let c = solver.get_circle(c_id).unwrap();
    assert!(
        (c.radius - 3.0).abs() < 1e-5,
        "radius should be 3.0, got {}",
        c.radius
    );
}
