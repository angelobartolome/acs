//! [`DistanceLineCircleConstraint`]: fixes the gap between a Line (the
//! segment) and a circle.

use nalgebra::DMatrix;

use crate::constraints::segment::segment_distance;
use crate::constraints::{Constraint, Var, set_row, xy};

/// Fixes the gap between a Line (the segment) and a circle: the distance
/// from the circle's center to the closest point of the segment, minus the
/// radius (negative while the segment cuts into the circle).
///
/// Entities:
///   - `line_pa_id`       – first point of the line (ax, ay)
///   - `line_pb_id`       – second point of the line (bx, by)
///   - `circle_center_id` – center point of the circle (cx, cy)
///   - `circle_id`        – the circle (radius r)
///
/// Residual: R = dist(c, segment ab) − r − distance
pub struct DistanceLineCircleConstraint {
    /// First point of the line (ax, ay).
    pub line_pa_id: String,
    /// Second point of the line (bx, by).
    pub line_pb_id: String,
    /// Center point of the circle (cx, cy).
    pub circle_center_id: String,
    /// The circle (radius r).
    pub circle_id: String,
    /// The gap between the segment and the circle.
    pub distance: f64,
}

impl DistanceLineCircleConstraint {
    /// The constraint over these entities, by ID (see the fields).
    pub fn new(
        line_pa_id: String,
        line_pb_id: String,
        circle_center_id: String,
        circle_id: String,
        distance: f64,
    ) -> Self {
        Self {
            line_pa_id,
            line_pb_id,
            circle_center_id,
            circle_id,
            distance,
        }
    }
}

impl Constraint for DistanceLineCircleConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [
            &xy(&self.circle_center_id)[..],
            &xy(&self.line_pa_id),
            &xy(&self.line_pb_id),
            &[Var::Radius(&self.circle_id)],
        ]
        .concat()
    }

    fn num_residuals(&self) -> usize {
        1
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        // x = [cx, cy, ax, ay, bx, by, r]
        let (d, gd) = segment_distance(x[0], x[1], x[2], x[3], x[4], x[5]);
        r[0] = d - x[6] - self.distance;
        set_row(j, 0, &gd);
        j[(0, 6)] = -1.0;
    }
}
