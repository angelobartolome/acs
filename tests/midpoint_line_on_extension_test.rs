use acs::{ConstraintSolver, ConstraintType, Point, SolverResult};

/// L1's midpoint starts beyond L2's far end and lands on L2's Extension
/// there, not back on the segment.
#[test]
fn midpoint_on_extension_stays_beyond_the_segment() {
    let mut solver = ConstraintSolver::new();
    solver.add_point(Point::new("l2a".into(), 0.0, 0.0, true));
    solver.add_point(Point::new("l2b".into(), 10.0, 0.0, true));
    solver.add_point(Point::new("l1a".into(), 14.0, 1.0, false));
    solver.add_point(Point::new("l1b".into(), 16.0, 3.0, false));
    solver
        .add_constraint(ConstraintType::MidpointOfLineOnExtension(
            "l1a".into(),
            "l1b".into(),
            "l2a".into(),
            "l2b".into(),
        ))
        .unwrap();

    let result = solver.solve().unwrap();
    assert!(matches!(result, SolverResult::Converged { .. }), "{result:?}");

    let a = solver.get_point("l1a".into()).unwrap();
    let b = solver.get_point("l1b".into()).unwrap();
    let (mx, my) = ((a.x + b.x) / 2.0, (a.y + b.y) / 2.0);
    assert!(my.abs() < 1e-8, "midpoint should lie on the Extension, got y = {my}");
    assert!((mx - 15.0).abs() < 1e-6, "midpoint should keep its x, got {mx}");
}
