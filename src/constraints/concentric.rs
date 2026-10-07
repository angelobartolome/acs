//! [`ConcentricConstraint`]: forces two circles to share the same center
//! (concentric).

#![allow(non_snake_case)]

use nalgebra::DMatrix;

use crate::constraints::{Constraint, Var, set_row, xy};

/// Forces two circles to share the same center (concentric).
///
/// Pass the center *point* IDs of each circle. Semantically identical to
/// `CoincidentConstraint` but carries a different geometric meaning.
///
/// Residuals:
///   R₀ = cx1 − cx2 = 0
///   R₁ = cy1 − cy2 = 0
pub struct ConcentricConstraint {
    /// The first circle's or arc's center point.
    pub center1_id: String,
    /// The second circle's or arc's center point.
    pub center2_id: String,
}

impl ConcentricConstraint {
    /// The constraint over these entities, by ID (see the fields).
    pub fn new(center1_id: String, center2_id: String) -> Self {
        Self {
            center1_id,
            center2_id,
        }
    }
}

impl Constraint for ConcentricConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [xy(&self.center1_id), xy(&self.center2_id)].concat()
    }

    fn num_residuals(&self) -> usize {
        2
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        r[0] = x[0] - x[2];
        r[1] = x[1] - x[3];
        set_row(j, 0, &[1.0, 0.0, -1.0, 0.0]);
        set_row(j, 1, &[0.0, 1.0, 0.0, -1.0]);
    }
}
