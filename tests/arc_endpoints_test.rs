//! An Arc keeps its start and end Points on itself (at its radius from its
//! center, at its start and end angles) without any constraint saying so.
//! Tested through `ConstraintSolver` so the JSON vocabulary can change
//! without touching these.

use acs::{Arc, ConstraintSolver, ConstraintType, Point, SolverResult};
use std::f64::consts::FRAC_PI_2;

fn converged(result: SolverResult) {
    assert!(matches!(result, SolverResult::Converged { .. }), "{result:?}");
}

fn xy(s: &ConstraintSolver, id: &str) -> (f64, f64) {
    let p = s.get_point(id.into()).unwrap();
    (p.x, p.y)
}

/// Asserts arc `id`'s endpoints sit on it.
fn assert_on_arc(s: &ConstraintSolver, id: &str) {
    let a = s.get_arc(id.into()).unwrap().clone();
    let (cx, cy) = xy(s, &a.center);
    for (end, t) in [(&a.start, a.start_angle), (&a.end, a.end_angle)] {
        let (x, y) = xy(s, end);
        assert!(
            (x - cx - a.radius * t.cos()).abs() < 1e-9 && (y - cy - a.radius * t.sin()).abs() < 1e-9,
            "{end} ({x}, {y}) is off {a:?} centered at ({cx}, {cy})"
        );
    }
}

/// A free quarter arc of radius 10 around the origin, its endpoints at `s`
/// and `e`; `fixed` fixes the arc and its three Points (Reference Geometry).
fn quarter_arc(s: (f64, f64), e: (f64, f64), fixed: bool) -> ConstraintSolver {
    let mut solver = ConstraintSolver::new();
    solver.add_point(Point::new("c".into(), 0.0, 0.0, fixed));
    solver.add_point(Point::new("s".into(), s.0, s.1, fixed));
    solver.add_point(Point::new("e".into(), e.0, e.1, fixed));
    solver.add_arc(Arc::new(
        "a".into(),
        "c".into(),
        "s".into(),
        "e".into(),
        10.0,
        0.0,
        FRAC_PI_2,
        fixed,
    ));
    solver
}

#[test]
fn endpoints_off_the_arc_land_on_it_without_any_constraint() {
    let mut s = quarter_arc((12.0, 1.0), (-1.0, 9.0), false);
    converged(s.solve().unwrap());
    assert_on_arc(&s, "a");
}

#[test]
fn a_free_arc_has_five_degrees_of_freedom_with_its_endpoints_determined() {
    let s = quarter_arc((10.0, 0.0), (0.0, 10.0), false);
    // Center (2) + radius (1) + angles (2); the endpoints follow.
    assert_eq!(s.dof(), 5);
}

#[test]
fn a_fixed_arc_around_a_fixed_center_fully_constrains_its_endpoints() {
    let mut s = ConstraintSolver::new();
    s.add_point(Point::new("c".into(), 0.0, 0.0, true));
    s.add_point(Point::new("s".into(), 11.0, 1.0, false));
    s.add_point(Point::new("e".into(), 1.0, 11.0, false));
    s.add_arc(Arc::new("a".into(), "c".into(), "s".into(), "e".into(), 10.0, 0.0, FRAC_PI_2, true));
    converged(s.solve().unwrap());
    assert_eq!(xy(&s, "s"), (10.0, 0.0));
    assert!((xy(&s, "e").0).abs() < 1e-12 && (xy(&s, "e").1 - 10.0).abs() < 1e-12);
    assert_eq!(s.dof(), 0);
    let fully = s.fully_constrained_entity_ids();
    for id in ["a", "c", "e", "s"] {
        assert!(fully.contains(&id.to_string()), "{id} not in {fully:?}");
    }
}

#[test]
fn a_reference_arc_and_its_endpoints_never_move() {
    let mut s = quarter_arc((10.0, 0.0), (0.0, 10.0), true);
    // A Line from the arc's end to a free point, pulled to a length that a
    // free arc could have helped meet.
    s.add_point(Point::new("q".into(), -3.0, 10.0, false));
    s.add_constraint(ConstraintType::Horizontal("e".into(), "q".into())).unwrap();
    s.add_constraint(ConstraintType::DistancePointPoint("e".into(), "q".into(), 4.0))
        .unwrap();
    // Drag the arc's start point away.
    s.add_temporary_constraint(ConstraintType::EqualX("s".into(), 20.0)).unwrap();
    s.add_temporary_constraint(ConstraintType::EqualY("s".into(), 5.0)).unwrap();
    let before = s.get_arc("a".into()).unwrap().clone();

    converged(s.solve().unwrap());
    assert_eq!(s.get_arc("a".into()).unwrap(), &before);
    assert_eq!(xy(&s, "c"), (0.0, 0.0));
    assert_eq!(xy(&s, "s"), (10.0, 0.0));
    assert_eq!(xy(&s, "e"), (0.0, 10.0));
    let d = s.diagnose();
    assert!(d.conflicting.is_empty() && d.redundant.is_empty(), "{d:?}");
}

#[test]
fn dragging_an_arc_endpoint_keeps_it_on_the_arc() {
    let mut s = quarter_arc((10.0, 0.0), (0.0, 10.0), false);
    // The center and radius are held; only the end angle can follow the drag.
    s.add_constraint(ConstraintType::EqualX("c".into(), 0.0)).unwrap();
    s.add_constraint(ConstraintType::EqualY("c".into(), 0.0)).unwrap();
    s.add_constraint(ConstraintType::FixedRadius("a".into(), 10.0)).unwrap();
    s.add_temporary_constraint(ConstraintType::EqualX("e".into(), -20.0)).unwrap();
    s.add_temporary_constraint(ConstraintType::EqualY("e".into(), 20.0)).unwrap();

    converged(s.solve().unwrap());
    // Held exactly while the goal drags.
    assert_on_arc(&s, "a");
    // Near the closest point to the cursor on the circle, at 135° (the goal
    // step converges slowly on a curved constraint, so only roughly).
    let (ex, ey) = xy(&s, "e");
    let h = 10.0 / 2f64.sqrt();
    assert!((ex + h).abs() < 0.1 && (ey - h).abs() < 0.1, "({ex}, {ey})");
}

#[test]
fn a_constraint_contradicting_the_arc_is_conflicting_and_the_arc_is_not_reported() {
    let mut s = ConstraintSolver::new();
    s.add_point(Point::new("c".into(), 0.0, 0.0, true));
    s.add_point(Point::new("s".into(), 10.0, 0.0, false));
    s.add_point(Point::new("e".into(), 0.0, 10.0, false));
    s.add_arc(Arc::new("a".into(), "c".into(), "s".into(), "e".into(), 10.0, 0.0, FRAC_PI_2, true));
    // The start Point must be 7 from the center, but the arc's radius is a fixed 10.
    s.add_constraint(ConstraintType::DistancePointPoint("c".into(), "s".into(), 7.0))
        .unwrap();

    assert!(matches!(s.solve().unwrap(), SolverResult::MaxIterationsReached { .. }));
    let d = s.diagnose();
    assert_eq!(d.conflicting, vec![0], "{d:?}");
    assert!(d.redundant.is_empty(), "{d:?}");
}

#[test]
fn a_constraint_the_arc_already_implies_is_redundant() {
    let mut s = quarter_arc((10.0, 0.0), (0.0, 10.0), false);
    // The radius already puts the start Point 10 from the center.
    s.add_constraint(ConstraintType::FixedRadius("a".into(), 10.0)).unwrap();
    s.add_constraint(ConstraintType::DistancePointPoint("c".into(), "s".into(), 10.0))
        .unwrap();
    converged(s.solve().unwrap());
    let d = s.diagnose();
    assert_eq!(d.redundant, vec![1], "{d:?}");
    assert!(d.conflicting.is_empty(), "{d:?}");
}

#[test]
fn lines_sharing_endpoints_of_two_arcs_chain_them() {
    // Two arcs joined by a Line from the first's end to the second's start.
    let mut s = quarter_arc((10.0, 0.0), (0.0, 10.0), false);
    s.add_point(Point::new("c2".into(), 30.0, 0.0, false));
    s.add_point(Point::new("s2".into(), 25.0, 0.0, false));
    s.add_point(Point::new("e2".into(), 30.0, 5.0, false));
    s.add_arc(Arc::new("b".into(), "c2".into(), "s2".into(), "e2".into(), 5.0, 1.0, 2.0, false));
    s.add_constraint(ConstraintType::Horizontal("e".into(), "s2".into())).unwrap();
    converged(s.solve().unwrap());
    assert_on_arc(&s, "a");
    assert_on_arc(&s, "b");
    assert!((xy(&s, "e").1 - xy(&s, "s2").1).abs() < 1e-9);
    // Two free arcs (5 each) less one Horizontal.
    assert_eq!(s.dof(), 9);
}
