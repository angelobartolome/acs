//! [`PerpendicularConstraint`]: forces two lines to be perpendicular (dot
//! product of direction vectors = 0).

#![allow(non_snake_case)]

use nalgebra::DMatrix;

use crate::constraints::{Constraint, Var, set_row, xy};

/// Forces two lines to be perpendicular (dot product of direction vectors = 0).
///
/// Line 1: p1 → p2, direction d1 = (x2−x1, y2−y1)
/// Line 2: p3 → p4, direction d2 = (x4−x3, y4−y3)
/// Residual: d1·d2 = dx1·dx2 + dy1·dy2 = 0
pub struct PerpendicularConstraint {
    /// Start of line L1.
    pub p1: String,
    /// End of line L1.
    pub p2: String,
    /// Start of line L2.
    pub p3: String,
    /// End of line L2.
    pub p4: String,
}

impl PerpendicularConstraint {
    /// The constraint over these entities, by ID (see the fields).
    pub fn new(p1: String, p2: String, p3: String, p4: String) -> Self {
        Self { p1, p2, p3, p4 }
    }
}

impl Constraint for PerpendicularConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [xy(&self.p1), xy(&self.p2), xy(&self.p3), xy(&self.p4)].concat()
    }

    fn num_residuals(&self) -> usize {
        1
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        let (dx1, dy1) = (x[2] - x[0], x[3] - x[1]);
        let (dx2, dy2) = (x[6] - x[4], x[7] - x[5]);
        r[0] = dx1 * dx2 + dy1 * dy2;
        set_row(j, 0, &[-dx2, -dy2, dx2, dy2, -dx1, -dy1, dx1, dy1]);
    }
}
