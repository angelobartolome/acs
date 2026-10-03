use nalgebra::DMatrix;

use crate::constraints::segment::line_signed_distance;
use crate::constraints::{Constraint, Var, set_row, xy};

/// Constrains a Line's Extension (the infinite line through its endpoints) to
/// be tangent to a circle. The Extension variant of
/// `TangentLineCircleConstraint`: the tangency point may lie anywhere on the
/// Extension, beyond either end of the segment.
///
/// Entities:
///   - `line_pa_id`       – first point of the line
///   - `line_pb_id`       – second point of the line
///   - `circle_center_id` – center point of the circle
///   - `circle_id`        – circle entity (radius at param 0)
///
/// Residual: R = |C / L| − r (the center is r away from the line through a, b).
pub struct TangentExtensionCircleConstraint {
    pub line_pa_id: String,
    pub line_pb_id: String,
    pub circle_center_id: String,
    pub circle_id: String,
}

impl TangentExtensionCircleConstraint {
    pub fn new(
        line_pa_id: String,
        line_pb_id: String,
        circle_center_id: String,
        circle_id: String,
    ) -> Self {
        Self {
            line_pa_id,
            line_pb_id,
            circle_center_id,
            circle_id,
        }
    }
}

impl Constraint for TangentExtensionCircleConstraint {
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
        // x = [cx, cy, ax, ay, bx, by, radius]
        let (d, gd) = line_signed_distance(x[0], x[1], x[2], x[3], x[4], x[5]);
        let sign = if d >= 0.0 { 1.0 } else { -1.0 };
        r[0] = d.abs() - x[6];
        set_row(j, 0, &gd.map(|v| sign * v));
        j[(0, 6)] = -1.0;
    }
}
