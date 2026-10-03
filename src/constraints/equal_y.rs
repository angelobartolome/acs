#![allow(non_snake_case)] // Makes sense for mathematical variables

use nalgebra::DMatrix;

use crate::constraints::{Constraint, Var, set_row};

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
    fn vars(&self) -> Vec<Var<'_>> {
        vec![Var::Y(&self.p1)]
    }

    fn num_residuals(&self) -> usize {
        1
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        r[0] = x[0] - self.y;
        set_row(j, 0, &[1.0]);
    }
}
