//! [`DistancePointCircleConstraint`]: fixes the gap between a point and a
//! circle, signed so the residual crosses zero linearly.

use nalgebra::DMatrix;

use crate::constraints::{Constraint, Var, set_row, xy};

/// Fixes the gap between a point and a circle, signed so the residual
/// crosses zero linearly: outside, `|p − c| − r` (negative while the point
/// is inside); with `internal`, `r − |p − c|` (negative while it is
/// outside).
///
/// Entities:
///   - `point_id`         – the point (px, py)
///   - `circle_center_id` – center point of the circle (cx, cy)
///   - `circle_id`        – the circle (radius r)
///
/// Residual: R = σ · (|p − c| − r) − distance, σ = −1 if `internal`, else +1
pub struct DistancePointCircleConstraint {
    /// The point (px, py).
    pub point_id: String,
    /// Center point of the circle (cx, cy).
    pub circle_center_id: String,
    /// The circle (radius r).
    pub circle_id: String,
    /// The gap between the point and the circle.
    pub distance: f64,
    /// The inside variant (`internal: true`).
    pub internal: bool,
}

impl DistancePointCircleConstraint {
    /// The constraint over these entities, by ID (see the fields).
    pub fn new(
        point_id: String,
        circle_center_id: String,
        circle_id: String,
        distance: f64,
        internal: bool,
    ) -> Self {
        Self {
            point_id,
            circle_center_id,
            circle_id,
            distance,
            internal,
        }
    }
}

impl Constraint for DistancePointCircleConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [
            &xy(&self.point_id)[..],
            &xy(&self.circle_center_id),
            &[Var::Radius(&self.circle_id)],
        ]
        .concat()
    }

    fn num_residuals(&self) -> usize {
        1
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        // x = [px, py, cx, cy, r]
        let (dx, dy) = (x[0] - x[2], x[1] - x[3]);
        let d = dx.hypot(dy);
        let (ux, uy) = if d > 1e-300 { (dx / d, dy / d) } else { (0.0, 0.0) };
        let sigma = if self.internal { -1.0 } else { 1.0 };
        r[0] = sigma * (d - x[4]) - self.distance;
        set_row(
            j,
            0,
            &[sigma * ux, sigma * uy, -sigma * ux, -sigma * uy, -sigma],
        );
    }
}
