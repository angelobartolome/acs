//! Shared geometry for constraints that measure against an Arc's span.
//!
//! An Arc sweeps counter-clockwise from its start angle to its end angle.
//! Its sweep is `(end − start) mod 2π` in `(0, 2π]`: equal angles mean a
//! full turn.

use std::f64::consts::TAU;

/// How far the direction angle `phi` (radians, around the arc's center)
/// falls outside the arc's span from `start` to `end`: 0 within it, else the
/// angle to the nearer end, positive past `end` and negative before `start`.
/// Returns the value and its partials w.r.t. `(phi, start, end)`.
pub(crate) fn span_overshoot(phi: f64, start: f64, end: f64) -> (f64, [f64; 3]) {
    let u = (phi - start).rem_euclid(TAU);
    let mut sweep = (end - start).rem_euclid(TAU);
    if sweep == 0.0 {
        sweep = TAU;
    }
    if u <= sweep {
        return (0.0, [0.0; 3]);
    }
    let past_end = u - sweep; // = phi − end (mod 2π)
    let before_start = TAU - u; // = start − phi (mod 2π)
    if past_end <= before_start {
        (past_end, [1.0, 0.0, -1.0])
    } else {
        (-before_start, [1.0, -1.0, 0.0])
    }
}
