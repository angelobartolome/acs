use acs::{Circle, ConstraintSolver, ConstraintType, Operand, Point, SolverResult};

fn two_circles(fixed_outer: bool) -> ConstraintSolver {
    let mut solver = ConstraintSolver::new();
    solver.add_point(Point::new("o".into(), 0.0, 0.0, true));
    solver.add_circle(Circle::new("outer".into(), "o".into(), 15.0, fixed_outer));
    solver.add_circle(Circle::new("inner".into(), "o".into(), 10.0, false));
    solver
}

fn converge(solver: &mut ConstraintSolver) {
    let result = solver.solve().unwrap();
    assert!(
        matches!(result, SolverResult::Converged { .. }),
        "{result:?}"
    );
}

/// param2 − param1 = difference, with a fixed outer radius.
#[test]
fn difference_of_radii_moves_the_free_one() {
    let mut solver = two_circles(true);
    solver
        .add_constraint(ConstraintType::Difference(
            Operand::Radius("inner".into()),
            Operand::Radius("outer".into()),
            Operand::Const(4.0),
        ))
        .unwrap();
    converge(&mut solver);
    assert!((solver.get_circle("outer".into()).unwrap().radius - 15.0).abs() < 1e-12);
    assert!((solver.get_circle("inner".into()).unwrap().radius - 11.0).abs() < 1e-8);
}

/// The difference itself may be a variable: here a Point's x.
#[test]
fn difference_may_be_a_variable() {
    let mut solver = two_circles(true);
    solver.add_point(Point::new("p".into(), 0.0, 0.0, false));
    solver
        .add_constraint(ConstraintType::Difference(
            Operand::Const(10.0),
            Operand::Radius("outer".into()),
            Operand::X("p".into()),
        ))
        .unwrap();
    converge(&mut solver);
    assert!((solver.get_point("p".into()).unwrap().x - 5.0).abs() < 1e-8);
}

/// A property reference to a missing entity is rejected when added.
#[test]
fn reference_to_a_missing_entity_is_rejected() {
    let mut solver = two_circles(false);
    let err = solver
        .add_constraint(ConstraintType::Difference(
            Operand::Radius("ghost".into()),
            Operand::Radius("outer".into()),
            Operand::Const(1.0),
        ))
        .unwrap_err();
    assert_eq!(err, "'ghost' is not a circle or arc");
}
