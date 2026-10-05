use std::f64::consts::{PI, TAU};

use nalgebra::DMatrix;

use crate::constraints::{Constraint, Var, set_row};

/// Fixes an arc's sweep, the counter-clockwise angle from its start angle to
/// its end angle, to `sweep` (`0 < sweep < 2π`, checked by
/// `create_constraint`).
///
/// The residual is the sweep unwrapped near the target: `β − α − sweep`
/// wrapped into `[−π, π)`. A sweep taken mod 2π would jump by 2π when the
/// end angle crosses the start (the 0/2π seam), as a drag across it does;
/// this one changes continuously there, and is discontinuous only half a
/// turn from the target.
///
/// Entities:
///   - `arc_id` – the arc (start angle α, end angle β)
///
/// Residual: R = wrap(β − α − sweep), wrap(t) = ((t + π) mod 2π) − π
pub struct ArcSweepConstraint {
    pub arc_id: String,
    pub sweep: f64,
}

impl ArcSweepConstraint {
    pub fn new(arc_id: String, sweep: f64) -> Self {
        Self { arc_id, sweep }
    }

    /// Whether `sweep` is a sweep this constraint accepts: `0 < sweep < 2π`.
    pub fn accepts(sweep: f64) -> bool {
        sweep > 0.0 && sweep < TAU
    }
}

impl Constraint for ArcSweepConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        vec![Var::StartAngle(&self.arc_id), Var::EndAngle(&self.arc_id)]
    }

    fn num_residuals(&self) -> usize {
        1
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        // x = [α, β]
        r[0] = (x[1] - x[0] - self.sweep + PI).rem_euclid(TAU) - PI;
        set_row(j, 0, &[-1.0, 1.0]);
    }
}
