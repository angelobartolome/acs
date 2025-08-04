#![allow(non_snake_case)] // Makes sense for mathematical variables
#![allow(unused_parens)]

use nalgebra::{DMatrix, DVector};

use crate::{ParameterManager, constraints::Constraint};

pub struct EqualYConstraint {
    pub p1: String, // Index of the first point
    pub y: f64,     // The y-coordinate to which the point should be equal
}

impl EqualYConstraint {
    pub fn new(p1: String, y: f64) -> Self {
        Self { p1, y }
    }
}

impl Constraint for EqualYConstraint {
    fn num_residuals(&self) -> usize {
        1
    }

    fn residual(&self, param_manager: &ParameterManager) -> DVector<f64> {
        // Get the y parameter for the point
        let p1_y_idx = param_manager
            .get_global_index(&self.p1, 1)
            .expect("Point not found");
        let params = param_manager.get_parameters();
        let y1 = params[p1_y_idx];

        DVector::from(vec![y1 - self.y])
    }

    fn jacobian(&self, param_manager: &ParameterManager) -> DMatrix<f64> {
        let total_params = param_manager.num_parameters();
        let mut J = DMatrix::<f64>::zeros(1, total_params);

        // Get the y parameter index for the point
        if let Some(p1_y_idx) = param_manager.get_global_index(&self.p1, 1) {
            J[(0, p1_y_idx)] = 1.0; // derivative wrt p1.y
        }

        J
    }
}
