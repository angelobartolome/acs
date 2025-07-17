use acs::{ConstraintSolver, SolverResult, constraints::base::ConstraintType, geometry::*};

#[test]
fn test_fixed_points() {
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

    // Try to solve - the movable point should adjust, fixed point should stay
    match solver.solve() {
        Ok(result) => {
            assert!(matches!(result, SolverResult::Converged { .. }));
        }
        Err(e) => panic!("Solver error: {}", e),
    }

    let new_fixed_point = Point::new(0, 1.0, 1.0, true);
    match solver.move_point(fixed_id, new_fixed_point) {
        Ok(_) => panic!("ERROR: Should not be able to move fixed point!"),
        Err(e) => assert_eq!(e, "Cannot move fixed point 0"),
    }

    solver.set_point_fixed(movable_id, true).unwrap();
    assert!(solver.is_point_fixed(movable_id));

    solver.set_point_fixed(movable_id, false).unwrap();
    assert!(!solver.is_point_fixed(movable_id));
}
