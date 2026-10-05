use nalgebra::DMatrix;

use crate::constraints::segment::line_signed_distance;
use crate::constraints::{Constraint, Var, xy};

/// Fixes the distance between two Lines: both endpoints of line b are
/// `distance` from line a's Extension, on the same side, so b is parallel
/// to a.
///
/// The side is the one b's midpoint is on (σ = +1 left of a₁→a₂, −1 right),
/// read at each evaluation; at a solution both endpoints are on it.
///
/// Entities:
///   - `a1_id`, `a2_id` – endpoints of line a
///   - `b1_id`, `b2_id` – endpoints of line b
///
/// Residuals:
///   R₀ = σ · C(b₁) / L − distance
///   R₁ = σ · C(b₂) / L − distance
/// with C(p) / L the signed distance from p to the line through a₁, a₂.
pub struct DistanceLineLineConstraint {
    pub a1_id: String,
    pub a2_id: String,
    pub b1_id: String,
    pub b2_id: String,
    pub distance: f64,
}

impl DistanceLineLineConstraint {
    pub fn new(a1_id: String, a2_id: String, b1_id: String, b2_id: String, distance: f64) -> Self {
        Self {
            a1_id,
            a2_id,
            b1_id,
            b2_id,
            distance,
        }
    }
}

impl Constraint for DistanceLineLineConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [
            &xy(&self.a1_id)[..],
            &xy(&self.a2_id),
            &xy(&self.b1_id),
            &xy(&self.b2_id),
        ]
        .concat()
    }

    fn num_residuals(&self) -> usize {
        2
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        // x = [a1x, a1y, a2x, a2y, b1x, b1y, b2x, b2y]
        let (d1, g1) = line_signed_distance(x[4], x[5], x[0], x[1], x[2], x[3]);
        let (d2, g2) = line_signed_distance(x[6], x[7], x[0], x[1], x[2], x[3]);
        let sigma = if d1 + d2 >= 0.0 { 1.0 } else { -1.0 };
        // Grad6 is over (px, py, ax, ay, bx, by): p is b₁ (columns 4, 5) or
        // b₂ (6, 7); a and b are a₁, a₂ (0..4).
        for (row, d, g, p) in [(0, d1, g1, 4), (1, d2, g2, 6)] {
            r[row] = sigma * d - self.distance;
            j[(row, p)] = sigma * g[0];
            j[(row, p + 1)] = sigma * g[1];
            for k in 0..4 {
                j[(row, k)] = sigma * g[k + 2];
            }
        }
    }
}
