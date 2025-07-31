#![allow(non_snake_case)] // Makes sense for mathematical variables
#![allow(unused_parens)]
use std::collections::HashMap;

use nalgebra::{DMatrix, DVector};

use crate::{Point, constraints::Constraint};

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
