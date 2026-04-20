#![allow(non_snake_case)]

use nalgebra::{DMatrix, DVector};

use crate::{ParameterManager, constraints::Constraint};

/// Forces a circle or arc to maintain a specific radius.
///
/// Residual: r − r₀ = 0
pub struct FixedRadiusConstraint {
    pub circle_id: String,
    pub target_radius: f64,
}

impl FixedRadiusConstraint {
    pub fn new(circle_id: String, target_radius: f64) -> Self {
        Self {
            circle_id,
            target_radius,
        }
    }
}

impl Constraint for FixedRadiusConstraint {
    fn num_residuals(&self) -> usize {
        1
    }

    fn residual(&self, pm: &ParameterManager) -> DVector<f64> {
        let r_idx = pm
            .get_global_index(&self.circle_id, 0)
            .expect("circle radius not found");
        let r = pm.get_parameters()[r_idx];
        DVector::from(vec![r - self.target_radius])
    }

    fn jacobian(&self, pm: &ParameterManager) -> DMatrix<f64> {
        let n = pm.num_parameters();
        let mut J = DMatrix::<f64>::zeros(1, n);
        if let Some(r_idx) = pm.get_global_index(&self.circle_id, 0) {
            J[(0, r_idx)] = 1.0;
        }
        J
    }
}
