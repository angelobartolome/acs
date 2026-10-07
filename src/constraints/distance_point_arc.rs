//! [`DistancePointArcConstraint`]: fixes the gap between a point and an arc.

use nalgebra::DMatrix;

use crate::constraints::arc_span::span_overshoot;
use crate::constraints::{Constraint, Var, set_row, xy};

/// Fixes the gap between a point and an arc: the point-to-circle gap of
/// `DistancePointCircleConstraint` (signed, outside or, with `internal`,
/// inside), with the nearest point of the arc on its span. The nearest
/// point of the arc's circle lies on the ray from the center through the
/// point, so the span row is `PointOnArcConstraint`'s: a point whose ray
/// misses the span isn't measured to an endpoint (that residual would jump
/// sign there); the solve turns the ray onto the span instead.
///
/// Entities:
///   - `point_id`  – the point (px, py)
///   - `center_id` – the arc's center point (cx, cy)
///   - `arc_id`    – the arc (radius r, start angle α, end angle β)
///
/// Residuals, with v = p − c, ρ = |v| and φ = atan2(vy, vx):
///   R₀ = σ · (ρ − r) − distance,  σ = −1 if `internal`, else +1
///   R₁ = r · overshoot(φ, α, β)   (the nearest point lies on the span)
pub struct DistancePointArcConstraint {
    /// The point (px, py).
    pub point_id: String,
    /// The arc's center point (cx, cy).
    pub center_id: String,
    /// The arc (radius r, start angle α, end angle β).
    pub arc_id: String,
    /// The gap between the point and the arc.
    pub distance: f64,
    /// The inside variant (`internal: true`).
    pub internal: bool,
}

impl DistancePointArcConstraint {
    /// The constraint over these entities, by ID (see the fields).
    pub fn new(
        point_id: String,
        center_id: String,
        arc_id: String,
        distance: f64,
        internal: bool,
    ) -> Self {
        Self {
            point_id,
            center_id,
            arc_id,
            distance,
            internal,
        }
    }
}

impl Constraint for DistancePointArcConstraint {
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
        let sigma = if self.internal { -1.0 } else { 1.0 };
        r[0] = sigma * (rho - rad) - self.distance;
        j[(0, 4)] = -sigma;
        if rho < 1e-300 {
            // The point sits on the center: no direction to measure.
            return;
        }
        let (ux, uy) = (vx / rho, vy / rho);
        set_row(
            j,
            0,
            &[sigma * ux, sigma * uy, -sigma * ux, -sigma * uy, -sigma],
        );

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
