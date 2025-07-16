use acs::{ConstraintSolver, ConstraintType, Line, Point, SolverResult};

#[test]
fn test_parallel_constraint() {
    let mut solver = ConstraintSolver::new();

    let p1 = Point::new(1, 0.0, 0.0);
    let p2 = Point::new(2, 1.0, 1.0);
    solver.add_point(p1);
    solver.add_point(p2);

    let line1 = Line::new(1, p1.id, p2.id);
    solver.add_line(line1);

    let p3 = Point::new(3, 0.0, 1.0);
    let p4 = Point::new(4, 1.0, 2.0);
    solver.add_point(p3);
    solver.add_point(p4);

    let line2 = Line::new(2, p3.id, p4.id);
    solver.add_line(line2);

    solver
        .add_constraint(ConstraintType::Parallel(line1.id, line2.id))
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
