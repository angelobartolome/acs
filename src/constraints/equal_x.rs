//! [`EqualXConstraint`]: a point is held at an x coordinate.

#![allow(non_snake_case)] // Makes sense for mathematical variables

use nalgebra::DMatrix;

use crate::constraints::{Constraint, Var, set_row};

/// A point is held at an x coordinate: `R = x − x₀`.
pub struct EqualXConstraint {
    /// The point.
    pub p1: String,
    /// The x coordinate it is held at.
    pub x: f64,
}

impl EqualXConstraint {
    /// The constraint over these entities, by ID (see the fields).
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
