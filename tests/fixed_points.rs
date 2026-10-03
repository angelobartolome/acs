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
        .expect("Failed to add vertical constraint");

    // Try to solve - the movable point should adjust, fixed point should stay
    match solver.solve() {
        Ok(result) => {
            assert!(matches!(result, SolverResult::Converged { .. }));
        }
        Err(e) => panic!("Solver error: {}", e),
    }

    // Check the final position of the movable point
    let final_movable = solver.get_point(movable_id.clone()).expect("Movable point should exist");
    let final_fixed = solver.get_point(fixed_id.clone()).expect("Fixed point should exist");
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

#[test]
fn add_constraint_rejects_unknown_or_wrong_kind_entities() {
    let mut solver = ConstraintSolver::new();
    solver.add_point(Point::new("p".into(), 0.0, 0.0, false));
    solver.add_point(Point::new("c".into(), 0.0, 0.0, false));
    solver.add_circle(Circle::new("circle".into(), "c".into(), 1.0, false));

    let missing = solver.add_constraint(ConstraintType::Horizontal("p".into(), "nope".into()));
    assert_eq!(missing, Err("'nope' is not a point".to_string()));

    let wrong_kind = solver.add_constraint(ConstraintType::FixedRadius("p".into(), 2.0));
    assert_eq!(wrong_kind, Err("'p' is not a circle or arc".to_string()));

    // Rejected constraints are not kept, so the sketch still solves.
    assert!(matches!(solver.solve(), Ok(SolverResult::Converged { .. })));
}

/// A component whose points are all fixed can't move: it's solved only if it
/// already holds, and the rest of the sketch still solves.
#[test]
fn all_fixed_component_reports_failure_when_violated() {
    let mut solver = ConstraintSolver::new();
    solver.add_point(Point::new("a".into(), 0.0, 0.0, true));
    solver.add_point(Point::new("b".into(), 1.0, 2.0, true));
    solver.add_point(Point::new("c".into(), 0.0, 0.0, false));
    solver.add_point(Point::new("d".into(), 3.0, 4.0, false));
    solver
        .add_constraint(ConstraintType::Horizontal("a".into(), "b".into()))
        .unwrap();
    solver
        .add_constraint(ConstraintType::Coincident("c".into(), "d".into()))
        .unwrap();

    let result = solver.solve().unwrap();
    assert!(matches!(result, SolverResult::MaxIterationsReached { .. }), "{result:?}");
    let d = solver.get_point("d".into()).unwrap();
    let c = solver.get_point("c".into()).unwrap();
    assert!((c.x - d.x).abs() < 1e-9 && (c.y - d.y).abs() < 1e-9);
}
