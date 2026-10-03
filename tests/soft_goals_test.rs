//! Temporary constraints are soft goals: the real constraints of a Component
//! hold exactly and the temporary ones (drag holds) are met as closely as the
//! real ones allow. Tested through `ConstraintSolver::add_temporary_constraint`
//! so the JSON vocabulary can change without touching these.

use acs::{Circle, ConstraintSolver, ConstraintType, Point, SolverResult};

fn converged(result: SolverResult) {
    assert!(matches!(result, SolverResult::Converged { .. }), "{result:?}");
}

fn xy(s: &ConstraintSolver, id: &str) -> (f64, f64) {
    let p = s.get_point(id.into()).unwrap();
    (p.x, p.y)
}

fn assert_close(actual: (f64, f64), expected: (f64, f64), what: &str) {
    assert!(
        (actual.0 - expected.0).abs() < 1e-8 && (actual.1 - expected.1).abs() < 1e-8,
        "{what}: expected {expected:?}, got {actual:?}"
    );
}

/// Holds `id` at the cursor (x, y) the way an editor drags a point.
fn drag(s: &mut ConstraintSolver, id: &str, x: f64, y: f64) {
    s.add_temporary_constraint(ConstraintType::EqualX(id.into(), x)).unwrap();
    s.add_temporary_constraint(ConstraintType::EqualY(id.into(), y)).unwrap();
}

/// A point at `p` on the fixed segment (0,0)-(10,5).
fn point_on_segment(p: (f64, f64)) -> ConstraintSolver {
    let mut s = ConstraintSolver::new();
    s.add_point(Point::new("a".into(), 0.0, 0.0, true));
    s.add_point(Point::new("b".into(), 10.0, 5.0, true));
    s.add_point(Point::new("p".into(), p.0, p.1, false));
    s.add_constraint(ConstraintType::PointOnLine("p".into(), "a".into(), "b".into()))
        .unwrap();
    s
}

/// Closest point to `c` on the segment (0,0)-(10,5).
fn closest_on_segment(c: (f64, f64)) -> (f64, f64) {
    let t = ((c.0 * 10.0 + c.1 * 5.0) / 125.0).clamp(0.0, 1.0);
    (10.0 * t, 5.0 * t)
}

#[test]
fn a_point_on_a_line_dragged_off_it_stays_on_it_closest_to_the_cursor() {
    let mut s = point_on_segment((4.0, 2.0));
    drag(&mut s, "p", 3.0, 6.0);
    converged(s.solve().unwrap());
    assert_close(xy(&s, "p"), closest_on_segment((3.0, 6.0)), "p");
    let d = s.diagnose();
    assert!(d.conflicting.is_empty() && d.redundant.is_empty(), "{d:?}");
    assert_eq!(s.dof(), 1);
}

#[test]
fn a_drag_sequence_slides_the_point_smoothly_along_the_line() {
    let dist = |a: (f64, f64), b: (f64, f64)| ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt();
    // The cursor sweeps an arc above the segment, past its far end.
    let cursor_at = |k: usize| {
        let a = k as f64 * 0.15;
        (4.0 + 7.0 * a.cos(), 3.0 + 7.0 * a.sin())
    };
    let mut p = (4.0, 2.0);
    let mut prev_cursor = p;
    for k in 0..=20 {
        let cursor = cursor_at(k);
        let mut s = point_on_segment(p);
        drag(&mut s, "p", cursor.0, cursor.1);
        converged(s.solve().unwrap());
        let next = xy(&s, "p");
        assert_close(next, closest_on_segment(cursor), &format!("step {k}"));
        // Smooth: the point never moves further than the cursor did.
        assert!(dist(next, p) <= dist(cursor, prev_cursor) + 1e-9, "step {k} jumped");
        p = next;
        prev_cursor = cursor;
    }
}

#[test]
fn a_point_dragged_past_the_segment_stops_at_its_end() {
    let mut s = point_on_segment((4.0, 2.0));
    drag(&mut s, "p", 14.0, 6.0);
    converged(s.solve().unwrap());
    assert_close(xy(&s, "p"), (10.0, 5.0), "p");
}

#[test]
fn dragging_a_fully_constrained_point_moves_nothing() {
    let mut s = ConstraintSolver::new();
    s.add_point(Point::new("a".into(), 0.0, 0.0, true));
    s.add_point(Point::new("p".into(), 3.0, 4.0, false));
    s.add_constraint(ConstraintType::Horizontal("a".into(), "p".into())).unwrap();
    s.add_constraint(ConstraintType::DistancePointPoint("a".into(), "p".into(), 5.0))
        .unwrap();
    converged(s.solve().unwrap());
    let solved = xy(&s, "p");

    // The same sketch, solved, then dragged.
    let mut s = ConstraintSolver::new();
    s.add_point(Point::new("a".into(), 0.0, 0.0, true));
    s.add_point(Point::new("p".into(), solved.0, solved.1, false));
    s.add_constraint(ConstraintType::Horizontal("a".into(), "p".into())).unwrap();
    s.add_constraint(ConstraintType::DistancePointPoint("a".into(), "p".into(), 5.0))
        .unwrap();
    drag(&mut s, "p", 9.0, 9.0);
    converged(s.solve().unwrap());
    assert_close(xy(&s, "p"), solved, "p");
    let d = s.diagnose();
    assert!(d.conflicting.is_empty() && d.redundant.is_empty(), "{d:?}");
    assert_eq!(s.dof(), 0);
    assert_eq!(s.fully_constrained_entity_ids(), ["a", "p"]);
}

/// A circle of radius 2 centered at (3, 0) on the fixed horizontal segment
/// (-100,0)-(100,0): its center is held on the segment.
fn circle_on_axis() -> ConstraintSolver {
    let mut s = ConstraintSolver::new();
    s.add_point(Point::new("l1".into(), -100.0, 0.0, true));
    s.add_point(Point::new("l2".into(), 100.0, 0.0, true));
    s.add_point(Point::new("cc".into(), 3.0, 0.0, false));
    s.add_circle(Circle::new("c".into(), "cc".into(), 2.0, false));
    s.add_constraint(ConstraintType::PointOnLine("cc".into(), "l1".into(), "l2".into()))
        .unwrap();
    s
}

/// A circle drag: the center at the cursor, the radius held.
fn drag_circle(s: &mut ConstraintSolver, x: f64, y: f64) {
    drag(s, "cc", x, y);
    s.add_temporary_constraint(ConstraintType::FixedRadius("c".into(), 2.0)).unwrap();
}

#[test]
fn a_dragged_circle_center_follows_the_cursor_where_the_constraints_allow() {
    let mut s = circle_on_axis();
    drag_circle(&mut s, 7.0, 5.0);
    converged(s.solve().unwrap());
    assert_close(xy(&s, "cc"), (7.0, 0.0), "center");
    assert!((s.get_circle("c".into()).unwrap().radius - 2.0).abs() < 1e-8);
}

#[test]
fn a_dragged_tangent_circle_stays_tangent() {
    // Tangent to the x axis: the center's height is the radius, so holding
    // both the cursor height and the radius can't be honoured exactly.
    let mut s = ConstraintSolver::new();
    s.add_point(Point::new("l1".into(), -100.0, 0.0, true));
    s.add_point(Point::new("l2".into(), 100.0, 0.0, true));
    s.add_point(Point::new("cc".into(), 3.0, 2.0, false));
    s.add_circle(Circle::new("c".into(), "cc".into(), 2.0, false));
    s.add_constraint(ConstraintType::TangentLineCircle(
        "l1".into(),
        "l2".into(),
        "cc".into(),
        "c".into(),
    ))
    .unwrap();
    drag_circle(&mut s, 7.0, 5.0);
    converged(s.solve().unwrap());
    let (x, y) = xy(&s, "cc");
    let r = s.get_circle("c".into()).unwrap().radius;
    assert!((y - r).abs() < 1e-9, "still tangent: y = {y}, r = {r}");
    // Least squares of (y - 5)² + (r - 2)² with y = r: halfway.
    assert!((x - 7.0).abs() < 1e-8 && (y - 3.5).abs() < 1e-7, "({x}, {y})");
}

#[test]
fn a_component_of_only_temporary_constraints_just_minimizes_them() {
    let mut s = ConstraintSolver::new();
    s.add_point(Point::new("p".into(), 0.0, 0.0, false));
    s.add_temporary_constraint(ConstraintType::EqualX("p".into(), 3.0)).unwrap();
    s.add_temporary_constraint(ConstraintType::EqualX("p".into(), 5.0)).unwrap();
    converged(s.solve().unwrap());
    assert!((xy(&s, "p").0 - 4.0).abs() < 1e-8);
    assert_eq!(s.dof(), 2);
}

#[test]
fn real_constraints_that_do_not_hold_are_solved_before_the_goals() {
    // The point starts off the segment and is dragged: it ends on it.
    let mut s = point_on_segment((0.0, 8.0));
    drag(&mut s, "p", 2.0, 7.0);
    converged(s.solve().unwrap());
    assert_close(xy(&s, "p"), closest_on_segment((2.0, 7.0)), "p");
}

#[test]
fn a_conflict_among_real_constraints_still_fails_and_is_reported() {
    let mut s = point_on_segment((4.0, 2.0));
    s.add_constraint(ConstraintType::EqualY("p".into(), 1.0)).unwrap();
    s.add_constraint(ConstraintType::EqualY("p".into(), 3.0)).unwrap();
    drag(&mut s, "p", 3.0, 6.0);
    assert!(matches!(s.solve().unwrap(), SolverResult::MaxIterationsReached { .. }));
    let d = s.diagnose();
    // Indices: 1 = y = 1, 2 = y = 3; the drag (3, 4) is never blamed.
    assert_eq!(d.conflicting, [1, 2]);
}
