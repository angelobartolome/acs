use acs::{constraints::base::ConstraintType, geometry::Point, solver::ConstraintSolver};

fn main() {
    let mut solver = ConstraintSolver::new();

    // Add some points
    let p1 = solver.add_point(Point::new(0.0, 0.0));
    let p2 = solver.add_point(Point::new(1.0, 1.0));
    let p3 = solver.add_point(Point::new(2.0, 0.5));

    // Add lines
    let line1 = solver.add_line(p1, p2).unwrap();
    let line2 = solver.add_line(p2, p3).unwrap();

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
