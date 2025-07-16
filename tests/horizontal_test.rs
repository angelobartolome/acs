use acs::{ConstraintSolver, ConstraintType, Point, SolverResult};

#[test]
fn test_horizontal_constraint() {
    let mut solver = ConstraintSolver::new();

    let p1 = solver.add_point(Point::new(0.0, 0.0));
    let p2 = solver.add_point(Point::new(1.0, 1.0));
    let line = solver.add_line(p1, p2).unwrap();

    solver
        .add_constraint(ConstraintType::Horizontal(line))
        .unwrap();
    let result = solver.solve().unwrap();

    match result {
        SolverResult::Converged { final_error, .. } => {
            assert!(final_error < 1e-6);
        }
        _ => panic!("Solver should have converged"),
    }

    let start = solver.get_point(p1).unwrap();
    let end = solver.get_point(p2).unwrap();
    solver.print_state();

    assert!(
        (start.y - end.y).abs() < 1e-4,
        "Points should be horizontally aligned"
    );
}
