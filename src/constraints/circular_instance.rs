use nalgebra::DMatrix;

use crate::constraints::{Constraint, Var, set_row, xy};

/// Constrains `pk` to be `p0` rotated about `center` by a fixed angle: one
/// instance of a circular array. PlaneGCS's `circular_instance`.
///
/// Entities:
///   - `p0_id`     – source point p0
///   - `pk_id`     – instance point pk
///   - `center_id` – rotation center c
///   - `angle`     – rotation θ in radians, counter-clockwise
///
/// Residuals: pk − (c + R(θ)·(p0 − c)), i.e.
///   r₀ = pkx − (cx + cosθ·(p0x − cx) − sinθ·(p0y − cy))
///   r₁ = pky − (cy + sinθ·(p0x − cx) + cosθ·(p0y − cy))
///
/// The center is a Guide (see `Constraint::guides`): the copy follows it, and
/// the solve moves it, if it's free, as the constraints and drags need.
pub struct CircularInstanceConstraint {
    pub p0_id: String,
    pub pk_id: String,
    pub center_id: String,
    pub angle: f64,
}

impl CircularInstanceConstraint {
    pub fn new(p0_id: String, pk_id: String, center_id: String, angle: f64) -> Self {
        Self {
            p0_id,
            pk_id,
            center_id,
            angle,
        }
    }
}

impl Constraint for CircularInstanceConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [xy(&self.p0_id), xy(&self.pk_id)].concat()
    }

    /// The center is a Guide: the input the copy is rotated about.
    fn guides(&self) -> Vec<Var<'_>> {
        xy(&self.center_id).to_vec()
    }

    fn num_residuals(&self) -> usize {
        2
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        // x = [p0x, p0y, pkx, pky | cx, cy]
        let (s, c) = self.angle.sin_cos();
        let (vx, vy) = (x[0] - x[4], x[1] - x[5]);
        r[0] = x[2] - (x[4] + c * vx - s * vy);
        r[1] = x[3] - (x[5] + s * vx + c * vy);
        // ∂r/∂center = −(I − R(θ)).
        set_row(j, 0, &[-c, s, 1.0, 0.0, c - 1.0, -s]);
        set_row(j, 1, &[-s, -c, 0.0, 1.0, s, c - 1.0]);
    }
}
