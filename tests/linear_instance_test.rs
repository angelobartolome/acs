use acs::{ConstraintSolver, ConstraintType, Point, SolverResult};

fn instance(dir: (&str, &str), base_distance: f64, n: f64) -> (f64, f64) {
    let mut solver = ConstraintSolver::new();
    solver.add_point(Point::new("d1".into(), 0.0, 0.0, true));
    solver.add_point(Point::new("d2".into(), 3.0, 4.0, true));
    solver.add_point(Point::new("p0".into(), 1.0, -1.0, true));
    solver.add_point(Point::new("pk".into(), 0.0, 0.0, false));
    solver
        .add_constraint(ConstraintType::LinearInstance(
            "p0".into(),
            "pk".into(),
            dir.0.into(),
            dir.1.into(),
            base_distance,
            n,
        ))
        .unwrap();
    let result = solver.solve().unwrap();
    assert!(
        matches!(result, SolverResult::Converged { .. }),
        "{result:?}"
    );
    let p = solver.get_point("pk".into()).unwrap();
    (p.x, p.y)
}

#[test]
fn translates_n_spacings_along_the_direction() {
    // Unit direction (0.6, 0.8); 2 · 5 = 10 along it.
    let (x, y) = instance(("d1", "d2"), 5.0, 2.0);
    assert!(
        (x - 7.0).abs() < 1e-8 && (y - 7.0).abs() < 1e-8,
        "({x}, {y})"
    );
}

/// An array is flipped by swapping the direction points.
#[test]
fn swapped_direction_points_flip_the_array() {
    let (x, y) = instance(("d2", "d1"), 5.0, 1.0);
    assert!(
        (x + 2.0).abs() < 1e-8 && (y + 5.0).abs() < 1e-8,
        "({x}, {y})"
    );
}

/// Only the direction of the axis matters, not its length.
#[test]
fn spacing_does_not_depend_on_the_axis_length() {
    let mut solver = ConstraintSolver::new();
    solver.add_point(Point::new("d1".into(), 0.0, 0.0, true));
    solver.add_point(Point::new("d2".into(), 0.5, 0.0, true));
    solver.add_point(Point::new("p0".into(), 0.0, 2.0, true));
    solver.add_point(Point::new("pk".into(), 0.0, 0.0, false));
    solver
        .add_constraint(ConstraintType::LinearInstance(
            "p0".into(),
            "pk".into(),
            "d1".into(),
            "d2".into(),
            3.0,
            1.0,
        ))
        .unwrap();
    solver.solve().unwrap();
    let p = solver.get_point("pk".into()).unwrap();
    assert!((p.x - 3.0).abs() < 1e-8 && (p.y - 2.0).abs() < 1e-8);
}
