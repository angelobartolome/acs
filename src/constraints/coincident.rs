#![allow(non_snake_case)] // Makes sense for mathematical variables
#![allow(unused_parens)]
use std::collections::HashMap;

use nalgebra::{DMatrix, DVector};

use crate::{ParameterManager, Point, constraints::Constraint};

pub struct CoincidentConstraint {
    pub p1: String, // Index of the first point
    pub p2: String, // Index of the second point
}

impl CoincidentConstraint {
    pub fn new(p1: String, p2: String) -> Self {
        Self { p1, p2 }
    }
}

impl Constraint for CoincidentConstraint {
    fn num_residuals(&self) -> usize {
        2
    }

    fn residual_parametric(&self, param_manager: &ParameterManager) -> DVector<f64> {
        // Get the parameter indices for both points
        let p1_x_idx = param_manager
            .get_global_index(&self.p1, 0)
            .expect("Point 1 not found");
        let p1_y_idx = param_manager
            .get_global_index(&self.p1, 1)
            .expect("Point 1 not found");
        let p2_x_idx = param_manager
            .get_global_index(&self.p2, 0)
            .expect("Point 2 not found");
        let p2_y_idx = param_manager
            .get_global_index(&self.p2, 1)
            .expect("Point 2 not found");

        let params = param_manager.get_parameters();
        let x1 = params[p1_x_idx];
        let y1 = params[p1_y_idx];
        let x2 = params[p2_x_idx];
        let y2 = params[p2_y_idx];

        DVector::from(vec![
            x1 - x2, // Residual for x-coordinates
            y1 - y2, // Residual for y-coordinates
        ])
    }

    fn jacobian_parametric(&self, param_manager: &ParameterManager) -> DMatrix<f64> {
        let total_params = param_manager.num_parameters();
        let mut J = DMatrix::<f64>::zeros(2, total_params);

        // Get the parameter indices for both points
        if let (Some(p1_x_idx), Some(p1_y_idx), Some(p2_x_idx), Some(p2_y_idx)) = (
            param_manager.get_global_index(&self.p1, 0),
            param_manager.get_global_index(&self.p1, 1),
            param_manager.get_global_index(&self.p2, 0),
            param_manager.get_global_index(&self.p2, 1),
        ) {
            // Row 0: X residual derivatives
            J[(0, p1_x_idx)] = 1.0; // d(x1-x2)/dx1
            J[(0, p2_x_idx)] = -1.0; // d(x1-x2)/dx2

            // Row 1: Y residual derivatives
            J[(1, p1_y_idx)] = 1.0; // d(y1-y2)/dy1
            J[(1, p2_y_idx)] = -1.0; // d(y1-y2)/dy2
        }

        J
    }

    fn residual(&self, points: &HashMap<String, Point>) -> DVector<f64> {
        let x1 = points[&self.p1].x;
        let y1 = points[&self.p1].y;
        let x2 = points[&self.p2].x;
        let y2 = points[&self.p2].y;

        DVector::from(vec![
            x1 - x2, // Residual for x-coordinates
            y1 - y2, // Residual for y-coordinates
        ])
    }

    fn jacobian(
        &self,
        points: &HashMap<String, Point>,
        id_to_index: &HashMap<String, usize>,
    ) -> DMatrix<f64> {
        let cols = points.len() * 2;
        let mut J = DMatrix::<f64>::zeros(2, cols);

        // Row 0: X residual
        J[(0, id_to_index[&self.p1] * 2)] = 1.0; // d(x1-x2)/dx1
        J[(0, id_to_index[&self.p2] * 2)] = -1.0; // d(x1-x2)/dx2

        // Row 1: Y residual
        J[(1, id_to_index[&self.p1] * 2 + 1)] = 1.0; // d(y1-y2)/dy1
        J[(1, id_to_index[&self.p2] * 2 + 1)] = -1.0; // d(y1-y2)/dy2

        J
    }
}
