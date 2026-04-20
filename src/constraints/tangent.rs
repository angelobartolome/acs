#![allow(non_snake_case)]

use nalgebra::{DMatrix, DVector};

use crate::{ParameterManager, constraints::Constraint};

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
    fn num_residuals(&self) -> usize {
        1
    }

    fn residual(&self, pm: &ParameterManager) -> DVector<f64> {
        let p = pm.get_parameters();
        let cx1 = p[pm.get_global_index(&self.c1_center_id, 0).expect("c1 center.x")];
        let cy1 = p[pm.get_global_index(&self.c1_center_id, 1).expect("c1 center.y")];
        let cx2 = p[pm.get_global_index(&self.c2_center_id, 0).expect("c2 center.x")];
        let cy2 = p[pm.get_global_index(&self.c2_center_id, 1).expect("c2 center.y")];
        let r1 = p[pm.get_global_index(&self.c1_id, 0).expect("c1.r")];
        let r2 = p[pm.get_global_index(&self.c2_id, 0).expect("c2.r")];

        let dx = cx2 - cx1;
        let dy = cy2 - cy1;
        let sum_r = r1 + r2;

        DVector::from(vec![dx * dx + dy * dy - sum_r * sum_r])
    }

    fn jacobian(&self, pm: &ParameterManager) -> DMatrix<f64> {
        let n = pm.num_parameters();
        let mut J = DMatrix::<f64>::zeros(1, n);

        let p = pm.get_parameters();
        let i_cx1 = pm.get_global_index(&self.c1_center_id, 0).expect("c1 center.x");
        let i_cy1 = pm.get_global_index(&self.c1_center_id, 1).expect("c1 center.y");
        let i_cx2 = pm.get_global_index(&self.c2_center_id, 0).expect("c2 center.x");
        let i_cy2 = pm.get_global_index(&self.c2_center_id, 1).expect("c2 center.y");
        let i_r1 = pm.get_global_index(&self.c1_id, 0).expect("c1.r");
        let i_r2 = pm.get_global_index(&self.c2_id, 0).expect("c2.r");

        let dx = p[i_cx2] - p[i_cx1];
        let dy = p[i_cy2] - p[i_cy1];
        let sum_r = p[i_r1] + p[i_r2];

        J[(0, i_cx1)] = -2.0 * dx;
        J[(0, i_cy1)] = -2.0 * dy;
        J[(0, i_cx2)] = 2.0 * dx;
        J[(0, i_cy2)] = 2.0 * dy;
        J[(0, i_r1)] = -2.0 * sum_r;
        J[(0, i_r2)] = -2.0 * sum_r;

        J
    }
}
