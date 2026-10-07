//! [`CoincidentConstraint`]: two points coincide.

#![allow(non_snake_case)] // Makes sense for mathematical variables

use nalgebra::DMatrix;

use crate::constraints::{Constraint, Var, set_row, xy};

/// Two points coincide.
///
/// Residuals: `R₀ = x2 − x1`, `R₁ = y2 − y1`.
pub struct CoincidentConstraint {
    /// The first point.
    pub p1: String,
    /// The second point.
    pub p2: String,
}

impl CoincidentConstraint {
    /// The constraint over these entities, by ID (see the fields).
    pub fn new(p1: String, p2: String) -> Self {
        Self { p1, p2 }
    }
}

impl Constraint for CoincidentConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [xy(&self.p1), xy(&self.p2)].concat()
    }

    fn num_residuals(&self) -> usize {
        2
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        r[0] = x[0] - x[2];
        r[1] = x[1] - x[3];
        set_row(j, 0, &[1.0, 0.0, -1.0, 0.0]);
        set_row(j, 1, &[0.0, 1.0, 0.0, -1.0]);
    }
}
