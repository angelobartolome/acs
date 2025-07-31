use acs::{ConstraintSolver, SolverResult, constraints::base::ConstraintType, geometry::*};

#[test]
fn test_fixed_points() {
    let mut solver = ConstraintSolver::new();

    // Create points - one fixed, one movable
    let fixed_point = Point::new(String::from("fixed"), 0.0, 0.0, true); // Fixed at origin
    let movable_point = Point::new(String::from("movable"), 3.0, 4.0, false); // Movable point

    let fixed_id = solver.add_point(fixed_point);
    let movable_id = solver.add_point(movable_point);

    // Add a vertical constraint to the line
    solver
        .add_constraint(ConstraintType::Vertical(
            fixed_id.clone(),
            movable_id.clone(),
        ))
        .unwrap();

    // Try to solve - the movable point should adjust, fixed point should stay
    match solver.solve() {
        Ok(result) => {
            assert!(matches!(result, SolverResult::Converged { .. }));
        }
        Err(e) => panic!("Solver error: {}", e),
    }

    // Check the final position of the movable point
    let final_movable = solver.get_point(movable_id.clone()).unwrap();
    let final_fixed = solver.get_point(fixed_id.clone()).unwrap();
    solver.print_state();

    assert!(
        (final_fixed.x - 0.0).abs() < 1e-6 && (final_fixed.y - 0.0).abs() < 1e-6,
        "Fixed point should remain at origin"
    );

    assert!(
        (final_movable.x - 0.0).abs() < 1e-6 && (final_movable.y - 4.0).abs() < 1e-6,
        "Movable point should be adjusted to y=4.0"
    );
}
