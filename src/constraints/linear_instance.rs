#![allow(non_snake_case)]

use nalgebra::DMatrix;

use crate::constraints::{Constraint, Var, set_row, xy};

/// Constrains `pk` to be `p0` translated `base_distance · n` along the
/// direction from `dir_p1` to `dir_p2`: the n-th instance of a linear array.
/// PlaneGCS's `linear_instance`.
///
/// Entities:
///   - `p0_id`         – source point p0
///   - `pk_id`         – instance point pk
///   - `dir_p1_id`     – start of the direction (d1)
///   - `dir_p2_id`     – end of the direction (d2)
///   - `base_distance` – spacing between consecutive instances
///   - `n`             – instance index (1 for the first copy)
///
/// With D = d2 − d1, L = |D|, u = D / L and s = base_distance · n:
///
/// Residuals: pk − p0 − s·u (x and y). For a degenerate direction (L ≈ 0)
/// the residual is pk − p0.
///
/// The direction points are Guides: a drag of the source or the copy never
/// moves them, while other real constraints can (see `Constraint::guides`).
pub struct LinearInstanceConstraint {
    pub p0_id: String,
    pub pk_id: String,
    pub dir_p1_id: String,
    pub dir_p2_id: String,
    pub base_distance: f64,
    pub n: f64,
}

impl LinearInstanceConstraint {
    pub fn new(
        p0_id: String,
        pk_id: String,
        dir_p1_id: String,
        dir_p2_id: String,
        base_distance: f64,
        n: f64,
    ) -> Self {
        Self {
            p0_id,
            pk_id,
            dir_p1_id,
            dir_p2_id,
            base_distance,
            n,
        }
    }
}

impl Constraint for LinearInstanceConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [xy(&self.p0_id), xy(&self.pk_id)].concat()
    }

    /// The direction points are Guides: a drag never moves them through this
    /// constraint.
    fn guides(&self) -> Vec<Var<'_>> {
        [xy(&self.dir_p1_id), xy(&self.dir_p2_id)].concat()
    }

    fn num_residuals(&self) -> usize {
        2
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        // x = [p0x, p0y, pkx, pky | d1x, d1y, d2x, d2y]
        let s = self.base_distance * self.n;
        let D = [x[6] - x[4], x[7] - x[5]];
        let L = (D[0] * D[0] + D[1] * D[1]).sqrt();
        let degenerate = L < 1e-12;
        let u = if degenerate {
            [0.0, 0.0]
        } else {
            [D[0] / L, D[1] / L]
        };

        for i in 0..2 {
            r[i] = x[2 + i] - x[i] - s * u[i];
            let mut g = [0.0; 8];
            g[i] = -1.0;
            g[2 + i] = 1.0;
            if !degenerate {
                // ∂u_i/∂D_k = (δ_ik − u_i·u_k) / L, with D = d2 − d1.
                for k in 0..2 {
                    let du = (f64::from(u8::from(i == k)) - u[i] * u[k]) / L;
                    g[4 + k] = s * du;
                    g[6 + k] = -s * du;
                }
            }
            set_row(j, i, &g);
        }
    }
}
