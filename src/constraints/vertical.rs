#![allow(non_snake_case)] // Makes sense for mathematical variables
#![allow(unused_parens)]

use nalgebra::{DMatrix, DVector};

use crate::{ParameterManager, constraints::Constraint};

pub struct VerticalConstraint {
    pub p1: String, // Index of the first point
    pub p2: String, // Index of the second point
}

impl VerticalConstraint {
    pub fn new(p1: String, p2: String) -> Self {
        Self { p1, p2 }
    }
}

impl Constraint for VerticalConstraint {
    fn num_residuals(&self) -> usize {
        1
    }

    fn residual(&self, param_manager: &ParameterManager) -> DVector<f64> {
        // Get x coordinates of both points
        let p1_x_idx = param_manager
            .get_global_index(&self.p1, 0)
            .expect("Point p1 not found in parameter manager");
        let p2_x_idx = param_manager
            .get_global_index(&self.p2, 0)
            .expect("Point p2 not found in parameter manager");

        let params = param_manager.get_parameters();
        let p1_x = params[p1_x_idx];
        let p2_x = params[p2_x_idx];

        DVector::from(vec![p1_x - p2_x])
    }

    fn jacobian(&self, param_manager: &ParameterManager) -> DMatrix<f64> {
        let total_params = param_manager.num_parameters();
        let mut J = DMatrix::<f64>::zeros(1, total_params);

        // Get global indices for x coordinates
        if let Some(p1_x_idx) = param_manager.get_global_index(&self.p1, 0) {
            J[(0, p1_x_idx)] = 1.0; // derivative wrt p1.x
        }

        if let Some(p2_x_idx) = param_manager.get_global_index(&self.p2, 0) {
            J[(0, p2_x_idx)] = -1.0; // derivative wrt p2.x
        }

        J
    }
}
