#![allow(non_snake_case)] // Makes sense for mathematical variables

use nalgebra::DMatrix;

use crate::constraints::{Constraint, Var, set_row, xy};

pub struct ParallelConstraint {
    pub p1: String, // Index of the first point (L1P1)
    pub p2: String, // Index of the second point (L1P2)
    pub p3: String, // Index of the third point (L2P1)
    pub p4: String, // Index of the fourth point (L2P2)
}

impl ParallelConstraint {
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
        r[0] = dx1 * dy2 - dy1 * dx2;
        set_row(j, 0, &[-dy2, dx2, dy2, -dx2, dy1, -dx1, -dy1, dx1]);
    }
}
