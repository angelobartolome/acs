#![allow(non_snake_case)] // Makes sense for mathematical variables

use nalgebra::DMatrix;

use crate::constraints::{Constraint, Var, set_row};

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
    fn vars(&self) -> Vec<Var<'_>> {
        vec![Var::Y(&self.p1), Var::Y(&self.p2)]
    }

    fn num_residuals(&self) -> usize {
        1
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        r[0] = x[0] - x[1];
        set_row(j, 0, &[1.0, -1.0]);
    }
}
