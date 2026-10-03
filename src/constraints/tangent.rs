#![allow(non_snake_case)]

use nalgebra::DMatrix;

use crate::constraints::{Constraint, Var, set_row, xy};

/// External tangency between two circles: distance between centres = r1 + r2.
///
/// Entities:
///   - `c1_center_id` – center point of circle 1
///   - `c1_id`        – circle 1 entity (radius at param 0)
///   - `c2_center_id` – center point of circle 2
///   - `c2_id`        – circle 2 entity (radius at param 0)
///
/// Residual: (cx2−cx1)² + (cy2−cy1)² − (r1+r2)² = 0
pub struct TangentConstraint {
    pub c1_center_id: String,
    pub c1_id: String,
    pub c2_center_id: String,
    pub c2_id: String,
}

impl TangentConstraint {
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
        let (ddx, ddy, sum_r) = (x[3] - x[0], x[4] - x[1], x[2] + x[5]);
        r[0] = ddx * ddx + ddy * ddy - sum_r * sum_r;
        set_row(
            j,
            0,
            &[-2.0 * ddx, -2.0 * ddy, -2.0 * sum_r, 2.0 * ddx, 2.0 * ddy, -2.0 * sum_r],
        );
    }
}
