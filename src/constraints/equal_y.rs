//! [`EqualYConstraint`]: a point is held at a y coordinate.

#![allow(non_snake_case)] // Makes sense for mathematical variables

use nalgebra::DMatrix;

use crate::constraints::{Constraint, Var, set_row};

/// A point is held at a y coordinate: `R = y − y₀`.
pub struct EqualYConstraint {
    /// The point.
    pub p1: String,
    /// The y coordinate it is held at.
    pub y: f64,
}

impl EqualYConstraint {
    /// The constraint over these entities, by ID (see the fields).
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
