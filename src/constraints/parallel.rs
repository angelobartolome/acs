#![allow(non_snake_case)] // Makes sense for mathematical variables
#![allow(unused_parens)]
use std::collections::HashMap;

use nalgebra::{DMatrix, DVector};

use crate::{ParameterManager, Point, constraints::Constraint};

pub struct ParallelConstraint {
    pub p1: String, // Index of the first point (L1P1)
    pub p2: String, // Index of the second point (L1P2)
    pub p3: String, // Index of the third point (L2P1)
    pub p4: String, // Index of the fourth point (L2P2)
}

impl ParallelConstraint {
    pub fn new(p1: String, p2: String, p3: String, p4: String) -> Self {
        Self { p1, p2, p3, p4 }
    }
}

impl Constraint for ParallelConstraint {
    fn num_residuals(&self) -> usize {
        1
    }

    fn residual_parametric(&self, param_manager: &ParameterManager) -> DVector<f64> {
        // Get the parameters for all four points
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
        let p3_x_idx = param_manager
            .get_global_index(&self.p3, 0)
            .expect("Point 3 not found");
        let p3_y_idx = param_manager
            .get_global_index(&self.p3, 1)
            .expect("Point 3 not found");
        let p4_x_idx = param_manager
            .get_global_index(&self.p4, 0)
            .expect("Point 4 not found");
        let p4_y_idx = param_manager
            .get_global_index(&self.p4, 1)
            .expect("Point 4 not found");

        let params = param_manager.get_parameters();
        let x1 = params[p1_x_idx];
        let y1 = params[p1_y_idx];
        let x2 = params[p2_x_idx];
        let y2 = params[p2_y_idx];
        let x3 = params[p3_x_idx];
        let y3 = params[p3_y_idx];
        let x4 = params[p4_x_idx];
        let y4 = params[p4_y_idx];

        let dx1 = x2 - x1;
        let dy1 = y2 - y1;
        let dx2 = x4 - x3;
        let dy2 = y4 - y3;

        DVector::from(vec![dx1 * dy2 - dy1 * dx2])
    }

    fn jacobian_parametric(&self, param_manager: &ParameterManager) -> DMatrix<f64> {
        let total_params = param_manager.num_parameters();
        let mut J = DMatrix::<f64>::zeros(1, total_params);

        // Get parameter indices
        if let (
            Some(p1_x_idx),
            Some(p1_y_idx),
            Some(p2_x_idx),
            Some(p2_y_idx),
            Some(p3_x_idx),
            Some(p3_y_idx),
            Some(p4_x_idx),
            Some(p4_y_idx),
        ) = (
            param_manager.get_global_index(&self.p1, 0),
            param_manager.get_global_index(&self.p1, 1),
            param_manager.get_global_index(&self.p2, 0),
            param_manager.get_global_index(&self.p2, 1),
            param_manager.get_global_index(&self.p3, 0),
            param_manager.get_global_index(&self.p3, 1),
            param_manager.get_global_index(&self.p4, 0),
            param_manager.get_global_index(&self.p4, 1),
        ) {
            let params = param_manager.get_parameters();
            let x1 = params[p1_x_idx];
            let y1 = params[p1_y_idx];
            let x2 = params[p2_x_idx];
            let y2 = params[p2_y_idx];
            let x3 = params[p3_x_idx];
            let y3 = params[p3_y_idx];
            let x4 = params[p4_x_idx];
            let y4 = params[p4_y_idx];

            // Partial derivatives of (dx1 * dy2 - dy1 * dx2)
            J[(0, p1_x_idx)] = -(y4 - y3); // ∂r/∂x1
            J[(0, p1_y_idx)] = (x4 - x3); // ∂r/∂y1
            J[(0, p2_x_idx)] = (y4 - y3); // ∂r/∂x2
            J[(0, p2_y_idx)] = -(x4 - x3); // ∂r/∂y2
            J[(0, p3_x_idx)] = (y2 - y1); // ∂r/∂x3
            J[(0, p3_y_idx)] = -(x2 - x1); // ∂r/∂y3
            J[(0, p4_x_idx)] = -(y2 - y1); // ∂r/∂x4
            J[(0, p4_y_idx)] = (x2 - x1); // ∂r/∂y4
        }

        J
    }

    fn residual(&self, points: &HashMap<String, Point>) -> DVector<f64> {
        let x1 = points[&self.p1].x;
        let y1 = points[&self.p1].y;
        let x2 = points[&self.p2].x;
        let y2 = points[&self.p2].y;
        let x3 = points[&self.p3].x;
        let y3 = points[&self.p3].y;
        let x4 = points[&self.p4].x;
        let y4 = points[&self.p4].y;

        let dx1 = x2 - x1;
        let dy1 = y2 - y1;
        let dx2 = x4 - x3;
        let dy2 = y4 - y3;

        DVector::from(vec![dx1 * dy2 - dy1 * dx2])
    }

    fn jacobian(
        &self,
        points: &HashMap<String, Point>,
        id_to_index: &HashMap<String, usize>,
    ) -> DMatrix<f64> {
        let x1 = points[&self.p1].x;
        let y1 = points[&self.p1].y;
        let x2 = points[&self.p2].x;
        let y2 = points[&self.p2].y;
        let x3 = points[&self.p3].x;
        let y3 = points[&self.p3].y;
        let x4 = points[&self.p4].x;
        let y4 = points[&self.p4].y;

        let cols = points.len() * 2;
        let mut J = DMatrix::<f64>::zeros(1, cols);

        // Derivatives
        J[(0, id_to_index[&self.p1] * 2)] = -(y4 - y3); // ∂r/∂x1
        J[(0, id_to_index[&self.p1] * 2 + 1)] = (x4 - x3); // ∂r/∂y1
        J[(0, id_to_index[&self.p2] * 2)] = (y4 - y3); // ∂r/∂x2
        J[(0, id_to_index[&self.p2] * 2 + 1)] = -(x4 - x3); // ∂r/∂y2
        J[(0, id_to_index[&self.p3] * 2)] = (y2 - y1); // ∂r/∂x3
        J[(0, id_to_index[&self.p3] * 2 + 1)] = -(x2 - x1); // ∂r/∂y3
        J[(0, id_to_index[&self.p4] * 2)] = -(y2 - y1); // ∂r/∂x4
        J[(0, id_to_index[&self.p4] * 2 + 1)] = (x2 - x1); // ∂r/∂y4

        /*
        J[(0, self.p1 * 2)] = -(y4 - y3); // ∂r/∂x1
        J[(0, self.p1 * 2 + 1)] = (x4 - x3); // ∂r/∂y1
        J[(0, self.p2 * 2)] = (y4 - y3); // ∂r/∂x2
        J[(0, self.p2 * 2 + 1)] = -(x4 - x3); // ∂r/∂y2
        J[(0, self.p3 * 2)] = (y2 - y1); // ∂r/∂x3
        J[(0, self.p3 * 2 + 1)] = -(x2 - x1); // ∂r/∂y3
        J[(0, self.p4 * 2)] = -(y2 - y1); // ∂r/∂x4
        J[(0, self.p4 * 2 + 1)] = (x2 - x1); // ∂r/∂y4
         */

        J
    }
}
