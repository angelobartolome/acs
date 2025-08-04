use acs::{Circle, EntityType, EqualRadiusConstraint, ParameterManager, constraints::Constraint};

#[test]
fn test_equal_radius_constraint_residual() {
    // Create two circles with different radii
    let circle1 = Circle::new("c1".to_string(), 0.0, 0.0, 5.0, false);
    let circle2 = Circle::new("c2".to_string(), 10.0, 10.0, 3.0, false);

    // Set up parameter manager
    let mut param_manager = ParameterManager::new();
    param_manager.register_entity("c1".to_string(), EntityType::Circle, &circle1);
    param_manager.register_entity("c2".to_string(), EntityType::Circle, &circle2);

    // Create equal radius constraint
    let constraint = EqualRadiusConstraint::new("c1".to_string(), "c2".to_string());

    // Test residual calculation
    let residual = constraint.residual_parametric(&param_manager);

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
    // Create two circles
    let circle1 = Circle::new("c1".to_string(), 0.0, 0.0, 5.0, false);
    let circle2 = Circle::new("c2".to_string(), 10.0, 10.0, 3.0, false);

    // Set up parameter manager
    let mut param_manager = ParameterManager::new();
    param_manager.register_entity("c1".to_string(), EntityType::Circle, &circle1);
    param_manager.register_entity("c2".to_string(), EntityType::Circle, &circle2);

    // Create equal radius constraint
    let constraint = EqualRadiusConstraint::new("c1".to_string(), "c2".to_string());

    // Test jacobian calculation
    let jacobian = constraint.jacobian_parametric(&param_manager);

    // Should have 1 row (one constraint) and 6 columns (2 circles × 3 parameters each)
    assert_eq!(jacobian.nrows(), 1);
    assert_eq!(jacobian.ncols(), 6);

    // Check that the derivatives are correct
    // Circle 1 radius parameter should have derivative 1.0 (at index 2)
    let c1_radius_idx = param_manager.get_global_index("c1", 2).unwrap();
    assert!((jacobian[(0, c1_radius_idx)] - 1.0).abs() < 1e-10);

    // Circle 2 radius parameter should have derivative -1.0 (at index 5)
    let c2_radius_idx = param_manager.get_global_index("c2", 2).unwrap();
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
    // Create two circles with equal radii
    let circle1 = Circle::new("c1".to_string(), 0.0, 0.0, 4.0, false);
    let circle2 = Circle::new("c2".to_string(), 10.0, 10.0, 4.0, false);

    // Set up parameter manager
    let mut param_manager = ParameterManager::new();
    param_manager.register_entity("c1".to_string(), EntityType::Circle, &circle1);
    param_manager.register_entity("c2".to_string(), EntityType::Circle, &circle2);

    // Create equal radius constraint
    let constraint = EqualRadiusConstraint::new("c1".to_string(), "c2".to_string());

    // Test residual calculation
    let residual = constraint.residual_parametric(&param_manager);

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
fn test_equal_radius_constraint_backward_compatibility() {
    // Test the legacy methods that should return zeros for backward compatibility
    use std::collections::HashMap;

    let constraint = EqualRadiusConstraint::new("c1".to_string(), "c2".to_string());
    let points = HashMap::new();
    let id_to_index = HashMap::new();

    // Test legacy residual method
    let residual = constraint.residual(&points);
    assert_eq!(residual.len(), 1);
    assert_eq!(residual[0], 0.0);

    // Test legacy jacobian method
    let jacobian = constraint.jacobian(&points, &id_to_index);
    assert_eq!(jacobian.nrows(), 1);
    assert_eq!(jacobian.ncols(), 0); // No points, so 0 columns
}
