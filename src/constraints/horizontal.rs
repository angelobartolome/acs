#![allow(non_snake_case)] // Makes sense for mathematical variables
#![allow(unused_parens)]

use nalgebra::{DMatrix, DVector};

use crate::{ParameterManager, constraints::Constraint};

pub struct HorizontalConstraint {
    pub p1: String, // Index of the first point
    pub p2: String, // Index of the second point
}

impl HorizontalConstraint {
    pub fn new(p1: String, p2: String) -> Self {
        Self { p1, p2 }
    }
}

impl Constraint for HorizontalConstraint {
    fn num_residuals(&self) -> usize {
        1
    }

    fn residual(&self, param_manager: &ParameterManager) -> DVector<f64> {
        // Get the y parameters for both points
        let p1_y_idx = param_manager
            .get_global_index(&self.p1, 1)
            .expect("Point 1 not found");
        let p2_y_idx = param_manager
            .get_global_index(&self.p2, 1)
            .expect("Point 2 not found");
        let params = param_manager.get_parameters();
        let y1 = params[p1_y_idx];
        let y2 = params[p2_y_idx];

        DVector::from(vec![y1 - y2])
    }

    fn jacobian(&self, param_manager: &ParameterManager) -> DMatrix<f64> {
        let total_params = param_manager.num_parameters();
        let mut J = DMatrix::<f64>::zeros(1, total_params);

        // Get the y parameter indices for both points
        if let (Some(p1_y_idx), Some(p2_y_idx)) = (
            param_manager.get_global_index(&self.p1, 1),
            param_manager.get_global_index(&self.p2, 1),
        ) {
            J[(0, p1_y_idx)] = 1.0; // derivative wrt p1.y
            J[(0, p2_y_idx)] = -1.0; // derivative wrt p2.y
        }

        J
    }
}
