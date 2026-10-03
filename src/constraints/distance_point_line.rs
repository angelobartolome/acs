use nalgebra::DMatrix;

use crate::constraints::segment::{segment_distance, segment_signed_distance};
use crate::constraints::{Constraint, Var, set_row, xy};

/// Constrains the distance from a point to a Line (the segment between its
/// endpoints) to equal `distance`.
///
/// Entities:
///   - `point_id`   – the free point p
///   - `line_pa_id` – first point of the line (a)
///   - `line_pb_id` – second point of the line (b)
///   - `distance`   – target distance d from p to the closest point of ab
///
/// Residual: D − d, where D is the distance from p to segment ab. For d = 0
/// the signed distance is used instead (identical to PointOnLine), since |·|
/// has no usable gradient at 0.
pub struct DistancePointLineConstraint {
    pub point_id: String,
    pub line_pa_id: String,
    pub line_pb_id: String,
    pub distance: f64,
}

impl DistancePointLineConstraint {
    pub fn new(
        point_id: String,
        line_pa_id: String,
        line_pb_id: String,
        distance: f64,
    ) -> Self {
        Self {
            point_id,
            line_pa_id,
            line_pb_id,
            distance,
        }
    }
}

impl Constraint for DistancePointLineConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [xy(&self.point_id), xy(&self.line_pa_id), xy(&self.line_pb_id)].concat()
    }

    fn num_residuals(&self) -> usize {
        1
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        let (d, g) = if self.distance == 0.0 {
            segment_signed_distance(x[0], x[1], x[2], x[3], x[4], x[5])
        } else {
            let (dist, g) = segment_distance(x[0], x[1], x[2], x[3], x[4], x[5]);
            (dist - self.distance, g)
        };
        r[0] = d;
        set_row(j, 0, &g);
    }
}
