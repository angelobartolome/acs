//! Ellipses through `ConstraintSolver`: a center Point, a focus Point and a
//! minor radius (`radmin`) the solver may move, with `PointOnEllipse`,
//! `TangentLineEllipse`, `EllipseAxisPoint` and `EllipseDiameter`.

use acs::{
    ConstraintSolver, ConstraintType, Ellipse, EllipseAxis, Operand, Point, SolverResult,
};

const TOL: f64 = 1e-8;

fn s(id: &str) -> String {
    id.to_string()
}

fn xy(solver: &ConstraintSolver, id: &str) -> (f64, f64) {
    let p = solver.get_point(s(id)).unwrap();
    (p.x, p.y)
}

fn dist((ax, ay): (f64, f64), (bx, by): (f64, f64)) -> f64 {
    (bx - ax).hypot(by - ay)
}

fn assert_converged(solver: &mut ConstraintSolver) {
    let result = solver.solve().unwrap();
    assert!(matches!(result, SolverResult::Converged { .. }), "{result:?}");
}

/// An ellipse `e` with center `c` and focus `f`; `fixed` holds all three
/// (Points and `radmin`).
fn add_ellipse(solver: &mut ConstraintSolver, c: (f64, f64), f: (f64, f64), radmin: f64, fixed: bool) {
    solver.add_point(Point::new(s("c"), c.0, c.1, fixed));
    solver.add_point(Point::new(s("f"), f.0, f.1, fixed));
    solver.add_ellipse(Ellipse::new(s("e"), s("c"), s("f"), radmin, fixed));
}

/// The axis-aligned 5 × 4 ellipse at the origin: foci (±3, 0), a = 5, b = 4.
fn fixed_5_4_ellipse(solver: &mut ConstraintSolver) {
    add_ellipse(solver, (0.0, 0.0), (3.0, 0.0), 4.0, true);
}

/// Sum of the distances from `p` to the foci of the ellipse `e`.
fn focal_sum(solver: &ConstraintSolver, p: (f64, f64)) -> f64 {
    let (c, f) = (xy(solver, "c"), xy(solver, "f"));
    let f2 = (2.0 * c.0 - f.0, 2.0 * c.1 - f.1);
    dist(p, f) + dist(p, f2)
}

fn major_radius(solver: &ConstraintSolver) -> f64 {
    let b = solver.get_ellipse(s("e")).unwrap().radmin;
    b.hypot(dist(xy(solver, "c"), xy(solver, "f")))
}

#[test]
fn a_point_is_pulled_onto_a_fixed_ellipse() {
    let mut solver = ConstraintSolver::new();
    fixed_5_4_ellipse(&mut solver);
    solver.add_point(Point::new(s("p"), 1.0, 6.0, false));
    solver
        .add_constraint(ConstraintType::PointOnEllipse(s("p"), s("c"), s("f"), s("e")))
        .unwrap();
    assert_converged(&mut solver);
    let (x, y) = xy(&solver, "p");
    assert!((x * x / 25.0 + y * y / 16.0 - 1.0).abs() < TOL, "({x}, {y})");
    assert!((focal_sum(&solver, (x, y)) - 10.0).abs() < TOL);
}

#[test]
fn a_fixed_point_on_the_ellipse_sets_its_minor_radius() {
    let mut solver = ConstraintSolver::new();
    solver.add_point(Point::new(s("c"), 0.0, 0.0, true));
    solver.add_point(Point::new(s("f"), 3.0, 0.0, true));
    solver.add_ellipse(Ellipse::new(s("e"), s("c"), s("f"), 4.0, false));
    solver.add_point(Point::new(s("p"), 0.0, 6.0, true));
    solver
        .add_constraint(ConstraintType::PointOnEllipse(s("p"), s("c"), s("f"), s("e")))
        .unwrap();
    assert_converged(&mut solver);
    // (0, 6) is a co-vertex: b = 6.
    assert!((solver.get_ellipse(s("e")).unwrap().radmin - 6.0).abs() < TOL);
}

/// A horizontal Line of length 4 with fixed x, free to move up and down.
fn horizontal_line(solver: &mut ConstraintSolver, y: f64, x0: f64) {
    solver.add_point(Point::new(s("la"), x0, y, false));
    solver.add_point(Point::new(s("lb"), x0 + 4.0, y + 0.5, false));
    for c in [
        ConstraintType::Horizontal(s("la"), s("lb")),
        ConstraintType::EqualX(s("la"), x0),
        ConstraintType::EqualX(s("lb"), x0 + 4.0),
    ] {
        solver.add_constraint(c).unwrap();
    }
}

#[test]
fn a_line_becomes_tangent_to_a_fixed_ellipse() {
    let mut solver = ConstraintSolver::new();
    fixed_5_4_ellipse(&mut solver);
    horizontal_line(&mut solver, 6.0, -2.0);
    solver
        .add_constraint(ConstraintType::TangentLineEllipse(s("la"), s("lb"), s("c"), s("f"), s("e")))
        .unwrap();
    assert_converged(&mut solver);
    // Touches the top co-vertex (0, 4).
    assert!((xy(&solver, "la").1 - 4.0).abs() < TOL);
    assert!((xy(&solver, "lb").1 - 4.0).abs() < TOL);
}

/// A rotated ellipse: center (1, 2), major axis at 30°, a = 5, b = 3 (so the
/// focal distance is 4).
fn rotated() -> ((f64, f64), (f64, f64), (f64, f64)) {
    let (c, u) = ((1.0, 2.0), (30f64.to_radians().cos(), 30f64.to_radians().sin()));
    (c, (c.0 + 4.0 * u.0, c.1 + 4.0 * u.1), u)
}

#[test]
fn a_free_line_becomes_tangent_to_a_rotated_ellipse() {
    let mut solver = ConstraintSolver::new();
    let (c, f, _) = rotated();
    add_ellipse(&mut solver, c, f, 3.0, true);
    solver.add_point(Point::new(s("la"), -6.0, 7.0, false));
    solver.add_point(Point::new(s("lb"), 8.0, 9.0, false));
    solver
        .add_constraint(ConstraintType::TangentLineEllipse(s("la"), s("lb"), s("c"), s("f"), s("e")))
        .unwrap();
    assert_converged(&mut solver);

    // Tangent: the line meets the ellipse in exactly one point. Sample the
    // segment: every point is on or outside, and the closest is on it.
    let (a, b) = (xy(&solver, "la"), xy(&solver, "lb"));
    let min = (0..=20000)
        .map(|i| {
            let t = i as f64 / 20000.0;
            focal_sum(&solver, (a.0 + t * (b.0 - a.0), a.1 + t * (b.1 - a.1))) - 10.0
        })
        .fold(f64::INFINITY, f64::min);
    assert!(min > -1e-9, "the segment cuts into the ellipse: {min}");
    assert!(min < 1e-5, "the segment doesn't reach the ellipse: {min}");
}

/// A Line is a segment: a vertical Line of length 1 above the ellipse's
/// right vertex isn't tangent until it slides down to reach the vertex.
#[test]
fn the_tangency_point_lies_on_the_segment() {
    let mut solver = ConstraintSolver::new();
    fixed_5_4_ellipse(&mut solver);
    solver.add_point(Point::new(s("la"), 6.0, 2.0, false));
    solver.add_point(Point::new(s("lb"), 6.0, 3.0, false));
    for c in [
        ConstraintType::Vertical(s("la"), s("lb")),
        ConstraintType::DistancePointPoint(s("la"), s("lb"), 1.0),
        ConstraintType::TangentLineEllipse(s("la"), s("lb"), s("c"), s("f"), s("e")),
    ] {
        solver.add_constraint(c).unwrap();
    }
    assert_converged(&mut solver);
    let (a, b) = (xy(&solver, "la"), xy(&solver, "lb"));
    assert!((a.0 - 5.0).abs() < TOL && (b.0 - 5.0).abs() < TOL, "{a:?} {b:?}");
    let (lo, hi) = (a.1.min(b.1), a.1.max(b.1));
    assert!(lo <= TOL && hi >= -TOL, "tangency point (5, 0) off the segment: {a:?} {b:?}");
}

/// An ellipse tool's sketch: an ellipse, its four axis points held by the
/// major and minor diameter alignments, and p2p distances fixing the
/// diameters. Every point starts a little off.
#[test]
fn the_ellipse_tool_sketch_solves_with_its_axis_points() {
    let mut solver = ConstraintSolver::new();
    let (c, f, u) = rotated();
    add_ellipse(&mut solver, c, (f.0 + 0.2, f.1 - 0.1), 2.7, false);
    let n = (-u.1, u.0);
    for (id, dir, r) in [("maj1", u, 5.0), ("maj2", u, -5.0), ("min1", n, 3.0), ("min2", n, -3.0)] {
        solver.add_point(Point::new(s(id), c.0 + r * dir.0 + 0.1, c.1 + r * dir.1 - 0.2, false));
    }
    for c in [
        ConstraintType::EllipseDiameter(s("maj1"), s("maj2"), s("c"), s("f"), s("e"), EllipseAxis::Major),
        ConstraintType::EllipseDiameter(s("min1"), s("min2"), s("c"), s("f"), s("e"), EllipseAxis::Minor),
        ConstraintType::DistancePointPoint(s("maj1"), s("maj2"), 10.0),
        ConstraintType::DistancePointPoint(s("min1"), s("min2"), 6.0),
    ] {
        solver.add_constraint(c).unwrap();
    }
    assert_converged(&mut solver);

    assert!((major_radius(&solver) - 5.0).abs() < TOL);
    assert!((solver.get_ellipse(s("e")).unwrap().radmin - 3.0).abs() < TOL);
    let (c, f) = (xy(&solver, "c"), xy(&solver, "f"));
    let k = dist(c, f);
    let u = ((f.0 - c.0) / k, (f.1 - c.1) / k);
    let n = (-u.1, u.0);
    let at = |id: &str, dir: (f64, f64), r: f64| {
        let p = xy(&solver, id);
        assert!(dist(p, (c.0 + r * dir.0, c.1 + r * dir.1)) < 1e-7, "{id} = {p:?}");
    };
    // Each point keeps the end it started by.
    at("maj1", u, 5.0);
    at("maj2", u, -5.0);
    at("min1", n, 3.0);
    at("min2", n, -3.0);
    // Free: the ellipse's position and rotation (3 DOF).
    assert_eq!(solver.dof(), 3);
}

#[test]
fn a_free_ellipse_has_five_degrees_of_freedom_and_its_diameters_add_none() {
    let mut solver = ConstraintSolver::new();
    add_ellipse(&mut solver, (0.0, 0.0), (3.0, 0.0), 4.0, false);
    // Any constraint over the ellipse puts its variables into a Component.
    solver.add_point(Point::new(s("p"), 0.0, 4.0, false));
    solver
        .add_constraint(ConstraintType::PointOnEllipse(s("p"), s("c"), s("f"), s("e")))
        .unwrap();
    assert_eq!(solver.dof(), 5 + 2 - 1);

    for (a, b, x, axis) in [("m1", "m2", 5.0, EllipseAxis::Major), ("n1", "n2", 4.0, EllipseAxis::Minor)] {
        let (p1, p2) = match axis {
            EllipseAxis::Major => ((x, 0.0), (-x, 0.0)),
            EllipseAxis::Minor => ((0.0, x), (0.0, -x)),
        };
        solver.add_point(Point::new(s(a), p1.0, p1.1, false));
        solver.add_point(Point::new(s(b), p2.0, p2.1, false));
        solver
            .add_constraint(ConstraintType::EllipseDiameter(s(a), s(b), s("c"), s("f"), s("e"), axis))
            .unwrap();
    }
    assert_converged(&mut solver);
    assert_eq!(solver.dof(), 5 + 2 - 1);
    assert!(solver.diagnose().redundant.is_empty());
}

#[test]
fn single_axis_points_sit_at_either_end() {
    let mut solver = ConstraintSolver::new();
    fixed_5_4_ellipse(&mut solver);
    for (id, x, y, axis) in [
        ("a", 4.6, 0.3, EllipseAxis::Major),
        ("b", -5.2, -0.4, EllipseAxis::Major),
        ("m", 0.3, 3.5, EllipseAxis::Minor),
        ("n", -0.2, -4.4, EllipseAxis::Minor),
    ] {
        solver.add_point(Point::new(s(id), x, y, false));
        solver
            .add_constraint(ConstraintType::EllipseAxisPoint(s(id), s("c"), s("f"), s("e"), axis))
            .unwrap();
    }
    assert_converged(&mut solver);
    for (id, x, y) in [("a", 5.0, 0.0), ("b", -5.0, 0.0), ("m", 0.0, 4.0), ("n", 0.0, -4.0)] {
        assert!(dist(xy(&solver, id), (x, y)) < TOL, "{id} = {:?}", xy(&solver, id));
    }
}

#[test]
fn dragging_a_major_axis_point_stretches_the_ellipse() {
    let mut solver = ConstraintSolver::new();
    solver.add_point(Point::new(s("c"), 0.0, 0.0, true));
    solver.add_point(Point::new(s("f"), 3.0, 0.0, false));
    solver.add_ellipse(Ellipse::new(s("e"), s("c"), s("f"), 4.0, false));
    solver.add_point(Point::new(s("m1"), 5.0, 0.0, false));
    solver.add_point(Point::new(s("m2"), -5.0, 0.0, false));
    for c in [
        ConstraintType::EllipseDiameter(s("m1"), s("m2"), s("c"), s("f"), s("e"), EllipseAxis::Major),
        // Hold radmin at 4 (a property reference, as in `equal`).
        ConstraintType::Equal(Operand::MinorRadius(s("e")), Operand::Const(4.0)),
    ] {
        solver.add_constraint(c).unwrap();
    }
    solver.add_temporary_constraint(ConstraintType::EqualX(s("m1"), 7.0)).unwrap();
    solver.add_temporary_constraint(ConstraintType::EqualY(s("m1"), 0.0)).unwrap();
    assert_converged(&mut solver);
    assert!(dist(xy(&solver, "m1"), (7.0, 0.0)) < 1e-6, "{:?}", xy(&solver, "m1"));
    assert!((major_radius(&solver) - 7.0).abs() < 1e-6);
    assert!((solver.get_ellipse(s("e")).unwrap().radmin - 4.0).abs() < TOL);
}

#[test]
fn fixed_radius_does_not_take_an_ellipse() {
    let mut solver = ConstraintSolver::new();
    fixed_5_4_ellipse(&mut solver);
    let err = solver
        .add_constraint(ConstraintType::FixedRadius(s("e"), 4.0))
        .unwrap_err();
    assert!(err.contains("'e' is not a circle or arc"), "{err}");
}

/// A fixed ellipse (Reference Geometry is always fixed) with fixed center and
/// focus never moves, whatever the constraints ask.
#[test]
fn a_fixed_ellipse_never_moves() {
    let mut solver = ConstraintSolver::new();
    fixed_5_4_ellipse(&mut solver);
    // A fixed Line at y = 6 can't be tangent to it.
    solver.add_point(Point::new(s("la"), -2.0, 6.0, true));
    solver.add_point(Point::new(s("lb"), 2.0, 6.0, true));
    solver
        .add_constraint(ConstraintType::TangentLineEllipse(s("la"), s("lb"), s("c"), s("f"), s("e")))
        .unwrap();
    solver.add_point(Point::new(s("p"), 1.0, 6.0, false));
    solver
        .add_constraint(ConstraintType::PointOnEllipse(s("p"), s("c"), s("f"), s("e")))
        .unwrap();
    let result = solver.solve().unwrap();
    assert!(matches!(result, SolverResult::MaxIterationsReached { .. }), "{result:?}");
    assert_eq!(xy(&solver, "c"), (0.0, 0.0));
    assert_eq!(xy(&solver, "f"), (3.0, 0.0));
    assert_eq!(solver.get_ellipse(s("e")).unwrap().radmin, 4.0);
}
