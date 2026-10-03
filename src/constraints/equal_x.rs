#![allow(non_snake_case)] // Makes sense for mathematical variables

use nalgebra::DMatrix;

use crate::constraints::{Constraint, Var, set_row};

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
    fn vars(&self) -> Vec<Var<'_>> {
        vec![Var::X(&self.p1)]
    }

    fn num_residuals(&self) -> usize {
        1
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        r[0] = x[0] - self.x;
        set_row(j, 0, &[1.0]);
    }
}
