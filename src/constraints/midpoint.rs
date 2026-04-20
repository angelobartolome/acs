#![allow(non_snake_case)]

use nalgebra::{DMatrix, DVector};

use crate::{ParameterManager, constraints::Constraint};

/// Constrains a point to be the midpoint of a line segment.
///
/// Residuals:
///   R₀ = mx − (ax + bx)/2 = 0
///   R₁ = my − (ay + by)/2 = 0
pub struct MidpointConstraint {
    pub midpoint_id: String,
    pub endpoint_a_id: String,
    pub endpoint_b_id: String,
}

impl MidpointConstraint {
    pub fn new(midpoint_id: String, endpoint_a_id: String, endpoint_b_id: String) -> Self {
        Self {
            midpoint_id,
            endpoint_a_id,
            endpoint_b_id,
        }
    }
}

impl Constraint for MidpointConstraint {
    fn num_residuals(&self) -> usize {
        2
    }

    fn residual(&self, pm: &ParameterManager) -> DVector<f64> {
        let p = pm.get_parameters();
        let mx = p[pm.get_global_index(&self.midpoint_id, 0).expect("m.x")];
        let my = p[pm.get_global_index(&self.midpoint_id, 1).expect("m.y")];
        let ax = p[pm.get_global_index(&self.endpoint_a_id, 0).expect("a.x")];
        let ay = p[pm.get_global_index(&self.endpoint_a_id, 1).expect("a.y")];
        let bx = p[pm.get_global_index(&self.endpoint_b_id, 0).expect("b.x")];
        let by = p[pm.get_global_index(&self.endpoint_b_id, 1).expect("b.y")];

        DVector::from(vec![
            mx - (ax + bx) / 2.0,
            my - (ay + by) / 2.0,
        ])
    }

    fn jacobian(&self, pm: &ParameterManager) -> DMatrix<f64> {
        let n = pm.num_parameters();
        let mut J = DMatrix::<f64>::zeros(2, n);

        let i_mx = pm.get_global_index(&self.midpoint_id, 0).expect("m.x");
        let i_my = pm.get_global_index(&self.midpoint_id, 1).expect("m.y");
        let i_ax = pm.get_global_index(&self.endpoint_a_id, 0).expect("a.x");
        let i_ay = pm.get_global_index(&self.endpoint_a_id, 1).expect("a.y");
        let i_bx = pm.get_global_index(&self.endpoint_b_id, 0).expect("b.x");
        let i_by = pm.get_global_index(&self.endpoint_b_id, 1).expect("b.y");

        // Row 0: R₀ = mx − (ax+bx)/2
        J[(0, i_mx)] = 1.0;
        J[(0, i_ax)] = -0.5;
        J[(0, i_bx)] = -0.5;

        // Row 1: R₁ = my − (ay+by)/2
        J[(1, i_my)] = 1.0;
        J[(1, i_ay)] = -0.5;
        J[(1, i_by)] = -0.5;

        J
    }
}
