use nalgebra::DMatrix;

use crate::constraints::segment::line_signed_distance;
use crate::constraints::{Constraint, Var, set_row, xy};

/// Constrains the perpendicular distance from a point to a Line's Extension
/// (the infinite line through its endpoints) to equal `distance`. The
/// Extension variant of `DistancePointLineConstraint`.
///
/// Entities:
///   - `point_id`   – the free point p
///   - `line_pa_id` – first point of the line (a)
///   - `line_pb_id` – second point of the line (b)
///   - `distance`   – target distance d from p to the line through a, b
///
/// Residual: |C / L| − d, the unsigned perpendicular distance minus d. For
/// d = 0 the signed distance C / L is used instead (identical to
/// PointOnExtension), since |·| has no usable gradient at 0.
pub struct DistancePointExtensionConstraint {
    pub point_id: String,
    pub line_pa_id: String,
    pub line_pb_id: String,
    pub distance: f64,
}

impl DistancePointExtensionConstraint {
    pub fn new(point_id: String, line_pa_id: String, line_pb_id: String, distance: f64) -> Self {
        Self {
            point_id,
            line_pa_id,
            line_pb_id,
            distance,
        }
    }
}

impl Constraint for DistancePointExtensionConstraint {
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
        if self.distance == 0.0 {
            r[0] = d;
            set_row(j, 0, &g);
        } else {
            let sign = if d >= 0.0 { 1.0 } else { -1.0 };
            r[0] = d.abs() - self.distance;
            set_row(j, 0, &g.map(|v| sign * v));
        }
    }
}
