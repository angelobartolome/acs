use acs::{ConstraintSolver, ConstraintType, Point, SolverResult};

#[test]
fn test_coincident_constraint() {
    let mut solver = ConstraintSolver::new();
    let p1 = Point::new(1, 0.0, 0.0, true);
    let p2 = Point::new(2, 3.0, 5.0, false);

    solver.add_point(p1);
    solver.add_point(p2);

    solver
        .add_constraint(ConstraintType::Coincident(p1.id, p2.id))
        .unwrap();

    let result = solver.solve().unwrap();

    match result {
        SolverResult::Converged { final_error, .. } => {
            assert!(final_error < 1e-6);
        }
        final_result => panic!("Solver should have converged, got: {:?}", final_result),
    }

    let start = solver.get_point(p1.id).unwrap();
    let end = solver.get_point(p2.id).unwrap();
    solver.print_state();

    assert!(
        (start.x - end.x).abs() < 1e-6 && (start.y - end.y).abs() < 1e-4,
        "Points should be coincident"
    );

    assert!(solver.is_point_fixed(p1.id), "Point 1 should be fixed");
    assert!(!solver.is_point_fixed(p2.id), "Point 2 should not be fixed");
}
