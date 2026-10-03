use nalgebra::DMatrix;

use crate::constraints::{Constraint, Var, set_row, xy};

/// Constrains the direction from one point to another: the vector p1 → p2
/// makes `angle` (radians, counter-clockwise from +x) with the x axis.
/// PlaneGCS's `p2p_angle`.
///
/// Entities:
///   - `p1_id` – start point
///   - `p2_id` – end point
///   - `angle` – target direction θ in radians
///
/// Let (dx, dy) = p2 − p1, rotated by −θ: u = dx·cosθ + dy·sinθ,
/// v = −dx·sinθ + dy·cosθ.
///
/// Residual: atan2(v, u), the signed angle (in (−π, π]) from θ to the
/// direction of p1 → p2. Unlike a sin/cos form it has no spurious root at
/// θ + π. Partials: ∂/∂dx = −dy / |d|², ∂/∂dy = dx / |d|²; zero when the
/// points coincide (the residual is then 0).
pub struct PointPointAngleConstraint {
    pub p1_id: String,
    pub p2_id: String,
    pub angle: f64,
}

impl PointPointAngleConstraint {
    pub fn new(p1_id: String, p2_id: String, angle: f64) -> Self {
        Self {
            p1_id,
            p2_id,
            angle,
        }
    }
}

impl Constraint for PointPointAngleConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [xy(&self.p1_id), xy(&self.p2_id)].concat()
    }

    fn num_residuals(&self) -> usize {
        1
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        // x = [x1, y1, x2, y2]
        let (dx, dy) = (x[2] - x[0], x[3] - x[1]);
        let (s, c) = self.angle.sin_cos();
        let u = dx * c + dy * s;
        let v = -dx * s + dy * c;
        r[0] = v.atan2(u);

        let r2 = dx * dx + dy * dy;
        if r2 < 1e-300 {
            return;
        }
        let (gx, gy) = (-dy / r2, dx / r2);
        set_row(j, 0, &[-gx, -gy, gx, gy]);
    }
}
