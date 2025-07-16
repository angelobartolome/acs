use acs::{ConstraintSolver, ConstraintType, Line, Point, SolverResult};

fn main() {
    // Create a new constraint solver
    let mut solver = ConstraintSolver::new();

    // Add points
    let p1_id = solver.add_point(Point::new(0, 0.0, 0.0));
    let p2_id = solver.add_point(Point::new(1, 1.0, 1.0));

    // Create a line between the points
    let line = solver.add_line(Line::new(2, p1_id, p2_id));

    // Add a vertical constraint to the line
    solver
        .add_constraint(ConstraintType::Vertical(line))
        .unwrap();

    // Solve the constraint system
    let result = solver.solve().unwrap();

    match result {
        SolverResult::Converged {
            iterations,
            final_error,
            ..
        } => {
            println!(
                "Converged in {} iterations with error {}",
                iterations, final_error
            );
        }
        SolverResult::MaxIterationsReached {
            iterations,
            final_error,
            ..
        } => {
            println!(
                "Max iterations ({}) reached with error {}",
                iterations, final_error
            );
        }
    }

    // Get the final positions
    let start = solver.get_point(p1_id).unwrap();
    let end = solver.get_point(p2_id).unwrap();
    println!(
        "Line endpoints: ({}, {}) to ({}, {})",
        start.x, start.y, end.x, end.y
    );
}
