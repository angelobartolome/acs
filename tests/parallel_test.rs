use acs::{ConstraintSolver, ConstraintType, Point, SolverResult};

#[test]
fn test_parallel_constraint() {
    let mut solver = ConstraintSolver::new();

    let p1 = Point::new(String::from("p1"), 0.0, 0.0, false);
    let p2 = Point::new(String::from("p2"), 1.0, 1.0, false);
    solver.add_point(p1);
    solver.add_point(p2);

    let p3 = Point::new(String::from("p3"), 0.0, 1.0, false);
    let p4 = Point::new(String::from("p4"), 1.0, 5.0, false);
    solver.add_point(p3);
    solver.add_point(p4);

    solver
        .add_constraint(ConstraintType::Parallel(
            String::from("p1"),
            String::from("p2"),
            String::from("p3"),
            String::from("p4"),
        ))
        .expect("Constraint should be added successfully");
    let result = solver.solve().expect("Solver should solve successfully");

    match result {
        SolverResult::Converged { final_error, .. } => {
            assert!(final_error < 1e-6);
        }
        _ => panic!("Solver should have converged"),
    }

    let start_a = solver.get_point(String::from("p1")).expect("Point p1 should exist");
    let end_a = solver.get_point(String::from("p2")).expect("Point p2 should exist");

    let start_b = solver.get_point(String::from("p3")).expect("Point p3 should exist");
    let end_b = solver.get_point(String::from("p4")).expect("Point p4 should exist");

    solver.print_state();

    let get_angle = |start: &Point, end: &Point| {
        let dy = end.y - start.y;
        let dx = end.x - start.x;
        dy.atan2(dx)
    };

    assert!(
        (get_angle(start_a, end_a) - get_angle(start_b, end_b)).abs() < 1e-4,
        "Lines should be parallel"
    );
}
