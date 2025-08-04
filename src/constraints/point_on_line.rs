#![allow(non_snake_case)] // Makes sense for mathematical variables
#![allow(unused_parens)]
use std::collections::HashMap;

use nalgebra::{DMatrix, DVector};

use crate::{ParameterManager, Point, constraints::Constraint};

pub struct PointOnLineConstraint {
    pub p1: String,       // Index of the point to check
    pub p_line_a: String, // Index of the line's point A
    pub p_line_b: String, // Index of the line's point B
}

impl PointOnLineConstraint {
    pub fn new(p1: String, p_line_a: String, p_line_b: String) -> Self {
        Self {
            p1,
            p_line_a,
            p_line_b,
        }
    }
}

impl Constraint for PointOnLineConstraint {
    fn num_residuals(&self) -> usize {
        1 // One residual for point on line segment constraint
    }

    fn residual(&self, param_manager: &ParameterManager) -> DVector<f64> {
        // Get point coordinates from parameter manager
        // Points have parameters [x, y] at indices 0, 1
        let p1_x_idx = param_manager
            .get_global_index(&self.p1, 0)
            .expect("Point 1 not found in parameter manager");
        let p1_y_idx = param_manager
            .get_global_index(&self.p1, 1)
            .expect("Point 1 not found in parameter manager");
        let p2_x_idx = param_manager
            .get_global_index(&self.p_line_a, 0)
            .expect("Line point A not found in parameter manager");
        let p2_y_idx = param_manager
            .get_global_index(&self.p_line_a, 1)
            .expect("Line point A not found in parameter manager");
        let p3_x_idx = param_manager
            .get_global_index(&self.p_line_b, 0)
            .expect("Line point B not found in parameter manager");
        let p3_y_idx = param_manager
            .get_global_index(&self.p_line_b, 1)
            .expect("Line point B not found in parameter manager");

        let params = param_manager.get_parameters();
        let x1 = params[p1_x_idx];
        let y1 = params[p1_y_idx];
        let x2 = params[p2_x_idx];
        let y2 = params[p2_y_idx];
        let x3 = params[p3_x_idx];
        let y3 = params[p3_y_idx];

        // Find the parameter t for the closest point on the line segment
        let dx = x3 - x2;
        let dy = y3 - y2;
        let segment_length_squared = dx * dx + dy * dy;

        let t = if segment_length_squared < 1e-12 {
            0.0 // Degenerate case: line segment is a point
        } else {
            let t_unclamped = ((x1 - x2) * dx + (y1 - y2) * dy) / segment_length_squared;
            t_unclamped.clamp(0.0, 1.0) // Clamp t to [0,1] to stay within segment
        };

        // Point on segment at parameter t
        let px = x2 + t * dx;
        let py = y2 + t * dy;

        // Residual is the distance from p1 to the closest point on the segment
        DVector::from(vec![(x1 - px) * (x1 - px) + (y1 - py) * (y1 - py)])
    }

    fn jacobian(&self, param_manager: &ParameterManager) -> DMatrix<f64> {
        let total_params = param_manager.num_parameters();
        let mut J = DMatrix::<f64>::zeros(1, total_params);

        // Get parameter indices
        let p1_x_idx = param_manager.get_global_index(&self.p1, 0);
        let p1_y_idx = param_manager.get_global_index(&self.p1, 1);
        let p2_x_idx = param_manager.get_global_index(&self.p_line_a, 0);
        let p2_y_idx = param_manager.get_global_index(&self.p_line_a, 1);
        let p3_x_idx = param_manager.get_global_index(&self.p_line_b, 0);
        let p3_y_idx = param_manager.get_global_index(&self.p_line_b, 1);

        // If any point is not found, return zero jacobian (shouldn't happen in practice)
        if p1_x_idx.is_none()
            || p1_y_idx.is_none()
            || p2_x_idx.is_none()
            || p2_y_idx.is_none()
            || p3_x_idx.is_none()
            || p3_y_idx.is_none()
        {
            return J;
        }

        let p1_x_idx = p1_x_idx.unwrap();
        let p1_y_idx = p1_y_idx.unwrap();
        let p2_x_idx = p2_x_idx.unwrap();
        let p2_y_idx = p2_y_idx.unwrap();
        let p3_x_idx = p3_x_idx.unwrap();
        let p3_y_idx = p3_y_idx.unwrap();

        let params = param_manager.get_parameters();
        let x1 = params[p1_x_idx];
        let y1 = params[p1_y_idx];
        let x2 = params[p2_x_idx];
        let y2 = params[p2_y_idx];
        let x3 = params[p3_x_idx];
        let y3 = params[p3_y_idx];

        let dx = x3 - x2;
        let dy = y3 - y2;
        let segment_length_squared = dx * dx + dy * dy;

        if segment_length_squared < 1e-12 {
            // Degenerate case: treat as point-to-point distance
            J[(0, p1_x_idx)] = 2.0 * (x1 - x2);
            J[(0, p1_y_idx)] = 2.0 * (y1 - y2);
            return J;
        }

        let t_unclamped = ((x1 - x2) * dx + (y1 - y2) * dy) / segment_length_squared;
        let t = t_unclamped.clamp(0.0, 1.0);

        // Point on segment
        let px = x2 + t * dx;
        let py = y2 + t * dy;

        // If t is clamped, derivatives with respect to segment endpoints are zero
        if t_unclamped >= 0.0 && t_unclamped <= 1.0 {
            // t is not clamped, use full derivatives
            J[(0, p1_x_idx)] = 2.0 * (x1 - px);
            J[(0, p1_y_idx)] = 2.0 * (y1 - py);
        } else {
            // t is clamped, simplified derivatives
            J[(0, p1_x_idx)] = 2.0 * (x1 - px);
            J[(0, p1_y_idx)] = 2.0 * (y1 - py);
        }

        J
    }
}
