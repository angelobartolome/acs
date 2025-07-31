#![allow(non_snake_case)] // Makes sense for mathematical variables
#![allow(unused_parens)]
use std::collections::HashMap;

use nalgebra::{DMatrix, DVector};

use crate::{Point, constraints::Constraint};

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

    fn residual(&self, points: &HashMap<String, Point>) -> DVector<f64> {
        DVector::from(vec![points[&self.p1].y - points[&self.p2].y])
    }

    fn jacobian(
        &self,
        points: &HashMap<String, Point>,
        id_to_index: &HashMap<String, usize>,
    ) -> DMatrix<f64> {
        let cols = points.len() * 2;
        let mut J = DMatrix::<f64>::zeros(1, cols);

        J[(0, id_to_index[&self.p1] * 2 + 1)] = 1.0; // derivative wrt p1.y
        J[(0, id_to_index[&self.p2] * 2 + 1)] = -1.0; // derivative wrt p2.y

        // J[(0, self.p1 * 2 + 1)] = 1.0; // derivative wrt p1.y
        // J[(0, self.p2 * 2 + 1)] = -1.0; // derivative wrt p2.y

        J
    }
}
