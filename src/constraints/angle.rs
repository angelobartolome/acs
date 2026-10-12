//! [`AngleConstraint`]: constrains the directed angle from line L1 to line L2
//! to a specific value.

#![allow(non_snake_case)]

use nalgebra::DMatrix;

use crate::constraints::directions::set_direction_row;
use crate::constraints::{Constraint, Var, xy};

/// Constrains the directed angle from line L1 to line L2 to a specific value.
///
/// Entities: four points (p1,p2,p3,p4) defining L1 = p1→p2 and L2 = p3→p4.
/// Target angle θ in radians.
///
/// Let d1 = (dx1, dy1) = (x2−x1, y2−y1)
///     d2 = (dx2, dy2) = (x4−x3, y4−y3)
///     dot   = dx1·dx2 + dy1·dy2
///     cross = dx1·dy2 − dy1·dx2
///
/// Residual, φ = atan2(cross, dot) less θ, modulo π, in radians (see SPEC.md,
/// Notation, Units):
///   R = φ − θ wrapped into [−π/2, π/2)
///
/// Like the earlier `dot·sin(θ) − cross·cos(θ)` (|d1||d2|·sin(θ − φ)), it
/// vanishes at φ = θ and also at φ = θ + π (d2 reversed).
pub struct AngleConstraint {
    /// Start of line L1.
    pub p1: String,
    /// End of line L1.
    pub p2: String,
    /// Start of line L2.
    pub p3: String,
    /// End of line L2.
    pub p4: String,
    /// The directed angle from L1 to L2, in radians.
    pub angle: f64,
}

impl AngleConstraint {
    /// The constraint over these entities, by ID (see the fields).
    pub fn new(p1: String, p2: String, p3: String, p4: String, angle: f64) -> Self {
        Self {
            p1,
            p2,
            p3,
            p4,
            angle,
        }
    }
}

impl Constraint for AngleConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [xy(&self.p1), xy(&self.p2), xy(&self.p3), xy(&self.p4)].concat()
    }

    fn num_residuals(&self) -> usize {
        1
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        let (dx1, dy1) = (x[2] - x[0], x[3] - x[1]);
        let (dx2, dy2) = (x[6] - x[4], x[7] - x[5]);
        let dot = dx1 * dx2 + dy1 * dy2;
        let cross = dx1 * dy2 - dy1 * dx2;
        let (s, c) = self.angle.sin_cos();
        // ∂dot/∂d1 = d2, ∂cross/∂d1 = (dy2, −dx2); ∂dot/∂d2 = d1,
        // ∂cross/∂d2 = (−dy1, dx1).
        let df1 = [dx2 * s - dy2 * c, dy2 * s + dx2 * c];
        let df2 = [dx1 * s + dy1 * c, dy1 * s - dx1 * c];
        set_direction_row([dx1, dy1], [dx2, dy2], self.angle, (dot * s - cross * c, df1, df2), r, j);
    }
}
