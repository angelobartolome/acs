//! [`ArcLengthConstraint`]: fixes an arc's length, `r · sweep`.

use std::f64::consts::TAU;

use nalgebra::DMatrix;

use crate::constraints::{Constraint, Var, set_row};

/// Fixes an arc's length, `r · sweep`.
///
/// The sweep is counter-clockwise from the start angle to the end angle,
/// `(β − α) mod 2π` in `(0, 2π]`: a sweep of 0 is a full turn, so the
/// length jumps only where the arc itself does, between a full circle and
/// none.
///
/// Entities:
///   - `arc_id` – the arc (radius r, start angle α, end angle β)
///
/// Residual: R = r · sweep − length
pub struct ArcLengthConstraint {
    /// The arc (radius r, start angle α, end angle β).
    pub arc_id: String,
    /// The arc length `r · sweep`.
    pub length: f64,
}

impl ArcLengthConstraint {
    /// The constraint over these entities, by ID (see the fields).
    pub fn new(arc_id: String, length: f64) -> Self {
        Self { arc_id, length }
    }
}

/// `(end − start) mod 2π` in `(0, 2π]`, a 0 sweep being a full turn.
pub(crate) fn arc_sweep(start: f64, end: f64) -> f64 {
    let s = (end - start).rem_euclid(TAU);
    if s <= 0.0 || s >= TAU { TAU } else { s }
}

impl Constraint for ArcLengthConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        vec![
            Var::Radius(&self.arc_id),
            Var::StartAngle(&self.arc_id),
            Var::EndAngle(&self.arc_id),
        ]
    }

    fn num_residuals(&self) -> usize {
        1
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        // x = [r, α, β]
        let sweep = arc_sweep(x[1], x[2]);
        r[0] = x[0] * sweep - self.length;
        set_row(j, 0, &[sweep, -x[0], x[0]]);
    }
}
