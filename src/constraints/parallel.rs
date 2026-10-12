//! [`ParallelConstraint`]: two lines are parallel.

#![allow(non_snake_case)] // Makes sense for mathematical variables

use std::f64::consts::PI;

use nalgebra::DMatrix;

use crate::constraints::directions::set_direction_row;
use crate::constraints::{Constraint, Var, xy};

/// Two lines, p1 → p2 and p3 → p4, are parallel (or antiparallel).
///
/// Residual, the angle between them modulo π, in radians (see SPEC.md,
/// Notation, Units):
///   R = atan2(d1 × d2, d1 · d2) wrapped into [−π/2, π/2),  d1 = p2 − p1, d2 = p4 − p3
pub struct ParallelConstraint {
    /// Start of line L1.
    pub p1: String,
    /// End of line L1.
    pub p2: String,
    /// Start of line L2.
    pub p3: String,
    /// End of line L2.
    pub p4: String,
}

impl ParallelConstraint {
    /// The constraint over these entities, by ID (see the fields).
    pub fn new(p1: String, p2: String, p3: String, p4: String) -> Self {
        Self { p1, p2, p3, p4 }
    }
}

impl Constraint for ParallelConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [xy(&self.p1), xy(&self.p2), xy(&self.p3), xy(&self.p4)].concat()
    }

    fn num_residuals(&self) -> usize {
        1
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        let (dx1, dy1) = (x[2] - x[0], x[3] - x[1]);
        let (dx2, dy2) = (x[6] - x[4], x[7] - x[5]);
        let cross = dx1 * dy2 - dy1 * dx2;
        set_direction_row([dx1, dy1], [dx2, dy2], 0.0, PI, (cross, [dy2, -dx2], [-dy1, dx1]), r, j);
    }
}
