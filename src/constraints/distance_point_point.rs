#![allow(non_snake_case)]

use nalgebra::{DMatrix, DVector};

use crate::{ParameterManager, constraints::Constraint};

/// Constrains the Euclidean distance between two points to a fixed value.
///
/// Residual (squared form, smooth everywhere):
///   R = (x2−x1)² + (y2−y1)² − d² = 0
pub struct DistancePointPointConstraint {
    pub p1_id: String,
    pub p2_id: String,
    pub distance: f64,
}

impl DistancePointPointConstraint {
    pub fn new(p1_id: String, p2_id: String, distance: f64) -> Self {
        Self {
            p1_id,
            p2_id,
            distance,
        }
    }
}

impl Constraint for DistancePointPointConstraint {
    fn num_residuals(&self) -> usize {
        1
    }

    fn residual(&self, pm: &ParameterManager) -> DVector<f64> {
        let p = pm.get_parameters();
        let x1 = p[pm.get_global_index(&self.p1_id, 0).expect("p1.x")];
        let y1 = p[pm.get_global_index(&self.p1_id, 1).expect("p1.y")];
        let x2 = p[pm.get_global_index(&self.p2_id, 0).expect("p2.x")];
        let y2 = p[pm.get_global_index(&self.p2_id, 1).expect("p2.y")];

        let dx = x2 - x1;
        let dy = y2 - y1;
        let d2 = self.distance * self.distance;

        DVector::from(vec![dx * dx + dy * dy - d2])
    }

    fn jacobian(&self, pm: &ParameterManager) -> DMatrix<f64> {
        let n = pm.num_parameters();
        let mut J = DMatrix::<f64>::zeros(1, n);

        let p = pm.get_parameters();
        let i_x1 = pm.get_global_index(&self.p1_id, 0).expect("p1.x");
        let i_y1 = pm.get_global_index(&self.p1_id, 1).expect("p1.y");
        let i_x2 = pm.get_global_index(&self.p2_id, 0).expect("p2.x");
        let i_y2 = pm.get_global_index(&self.p2_id, 1).expect("p2.y");

        let dx = p[i_x2] - p[i_x1];
        let dy = p[i_y2] - p[i_y1];

        J[(0, i_x1)] = -2.0 * dx;
        J[(0, i_y1)] = -2.0 * dy;
        J[(0, i_x2)] = 2.0 * dx;
        J[(0, i_y2)] = 2.0 * dy;

        J
    }
}
