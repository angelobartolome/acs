//! [`EqualLengthConstraint`]: constrains two line segments to have the same
//! length.

#![allow(non_snake_case)]

use nalgebra::DMatrix;

use crate::constraints::directions::unit;
use crate::constraints::{Constraint, Var, set_row, xy};

/// Constrains two line segments to have the same length.
///
/// Segment 1: p1 → p2, Segment 2: p3 → p4.
///
/// Residual, a length (see SPEC.md, Notation, Units):
///   R = |p2 − p1| − |p4 − p3|
pub struct EqualLengthConstraint {
    /// Start of segment 1.
    pub p1: String,
    /// End of segment 1.
    pub p2: String,
    /// Start of segment 2.
    pub p3: String,
    /// End of segment 2.
    pub p4: String,
}

impl EqualLengthConstraint {
    /// The constraint over these entities, by ID (see the fields).
    pub fn new(p1: String, p2: String, p3: String, p4: String) -> Self {
        Self { p1, p2, p3, p4 }
    }
}

impl Constraint for EqualLengthConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [xy(&self.p1), xy(&self.p2), xy(&self.p3), xy(&self.p4)].concat()
    }

    fn num_residuals(&self) -> usize {
        1
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        let (u1, n1) = unit(x[2] - x[0], x[3] - x[1]);
        let (u2, n2) = unit(x[6] - x[4], x[7] - x[5]);
        r[0] = n1 - n2;
        set_row(j, 0, &[-u1[0], -u1[1], u1[0], u1[1], u2[0], u2[1], -u2[0], -u2[1]]);
    }
}
