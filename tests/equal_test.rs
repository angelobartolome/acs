use acs::{Circle, ConstraintSolver, ConstraintType, Operand, Point, SolverResult};

fn converge(solver: &mut ConstraintSolver) {
    let result = solver.solve().unwrap();
    assert!(
        matches!(result, SolverResult::Converged { .. }),
        "{result:?}"
    );
}

#[test]
fn equal_radii_meet() {
    let mut solver = ConstraintSolver::new();
    solver.add_point(Point::new("o1".into(), 0.0, 0.0, true));
    solver.add_point(Point::new("o2".into(), 9.0, 0.0, true));
    solver.add_circle(Circle::new("c1".into(), "o1".into(), 3.0, true));
    solver.add_circle(Circle::new("c2".into(), "o2".into(), 1.0, false));
    solver
        .add_constraint(ConstraintType::Equal(
            Operand::Radius("c2".into()),
            Operand::Radius("c1".into()),
        ))
        .unwrap();
    converge(&mut solver);
    assert!((solver.get_circle("c2".into()).unwrap().radius - 3.0).abs() < 1e-8);
}

#[test]
fn equal_to_a_constant_sets_a_coordinate() {
    let mut solver = ConstraintSolver::new();
    solver.add_point(Point::new("p".into(), 1.0, 2.0, false));
    solver
        .add_constraint(ConstraintType::Equal(
            Operand::Y("p".into()),
            Operand::Const(-4.0),
        ))
        .unwrap();
    converge(&mut solver);
    let p = solver.get_point("p".into()).unwrap();
    assert!((p.y + 4.0).abs() < 1e-8 && (p.x - 1.0).abs() < 1e-12);
}
