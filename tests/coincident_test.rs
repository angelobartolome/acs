use acs::{ConstraintSolver, ConstraintType, Point, SolverResult};

#[test]
fn test_coincident_constraint() {
    let mut solver = ConstraintSolver::new();
    let p1 = Point::new(String::from("p1"), 0.0, 0.0, false);
    let p2 = Point::new(String::from("p2"), 3.0, 5.0, false);

    solver.add_point(p1);
    solver.add_point(p2);

    solver
        .add_constraint(ConstraintType::Coincident(
            String::from("p1"),
            String::from("p2"),
        ))
        .unwrap();

    let result = solver.solve().unwrap();

    match result {
        SolverResult::Converged { final_error, .. } => {
            assert!(final_error < 1e-6);
        }
        final_result => panic!("Solver should have converged, got: {:?}", final_result),
    }

    let point_a = solver.get_point(String::from("p1")).unwrap();
    let point_b = solver.get_point(String::from("p2")).unwrap();
    solver.print_state();

    assert!(
        (point_a.x - point_b.x).abs() < 1e-6 && (point_a.y - point_b.y).abs() < 1e-4,
        "Points should be coincident"
    );
}
