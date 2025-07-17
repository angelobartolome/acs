use acs::{ConstraintSolver, constraints::base::ConstraintType, geometry::*};

fn main() {
    let mut solver = ConstraintSolver::new();

    // Create points - one fixed, one movable
    let fixed_point = Point::new(0, 0.0, 0.0, true); // Fixed at origin
    let movable_point = Point::new(1, 3.0, 4.0, false); // Movable point

    let fixed_id = solver.add_point(fixed_point);
    let movable_id = solver.add_point(movable_point);

    // Create a line between them
    let line = Line::new(0, fixed_id, movable_id);
    solver.add_line(line);

    // Add a vertical constraint to the line
    solver.add_constraint(ConstraintType::Vertical(0)).unwrap();

    println!("Initial state:");
    solver.print_state();

    // Try to solve - the movable point should adjust, fixed point should stay
    match solver.solve() {
        Ok(result) => {
            println!("\nSolver result: {:?}", result);
            println!("\nFinal state:");
            solver.print_state();
        }
        Err(e) => println!("Solver error: {}", e),
    }

    // Try to move the fixed point - should fail
    println!("\nTrying to move fixed point...");
    let new_fixed_point = Point::new(0, 1.0, 1.0, true);
    match solver.move_point(fixed_id, new_fixed_point) {
        Ok(_) => println!("ERROR: Should not be able to move fixed point!"),
        Err(e) => println!("Correctly prevented moving fixed point: {}", e),
    }

    // Try to move the movable point - should work
    println!("\nTrying to move movable point...");
    let new_movable_point = Point::new(1, 5.0, 6.0, false);
    match solver.move_point(movable_id, new_movable_point) {
        Ok(result) => {
            println!("Successfully moved movable point: {:?}", result);
            println!("\nState after moving movable point:");
            solver.print_state();
        }
        Err(e) => println!("Error moving movable point: {}", e),
    }

    // Test setting a point as fixed/unfixed
    println!("\nTesting set_point_fixed...");
    println!(
        "Point {} is fixed: {}",
        movable_id,
        solver.is_point_fixed(movable_id)
    );

    solver.set_point_fixed(movable_id, true).unwrap();
    println!(
        "After setting as fixed, point {} is fixed: {}",
        movable_id,
        solver.is_point_fixed(movable_id)
    );

    solver.set_point_fixed(movable_id, false).unwrap();
    println!(
        "After setting as movable, point {} is fixed: {}",
        movable_id,
        solver.is_point_fixed(movable_id)
    );
}
