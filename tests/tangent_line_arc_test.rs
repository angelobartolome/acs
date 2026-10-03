use acs::{Arc, ConstraintSolver, ConstraintType, Point, SolverResult};
use std::f64::consts::FRAC_PI_2;

/// A fixed quarter arc of radius 5 around the origin (angles 0 to π/2) and a
/// free Line from `a` to `b` made tangent to it. Returns the solved line.
fn solve_tangent_to_quarter_arc(a: (f64, f64), b: (f64, f64)) -> ((f64, f64), (f64, f64)) {
    let mut solver = ConstraintSolver::new();
    for (id, px, py) in [("c", 0.0, 0.0), ("s", 5.0, 0.0), ("e", 0.0, 5.0)] {
        solver.add_point(Point::new(id.into(), px, py, true));
    }
    solver.add_arc(Arc::new(
        "a".into(),
        "c".into(),
        "s".into(),
        "e".into(),
        5.0,
        0.0,
        FRAC_PI_2,
        true,
    ));
    solver.add_point(Point::new("la".into(), a.0, a.1, false));
    solver.add_point(Point::new("lb".into(), b.0, b.1, false));
    solver
        .add_constraint(ConstraintType::TangentLineArc(
            "la".into(),
            "lb".into(),
            "c".into(),
            "a".into(),
        ))
        .unwrap();

    let result = solver.solve().unwrap();
    assert!(matches!(result, SolverResult::Converged { .. }), "{result:?}");
    let p = |id: &str| {
        let p = solver.get_point(id.into()).unwrap();
        (p.x, p.y)
    };
    (p("la"), p("lb"))
}

/// The tangency point (foot of the perpendicular from the origin) and its
/// parameter along the line.
fn tangency((ax, ay): (f64, f64), (bx, by): (f64, f64)) -> ((f64, f64), f64) {
    let (dx, dy) = (bx - ax, by - ay);
    let t = -(ax * dx + ay * dy) / (dx * dx + dy * dy);
    ((ax + t * dx, ay + t * dy), t)
}

#[test]
fn line_becomes_tangent_on_the_arcs_span_and_the_segment() {
    let (a, b) = solve_tangent_to_quarter_arc((1.0, 7.0), (7.0, 1.0));
    let ((fx, fy), t) = tangency(a, b);
    assert!(((fx * fx + fy * fy).sqrt() - 5.0).abs() < 1e-8);
    assert!((-1e-8..=1.0 + 1e-8).contains(&t));
    assert!(fx >= -1e-8 && fy >= -1e-8, "on the quarter arc: ({fx}, {fy})");
}

/// Tangent to the arc's circle at (0, −5), which is off the arc's span: the
/// line is moved round until it touches the arc itself.
#[test]
fn tangency_off_the_span_is_moved_onto_it() {
    let (a, b) = solve_tangent_to_quarter_arc((-3.0, -5.0), (3.0, -5.0));
    let ((fx, fy), t) = tangency(a, b);
    assert!(((fx * fx + fy * fy).sqrt() - 5.0).abs() < 1e-8);
    assert!((-1e-8..=1.0 + 1e-8).contains(&t));
    assert!(fx >= -1e-8 && fy >= -1e-8, "on the quarter arc: ({fx}, {fy})");
}
