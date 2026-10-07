//! [`FixedRadiusConstraint`]: forces a circle or arc to maintain a specific
//! radius.

#![allow(non_snake_case)]

use nalgebra::DMatrix;

use crate::constraints::{Constraint, Var, set_row};

/// Forces a circle or arc to maintain a specific radius.
///
/// Residual: r − r₀ = 0
pub struct FixedRadiusConstraint {
    /// The circle or arc.
    pub circle_id: String,
    /// The radius it is held at.
    pub target_radius: f64,
}

impl FixedRadiusConstraint {
    /// The constraint over these entities, by ID (see the fields).
    pub fn new(circle_id: String, target_radius: f64) -> Self {
        Self {
            circle_id,
            target_radius,
        }
    }
}

impl Constraint for FixedRadiusConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        vec![Var::Radius(&self.circle_id)]
    }

    fn num_residuals(&self) -> usize {
        1
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        r[0] = x[0] - self.target_radius;
        set_row(j, 0, &[1.0]);
    }
}
