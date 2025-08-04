#![allow(non_snake_case)] // Makes sense for mathematical variables
#![allow(unused_parens)]
use std::collections::HashMap;

use nalgebra::{DMatrix, DVector};

use crate::{ParameterManager, Point, constraints::Constraint};

pub struct EqualXConstraint {
    pub p1: String, // Index of the first point
    pub x: f64,     // The x-coordinate to which the point should be equal
}

impl EqualXConstraint {
    pub fn new(p1: String, x: f64) -> Self {
        Self { p1, x }
    }
}

impl Constraint for EqualXConstraint {
    fn num_residuals(&self) -> usize {
        1
    }

    fn residual_parametric(&self, param_manager: &ParameterManager) -> DVector<f64> {
        // Get the x parameter for the point
        let p1_x_idx = param_manager
            .get_global_index(&self.p1, 0)
            .expect("Point not found");
        let params = param_manager.get_parameters();
        let x1 = params[p1_x_idx];

        DVector::from(vec![x1 - self.x])
    }

    fn jacobian_parametric(&self, param_manager: &ParameterManager) -> DMatrix<f64> {
        let total_params = param_manager.num_parameters();
        let mut J = DMatrix::<f64>::zeros(1, total_params);

        // Get the x parameter index for the point
        if let Some(p1_x_idx) = param_manager.get_global_index(&self.p1, 0) {
            J[(0, p1_x_idx)] = 1.0; // derivative wrt p1.x
        }

        J
    }

    fn residual(&self, points: &HashMap<String, Point>) -> DVector<f64> {
        DVector::from(vec![points[&self.p1].x - self.x])
    }

    fn jacobian(
        &self,
        points: &HashMap<String, Point>,
        id_to_index: &HashMap<String, usize>,
    ) -> DMatrix<f64> {
        let cols = points.len() * 2;
        let mut J = DMatrix::<f64>::zeros(1, cols);

        // J[(0, self.p1 * 2)] = 1.0; // derivative wrt p1.x
        J[(0, id_to_index[&self.p1] * 2)] = 1.0; // derivative wrt p1.x

        J
    }
}
