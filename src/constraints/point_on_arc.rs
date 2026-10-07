//! [`PointOnArcConstraint`]: constrains a point to lie on an arc's span.

use nalgebra::DMatrix;

use crate::constraints::arc_span::span_overshoot;
use crate::constraints::{Constraint, Var, set_row, xy};

/// Constrains a point to lie on an arc's span: on its circle, between its
/// start and end angles (counter-clockwise).
///
/// Entities:
///   - `point_id`  – the point (px, py)
///   - `center_id` – the arc's center point (cx, cy)
///   - `arc_id`    – the arc (radius r, start angle α, end angle β)
///
/// Residuals, with v = p − c, ρ = |v| and φ = atan2(vy, vx):
///   R₀ = ρ − r                   (on the circle)
///   R₁ = r · overshoot(φ, α, β)  (on the span: 0 while φ is within it,
///                                 else the arc length past the nearer end)
pub struct PointOnArcConstraint {
    /// The point (px, py).
    pub point_id: String,
    /// The arc's center point (cx, cy).
    pub center_id: String,
    /// The arc (radius r, start angle α, end angle β).
    pub arc_id: String,
}

impl PointOnArcConstraint {
    /// The constraint over these entities, by ID (see the fields).
    pub fn new(point_id: String, center_id: String, arc_id: String) -> Self {
        Self {
            point_id,
            center_id,
            arc_id,
        }
    }
}

impl Constraint for PointOnArcConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [
            &xy(&self.point_id)[..],
            &xy(&self.center_id),
            &[
                Var::Radius(&self.arc_id),
                Var::StartAngle(&self.arc_id),
                Var::EndAngle(&self.arc_id),
            ],
        ]
        .concat()
    }

    fn num_residuals(&self) -> usize {
        2
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        // x = [px, py, cx, cy, r, α, β]
        let (vx, vy, rad) = (x[0] - x[2], x[1] - x[3], x[4]);
        let rho2 = vx * vx + vy * vy;
        let rho = rho2.sqrt();
        r[0] = rho - rad;
        j[(0, 4)] = -1.0;
        if rho < 1e-300 {
            // The point sits on the center: no direction to measure.
            return;
        }
        set_row(j, 0, &[vx / rho, vy / rho, -vx / rho, -vy / rho, -1.0]);

        // ∂φ/∂(px, py, cx, cy)
        let dphi = [-vy / rho2, vx / rho2, vy / rho2, -vx / rho2];
        let (over, [d_phi, d_start, d_end]) = span_overshoot(vy.atan2(vx), x[5], x[6]);
        r[1] = rad * over;
        let row: Vec<f64> = dphi
            .iter()
            .map(|g| rad * d_phi * g)
            .chain([over, rad * d_start, rad * d_end])
            .collect();
        set_row(j, 1, &row);
    }
}
