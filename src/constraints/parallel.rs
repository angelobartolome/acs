#![allow(non_snake_case)] // Makes sense for mathematical variables
#![allow(unused_parens)]
use std::collections::HashMap;

use nalgebra::{DMatrix, DVector};

use crate::{Point, constraints::Constraint};

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
