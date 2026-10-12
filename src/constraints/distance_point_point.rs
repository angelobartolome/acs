//! [`DistancePointPointConstraint`]: constrains the Euclidean distance
//! between two points to a fixed value.

#![allow(non_snake_case)]

use nalgebra::DMatrix;

use crate::constraints::directions::unit;
use crate::constraints::{Constraint, Var, set_row, xy};

/// Constrains the Euclidean distance between two points to a fixed value.
///
/// Residual, a length (see SPEC.md, Notation, Units):
///   R = |p2 − p1| − d
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
        let (u, n) = unit(x[2] - x[0], x[3] - x[1]);
        r[0] = n - self.distance;
        set_row(j, 0, &[-u[0], -u[1], u[0], u[1]]);
    }
}
