#![allow(non_snake_case)]

use nalgebra::{DMatrix, DVector};

use crate::{ParameterManager, constraints::Constraint};

/// Forces two circles to share the same center (concentric).
///
/// Pass the center *point* IDs of each circle. Semantically identical to
/// `CoincidentConstraint` but carries a different geometric meaning.
///
/// Residuals:
///   R₀ = cx1 − cx2 = 0
///   R₁ = cy1 − cy2 = 0
pub struct ConcentricConstraint {
    pub center1_id: String,
    pub center2_id: String,
}

impl ConcentricConstraint {
    pub fn new(center1_id: String, center2_id: String) -> Self {
        Self {
            center1_id,
            center2_id,
        }
    }
}

impl Constraint for ConcentricConstraint {
    fn num_residuals(&self) -> usize {
        2
    }

    fn residual(&self, pm: &ParameterManager) -> DVector<f64> {
        let p = pm.get_parameters();
        let cx1 = p[pm.get_global_index(&self.center1_id, 0).expect("center1.x")];
        let cy1 = p[pm.get_global_index(&self.center1_id, 1).expect("center1.y")];
        let cx2 = p[pm.get_global_index(&self.center2_id, 0).expect("center2.x")];
        let cy2 = p[pm.get_global_index(&self.center2_id, 1).expect("center2.y")];
        DVector::from(vec![cx1 - cx2, cy1 - cy2])
    }

    fn jacobian(&self, pm: &ParameterManager) -> DMatrix<f64> {
        let n = pm.num_parameters();
        let mut J = DMatrix::<f64>::zeros(2, n);

        let i_cx1 = pm.get_global_index(&self.center1_id, 0).expect("center1.x");
        let i_cy1 = pm.get_global_index(&self.center1_id, 1).expect("center1.y");
        let i_cx2 = pm.get_global_index(&self.center2_id, 0).expect("center2.x");
        let i_cy2 = pm.get_global_index(&self.center2_id, 1).expect("center2.y");

        J[(0, i_cx1)] = 1.0;
        J[(0, i_cx2)] = -1.0;
        J[(1, i_cy1)] = 1.0;
        J[(1, i_cy2)] = -1.0;

        J
    }
}
