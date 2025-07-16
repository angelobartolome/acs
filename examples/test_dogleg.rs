use acs::{Line, constraints::base::ConstraintType, geometry::Point, solver::ConstraintSolver};

fn main() {
    let mut solver = ConstraintSolver::new();

    // Add some points
    let p1 = solver.add_point(Point::new(1, 0.0, 0.0));
    let p2 = solver.add_point(Point::new(2, 1.0, 1.0));
    let p3 = solver.add_point(Point::new(3, 2.0, 0.5));

    // Add lines
    let line1 = solver.add_line(Line::new(1, p1, p2));
    let line2 = solver.add_line(Line::new(2, p2, p3));

    // Add constraints
    solver
        .add_constraint(ConstraintType::Vertical(line1))
        .unwrap();
    solver
        .add_constraint(ConstraintType::Horizontal(line2))
        .unwrap();

    println!("Initial state:");
    solver.print_state();

    // Solve
    match solver.solve() {
        Ok(result) => {
            println!("\nSolution: {:?}", result);
            println!("\nFinal state:");
            solver.print_state();
        }
        Err(e) => println!("Error: {}", e),
    }
}
