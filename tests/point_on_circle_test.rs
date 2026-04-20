use acs::{Circle, ConstraintSolver, ConstraintType, Point, SolverResult};

#[test]
fn test_point_on_circle_constraint() {
    let mut solver = ConstraintSolver::new();

    // Circle centered at origin with radius 5 (fixed)
    solver.add_point(Point::new("center".into(), 0.0, 0.0, true));
    let c_id = solver.add_circle(Circle::new("c".into(), "center".into(), 5.0, true));

    // Point starts off the circle, should move onto it
    solver.add_point(Point::new("p".into(), 3.0, 3.0, false));

    solver
        .add_constraint(ConstraintType::PointOnCircle(
            "p".into(),
            "center".into(),
            c_id,
        ))
        .expect("add point-on-circle");

    let result = solver.solve().expect("solve");
    match result {
        SolverResult::Converged { final_error, .. } => assert!(final_error < 1e-6),
        _ => panic!("did not converge"),
    }

    let p = solver.get_point("p".into()).unwrap();
    let dist = (p.x * p.x + p.y * p.y).sqrt();
    assert!(
        (dist - 5.0).abs() < 1e-4,
        "point should be on circle (r=5), dist = {}",
        dist
    );
}
