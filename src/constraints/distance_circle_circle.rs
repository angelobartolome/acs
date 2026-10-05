use nalgebra::DMatrix;

use crate::constraints::{Constraint, Var, set_row, xy};

/// Fixes the gap between two circles, signed so the residual crosses zero
/// linearly: outside, `|c1 − c2| − r1 − r2` (negative while they overlap);
/// with `internal`, the gap between the smaller circle and the inside of
/// the larger, `|r1 − r2| − |c1 − c2|` (negative while the smaller pokes
/// out). Either circle may be the larger, so `a` and `b` commute.
///
/// Entities:
///   - `c1_center_id` – center point of circle 1 (x1, y1)
///   - `c1_id`        – circle 1 (radius r1)
///   - `c2_center_id` – center point of circle 2 (x2, y2)
///   - `c2_id`        – circle 2 (radius r2)
///
/// Residual: R = |c1 − c2| − r1 − r2 − distance, or (internal)
///           R = |r1 − r2| − |c1 − c2| − distance
pub struct DistanceCircleCircleConstraint {
    pub c1_center_id: String,
    pub c1_id: String,
    pub c2_center_id: String,
    pub c2_id: String,
    pub distance: f64,
    pub internal: bool,
}

impl DistanceCircleCircleConstraint {
    pub fn new(
        c1_center_id: String,
        c1_id: String,
        c2_center_id: String,
        c2_id: String,
        distance: f64,
        internal: bool,
    ) -> Self {
        Self {
            c1_center_id,
            c1_id,
            c2_center_id,
            c2_id,
            distance,
            internal,
        }
    }
}

impl Constraint for DistanceCircleCircleConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [
            &xy(&self.c1_center_id)[..],
            &[Var::Radius(&self.c1_id)],
            &xy(&self.c2_center_id),
            &[Var::Radius(&self.c2_id)],
        ]
        .concat()
    }

    fn num_residuals(&self) -> usize {
        1
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        // x = [x1, y1, r1, x2, y2, r2]
        let (dx, dy) = (x[3] - x[0], x[4] - x[1]);
        let d = dx.hypot(dy);
        // ∂d/∂c2 = u, ∂d/∂c1 = −u
        let (ux, uy) = if d > 1e-300 { (dx / d, dy / d) } else { (0.0, 0.0) };
        if self.internal {
            let s = if x[2] >= x[5] { 1.0 } else { -1.0 };
            r[0] = s * (x[2] - x[5]) - d - self.distance;
            set_row(j, 0, &[ux, uy, s, -ux, -uy, -s]);
        } else {
            r[0] = d - x[2] - x[5] - self.distance;
            set_row(j, 0, &[-ux, -uy, -1.0, ux, uy, -1.0]);
        }
    }
}
