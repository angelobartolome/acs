//! [`MidpointConstraint`]: constrains a point to be the midpoint of a line
//! segment.

#![allow(non_snake_case)]

use nalgebra::DMatrix;

use crate::constraints::{Constraint, Var, set_row, xy};

/// Constrains a point to be the midpoint of a line segment.
///
/// Residuals:
///   R₀ = mx − (ax + bx)/2 = 0
///   R₁ = my − (ay + by)/2 = 0
pub struct MidpointConstraint {
    /// The point held at the middle.
    pub midpoint_id: String,
    /// The segment's first endpoint.
    pub endpoint_a_id: String,
    /// The segment's second endpoint.
    pub endpoint_b_id: String,
}

impl MidpointConstraint {
    /// The constraint over these entities, by ID (see the fields).
    pub fn new(midpoint_id: String, endpoint_a_id: String, endpoint_b_id: String) -> Self {
        Self {
            midpoint_id,
            endpoint_a_id,
            endpoint_b_id,
        }
    }
}

impl Constraint for MidpointConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [xy(&self.midpoint_id), xy(&self.endpoint_a_id), xy(&self.endpoint_b_id)].concat()
    }

    fn num_residuals(&self) -> usize {
        2
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        // x = [mx, my, ax, ay, bx, by]
        r[0] = x[0] - (x[2] + x[4]) / 2.0;
        r[1] = x[1] - (x[3] + x[5]) / 2.0;
        set_row(j, 0, &[1.0, 0.0, -0.5, 0.0, -0.5, 0.0]);
        set_row(j, 1, &[0.0, 1.0, 0.0, -0.5, 0.0, -0.5]);
    }
}
