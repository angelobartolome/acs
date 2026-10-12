//! [`TangentConstraint`]: external tangency between two circles.

#![allow(non_snake_case)]

use nalgebra::DMatrix;

use crate::constraints::directions::unit;
use crate::constraints::{Constraint, Var, set_row, xy};

/// External tangency between two circles: distance between centres = r1 + r2.
///
/// Entities:
///   - `c1_center_id` – center point of circle 1
///   - `c1_id`        – circle 1 entity (radius at param 0)
///   - `c2_center_id` – center point of circle 2
///   - `c2_id`        – circle 2 entity (radius at param 0)
///
/// Residual, a length (see SPEC.md, Notation, Units):
///   R = |c2 − c1| − (r1 + r2)
pub struct TangentConstraint {
    /// Center point of circle 1.
    pub c1_center_id: String,
    /// Circle 1 entity (radius at param 0).
    pub c1_id: String,
    /// Center point of circle 2.
    pub c2_center_id: String,
    /// Circle 2 entity (radius at param 0).
    pub c2_id: String,
}

impl TangentConstraint {
    /// The constraint over these entities, by ID (see the fields).
    pub fn new(
        c1_center_id: String,
        c1_id: String,
        c2_center_id: String,
        c2_id: String,
    ) -> Self {
        Self {
            c1_center_id,
            c1_id,
            c2_center_id,
            c2_id,
        }
    }
}

impl Constraint for TangentConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [&xy(&self.c1_center_id)[..], &[Var::Radius(&self.c1_id)], &xy(&self.c2_center_id), &[Var::Radius(&self.c2_id)]].concat()
    }

    fn num_residuals(&self) -> usize {
        1
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        // x = [cx1, cy1, r1, cx2, cy2, r2]
        let (u, n) = unit(x[3] - x[0], x[4] - x[1]);
        r[0] = n - (x[2] + x[5]);
        set_row(j, 0, &[-u[0], -u[1], -1.0, u[0], u[1], -1.0]);
    }
}
