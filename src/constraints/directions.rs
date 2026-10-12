//! Shared plumbing for the kernels whose residual is a length or a relation
//! between two directions, never a product of lengths: [`unit`] for the
//! gradient of a distance `|v|`, and [`set_direction_row`] for a relation
//! between two lines' directions, written as the angle between them.
//!
//! A product of lengths rounds to about 2e-16 times the coordinates times
//! the lengths, which outgrows `TOLF` (1e-10) at coordinates in the
//! thousands: a sketch in mm that solves exactly reports `failed`.

use std::f64::consts::{FRAC_PI_2, PI};

use nalgebra::DMatrix;

use crate::constraints::set_row;

/// `v / |v|` and `|v|`: the gradient of the distance `|v|` with respect to
/// `v`. A zero `v` gets `(1, 0)`, one of the distance's subgradients there,
/// so a row like `|p − q| − d` keeps a direction to move along while `p`
/// and `q` coincide.
pub(crate) fn unit(vx: f64, vy: f64) -> ([f64; 2], f64) {
    let n = vx.hypot(vy);
    if n > 0.0 { ([vx / n, vy / n], n) } else { ([1.0, 0.0], 0.0) }
}

/// Row 0 of a constraint over lines p1→p2 and p3→p4 (variables `[p1, p2,
/// p3, p4]`, x then y), with directions `d1 = p2 − p1`, `d2 = p4 − p3`,
/// holding the angle φ from d1 to d2 at `target` modulo π (a line's
/// direction either way): `R = φ − target` wrapped into [−π/2, π/2).
///
///   φ = atan2(d1 × d2, d1 · d2),  ∂φ/∂d1 = (dy1, −dx1)/|d1|²,  ∂φ/∂d2 = (−dy2, dx2)/|d2|²
///
/// The angle, not its sine or cosine: those are flat where they peak (a
/// cosine row has a zero gradient for lines that start parallel, and
/// `perpendicular` never moves them), while the wrapped angle changes at
/// rate 1 everywhere and crosses zero linearly at both roots, `target` and
/// `target + π`. A zero-length line has no direction; the row falls back
/// to the form `f` (`d1 × d2` for parallel, `d1 · d2` for perpendicular)
/// with partials `df1`, `df2`, which still moves the line's ends.
pub(crate) fn set_direction_row(
    d1: [f64; 2],
    d2: [f64; 2],
    target: f64,
    (f, df1, df2): (f64, [f64; 2], [f64; 2]),
    r: &mut [f64],
    j: &mut DMatrix<f64>,
) {
    let (n1sq, n2sq) = (d1[0] * d1[0] + d1[1] * d1[1], d2[0] * d2[0] + d2[1] * d2[1]);
    let (g1, g2) = if n1sq > 0.0 && n2sq > 0.0 {
        let cross = d1[0] * d2[1] - d1[1] * d2[0];
        let dot = d1[0] * d2[0] + d1[1] * d2[1];
        let phi = cross.atan2(dot);
        r[0] = (phi - target + FRAC_PI_2).rem_euclid(PI) - FRAC_PI_2;
        ([d1[1] / n1sq, -d1[0] / n1sq], [-d2[1] / n2sq, d2[0] / n2sq])
    } else {
        r[0] = f;
        (df1, df2)
    };
    // d1 = p2 − p1, d2 = p4 − p3.
    set_row(j, 0, &[-g1[0], -g1[1], g1[0], g1[1], -g2[0], -g2[1], g2[0], g2[1]]);
}
