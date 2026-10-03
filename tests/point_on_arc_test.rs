use acs::{Arc, ConstraintSolver, ConstraintType, Point, SolverResult};
use std::f64::consts::FRAC_PI_2;

/// A fixed quarter arc of radius 5 around the origin, from angle 0 to π/2,
/// and a free point `p` at (x, y) constrained onto it.
fn solve_point_on_quarter_arc(x: f64, y: f64) -> (f64, f64) {
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
    solver.add_point(Point::new("p".into(), x, y, false));
    solver
        .add_constraint(ConstraintType::PointOnArc("p".into(), "c".into(), "a".into()))
        .unwrap();

    let result = solver.solve().unwrap();
    assert!(matches!(result, SolverResult::Converged { .. }), "{result:?}");
    let p = solver.get_point("p".into()).unwrap();
    (p.x, p.y)
}

#[test]
fn point_inside_the_span_moves_radially_onto_the_arc() {
    let (x, y) = solve_point_on_quarter_arc(3.0, 3.0);
    let r = (x * x + y * y).sqrt();
    assert!((r - 5.0).abs() < 1e-8);
    assert!((x - y).abs() < 1e-6, "stays on the 45° ray: ({x}, {y})");
}

/// On the arc's circle but past its end: pulled back onto the span.
#[test]
fn point_on_the_circle_but_off_the_span_is_pulled_onto_it() {
    let (x, y) = solve_point_on_quarter_arc(-4.0, 3.0);
    assert!(((x * x + y * y).sqrt() - 5.0).abs() < 1e-8);
    assert!(x >= -1e-8 && y >= -1e-8, "in the first quadrant: ({x}, {y})");
}
