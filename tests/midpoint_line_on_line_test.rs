use acs::constraints::Constraint;
use acs::constraints::midpoint_line_on_line::MidpointOfLineOnLineConstraint;
use acs::geometry::Point;
use acs::parameter_system::{EntityType, ParameterManager};

#[test]
fn fd_check_midpoint_line_on_line_jacobian() {
    let mut pm = ParameterManager::new();
    for (id, x, y) in [
        ("l1a", 0.1, 0.2),
        ("l1b", 2.3, 4.1),
        ("l2a", 0.3, 0.7),
        ("l2b", 5.0, 1.0),
    ] {
        pm.register_entity(
            id.to_string(),
            EntityType::Point,
            &Point::new(id.to_string(), x, y, false),
        );
    }
    let c = MidpointOfLineOnLineConstraint::new(
        "l1a".into(),
        "l1b".into(),
        "l2a".into(),
        "l2b".into(),
    );

    let jac = c.jacobian(&pm);
    let base = c.residual(&pm)[0];
    let n = pm.num_parameters();
    let h = 1e-7;

    for i in 0..n {
        let orig = pm.get_parameters()[i];
        pm.get_parameters_mut()[i] = orig + h;
        let fd = (c.residual(&pm)[0] - base) / h;
        pm.get_parameters_mut()[i] = orig;
        assert!(
            (jac[(0, i)] - fd).abs() < 1e-4,
            "param {i}: analytic {} vs fd {fd}",
            jac[(0, i)]
        );
    }
}
