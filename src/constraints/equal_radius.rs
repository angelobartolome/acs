#![allow(non_snake_case)] // Makes sense for mathematical variables
#![allow(unused_parens)]

use nalgebra::{DMatrix, DVector};

use crate::{ParameterManager, constraints::Constraint};

pub struct EqualRadiusConstraint {
    pub circle1_id: String, // ID of the first circle
    pub circle2_id: String, // ID of the second circle
}

impl EqualRadiusConstraint {
    pub fn new(circle1_id: String, circle2_id: String) -> Self {
        Self {
            circle1_id,
            circle2_id,
        }
    }
}

impl Constraint for EqualRadiusConstraint {
    fn num_residuals(&self) -> usize {
        1
    }

    fn residual(&self, param_manager: &ParameterManager) -> DVector<f64> {
        // Get radius parameters of both circles
        // Circles now have parameters [radius] with radius at index 0
        let c1_radius_idx = param_manager
            .get_global_index(&self.circle1_id, 0)
            .expect("Circle 1 not found in parameter manager");
        let c2_radius_idx = param_manager
            .get_global_index(&self.circle2_id, 0)
            .expect("Circle 2 not found in parameter manager");

        let params = param_manager.get_parameters();
        let c1_radius = params[c1_radius_idx];
        let c2_radius = params[c2_radius_idx];

        // Residual: radius1 - radius2 = 0
        DVector::from(vec![c1_radius - c2_radius])
    }

    fn jacobian(&self, param_manager: &ParameterManager) -> DMatrix<f64> {
        let total_params = param_manager.num_parameters();
        let mut J = DMatrix::<f64>::zeros(1, total_params);

        // Get global indices for radius parameters
        if let Some(c1_radius_idx) = param_manager.get_global_index(&self.circle1_id, 0) {
            J[(0, c1_radius_idx)] = 1.0; // derivative wrt circle1.radius
        }

        if let Some(c2_radius_idx) = param_manager.get_global_index(&self.circle2_id, 0) {
            J[(0, c2_radius_idx)] = -1.0; // derivative wrt circle2.radius
        }

        J
    }
}
