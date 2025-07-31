use acs::{ConstraintSolver, ConstraintType, Point, SolverResult};

#[test]
fn test_vertical_constraint() {
    let mut solver = ConstraintSolver::new();
    let p1 = Point::new(String::from("p1"), 0.0, 0.0, false);
    let p2 = Point::new(String::from("p2"), 1.0, 1.0, false);

    solver.add_point(p1);
    solver.add_point(p2);

    solver
        .add_constraint(ConstraintType::Vertical(
            String::from("p1"),
            String::from("p2"),
        ))
        .unwrap();
    let result = solver.solve().unwrap();

    match result {
        SolverResult::Converged { final_error, .. } => {
            assert!(final_error < 1e-6);
        }
        _ => panic!("Solver should have converged"),
    }

    let start = solver.get_point(String::from("p1")).unwrap();
    let end = solver.get_point(String::from("p2")).unwrap();
    solver.print_state();

    assert!(
        (start.x - end.x).abs() < 1e-6,
        "Points should be vertically aligned"
    );
}
