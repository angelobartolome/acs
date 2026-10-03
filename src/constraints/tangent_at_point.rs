use nalgebra::DMatrix;

use crate::constraints::{Constraint, Var, set_row, xy};

/// A Line tangent to an arc at a Point they share: the line's direction is
/// perpendicular to the arc's radius at that point.
///
/// Used instead of the distance form (`TangentLineArc`: distance from the
/// center to the line = radius) when a line endpoint *is* one of the arc's
/// endpoints. Through a point already on the arc, that distance can only fall
/// short of the radius, so it changes quadratically as the line turns and its
/// Jacobian row vanishes at tangency: diagnosis then reports drag-dependent
/// Redundant constraints and `dof` comes out too high. The angle at the point
/// changes linearly. The shared point lies on the line (it's an endpoint) and
/// on the arc (the arc's implicit rules), so no segment or span check is
/// needed.
///
/// Entities:
///   - `point_id`   – the shared point p (a line endpoint and an arc endpoint)
///   - `other_id`   – the line's other endpoint o
///   - `center_id`  – the arc's center c
///
/// Let u = o − p (the line's direction) and v = p − c (the radius at p).
///
/// Residual: u·v / (|u|·|v|), the cosine of the angle between them (0 at
/// tangency). For a degenerate line or radius (|u| or |v| ≈ 0) the
/// unnormalized u·v is used.
pub struct TangentAtPointConstraint {
    pub point_id: String,
    pub other_id: String,
    pub center_id: String,
}

impl TangentAtPointConstraint {
    pub fn new(point_id: String, other_id: String, center_id: String) -> Self {
        Self {
            point_id,
            other_id,
            center_id,
        }
    }
}

impl Constraint for TangentAtPointConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [xy(&self.point_id), xy(&self.other_id), xy(&self.center_id)].concat()
    }

    fn num_residuals(&self) -> usize {
        1
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        // x = [px, py, ox, oy, cx, cy]
        let (ux, uy) = (x[2] - x[0], x[3] - x[1]);
        let (vx, vy) = (x[0] - x[4], x[1] - x[5]);
        let f = ux * vx + uy * vy;
        // ∂f/∂(px, py, ox, oy, cx, cy): u depends on p with −1, v with +1.
        let df = [ux - vx, uy - vy, vx, vy, -ux, -uy];

        let (nu, nv) = ((ux * ux + uy * uy).sqrt(), (vx * vx + vy * vy).sqrt());
        if nu < 1e-12 || nv < 1e-12 {
            r[0] = f;
            set_row(j, 0, &df);
            return;
        }

        // r = f / (|u||v|)  ⇒  ∂r = ∂f/(|u||v|) − r·(∂|u|/|u| + ∂|v|/|v|)
        let n = nu * nv;
        r[0] = f / n;
        let dnu = [-ux / nu, -uy / nu, ux / nu, uy / nu, 0.0, 0.0];
        let dnv = [vx / nv, vy / nv, 0.0, 0.0, -vx / nv, -vy / nv];
        let mut g = [0.0; 6];
        for k in 0..6 {
            g[k] = df[k] / n - r[0] * (dnu[k] / nu + dnv[k] / nv);
        }
        set_row(j, 0, &g);
    }
}
