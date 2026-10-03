#![allow(non_snake_case)]

use nalgebra::DMatrix;

use crate::constraints::{Constraint, Var, set_row, xy};

/// Constrains two line segments to have the same length.
///
/// Segment 1: p1 → p2, Segment 2: p3 → p4.
///
/// Residual (squared form, smooth everywhere):
///   R = (dx1² + dy1²) − (dx2² + dy2²) = 0
///
/// where dx1 = x2−x1, dy1 = y2−y1, dx2 = x4−x3, dy2 = y4−y3.
pub struct EqualLengthConstraint {
    pub p1: String,
    pub p2: String,
    pub p3: String,
    pub p4: String,
}

impl EqualLengthConstraint {
    pub fn new(p1: String, p2: String, p3: String, p4: String) -> Self {
        Self { p1, p2, p3, p4 }
    }
}

impl Constraint for EqualLengthConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [xy(&self.p1), xy(&self.p2), xy(&self.p3), xy(&self.p4)].concat()
    }

    fn num_residuals(&self) -> usize {
        1
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        let (dx1, dy1) = (x[2] - x[0], x[3] - x[1]);
        let (dx2, dy2) = (x[6] - x[4], x[7] - x[5]);
        r[0] = (dx1 * dx1 + dy1 * dy1) - (dx2 * dx2 + dy2 * dy2);
        set_row(
            j,
            0,
            &[
                -2.0 * dx1,
                -2.0 * dy1,
                2.0 * dx1,
                2.0 * dy1,
                2.0 * dx2,
                2.0 * dy2,
                -2.0 * dx2,
                -2.0 * dy2,
            ],
        );
    }
}
