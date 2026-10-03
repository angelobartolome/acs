//! Guides: a mirror's axis, a rotation's center and a translation's
//! direction. They are ordinary unknowns in every solve, drags included:
//! moving a Guide (by a drag or by other constraints) carries the copy with
//! it; dragging the source or the copy moves the other one, and also moves a
//! free Guide as needed (pin it to keep it put); every vertex of a patterned
//! polygon drags alike.

use acs::{Circle, ConstraintSolver, ConstraintType, Line, Point, SolverResult};

const EPS: f64 = 1e-8;

fn s(id: &str) -> String {
    id.to_string()
}

fn solver_with(points: &[(&str, f64, f64, bool)]) -> ConstraintSolver {
    let mut solver = ConstraintSolver::new();
    for &(id, x, y, fixed) in points {
        solver.add_point(Point::new(s(id), x, y, fixed));
    }
    solver
}

/// Holds `id` at `(x, y)` with temporary constraints, as a drag does.
fn drag(solver: &mut ConstraintSolver, id: &str, x: f64, y: f64) {
    solver
        .add_temporary_constraint(ConstraintType::EqualX(s(id), x))
        .unwrap();
    solver
        .add_temporary_constraint(ConstraintType::EqualY(s(id), y))
        .unwrap();
}

/// Holds `id` where it is with real constraints (a pinned Guide).
fn pin(solver: &mut ConstraintSolver, id: &str) {
    let (x, y) = at(solver, id);
    solver.add_constraint(ConstraintType::EqualX(s(id), x)).unwrap();
    solver.add_constraint(ConstraintType::EqualY(s(id), y)).unwrap();
}

fn solve(solver: &mut ConstraintSolver) {
    let result = solver.solve().unwrap();
    assert!(matches!(result, SolverResult::Converged { .. }), "{result:?}");
}

fn at(solver: &ConstraintSolver, id: &str) -> (f64, f64) {
    let p = solver.get_point(s(id)).unwrap();
    (p.x, p.y)
}

fn assert_at(solver: &ConstraintSolver, id: &str, x: f64, y: f64) {
    let (px, py) = at(solver, id);
    assert!(
        (px - x).abs() < EPS && (py - y).abs() < EPS,
        "{id} at ({px}, {py}), expected ({x}, {y})"
    );
}

// ── Mirror ──────────────────────────────────────────────────────────────────

/// Free axis m1 (0, 0) → m2 (2, 0); free source a (1, 3) and image b (1, −3).
fn mirror_sketch(source_fixed: bool) -> ConstraintSolver {
    let mut solver = solver_with(&[
        ("m1", 0.0, 0.0, false),
        ("m2", 2.0, 0.0, false),
        ("a", 1.0, 3.0, source_fixed),
        ("b", 1.0, -3.0, false),
    ]);
    solver
        .add_constraint(ConstraintType::MirrorPointExtension(s("a"), s("b"), s("m1"), s("m2")))
        .unwrap();
    solver
}

#[test]
fn moving_the_mirror_source_moves_the_image_across_a_pinned_axis() {
    let mut solver = mirror_sketch(false);
    pin(&mut solver, "m1");
    pin(&mut solver, "m2");
    drag(&mut solver, "a", 4.0, 5.0);
    solve(&mut solver);
    assert_at(&solver, "b", 4.0, -5.0);
    assert_at(&solver, "m1", 0.0, 0.0);
    assert_at(&solver, "m2", 2.0, 0.0);
}

#[test]
fn moving_the_mirror_image_moves_the_source_across_a_pinned_axis() {
    let mut solver = mirror_sketch(false);
    pin(&mut solver, "m1");
    pin(&mut solver, "m2");
    drag(&mut solver, "b", 2.0, -1.0);
    solve(&mut solver);
    assert_at(&solver, "a", 2.0, 1.0);
    assert_at(&solver, "m1", 0.0, 0.0);
    assert_at(&solver, "m2", 2.0, 0.0);
}

#[test]
fn moving_the_mirror_axis_moves_the_image() {
    let mut solver = mirror_sketch(true);
    pin(&mut solver, "m1");
    // The axis becomes the line y = x: (1, 3) mirrors to (3, 1).
    drag(&mut solver, "m2", 2.0, 2.0);
    solve(&mut solver);
    assert_at(&solver, "m2", 2.0, 2.0);
    assert_at(&solver, "m1", 0.0, 0.0);
    assert_at(&solver, "b", 3.0, 1.0);
}

#[test]
fn a_mirror_axis_moved_by_other_constraints_carries_the_image() {
    let mut solver = mirror_sketch(true);
    // m1 held at (0, 0) and m2 pinned to (0, 2) by permanent constraints: the
    // axis is x = 0. (With m1 free, the solve could move it as well.)
    solver.add_constraint(ConstraintType::EqualX(s("m1"), 0.0)).unwrap();
    solver.add_constraint(ConstraintType::EqualY(s("m1"), 0.0)).unwrap();
    solver.add_constraint(ConstraintType::EqualX(s("m2"), 0.0)).unwrap();
    solver.add_constraint(ConstraintType::EqualY(s("m2"), 2.0)).unwrap();
    solve(&mut solver);
    assert_at(&solver, "m2", 0.0, 2.0);
    assert_at(&solver, "b", -1.0, 3.0);
    assert_eq!(solver.diagnose(), Default::default());
}

#[test]
fn a_free_mirror_axis_keeps_its_degrees_of_freedom() {
    let mut solver = mirror_sketch(true);
    solve(&mut solver);
    // The axis's two free points keep 4 DOF; the image follows the (fixed)
    // source and the axis, so it moves when the axis does.
    assert_eq!(solver.dof(), 4);
    let full = solver.fully_constrained_entity_ids();
    assert!(!full.contains(&s("m1")) && !full.contains(&s("m2")), "{full:?}");
    assert!(!full.contains(&s("b")), "{full:?}");
    assert_eq!(solver.diagnose(), Default::default());
}

// ── Rotation ────────────────────────────────────────────────────────────────

/// Free center c (1, 1); p0 (3, 1) rotated a quarter turn is pk (1, 3).
fn rotation_sketch(source_fixed: bool) -> ConstraintSolver {
    let mut solver = solver_with(&[
        ("c", 1.0, 1.0, false),
        ("p0", 3.0, 1.0, source_fixed),
        ("pk", 1.0, 3.0, false),
    ]);
    solver
        .add_constraint(ConstraintType::CircularInstance(
            s("p0"),
            s("pk"),
            s("c"),
            std::f64::consts::FRAC_PI_2,
        ))
        .unwrap();
    solver
}

#[test]
fn moving_the_rotation_source_moves_the_copy_about_a_pinned_center() {
    let mut solver = rotation_sketch(false);
    pin(&mut solver, "c");
    drag(&mut solver, "p0", 4.0, 2.0);
    solve(&mut solver);
    assert_at(&solver, "pk", 0.0, 4.0);
    assert_at(&solver, "c", 1.0, 1.0);
}

#[test]
fn moving_the_rotation_copy_moves_the_source_about_a_pinned_center() {
    let mut solver = rotation_sketch(false);
    pin(&mut solver, "c");
    drag(&mut solver, "pk", 1.0, 5.0);
    solve(&mut solver);
    assert_at(&solver, "p0", 5.0, 1.0);
    assert_at(&solver, "c", 1.0, 1.0);
}

#[test]
fn moving_the_rotation_center_moves_the_copy() {
    let mut solver = rotation_sketch(true);
    drag(&mut solver, "c", 2.0, 1.0);
    solve(&mut solver);
    assert_at(&solver, "c", 2.0, 1.0);
    assert_at(&solver, "pk", 2.0, 2.0);
}

#[test]
fn a_rotation_center_moved_by_other_constraints_carries_the_copy() {
    let mut solver = rotation_sketch(true);
    solver.add_constraint(ConstraintType::EqualX(s("c"), 3.0)).unwrap();
    solver.add_constraint(ConstraintType::EqualY(s("c"), 3.0)).unwrap();
    solve(&mut solver);
    assert_at(&solver, "pk", 5.0, 3.0);
    assert_eq!(solver.diagnose(), Default::default());
}

#[test]
fn a_free_rotation_center_keeps_its_degrees_of_freedom() {
    let mut solver = rotation_sketch(true);
    solve(&mut solver);
    assert_eq!(solver.dof(), 2);
    let full = solver.fully_constrained_entity_ids();
    assert!(!full.contains(&s("c")), "{full:?}");
    // The copy follows the free center.
    assert!(!full.contains(&s("pk")), "{full:?}");
}

// ── Translation ─────────────────────────────────────────────────────────────

/// Free direction d1 (0, 0) → d2 (1, 0); pk is p0 (0, 1) moved 2·1.5 along it.
fn translation_sketch(source_fixed: bool) -> ConstraintSolver {
    let mut solver = solver_with(&[
        ("d1", 0.0, 0.0, false),
        ("d2", 1.0, 0.0, false),
        ("p0", 0.0, 1.0, source_fixed),
        ("pk", 3.0, 1.0, false),
    ]);
    solver
        .add_constraint(ConstraintType::LinearInstance(
            s("p0"),
            s("pk"),
            s("d1"),
            s("d2"),
            1.5,
            2.0,
        ))
        .unwrap();
    solver
}

#[test]
fn moving_the_translation_source_moves_the_copy_along_a_pinned_direction() {
    let mut solver = translation_sketch(false);
    pin(&mut solver, "d1");
    pin(&mut solver, "d2");
    drag(&mut solver, "p0", 1.0, 2.0);
    solve(&mut solver);
    assert_at(&solver, "pk", 4.0, 2.0);
    assert_at(&solver, "d1", 0.0, 0.0);
    assert_at(&solver, "d2", 1.0, 0.0);
}

#[test]
fn moving_the_translation_copy_moves_the_source_along_a_pinned_direction() {
    let mut solver = translation_sketch(false);
    pin(&mut solver, "d1");
    pin(&mut solver, "d2");
    drag(&mut solver, "pk", 5.0, -1.0);
    solve(&mut solver);
    assert_at(&solver, "p0", 2.0, -1.0);
    assert_at(&solver, "d1", 0.0, 0.0);
    assert_at(&solver, "d2", 1.0, 0.0);
}

#[test]
fn moving_the_translation_direction_moves_the_copy() {
    let mut solver = translation_sketch(true);
    pin(&mut solver, "d1");
    drag(&mut solver, "d2", 0.0, 4.0);
    solve(&mut solver);
    assert_at(&solver, "d2", 0.0, 4.0);
    assert_at(&solver, "d1", 0.0, 0.0);
    assert_at(&solver, "pk", 0.0, 4.0);
}

#[test]
fn a_translation_direction_moved_by_other_constraints_carries_the_copy() {
    let mut solver = translation_sketch(true);
    // d1 held at (0, 0) too: with it free, the solve could move it as well.
    solver.add_constraint(ConstraintType::EqualX(s("d1"), 0.0)).unwrap();
    solver.add_constraint(ConstraintType::EqualY(s("d1"), 0.0)).unwrap();
    solver.add_constraint(ConstraintType::EqualX(s("d2"), 0.0)).unwrap();
    solver.add_constraint(ConstraintType::EqualY(s("d2"), -1.0)).unwrap();
    solve(&mut solver);
    assert_at(&solver, "pk", 0.0, -2.0);
    assert_eq!(solver.diagnose(), Default::default());
}

#[test]
fn a_free_translation_direction_keeps_its_degrees_of_freedom() {
    let mut solver = translation_sketch(true);
    solve(&mut solver);
    assert_eq!(solver.dof(), 4);
    let full = solver.fully_constrained_entity_ids();
    assert!(!full.contains(&s("d1")) && !full.contains(&s("d2")), "{full:?}");
    // The copy follows the free direction.
    assert!(!full.contains(&s("pk")), "{full:?}");
}

// ── Components ──────────────────────────────────────────────────────────────

/// Two mirrors share an axis whose end is pinned by its own constraints:
/// the Guide joins them into one solve and both images follow it.
#[test]
fn copies_sharing_a_guide_follow_it_together() {
    let mut solver = solver_with(&[
        ("m1", 0.0, 0.0, true),
        ("m2", 2.0, 0.0, false),
        ("a", 1.0, 3.0, true),
        ("b", 1.0, -3.0, false),
        ("c", 2.0, 1.0, true),
        ("d", 2.0, -1.0, false),
    ]);
    for (src, img) in [("a", "b"), ("c", "d")] {
        solver
            .add_constraint(ConstraintType::MirrorPointExtension(s(src), s(img), s("m1"), s("m2")))
            .unwrap();
    }
    drag(&mut solver, "m2", 0.0, 2.0);
    solve(&mut solver);
    assert_at(&solver, "b", -1.0, 3.0);
    assert_at(&solver, "d", -2.0, 1.0);
}

/// Real constraints move a free axis: with source and image both fixed, the
/// solve turns the axis onto their perpendicular bisector.
#[test]
fn a_mirror_moves_a_free_axis_to_fit() {
    let mut solver = solver_with(&[
        ("m1", 0.0, 0.0, false),
        ("m2", 2.0, 0.0, false),
        ("a", 1.0, 3.0, true),
        ("b", 3.0, 1.0, true),
    ]);
    solver
        .add_constraint(ConstraintType::MirrorPointExtension(s("a"), s("b"), s("m1"), s("m2")))
        .unwrap();
    let result = solver.solve().unwrap();
    assert!(matches!(result, SolverResult::Converged { .. }), "{result:?}");
    // The perpendicular bisector of a (1, 3) and b (3, 1) is y = x.
    for m in ["m1", "m2"] {
        let (x, y) = at(&solver, m);
        assert!((x - y).abs() < 1e-8, "{m} at ({x}, {y}) is off y = x");
    }
    assert_eq!(solver.diagnose(), Default::default());
}

/// Soft goals and Guides together: the dragged source can only slide along
/// its line, so the drag goal can't be fully met. The source still slides as
/// close to the cursor as it can, the image follows, and the free mirror axis
/// never moves.
#[test]
fn constrained_drag_of_a_mirrored_point_across_a_pinned_axis() {
    let mut solver = solver_with(&[
        ("a1", 0.0, -5.0, false),
        ("a2", 0.0, 5.0, false),
        ("l1", 1.0, 0.0, true),
        ("l2", 1.0, 10.0, true),
        ("src", 1.0, 2.0, false),
        ("img", -1.0, 2.0, false),
    ]);
    solver
        .add_constraint(ConstraintType::PointOnLine(s("src"), s("l1"), s("l2")))
        .unwrap();
    solver
        .add_constraint(ConstraintType::MirrorPointExtension(s("src"), s("img"), s("a1"), s("a2")))
        .unwrap();
    pin(&mut solver, "a1");
    pin(&mut solver, "a2");
    drag(&mut solver, "src", 4.0, 6.0);

    let result = solver.solve().unwrap();
    assert!(matches!(result, SolverResult::Converged { .. }), "{result:?}");

    let p = |id: &str| {
        let q = solver.get_point(s(id)).unwrap();
        (q.x, q.y)
    };
    let ((a1x, a1y), (a2x, a2y)) = (p("a1"), p("a2"));
    assert!(a1x.abs() < EPS && (a1y + 5.0).abs() < EPS, "axis start moved: ({a1x}, {a1y})");
    assert!(a2x.abs() < EPS && (a2y - 5.0).abs() < EPS, "axis end moved: ({a2x}, {a2y})");
    let (sx, sy) = p("src");
    assert!((sx - 1.0).abs() < EPS && (sy - 6.0).abs() < 1e-6, "src slides along its line: ({sx}, {sy})");
    let (ix, iy) = p("img");
    assert!((ix + sx).abs() < EPS && (iy - sy).abs() < EPS, "image mirrors src: ({ix}, {iy})");
}

// ── Polygon ─────────────────────────────────────────────────────────────────

/// A pentagon built as a polygon tool builds it: one side tangent to a
/// dimensioned incircle about a free center, the other vertices rotated copies
/// of that side's endpoints about the center (every vertex shared by its two
/// sides, so 8 rotations, 4 of them implied by the others).
fn polygon(center_fixed: bool) -> ConstraintSolver {
    let (n, r, cx, cy) = (5usize, 20.0, 3.0, -2.0);
    let step = 2.0 * std::f64::consts::PI / n as f64;
    let big = r / (std::f64::consts::PI / n as f64).cos();
    let v = |k: usize| format!("v{}", k % n);
    let mut solver = solver_with(&[("c", cx, cy, center_fixed)]);
    for k in 0..n {
        let t = 0.3 + k as f64 * step;
        solver.add_point(Point::new(v(k), cx + big * t.cos(), cy + big * t.sin(), false));
        solver.add_line(Line::new(format!("l{k}"), v(k), v(k + 1)));
    }
    solver.add_circle(Circle::new(s("incircle"), s("c"), r, false));
    let constraints = [
        ConstraintType::FixedRadius(s("incircle"), r),
        ConstraintType::TangentLineCircle(s("v0"), s("v1"), s("c"), s("incircle")),
    ];
    for c in constraints {
        solver.add_constraint(c).unwrap();
    }
    for copy in 1..n {
        let angle = copy as f64 * step;
        for (source, image) in [(s("v0"), v(copy)), (s("v1"), v(copy + 1))] {
            solver
                .add_constraint(ConstraintType::CircularInstance(source, image, s("c"), angle))
                .unwrap();
        }
    }
    solver
}

/// Horizontal on the bottom side (l3, between the two lowest vertices) has to
/// turn the polygon, and with the center free the tangency moves the center
/// as it does: the rotations move it with them, so the solve converges.
#[test]
fn a_patterned_polygon_turns_about_a_free_center() {
    for center_fixed in [true, false] {
        let mut solver = polygon(center_fixed);
        solver.add_constraint(ConstraintType::Horizontal(s("v3"), s("v4"))).unwrap();
        let result = solver.solve().unwrap();
        assert!(matches!(result, SolverResult::Converged { .. }), "center fixed {center_fixed}: {result:?}");
        let ((_, y3), (_, y4)) = (at(&solver, "v3"), at(&solver, "v4"));
        assert!((y3 - y4).abs() < EPS, "center fixed {center_fixed}: bottom side not horizontal");
        assert!(solver.diagnose().conflicting.is_empty(), "center fixed {center_fixed}");
    }
}

/// Every vertex of the pattern drags the same way, copies included: the
/// polygon follows the cursor (its center is free, so it can move) and stays
/// regular. Holding the center during a drag made a copy's drag chase the
/// center the tangency moved, and the solve blew up.
#[test]
fn every_vertex_of_a_patterned_polygon_drags_alike() {
    for k in 0..5 {
        let mut solver = polygon(false);
        let v = format!("v{k}");
        let (x, y) = at(&solver, &v);
        drag(&mut solver, &v, x + 3.0, y + 2.0);
        solve(&mut solver);
        assert_at(&solver, &v, x + 3.0, y + 2.0);
        let side = |i: usize| {
            let (a, b) = (at(&solver, &format!("v{i}")), at(&solver, &format!("v{}", (i + 1) % 5)));
            (b.0 - a.0).hypot(b.1 - a.1)
        };
        for i in 1..5 {
            assert!((side(i) - side(0)).abs() < 1e-8, "v{k}: side {i} is {} vs {}", side(i), side(0));
        }
    }
}

/// Dragging a mirror's image when nothing pins the axis: the drag reaches the
/// cursor and the mirror still holds; the solver may move the axis too (the
/// smallest change), as it may any free geometry.
#[test]
fn dragging_a_mirror_image_across_a_free_axis_keeps_the_mirror() {
    let mut solver = mirror_sketch(false);
    drag(&mut solver, "b", 2.0, -1.0);
    solve(&mut solver);
    assert_at(&solver, "b", 2.0, -1.0);
    // a and b are mirror images across the line through m1 and m2.
    let ((ax, ay), (bx, by), (m1x, m1y), (m2x, m2y)) =
        (at(&solver, "a"), at(&solver, "b"), at(&solver, "m1"), at(&solver, "m2"));
    let (dx, dy) = (m2x - m1x, m2y - m1y);
    let mid = ((ax + bx) / 2.0, (ay + by) / 2.0);
    let on_axis = dx * (mid.1 - m1y) - dy * (mid.0 - m1x);
    let perpendicular = dx * (bx - ax) + dy * (by - ay);
    assert!(on_axis.abs() < 1e-8 && perpendicular.abs() < 1e-8, "not mirror images");
}
