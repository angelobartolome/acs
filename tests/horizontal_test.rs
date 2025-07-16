use acs::{ConstraintSolver, ConstraintType, Line, Point, SolverResult};

#[test]
fn test_horizontal_constraint() {
    let mut solver = ConstraintSolver::new();
    let p1 = Point::new(1, 0.0, 0.0);
    let p2 = Point::new(2, 1.0, 1.0);

    solver.add_point(p1);
    solver.add_point(p2);

    let line = Line::new(1, p1.id, p2.id);
    solver.add_line(line);

    solver
        .add_constraint(ConstraintType::Horizontal(line.id))
        .unwrap();
    let result = solver.solve().unwrap();

    match result {
        SolverResult::Converged { final_error, .. } => {
            assert!(final_error < 1e-6);
        }
        _ => panic!("Solver should have converged"),
    }

    let start = solver.get_point(p1.id).unwrap();
    let end = solver.get_point(p2.id).unwrap();
    solver.print_state();

    assert!(
        (start.y - end.y).abs() < 1e-4,
        "Points should be horizontally aligned"
    );
}
