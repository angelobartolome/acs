#![allow(non_snake_case)]

use nalgebra::DMatrix;

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
/// Residual: dot·sin(θ) − cross·cos(θ) = 0
///
/// This encodes atan2(cross, dot) = θ, i.e. the angle from d1 to d2 is θ.
pub struct AngleConstraint {
    pub p1: String,
    pub p2: String,
    pub p3: String,
    pub p4: String,
    pub angle: f64,
}

impl AngleConstraint {
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
        r[0] = dot * s - cross * c;

        let d_dot = [-dx2, -dy2, dx2, dy2, -dx1, -dy1, dx1, dy1];
        let d_cross = [-dy2, dx2, dy2, -dx2, dy1, -dx1, -dy1, dx1];
        for k in 0..8 {
            j[(0, k)] = d_dot[k] * s - d_cross[k] * c;
        }
    }
}
