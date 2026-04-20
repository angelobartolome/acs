use acs::{ConstraintSolver, ConstraintType, Point};

// ── Unit tests for find_components ─────────────────────────────────────────

#[test]
fn test_find_components_two_independent() {
    // Two constraints that share no entity IDs → 2 components.
    use acs::component_graph::find_components;
    let types = vec![
        ConstraintType::Vertical("p1".into(), "p2".into()),
        ConstraintType::Vertical("p3".into(), "p4".into()),
    ];
    let cc = find_components(&types);
    assert_eq!(cc.components.len(), 2);
    let mut sizes: Vec<usize> = cc.components.iter().map(|c| c.len()).collect();
    sizes.sort();
    assert_eq!(sizes, vec![1, 1]);
}

#[test]
fn test_find_components_connected_via_shared_entity() {
    // Three constraints: Vertical(p1,p2), Horizontal(p3,p4), Coincident(p2,p3)
    // The Coincident bridges the two groups → 1 component.
    use acs::component_graph::find_components;
    let types = vec![
        ConstraintType::Vertical("p1".into(), "p2".into()),
        ConstraintType::Horizontal("p3".into(), "p4".into()),
        ConstraintType::Coincident("p2".into(), "p3".into()),
    ];
    let cc = find_components(&types);
    assert_eq!(cc.components.len(), 1);
    assert_eq!(cc.components[0].len(), 3);
}

#[test]
fn test_find_components_empty() {
    use acs::component_graph::find_components;
    let cc = find_components(&[]);
    assert_eq!(cc.components.len(), 0);
}

#[test]
fn test_find_components_single_entity_constraint() {
    use acs::component_graph::find_components;
    let types = vec![ConstraintType::EqualX("p1".into(), 5.0)];
    let cc = find_components(&types);
    assert_eq!(cc.components.len(), 1);
    assert_eq!(cc.components[0], vec![0]);
}

// ── entity_ids exhaustive coverage ────────────────────────────────────────

#[test]
fn test_entity_ids_all_variants() {
    assert_eq!(
        ConstraintType::Vertical("a".into(), "b".into()).entity_ids(),
        vec!["a", "b"]
    );
    assert_eq!(
        ConstraintType::Horizontal("a".into(), "b".into()).entity_ids(),
        vec!["a", "b"]
    );
    assert_eq!(
        ConstraintType::Coincident("a".into(), "b".into()).entity_ids(),
        vec!["a", "b"]
    );
    assert_eq!(
        ConstraintType::EqualX("a".into(), 1.0).entity_ids(),
        vec!["a"]
    );
    assert_eq!(
        ConstraintType::EqualY("a".into(), 1.0).entity_ids(),
        vec!["a"]
    );
    assert_eq!(
        ConstraintType::FixedRadius("c".into(), 5.0).entity_ids(),
        vec!["c"]
    );
    assert_eq!(
        ConstraintType::EqualRadius("c1".into(), "c2".into()).entity_ids(),
        vec!["c1", "c2"]
    );
    assert_eq!(
        ConstraintType::Concentric("a".into(), "b".into()).entity_ids(),
        vec!["a", "b"]
    );
    assert_eq!(
        ConstraintType::DistancePointPoint("a".into(), "b".into(), 3.0).entity_ids(),
        vec!["a", "b"]
    );
    assert_eq!(
        ConstraintType::PointOnLine("p".into(), "a".into(), "b".into()).entity_ids(),
        vec!["p", "a", "b"]
    );
    assert_eq!(
        ConstraintType::Midpoint("m".into(), "a".into(), "b".into()).entity_ids(),
        vec!["m", "a", "b"]
    );
    assert_eq!(
        ConstraintType::PointOnCircle("p".into(), "cc".into(), "c".into()).entity_ids(),
        vec!["p", "cc", "c"]
    );
    assert_eq!(
        ConstraintType::DistancePointLine("p".into(), "a".into(), "b".into(), 2.0).entity_ids(),
        vec!["p", "a", "b"]
    );
    assert_eq!(
        ConstraintType::Parallel("a".into(), "b".into(), "c".into(), "d".into()).entity_ids(),
        vec!["a", "b", "c", "d"]
    );
    assert_eq!(
        ConstraintType::Perpendicular("a".into(), "b".into(), "c".into(), "d".into()).entity_ids(),
        vec!["a", "b", "c", "d"]
    );
    assert_eq!(
        ConstraintType::Tangent("a".into(), "b".into(), "c".into(), "d".into()).entity_ids(),
        vec!["a", "b", "c", "d"]
    );
    assert_eq!(
        ConstraintType::TangentLineCircle("a".into(), "b".into(), "c".into(), "d".into())
            .entity_ids(),
        vec!["a", "b", "c", "d"]
    );
    assert_eq!(
        ConstraintType::Angle("a".into(), "b".into(), "c".into(), "d".into(), 1.0).entity_ids(),
        vec!["a", "b", "c", "d"]
    );
    assert_eq!(
        ConstraintType::EqualLength("a".into(), "b".into(), "c".into(), "d".into()).entity_ids(),
        vec!["a", "b", "c", "d"]
    );
    assert_eq!(
        ConstraintType::Symmetric("a".into(), "b".into(), "c".into(), "d".into()).entity_ids(),
        vec!["a", "b", "c", "d"]
    );
    assert_eq!(
        ConstraintType::MidpointOfLineOnLine("a".into(), "b".into(), "c".into(), "d".into())
            .entity_ids(),
        vec!["a", "b", "c", "d"]
    );
}

// ── Integration: pre-solver skips already-satisfied components ─────────────

#[test]
fn test_presolver_skips_satisfied_component() {
    // Two coincident pairs: (p1,p2) already coincident; (p3,p4) not.
    // With max_iterations=0 the dogleg can't move anything, so if the
    // satisfied pair is (incorrectly) fed to the solver it doesn't matter —
    // but the unsatisfied pair should report MaxIterationsReached.
    let mut solver = ConstraintSolver::new();

    // Already coincident
    solver.add_point(Point::new("p1".into(), 1.0, 2.0, false));
    solver.add_point(Point::new("p2".into(), 1.0, 2.0, false));
    solver.add_constraint(ConstraintType::Coincident("p1".into(), "p2".into())).unwrap();

    // Not coincident
    solver.add_point(Point::new("p3".into(), 0.0, 0.0, false));
    solver.add_point(Point::new("p4".into(), 5.0, 5.0, false));
    solver.add_constraint(ConstraintType::Coincident("p3".into(), "p4".into())).unwrap();

    solver.set_max_iterations(0);
    let result = solver.solve().unwrap();

    // The satisfied component was skipped, so only the (p3,p4) pair
    // went to the solver with max_iterations=0 → MaxIterationsReached.
    match result {
        acs::SolverResult::MaxIterationsReached { .. } => {}
        other => panic!("expected MaxIterationsReached, got {:?}", other),
    }

    // The already-satisfied pair must be unchanged.
    let p1 = solver.get_point("p1".into()).unwrap();
    let p2 = solver.get_point("p2".into()).unwrap();
    assert!((p1.x - p2.x).abs() < 1e-10);
    assert!((p1.y - p2.y).abs() < 1e-10);
}

#[test]
fn test_presolver_all_satisfied_returns_converged() {
    // Both pairs already coincident → both skipped → Converged with 0 iterations.
    let mut solver = ConstraintSolver::new();

    solver.add_point(Point::new("p1".into(), 3.0, 3.0, false));
    solver.add_point(Point::new("p2".into(), 3.0, 3.0, false));
    solver.add_constraint(ConstraintType::Coincident("p1".into(), "p2".into())).unwrap();

    solver.add_point(Point::new("p3".into(), 7.0, 1.0, false));
    solver.add_point(Point::new("p4".into(), 7.0, 1.0, false));
    solver.add_constraint(ConstraintType::Coincident("p3".into(), "p4".into())).unwrap();

    solver.set_max_iterations(0); // would fail if solver actually ran
    let result = solver.solve().unwrap();

    match result {
        acs::SolverResult::Converged { iterations, .. } => {
            assert_eq!(iterations, 0);
        }
        other => panic!("expected Converged{{0}}, got {:?}", other),
    }
}

#[test]
fn test_presolver_two_independent_components_both_solved() {
    // Two independent horizontal constraints, both unsatisfied.
    // Each forms its own component; both should be solved.
    let mut solver = ConstraintSolver::new();

    solver.add_point(Point::new("p1".into(), 0.0, 0.0, true));
    solver.add_point(Point::new("p2".into(), 1.0, 5.0, false));
    solver.add_constraint(ConstraintType::Horizontal("p1".into(), "p2".into())).unwrap();

    solver.add_point(Point::new("p3".into(), 0.0, 0.0, true));
    solver.add_point(Point::new("p4".into(), 2.0, 9.0, false));
    solver.add_constraint(ConstraintType::Horizontal("p3".into(), "p4".into())).unwrap();

    let result = solver.solve().unwrap();
    match result {
        acs::SolverResult::Converged { .. } => {}
        other => panic!("expected Converged, got {:?}", other),
    }

    let p1 = solver.get_point("p1".into()).unwrap();
    let p2 = solver.get_point("p2".into()).unwrap();
    assert!((p1.y - p2.y).abs() < 1e-6, "p1/p2 not horizontal");

    let p3 = solver.get_point("p3".into()).unwrap();
    let p4 = solver.get_point("p4".into()).unwrap();
    assert!((p3.y - p4.y).abs() < 1e-6, "p3/p4 not horizontal");
}

#[test]
fn test_no_constraints_returns_converged() {
    let mut solver = ConstraintSolver::new();
    solver.add_point(Point::new("p1".into(), 1.0, 2.0, false));
    let result = solver.solve().unwrap();
    match result {
        acs::SolverResult::Converged { iterations, .. } => assert_eq!(iterations, 0),
        other => panic!("expected Converged{{0}}, got {:?}", other),
    }
}
