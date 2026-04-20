#![allow(non_snake_case)]

use nalgebra::{DMatrix, DVector};

use crate::{ParameterManager, constraints::Constraint};

/// Forces two lines to be perpendicular (dot product of direction vectors = 0).
///
/// Line 1: p1 → p2, direction d1 = (x2−x1, y2−y1)
/// Line 2: p3 → p4, direction d2 = (x4−x3, y4−y3)
/// Residual: d1·d2 = dx1·dx2 + dy1·dy2 = 0
pub struct PerpendicularConstraint {
    pub p1: String,
    pub p2: String,
    pub p3: String,
    pub p4: String,
}

impl PerpendicularConstraint {
    pub fn new(p1: String, p2: String, p3: String, p4: String) -> Self {
        Self { p1, p2, p3, p4 }
    }
}

impl Constraint for PerpendicularConstraint {
    fn num_residuals(&self) -> usize {
        1
    }

    fn residual(&self, pm: &ParameterManager) -> DVector<f64> {
        let x1 = pm.get_parameters()[pm.get_global_index(&self.p1, 0).expect("p1.x")];
        let y1 = pm.get_parameters()[pm.get_global_index(&self.p1, 1).expect("p1.y")];
        let x2 = pm.get_parameters()[pm.get_global_index(&self.p2, 0).expect("p2.x")];
        let y2 = pm.get_parameters()[pm.get_global_index(&self.p2, 1).expect("p2.y")];
        let x3 = pm.get_parameters()[pm.get_global_index(&self.p3, 0).expect("p3.x")];
        let y3 = pm.get_parameters()[pm.get_global_index(&self.p3, 1).expect("p3.y")];
        let x4 = pm.get_parameters()[pm.get_global_index(&self.p4, 0).expect("p4.x")];
        let y4 = pm.get_parameters()[pm.get_global_index(&self.p4, 1).expect("p4.y")];

        let dx1 = x2 - x1;
        let dy1 = y2 - y1;
        let dx2 = x4 - x3;
        let dy2 = y4 - y3;

        DVector::from(vec![dx1 * dx2 + dy1 * dy2])
    }

    fn jacobian(&self, pm: &ParameterManager) -> DMatrix<f64> {
        let n = pm.num_parameters();
        let mut J = DMatrix::<f64>::zeros(1, n);

        let i1x = pm.get_global_index(&self.p1, 0).expect("p1.x");
        let i1y = pm.get_global_index(&self.p1, 1).expect("p1.y");
        let i2x = pm.get_global_index(&self.p2, 0).expect("p2.x");
        let i2y = pm.get_global_index(&self.p2, 1).expect("p2.y");
        let i3x = pm.get_global_index(&self.p3, 0).expect("p3.x");
        let i3y = pm.get_global_index(&self.p3, 1).expect("p3.y");
        let i4x = pm.get_global_index(&self.p4, 0).expect("p4.x");
        let i4y = pm.get_global_index(&self.p4, 1).expect("p4.y");

        let p = pm.get_parameters();
        let dx1 = p[i2x] - p[i1x];
        let dy1 = p[i2y] - p[i1y];
        let dx2 = p[i4x] - p[i3x];
        let dy2 = p[i4y] - p[i3y];

        J[(0, i1x)] = -dx2;
        J[(0, i1y)] = -dy2;
        J[(0, i2x)] = dx2;
        J[(0, i2y)] = dy2;
        J[(0, i3x)] = -dx1;
        J[(0, i3y)] = -dy1;
        J[(0, i4x)] = dx1;
        J[(0, i4y)] = dy1;

        J
    }
}
