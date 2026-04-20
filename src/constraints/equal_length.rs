#![allow(non_snake_case)]

use nalgebra::{DMatrix, DVector};

use crate::{ParameterManager, constraints::Constraint};

/// Constrains two line segments to have the same length.
///
/// Segment 1: p1 → p2, Segment 2: p3 → p4.
///
/// Residual (squared form, smooth everywhere):
///   R = (dx1² + dy1²) − (dx2² + dy2²) = 0
///
/// where dx1 = x2−x1, dy1 = y2−y1, dx2 = x4−x3, dy2 = y4−y3.
pub struct EqualLengthConstraint {
    pub p1: String,
    pub p2: String,
    pub p3: String,
    pub p4: String,
}

impl EqualLengthConstraint {
    pub fn new(p1: String, p2: String, p3: String, p4: String) -> Self {
        Self { p1, p2, p3, p4 }
    }
}

impl Constraint for EqualLengthConstraint {
    fn num_residuals(&self) -> usize {
        1
    }

    fn residual(&self, pm: &ParameterManager) -> DVector<f64> {
        let p = pm.get_parameters();
        let x1 = p[pm.get_global_index(&self.p1, 0).expect("p1.x")];
        let y1 = p[pm.get_global_index(&self.p1, 1).expect("p1.y")];
        let x2 = p[pm.get_global_index(&self.p2, 0).expect("p2.x")];
        let y2 = p[pm.get_global_index(&self.p2, 1).expect("p2.y")];
        let x3 = p[pm.get_global_index(&self.p3, 0).expect("p3.x")];
        let y3 = p[pm.get_global_index(&self.p3, 1).expect("p3.y")];
        let x4 = p[pm.get_global_index(&self.p4, 0).expect("p4.x")];
        let y4 = p[pm.get_global_index(&self.p4, 1).expect("p4.y")];

        let dx1 = x2 - x1;
        let dy1 = y2 - y1;
        let dx2 = x4 - x3;
        let dy2 = y4 - y3;

        DVector::from(vec![(dx1 * dx1 + dy1 * dy1) - (dx2 * dx2 + dy2 * dy2)])
    }

    fn jacobian(&self, pm: &ParameterManager) -> DMatrix<f64> {
        let n = pm.num_parameters();
        let mut J = DMatrix::<f64>::zeros(1, n);

        let p = pm.get_parameters();
        let i1x = pm.get_global_index(&self.p1, 0).expect("p1.x");
        let i1y = pm.get_global_index(&self.p1, 1).expect("p1.y");
        let i2x = pm.get_global_index(&self.p2, 0).expect("p2.x");
        let i2y = pm.get_global_index(&self.p2, 1).expect("p2.y");
        let i3x = pm.get_global_index(&self.p3, 0).expect("p3.x");
        let i3y = pm.get_global_index(&self.p3, 1).expect("p3.y");
        let i4x = pm.get_global_index(&self.p4, 0).expect("p4.x");
        let i4y = pm.get_global_index(&self.p4, 1).expect("p4.y");

        let dx1 = p[i2x] - p[i1x];
        let dy1 = p[i2y] - p[i1y];
        let dx2 = p[i4x] - p[i3x];
        let dy2 = p[i4y] - p[i3y];

        J[(0, i1x)] = -2.0 * dx1;
        J[(0, i1y)] = -2.0 * dy1;
        J[(0, i2x)] = 2.0 * dx1;
        J[(0, i2y)] = 2.0 * dy1;
        J[(0, i3x)] = 2.0 * dx2;
        J[(0, i3y)] = 2.0 * dy2;
        J[(0, i4x)] = -2.0 * dx2;
        J[(0, i4y)] = -2.0 * dy2;

        J
    }
}
