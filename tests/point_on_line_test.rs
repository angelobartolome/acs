use acs::{ConstraintSolver, ConstraintType, Point, SolverResult};

#[test]
fn test_point_on_line_constraint() {
    let mut solver = ConstraintSolver::new();

    // Create a vertical line from (2,0) to (2,4)
    let p1 = Point::new("1".into(), 0.0, 0.0, true); // Fixed start point
    let p2 = Point::new("2".into(), 0.0, 4.0, true); // Fixed end point

    let p3 = Point::new("3".into(), 1.0, 1.0, false); // Point to be constrained

    solver.add_point(p1);
    solver.add_point(p2);
    solver.add_point(p3);

    solver
        .add_constraint(ConstraintType::PointOnLine(
            "3".into(),
            "1".into(),
            "2".into(),
        ))
        .expect("Constraint should be added successfully");

    let result = solver.solve().expect("Solver should solve successfully");

    match result {
        SolverResult::Converged { final_error, .. } => {
            assert!(final_error < 1e-3);
        }
        final_result => panic!("Solver should have converged, got: {final_result:?}"),
    }

    let constrained_point = solver.get_point("3".into()).expect("Point 3 should exist");

    // The point should have x-coordinate = 0
    let x = constrained_point.x;
    assert!(
        x.abs() < 0.001,
        "Expected point to be on the vertical line at x = 0, got x = {x}"
    );
}
