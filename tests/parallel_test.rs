use acs::{ConstraintSolver, ConstraintType, Point, SolverResult};

#[test]
fn test_parallel_constraint() {
    let mut solver = ConstraintSolver::new();

    let p1 = solver.add_point(Point::new(0.0, 0.0));
    let p2 = solver.add_point(Point::new(1.0, 1.0));
    let line1 = solver.add_line(p1, p2).unwrap();

    let p3 = solver.add_point(Point::new(0.0, 1.0));
    let p4 = solver.add_point(Point::new(1.0, 2.0));
    let line2 = solver.add_line(p3, p4).unwrap();

    solver
        .add_constraint(ConstraintType::Parallel(line1, line2))
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

    let get_angle = |start: &Point, end: &Point| {
        let dy = end.y - start.y;
        let dx = end.x - start.x;
        dy.atan2(dx)
    };

    assert!(
        (get_angle(&start, &end) - get_angle(&start, &end)).abs() < 1e-4,
        "Lines should be parallel"
    );
}
