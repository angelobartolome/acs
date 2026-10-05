#![allow(non_snake_case)]

use nalgebra::DMatrix;

use crate::constraints::segment::line_signed_distance;
use crate::constraints::{Constraint, Var, set_row, xy};

/// Constrains `pB` to be the mirror image of `pA` across a Line's Extension
/// (the infinite line through the axis endpoints). Native `mirror`: a
/// mirror reflects across the whole line, so a point beside or beyond the
/// axis segment's ends still has its image.
///
/// Entities:
///   - `pA_id`      – source point A
///   - `pB_id`      – mirrored point B
///   - `axis_pa_id` – first point of the axis (a)
///   - `axis_pb_id` – second point of the axis (b)
///
/// With D = b − a, L = |D|, unit normal n̂ = (−Dy, Dx) / L and
/// s = `line_signed_distance(A; a, b)` (the signed distance from A to the
/// Extension), the reflection of A is A' = A − 2·s·n̂.
///
/// Residuals: B − A' (x and y), linear in B. For a degenerate axis
/// (L ≈ 0) the residual is B − A.
///
/// The axis points are Guides (see `Constraint::guides`): moving the axis
/// moves the image, and the solve moves a free axis as needed.
pub struct MirrorPointExtensionConstraint {
    pub pA_id: String,
    pub pB_id: String,
    pub axis_pa_id: String,
    pub axis_pb_id: String,
}

impl MirrorPointExtensionConstraint {
    pub fn new(pA_id: String, pB_id: String, axis_pa_id: String, axis_pb_id: String) -> Self {
        Self {
            pA_id,
            pB_id,
            axis_pa_id,
            axis_pb_id,
        }
    }
}

impl Constraint for MirrorPointExtensionConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [xy(&self.pA_id), xy(&self.pB_id)].concat()
    }

    /// The axis is a Guide: the input the image is mirrored across.
    fn guides(&self) -> Vec<Var<'_>> {
        [xy(&self.axis_pa_id), xy(&self.axis_pb_id)].concat()
    }

    fn num_residuals(&self) -> usize {
        2
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        // x = [Ax, Ay, Bx, By | ax, ay, bx, by]
        let (Ax, Ay, Bx, By) = (x[0], x[1], x[2], x[3]);
        let (ax, ay, bx, by) = (x[4], x[5], x[6], x[7]);
        let (Dx, Dy) = (bx - ax, by - ay);
        let l2 = Dx * Dx + Dy * Dy;

        if l2 < 1e-24 {
            r[0] = Bx - Ax;
            r[1] = By - Ay;
            set_row(j, 0, &[-1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
            set_row(j, 1, &[0.0, -1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0]);
            return;
        }

        // A' = A − 2·s·n̂, with s the signed distance from A to the Extension
        // and n̂ = (−Dy, Dx) / L its unit normal (the side s is positive on).
        // n̂ depends only on the axis, and ∂s/∂A = n̂.
        let (s, ds6) = line_signed_distance(Ax, Ay, ax, ay, bx, by);
        let L = l2.sqrt();
        let u = [Dx / L, Dy / L];
        let n = [-u[1], u[0]];
        // n̂ = R·u with R the quarter turn and ∂u/∂D = (I − u·uᵀ)/L, so
        // ∂n̂_i/∂D_k = Σ_m R_im·(δ_mk − u_m·u_k)/L, with D = b − a.
        let R = [[0.0, -1.0], [1.0, 0.0]];
        let dn = |i: usize, k: usize| -> f64 {
            (0..2)
                .map(|m| R[i][m] * (f64::from(u8::from(m == k)) - u[m] * u[k]))
                .sum::<f64>()
                / L
        };

        // r_i = B_i − A_i + 2·s·n̂_i
        let A = [Ax, Ay];
        let B = [Bx, By];
        for i in 0..2 {
            r[i] = B[i] - A[i] + 2.0 * s * n[i];
            let mut g = [0.0; 8];
            g[0] = 2.0 * ds6[0] * n[i];
            g[1] = 2.0 * ds6[1] * n[i];
            g[i] -= 1.0; // ∂/∂A_i
            g[2 + i] += 1.0; // ∂/∂B_i
            for k in 0..2 {
                g[4 + k] = 2.0 * (ds6[2 + k] * n[i] - s * dn(i, k)); // ∂/∂a_k
                g[6 + k] = 2.0 * (ds6[4 + k] * n[i] + s * dn(i, k)); // ∂/∂b_k
            }
            set_row(j, i, &g);
        }
    }
}
