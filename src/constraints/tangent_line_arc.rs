use nalgebra::DMatrix;

use crate::constraints::arc_span::span_overshoot;
use crate::constraints::segment::{foot_overshoot, line_signed_distance};
use crate::constraints::{Constraint, Var, set_row, xy};

/// Constrains a Line (the segment between its endpoints) to be tangent to an
/// arc, touching it on both the segment and the arc's span. The arc analogue
/// of `TangentLineCircleConstraint`.
///
/// Entities:
///   - `line_pa_id` – first point of the line (a)
///   - `line_pb_id` – second point of the line (b)
///   - `center_id`  – the arc's center point (c)
///   - `arc_id`     – the arc (radius r, start angle α, end angle β)
///
/// Residuals, with d the signed distance from c to the line through a, b:
///   R₀ = |d| − r                 (the line touches the arc's circle)
///   R₁ = overshoot               (the tangency point, the foot of the
///                                 perpendicular from c, lies on the segment)
///   R₂ = r · overshoot(φ, α, β)  (and on the arc's span; φ is the direction
///                                 from c to the foot, the line's direction
///                                 turned ∓90° by the side c is on)
pub struct TangentLineArcConstraint {
    pub line_pa_id: String,
    pub line_pb_id: String,
    pub center_id: String,
    pub arc_id: String,
}

impl TangentLineArcConstraint {
    pub fn new(line_pa_id: String, line_pb_id: String, center_id: String, arc_id: String) -> Self {
        Self {
            line_pa_id,
            line_pb_id,
            center_id,
            arc_id,
        }
    }
}

impl Constraint for TangentLineArcConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [
            &xy(&self.center_id)[..],
            &xy(&self.line_pa_id),
            &xy(&self.line_pb_id),
            &[
                Var::Radius(&self.arc_id),
                Var::StartAngle(&self.arc_id),
                Var::EndAngle(&self.arc_id),
            ],
        ]
        .concat()
    }

    fn num_residuals(&self) -> usize {
        3
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        // x = [cx, cy, ax, ay, bx, by, r, α, β]
        let rad = x[6];
        let (d, gd) = line_signed_distance(x[0], x[1], x[2], x[3], x[4], x[5]);
        let sign = if d >= 0.0 { 1.0 } else { -1.0 };
        r[0] = d.abs() - rad;
        set_row(j, 0, &gd.map(|v| sign * v));
        j[(0, 6)] = -1.0;

        let (over, go) = foot_overshoot(x[0], x[1], x[2], x[3], x[4], x[5]);
        r[1] = over;
        set_row(j, 1, &go);

        // Direction from c to the foot: sign·(dy, −dx), so
        // φ = atan2(−sign·dx, sign·dy), and ∂φ/∂(dx, dy) = (−dy, dx) / L².
        let (dx, dy) = (x[4] - x[2], x[5] - x[3]);
        let l2 = dx * dx + dy * dy;
        if l2 < 1e-24 {
            return; // Degenerate line: no direction.
        }
        let phi = (-sign * dx).atan2(sign * dy);
        let (o, [d_phi, d_start, d_end]) = span_overshoot(phi, x[7], x[8]);
        r[2] = rad * o;
        let k = rad * d_phi / l2;
        // ∂φ/∂(ax, ay, bx, by) = (dy, −dx, −dy, dx) / L²; c doesn't turn φ.
        set_row(
            j,
            2,
            &[0.0, 0.0, k * dy, -k * dx, -k * dy, k * dx, o, rad * d_start, rad * d_end],
        );
    }
}
