use nalgebra::DMatrix;

use crate::constraints::segment::line_signed_distance;
use crate::constraints::{Constraint, Var, set_row, xy};

/// Constrains the signed perpendicular distance from a point to a Line's
/// Extension (the infinite line through its endpoints), so the point stays on
/// one side of it. Used by Linked Offsets, whose endpoints routinely stick out
/// past their source line, so it always measures against the Extension.
///
/// Entities:
///   - `point_id`   – the point p
///   - `line_pa_id` – first point of the line (a)
///   - `line_pb_id` – second point of the line (b)
///   - `distance`   – target distance d
///   - `side`       – +1 for the left of a→b, −1 for the right (any negative
///     number counts as −1, anything else as +1)
///
/// Residual: side · C / L − d, with C = cross(b − a, p − a) and L = |b − a|.
pub struct SignedDistancePointExtensionConstraint {
    pub point_id: String,
    pub line_pa_id: String,
    pub line_pb_id: String,
    pub distance: f64,
    pub side: f64,
}

impl SignedDistancePointExtensionConstraint {
    pub fn new(
        point_id: String,
        line_pa_id: String,
        line_pb_id: String,
        distance: f64,
        side: f64,
    ) -> Self {
        Self {
            point_id,
            line_pa_id,
            line_pb_id,
            distance,
            side: if side < 0.0 { -1.0 } else { 1.0 },
        }
    }
}

impl Constraint for SignedDistancePointExtensionConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [
            xy(&self.point_id),
            xy(&self.line_pa_id),
            xy(&self.line_pb_id),
        ]
        .concat()
    }

    fn num_residuals(&self) -> usize {
        1
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        let (d, g) = line_signed_distance(x[0], x[1], x[2], x[3], x[4], x[5]);
        r[0] = self.side * d - self.distance;
        set_row(j, 0, &g.map(|v| self.side * v));
    }
}
