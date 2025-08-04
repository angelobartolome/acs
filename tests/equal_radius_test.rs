use acs::{
    Circle, ConstraintSolver, ConstraintType, EntityType, EqualRadiusConstraint, ParameterManager,
    Point, SolverResult, constraints::Constraint,
};

#[test]
fn test_equal_radius_constraint_residual() {
    // Create center points for the circles
    let center1 = Point::new("center1".to_string(), 0.0, 0.0, false);
    let center2 = Point::new("center2".to_string(), 10.0, 10.0, false);

    // Create two circles with different radii
    let circle1 = Circle::new("c1".to_string(), "center1".to_string(), 5.0, false);
    let circle2 = Circle::new("c2".to_string(), "center2".to_string(), 3.0, false);

    // Set up parameter manager
    let mut param_manager = ParameterManager::new();
    param_manager.register_entity("center1".to_string(), EntityType::Point, &center1);
    param_manager.register_entity("center2".to_string(), EntityType::Point, &center2);
    param_manager.register_entity("c1".to_string(), EntityType::Circle, &circle1);
    param_manager.register_entity("c2".to_string(), EntityType::Circle, &circle2);

    // Create equal radius constraint
    let constraint = EqualRadiusConstraint::new("c1".to_string(), "c2".to_string());

    // Test residual calculation
    let residual = constraint.residual(&param_manager);

    // Expected residual: radius1 - radius2 = 5.0 - 3.0 = 2.0
    assert_eq!(residual.len(), 1);
    assert!(
        (residual[0] - 2.0).abs() < 1e-10,
        "Expected residual of 2.0, got {}",
        residual[0]
    );
}

#[test]
fn test_equal_radius_constraint_jacobian() {
    // Create center points for the circles
    let center1 = Point::new("center1".to_string(), 0.0, 0.0, false);
    let center2 = Point::new("center2".to_string(), 10.0, 10.0, false);

    // Create two circles
    let circle1 = Circle::new("c1".to_string(), "center1".to_string(), 5.0, false);
    let circle2 = Circle::new("c2".to_string(), "center2".to_string(), 3.0, false);

    // Set up parameter manager
    let mut param_manager = ParameterManager::new();
    param_manager.register_entity("center1".to_string(), EntityType::Point, &center1);
    param_manager.register_entity("center2".to_string(), EntityType::Point, &center2);
    param_manager.register_entity("c1".to_string(), EntityType::Circle, &circle1);
    param_manager.register_entity("c2".to_string(), EntityType::Circle, &circle2);

    // Create equal radius constraint
    let constraint = EqualRadiusConstraint::new("c1".to_string(), "c2".to_string());

    // Test jacobian calculation
    let jacobian = constraint.jacobian(&param_manager);

    // Should have 1 row (one constraint) and 6 columns (2 points × 2 parameters + 2 circles × 1 parameter each)
    assert_eq!(jacobian.nrows(), 1);
    assert_eq!(jacobian.ncols(), 6);

    // Check that the derivatives are correct
    // Circle 1 radius parameter should have derivative 1.0 (at index 0 for circle parameters)
    let c1_radius_idx = param_manager.get_global_index("c1", 0).unwrap();
    assert!((jacobian[(0, c1_radius_idx)] - 1.0).abs() < 1e-10);

    // Circle 2 radius parameter should have derivative -1.0 (at index 0 for circle parameters)
    let c2_radius_idx = param_manager.get_global_index("c2", 0).unwrap();
    assert!((jacobian[(0, c2_radius_idx)] - (-1.0)).abs() < 1e-10);

    // All other entries should be zero
    for i in 0..jacobian.ncols() {
        if i != c1_radius_idx && i != c2_radius_idx {
            assert!(
                (jacobian[(0, i)]).abs() < 1e-10,
                "Non-zero entry at index {}: {}",
                i,
                jacobian[(0, i)]
            );
        }
    }
}

#[test]
fn test_equal_radius_constraint_zero_residual() {
    // Create center points for the circles
    let center1 = Point::new("center1".to_string(), 0.0, 0.0, false);
    let center2 = Point::new("center2".to_string(), 10.0, 10.0, false);

    // Create two circles with equal radii
    let circle1 = Circle::new("c1".to_string(), "center1".to_string(), 4.0, false);
    let circle2 = Circle::new("c2".to_string(), "center2".to_string(), 4.0, false);

    // Set up parameter manager
    let mut param_manager = ParameterManager::new();
    param_manager.register_entity("center1".to_string(), EntityType::Point, &center1);
    param_manager.register_entity("center2".to_string(), EntityType::Point, &center2);
    param_manager.register_entity("c1".to_string(), EntityType::Circle, &circle1);
    param_manager.register_entity("c2".to_string(), EntityType::Circle, &circle2);

    // Create equal radius constraint
    let constraint = EqualRadiusConstraint::new("c1".to_string(), "c2".to_string());

    // Test residual calculation
    let residual = constraint.residual(&param_manager);

    // Expected residual: radius1 - radius2 = 4.0 - 4.0 = 0.0
    assert_eq!(residual.len(), 1);
    assert!(
        residual[0].abs() < 1e-10,
        "Expected residual of 0.0, got {}",
        residual[0]
    );
}

#[test]
fn test_equal_radius_constraint_num_residuals() {
    let constraint = EqualRadiusConstraint::new("c1".to_string(), "c2".to_string());
    assert_eq!(constraint.num_residuals(), 1);
}

#[test]
fn test_equal_radius_constraint_solver_integration() {
    // Test that the equal radius constraint actually resizes circles when solved
    let mut solver = ConstraintSolver::new();

    // Create center points for the circles
    let center1_id = solver.add_point(Point::new("center1".to_string(), 0.0, 0.0, false));
    let center2_id = solver.add_point(Point::new("center2".to_string(), 10.0, 10.0, false));

    // Create two circles with different radii (10 and 3)
    let circle1_id = solver.add_circle(Circle::new(
        "c1".to_string(),
        center1_id.clone(),
        10.0,
        false,
    ));
    let circle2_id = solver.add_circle(Circle::new(
        "c2".to_string(),
        center2_id.clone(),
        3.0,
        false,
    ));

    // Verify initial radii are different
    let initial_circle1 = solver.get_circle(circle1_id.clone()).unwrap();
    let initial_circle2 = solver.get_circle(circle2_id.clone()).unwrap();
    assert!(
        (initial_circle1.radius - 10.0).abs() < 1e-10,
        "Initial circle1 radius should be 10.0"
    );
    assert!(
        (initial_circle2.radius - 3.0).abs() < 1e-10,
        "Initial circle2 radius should be 3.0"
    );

    // Add equal radius constraint
    solver
        .add_constraint(ConstraintType::EqualRadius(
            circle1_id.clone(),
            circle2_id.clone(),
        ))
        .unwrap();

    // Solve the constraint system
    let result = solver.solve().unwrap();

    // Verify the solver converged
    match result {
        SolverResult::Converged { final_error, .. } => {
            assert!(
                final_error < 1e-6,
                "Expected small final error, got {}",
                final_error
            );
        }
        _ => panic!("Solver should have converged"),
    }

    // Verify that both circles now have the same radius
    let final_circle1 = solver.get_circle(circle1_id).unwrap();
    let final_circle2 = solver.get_circle(circle2_id).unwrap();

    assert!(
        (final_circle1.radius - final_circle2.radius).abs() < 1e-6,
        "Circles should have equal radii after solving: c1={}, c2={}",
        final_circle1.radius,
        final_circle2.radius
    );

    // The final radius should be somewhere between the original values (since both circles are free to change)
    // Expected behavior: both radii will converge to a value between 3.0 and 10.0
    assert!(
        final_circle1.radius >= 3.0 && final_circle1.radius <= 10.0,
        "Final radius should be between 3.0 and 10.0, got {}",
        final_circle1.radius
    );

    println!("Initial radii: c1={}, c2={}", 10.0, 3.0);
    println!(
        "Final radii: c1={}, c2={}",
        final_circle1.radius, final_circle2.radius
    );
}
