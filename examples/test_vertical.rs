use acs::{ConstraintSolver, ConstraintType, Point};

fn main() {
    println!("Testing vertical constraint with dogleg solver...");

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
    let x_diff = (start.x - end.x).abs();
    println!("X difference: {}", x_diff);

    if x_diff < 1e-6 {
        println!("✓ Test PASSED: Points are vertically aligned");
    } else {
        println!(
            "✗ Test FAILED: Points are not vertically aligned (diff: {})",
            x_diff
        );
    }
}
