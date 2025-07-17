use acs::{ConstraintSolver, ConstraintType, Line, Point, SolverResult};

#[test]
fn test_vertical_constraint() {
    let mut solver = ConstraintSolver::new();
    let p1 = Point::new(1, 0.0, 0.0, false);
    let p2 = Point::new(2, 1.0, 1.0, false);

    solver.add_point(p1);
    solver.add_point(p2);

    let line = Line::new(1, p1.id, p2.id);
    solver.add_line(line);

    solver
        .add_constraint(ConstraintType::Vertical(line.id))
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
        (start.x - end.x).abs() < 1e-6,
        "Points should be vertically aligned"
    );
}
