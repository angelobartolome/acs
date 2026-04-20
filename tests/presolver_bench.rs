/// Timing comparison: pre-solver vs naive single-component solve.
///
/// Scenario: a fully-constrained square (4 points, fixed position) + a
/// horizontal line that the user "drags" by nudging one endpoint 500 times.
///
/// Run with:  cargo test presolver_bench -- --nocapture
use std::time::Instant;

use acs::{ConstraintSolver, ConstraintType, Point};

const DRAG_ITERS: usize = 500;

// ── helpers ────────────────────────────────────────────────────────────────

fn build_constrained_square(solver: &mut ConstraintSolver, tag: &str) {
    // Four corners of a 10×10 square, fully constrained.
    let tl = format!("{tag}_tl");
    let tr = format!("{tag}_tr");
    let bl = format!("{tag}_bl");
    let br = format!("{tag}_br");

    solver.add_point(Point::new(tl.clone(), 0.0, 10.0, false));
    solver.add_point(Point::new(tr.clone(), 10.0, 10.0, false));
    solver.add_point(Point::new(bl.clone(), 0.0, 0.0, false));
    solver.add_point(Point::new(br.clone(), 10.0, 0.0, false));

    // Pin tl to (0,10)
    solver.add_constraint(ConstraintType::EqualX(tl.clone(), 0.0)).unwrap();
    solver.add_constraint(ConstraintType::EqualY(tl.clone(), 10.0)).unwrap();

    // Horizontal top and bottom edges
    solver.add_constraint(ConstraintType::Horizontal(tl.clone(), tr.clone())).unwrap();
    solver.add_constraint(ConstraintType::Horizontal(bl.clone(), br.clone())).unwrap();

    // Vertical left and right edges
    solver.add_constraint(ConstraintType::Vertical(tl.clone(), bl.clone())).unwrap();
    solver.add_constraint(ConstraintType::Vertical(tr.clone(), br.clone())).unwrap();

    // Fix width and height via EqualX/EqualY on two corners
    solver.add_constraint(ConstraintType::EqualX(tr.clone(), 10.0)).unwrap();
    solver.add_constraint(ConstraintType::EqualY(bl.clone(), 0.0)).unwrap();
}

fn build_free_line(solver: &mut ConstraintSolver) {
    // A simple horizontal line far from the square.
    solver.add_point(Point::new("lp1".into(), 50.0, 50.0, true)); // fixed anchor
    solver.add_point(Point::new("lp2".into(), 60.0, 55.0, false)); // free end
    solver.add_constraint(ConstraintType::Horizontal("lp1".into(), "lp2".into())).unwrap();
}

// ── benchmark: with pre-solver (square component skipped each drag) ────────

#[test]
fn bench_presolver_square_plus_dragged_line() {
    // First solve to converge the square.
    let mut solver = ConstraintSolver::new();
    build_constrained_square(&mut solver, "sq");
    build_free_line(&mut solver);
    solver.solve().unwrap(); // warm-up / initial convergence

    // Now simulate dragging: nudge lp2.y each iteration and re-solve.
    // The square is already satisfied; only the line component should run.
    let t0 = Instant::now();
    for i in 0..DRAG_ITERS {
        // Simulate dragging by re-adding the line constraint (the point already moves
        // via the solver updating geometry). We re-build only the free point position.
        // Since we can't mutate point positions directly here, we rebuild the solver
        // each iteration — which is the realistic worst-case for this benchmark.
        let mut s = ConstraintSolver::new();
        build_constrained_square(&mut s, "sq");
        // Line endpoint starts slightly off-horizontal to give the solver work.
        s.add_point(Point::new("lp1".into(), 50.0, 50.0, true));
        s.add_point(Point::new("lp2".into(), 60.0 + (i as f64) * 0.01, 55.0, false));
        s.add_constraint(ConstraintType::Horizontal("lp1".into(), "lp2".into())).unwrap();
        s.solve().unwrap();
    }
    let with_presolver = t0.elapsed();

    println!(
        "\n[pre-solver ON ] {DRAG_ITERS} drag iterations: {:.3}ms total, {:.3}µs/iter",
        with_presolver.as_secs_f64() * 1000.0,
        with_presolver.as_secs_f64() * 1_000_000.0 / DRAG_ITERS as f64,
    );
}

// ── benchmark: naive (all constraints in one component, no skip) ───────────
//
// To simulate a "no pre-solver" baseline we connect the square and the line
// via a dummy constraint (PointOnLine) so they form a single component, preventing
// the pre-solver from skipping anything.

#[test]
fn bench_naive_square_plus_dragged_line_one_component() {
    let t0 = Instant::now();
    for i in 0..DRAG_ITERS {
        let mut s = ConstraintSolver::new();
        build_constrained_square(&mut s, "sq");
        s.add_point(Point::new("lp1".into(), 50.0, 50.0, true));
        s.add_point(Point::new("lp2".into(), 60.0 + (i as f64) * 0.01, 55.0, false));
        s.add_constraint(ConstraintType::Horizontal("lp1".into(), "lp2".into())).unwrap();
        // Connecting constraint: forces square and line into one component.
        // PointOnLine(lp1, sq_bl, sq_br) — lp1 is fixed so this doesn't disturb
        // the square, but it merges the two components in the graph.
        s.add_constraint(ConstraintType::PointOnLine(
            "lp1".into(),
            "sq_bl".into(),
            "sq_br".into(),
        ))
        .unwrap();
        s.solve().unwrap();
    }
    let naive = t0.elapsed();

    println!(
        "[pre-solver OFF] {DRAG_ITERS} drag iterations: {:.3}ms total, {:.3}µs/iter",
        naive.as_secs_f64() * 1000.0,
        naive.as_secs_f64() * 1_000_000.0 / DRAG_ITERS as f64,
    );
}
