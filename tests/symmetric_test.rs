use acs::{ConstraintSolver, ConstraintType, Point};

/// p and q symmetric about the y-axis (x=0 line through (0,-1)→(0,1)).
/// p fixed at (3,2); q should move to (−3,2).
#[test]
fn test_symmetric_about_y_axis() {
    let mut solver = ConstraintSolver::new();

    // Symmetry axis: vertical line (x=0)
    solver.add_point(Point::new("la".into(), 0.0, -1.0, true));
    solver.add_point(Point::new("lb".into(), 0.0, 1.0, true));

    // Fixed point
    solver.add_point(Point::new("p".into(), 3.0, 2.0, true));
    // Mirror point (free)
    solver.add_point(Point::new("q".into(), 1.0, 0.0, false));

    solver
        .add_constraint(ConstraintType::Symmetric(
            "p".into(),
            "q".into(),
            "la".into(),
            "lb".into(),
        ))
        .expect("add symmetric");

    solver.set_max_iterations(500);
    solver.solve().expect("solve");

    let q = solver.get_point("q".into()).unwrap();
    assert!(
        (q.x - (-3.0)).abs() < 1e-2 && (q.y - 2.0).abs() < 1e-2,
        "q should be (-3, 2), got ({}, {})",
        q.x,
        q.y
    );
}

/// p and q symmetric about a horizontal axis y=0.
/// p fixed at (1, 4); q should move to (1, −4).
#[test]
fn test_symmetric_about_x_axis() {
    let mut solver = ConstraintSolver::new();

    // Symmetry axis: horizontal line y=0
    solver.add_point(Point::new("la".into(), -1.0, 0.0, true));
    solver.add_point(Point::new("lb".into(), 1.0, 0.0, true));

    solver.add_point(Point::new("p".into(), 1.0, 4.0, true));
    solver.add_point(Point::new("q".into(), 1.0, 2.0, false)); // will move to (1,-4)

    solver
        .add_constraint(ConstraintType::Symmetric(
            "p".into(),
            "q".into(),
            "la".into(),
            "lb".into(),
        ))
        .expect("add symmetric");

    solver.set_max_iterations(500);
    solver.solve().expect("solve");

    let q = solver.get_point("q".into()).unwrap();
    assert!(
        (q.x - 1.0).abs() < 1e-2 && (q.y - (-4.0)).abs() < 1e-2,
        "q should be (1, -4), got ({}, {})",
        q.x,
        q.y
    );
}
