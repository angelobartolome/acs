use acs::{ConstraintSolver, ConstraintType, Point, SolverResult};

fn main() {
    let mut solver = ConstraintSolver::new();

    let p1 = solver.add_point(Point::new(0.0, 0.0));
    let p2 = solver.add_point(Point::new(1.0, 1.0));
    let line = solver.add_line(p1, p2).unwrap();

    println!("Initial state:");
    solver.print_state();

    solver
        .add_constraint(ConstraintType::Vertical(line))
        .unwrap();

    println!(
        "Initial constraint error: {}",
        solver.get_constraint_error()
    );

    let result = solver.solve().unwrap();

    println!("Result: {:?}", result);
    println!("Final state:");
    solver.print_state();

    let start = solver.get_point(p1).unwrap();
    let end = solver.get_point(p2).unwrap();

    println!("Start point: ({}, {})", start.x, start.y);
    println!("End point: ({}, {})", end.x, end.y);
    println!("X difference: {}", (start.x - end.x).abs());
}
