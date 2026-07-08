use acs::{ConstraintSolver, ConstraintType, Point};

/// A single free point has 2 DOF -> not fully constrained.
#[test]
fn test_free_point_not_constrained() {
    let mut solver = ConstraintSolver::new();
    solver.add_point(Point::new("p1".into(), 0.0, 0.0, false));

    let ids = solver.fully_constrained_entity_ids();
    assert!(
        ids.is_empty(),
        "a free point with no constraints must not be fully constrained, got {ids:?}"
    );
}

/// A fixed point has 0 DOF -> fully constrained even without constraints.
#[test]
fn test_fixed_point_is_constrained() {
    let mut solver = ConstraintSolver::new();
    solver.add_point(Point::new("p1".into(), 3.0, 4.0, true));

    let ids = solver.fully_constrained_entity_ids();
    assert_eq!(ids, vec!["p1".to_string()]);
}

/// A fixed anchor plus a distance + horizontal constraint pins the second point
/// completely: it can only sit at a fixed offset on the horizontal line. Both
/// points should be fully constrained.
#[test]
fn test_pinned_point_is_constrained() {
    let mut solver = ConstraintSolver::new();
    solver.add_point(Point::new("a".into(), 0.0, 0.0, true));
    solver.add_point(Point::new("b".into(), 1.0, 1.0, false));

    solver
        .add_constraint(ConstraintType::Horizontal("a".into(), "b".into()))
        .unwrap();
    solver
        .add_constraint(ConstraintType::DistancePointPoint(
            "a".into(),
            "b".into(),
            5.0,
        ))
        .unwrap();

    solver.solve().expect("solve should converge");

    let ids = solver.fully_constrained_entity_ids();
    assert!(ids.contains(&"a".to_string()), "fixed anchor a should be constrained: {ids:?}");
    assert!(ids.contains(&"b".to_string()), "pinned point b should be constrained: {ids:?}");
}

/// Only a horizontal constraint between a fixed anchor and a free point leaves
/// the free point able to slide along x -> 1 DOF remaining -> not fully constrained.
#[test]
fn test_underconstrained_point_not_constrained() {
    let mut solver = ConstraintSolver::new();
    solver.add_point(Point::new("a".into(), 0.0, 0.0, true));
    solver.add_point(Point::new("b".into(), 1.0, 1.0, false));

    solver
        .add_constraint(ConstraintType::Horizontal("a".into(), "b".into()))
        .unwrap();

    solver.solve().expect("solve should converge");

    let ids = solver.fully_constrained_entity_ids();
    assert!(ids.contains(&"a".to_string()), "fixed anchor a should be constrained");
    assert!(
        !ids.contains(&"b".to_string()),
        "b can still slide in x and should NOT be fully constrained: {ids:?}"
    );
}
