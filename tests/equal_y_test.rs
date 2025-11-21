use acs::{ConstraintSolver, ConstraintType, Point, SolverResult};

#[test]
fn test_equal_y_constraint() {
    let mut solver = ConstraintSolver::new();
    let p1 = Point::new(String::from("p1"), 7.0, 12.0, false);

    solver.add_point(p1);

    solver
        .add_constraint(ConstraintType::EqualY(String::from("p1"), 5.0))
        .expect("Constraint should be added successfully");

    let result = solver.solve().expect("Solver should solve successfully");

    match result {
        SolverResult::Converged { final_error, .. } => {
            assert!(final_error < 1e-6);
        }
        final_result => panic!("Solver should have converged, got: {:?}", final_result),
    }

    let start = solver.get_point(String::from("p1")).expect("Point p1 should exist");
    solver.print_state();

    assert!(
        (start.y - 5.0).abs() < 1e-6,
        "Point should have y-coordinate equal to 5.0"
    );
}
