use acs::{ConstraintSolver, ConstraintType, Point, SolverResult};

#[test]
fn test_equal_x_constraint() {
    let mut solver = ConstraintSolver::new();
    let p1 = Point::new(String::from("p1"), 12.0, 7.0, false);

    solver.add_point(p1);

    solver
        .add_constraint(ConstraintType::EqualX(String::from("p1"), 5.0))
        .expect("Constraint should be added successfully");

    let result = solver.solve().expect("Solver should solve successfully");

    match result {
        SolverResult::Converged { final_error, .. } => {
            assert!(final_error < 1e-6);
        }
        final_result => panic!("Solver should have converged, got: {final_result:?}"),
    }

    let start = solver.get_point(String::from("p1")).expect("Point p1 should exist");
    solver.print_state();

    assert!(
        (start.x - 5.0).abs() < 1e-6,
        "Point should have x-coordinate equal to 5.0"
    );
}
