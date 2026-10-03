use acs::constraints::Constraint;
use acs::constraints::midpoint_line_on_line::MidpointOfLineOnLineConstraint;
use acs::geometry::Point;
use acs::var_registry::{EntityType, VarRegistry};

#[test]
fn fd_check_midpoint_line_on_line_jacobian() {
    let mut pm = VarRegistry::new();
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
    let n = pm.num_vars();
    let h = 1e-7;

    for i in 0..n {
        let orig = pm.values()[i];
        pm.values_mut()[i] = orig + h;
        let fd = (c.residual(&pm)[0] - base) / h;
        pm.values_mut()[i] = orig;
        assert!(
            (jac[(0, i)] - fd).abs() < 1e-4,
            "param {i}: analytic {} vs fd {fd}",
            jac[(0, i)]
        );
    }
}

/// L2 is a segment: L1's midpoint must land between L2's endpoints.
#[test]
fn midpoint_lands_within_segment() {
    use acs::{ConstraintSolver, ConstraintType, SolverResult};

    let mut solver = ConstraintSolver::new();
    solver.add_point(Point::new("l2a".into(), 0.0, 0.0, true));
    solver.add_point(Point::new("l2b".into(), 1.0, 0.0, true));
    solver.add_point(Point::new("l1a".into(), 4.0, 1.0, false));
    solver.add_point(Point::new("l1b".into(), 6.0, 1.0, false));
    solver
        .add_constraint(ConstraintType::MidpointOfLineOnLine(
            "l1a".into(),
            "l1b".into(),
            "l2a".into(),
            "l2b".into(),
        ))
        .unwrap();

    let result = solver.solve().unwrap();
    assert!(matches!(result, SolverResult::Converged { .. }), "{result:?}");

    let a = solver.get_point("l1a".into()).unwrap().clone();
    let b = solver.get_point("l1b".into()).unwrap().clone();
    let (mx, my) = ((a.x + b.x) / 2.0, (a.y + b.y) / 2.0);
    assert!(my.abs() < 1e-8, "midpoint should be on the segment, got y = {my}");
    assert!((0.0..=1.0 + 1e-8).contains(&mx), "midpoint x = {mx} should be within [0, 1]");
}
