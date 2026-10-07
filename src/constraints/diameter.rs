//! [`DiameterConstraint`]: fixes a circle's or arc's diameter.

use nalgebra::DMatrix;

use crate::constraints::{Constraint, Var};

/// Fixes a circle's or arc's diameter: the dimension holds the number given,
/// not a radius of half of it.
///
/// Entities:
///   - `circle_id` – the circle or arc (radius r)
///
/// Residual: R = 2r − diameter
pub struct DiameterConstraint {
    /// The circle or arc (radius r).
    pub circle_id: String,
    /// The diameter, `2r`.
    pub diameter: f64,
}

impl DiameterConstraint {
    /// The constraint over these entities, by ID (see the fields).
    pub fn new(circle_id: String, diameter: f64) -> Self {
        Self {
            circle_id,
            diameter,
        }
    }
}

impl Constraint for DiameterConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        vec![Var::Radius(&self.circle_id)]
    }

    fn num_residuals(&self) -> usize {
        1
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        // x = [radius]
        r[0] = 2.0 * x[0] - self.diameter;
        j[(0, 0)] = 2.0;
    }
}
