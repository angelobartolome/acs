//! [`DistancePointPointConstraint`]: constrains the Euclidean distance
//! between two points to a fixed value.

#![allow(non_snake_case)]

use nalgebra::DMatrix;

use crate::constraints::{Constraint, Var, set_row, xy};

/// Constrains the Euclidean distance between two points to a fixed value.
///
/// Residual (squared form, smooth everywhere):
///   R = (x2−x1)² + (y2−y1)² − d² = 0
pub struct DistancePointPointConstraint {
    /// The first point.
    pub p1_id: String,
    /// The second point.
    pub p2_id: String,
    /// The distance between them.
    pub distance: f64,
}

impl DistancePointPointConstraint {
    /// The constraint over these entities, by ID (see the fields).
    pub fn new(p1_id: String, p2_id: String, distance: f64) -> Self {
        Self {
            p1_id,
            p2_id,
            distance,
        }
    }
}

impl Constraint for DistancePointPointConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [xy(&self.p1_id), xy(&self.p2_id)].concat()
    }

    fn num_residuals(&self) -> usize {
        1
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        let (dx, dy) = (x[2] - x[0], x[3] - x[1]);
        r[0] = dx * dx + dy * dy - self.distance * self.distance;
        set_row(j, 0, &[-2.0 * dx, -2.0 * dy, 2.0 * dx, 2.0 * dy]);
    }
}
