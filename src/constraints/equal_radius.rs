#![allow(non_snake_case)] // Makes sense for mathematical variables

use nalgebra::DMatrix;

use crate::constraints::{Constraint, Var, set_row};

pub struct EqualRadiusConstraint {
    pub circle1_id: String, // ID of the first circle
    pub circle2_id: String, // ID of the second circle
}

impl EqualRadiusConstraint {
    pub fn new(circle1_id: String, circle2_id: String) -> Self {
        Self {
            circle1_id,
            circle2_id,
        }
    }
}

impl Constraint for EqualRadiusConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        vec![Var::Radius(&self.circle1_id), Var::Radius(&self.circle2_id)]
    }

    fn num_residuals(&self) -> usize {
        1
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        r[0] = x[0] - x[1];
        set_row(j, 0, &[1.0, -1.0]);
    }
}
