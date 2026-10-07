//! [`VerticalConstraint`]: two points (a Line's endpoints) are plumb.

#![allow(non_snake_case)] // Makes sense for mathematical variables

use nalgebra::DMatrix;

use crate::constraints::{Constraint, Var, set_row};

/// Two points (a Line's endpoints) are plumb: `R = x2 − x1`.
pub struct VerticalConstraint {
    /// The first point (a Line's start).
    pub p1: String,
    /// The second point (a Line's end).
    pub p2: String,
}

impl VerticalConstraint {
    /// The constraint over these entities, by ID (see the fields).
    pub fn new(p1: String, p2: String) -> Self {
        Self { p1, p2 }
    }
}

impl Constraint for VerticalConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        vec![Var::X(&self.p1), Var::X(&self.p2)]
    }

    fn num_residuals(&self) -> usize {
        1
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        r[0] = x[0] - x[1];
        set_row(j, 0, &[1.0, -1.0]);
    }
}
