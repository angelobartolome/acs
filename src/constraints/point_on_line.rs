#![allow(non_snake_case)] // Makes sense for mathematical variables
#![allow(unused_parens)]
use std::collections::HashMap;

use nalgebra::{DMatrix, DVector};

use crate::{Point, constraints::Constraint};

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

    fn residual(&self, points: &HashMap<String, Point>) -> DVector<f64> {
        let x1 = points[&self.p1].x;
        let y1 = points[&self.p1].y;
        let x2 = points[&self.p_line_a].x;
        let y2 = points[&self.p_line_a].y;
        let x3 = points[&self.p_line_b].x;
        let y3 = points[&self.p_line_b].y;

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

    fn jacobian(
        &self,
        points: &HashMap<String, Point>,
        id_to_index: &HashMap<String, usize>,
    ) -> DMatrix<f64> {
        let cols = points.len() * 2;
        let mut J = DMatrix::<f64>::zeros(1, cols);

        let x1 = points[&self.p1].x;
        let y1 = points[&self.p1].y;
        let x2 = points[&self.p_line_a].x;
        let y2 = points[&self.p_line_a].y;
        let x3 = points[&self.p_line_b].x;
        let y3 = points[&self.p_line_b].y;

        let dx = x3 - x2;
        let dy = y3 - y2;
        let segment_length_squared = dx * dx + dy * dy;

        if segment_length_squared < 1e-12 {
            // Degenerate case: treat as point-to-point distance
            J[(0, id_to_index[&self.p1] * 2)] = 2.0 * (x1 - x2);
            J[(0, id_to_index[&self.p1] * 2 + 1)] = 2.0 * (y1 - y2);
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
            J[(0, id_to_index[&self.p1] * 2)] = 2.0 * (x1 - px);
            J[(0, id_to_index[&self.p1] * 2 + 1)] = 2.0 * (y1 - py);
        } else {
            // t is clamped, simplified derivatives
            J[(0, id_to_index[&self.p1] * 2)] = 2.0 * (x1 - px);
            J[(0, id_to_index[&self.p1] * 2 + 1)] = 2.0 * (y1 - py);
        }

        J
    }
}
