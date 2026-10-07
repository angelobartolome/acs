use nalgebra::DMatrix;

use crate::constraints::arc_span::span_overshoot;
use crate::constraints::segment::{line_signed_distance, segment_distance};
use crate::constraints::{Constraint, Var, set_row, xy};

/// Fixes the gap between a Line and an arc: the line–circle gap of
/// `DistanceLineCircleConstraint` (to the segment) or, with `extension`,
/// of `DistanceExtensionCircleConstraint` (to the Extension), with the
/// nearest point of the arc on its span. The nearest point of the arc's
/// circle lies on the ray from the center through the nearest point `q` of
/// the segment (the foot of the perpendicular, or the nearer endpoint past
/// an end; for the Extension always the foot), so the span row keeps that
/// ray on the span, as `TangentLineArcConstraint` does for the tangency
/// point. A Line whose nearest point lies off the span isn't measured to an
/// arc endpoint (that residual would be kinked); the solve turns the ray
/// onto the span instead.
///
/// Entities:
///   - `line_pa_id` – first point of the line (a)
///   - `line_pb_id` – second point of the line (b)
///   - `center_id`  – the arc's center point (c)
///   - `arc_id`     – the arc (radius r, start angle α, end angle β)
///
/// Residuals, with φ the direction of q − c:
///   R₀ = dist(c, segment ab) − r − distance   (segment)
///   R₀ = |C / L| − r − distance               (`extension`)
///   R₁ = r · overshoot(φ, α, β)               (the nearest point lies on
///                                             the span)
pub struct DistanceLineArcConstraint {
    pub line_pa_id: String,
    pub line_pb_id: String,
    pub center_id: String,
    pub arc_id: String,
    pub distance: f64,
    pub extension: bool,
}

impl DistanceLineArcConstraint {
    pub fn new(
        line_pa_id: String,
        line_pb_id: String,
        center_id: String,
        arc_id: String,
        distance: f64,
        extension: bool,
    ) -> Self {
        Self {
            line_pa_id,
            line_pb_id,
            center_id,
            arc_id,
            distance,
            extension,
        }
    }
}

const DEGENERATE_L2: f64 = 1e-24;

impl Constraint for DistanceLineArcConstraint {
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
        2
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        // x = [cx, cy, ax, ay, bx, by, r, α, β]
        let rad = x[6];
        let (cx, cy, ax, ay, bx, by) = (x[0], x[1], x[2], x[3], x[4], x[5]);
        let (sd, gs) = line_signed_distance(cx, cy, ax, ay, bx, by);
        let sign = if sd >= 0.0 { 1.0 } else { -1.0 };
        let (d, gd) = if self.extension {
            (sd.abs(), gs.map(|v| sign * v))
        } else {
            segment_distance(cx, cy, ax, ay, bx, by)
        };
        r[0] = d - rad - self.distance;
        set_row(j, 0, &gd);
        j[(0, 6)] = -1.0;

        // φ and its partials w.r.t. (cx, cy, ax, ay, bx, by).
        let (dx, dy) = (bx - ax, by - ay);
        let l2 = dx * dx + dy * dy;
        let t = if l2 < DEGENERATE_L2 {
            0.0
        } else {
            ((cx - ax) * dx + (cy - ay) * dy) / l2
        };
        let (phi, dphi) = if l2 >= DEGENERATE_L2 && (self.extension || (0.0..=1.0).contains(&t)) {
            // q is the foot: q − c points sign·(dy, −dx), whatever c is, so
            // φ = atan2(−sign·dx, sign·dy) and ∂φ/∂(dx, dy) = (−dy, dx) / L².
            if sd.abs() < 1e-300 {
                return; // The center lies on the line: no direction.
            }
            let phi = (-sign * dx).atan2(sign * dy);
            (phi, [0.0, 0.0, dy / l2, -dx / l2, -dy / l2, dx / l2])
        } else {
            if self.extension {
                return; // Degenerate line: no direction.
            }
            // Past an end (or a degenerate segment), q is that endpoint e.
            let e_is_a = t < 0.0 || l2 < DEGENERATE_L2;
            let (ex, ey) = if e_is_a { (ax, ay) } else { (bx, by) };
            let (vx, vy) = (ex - cx, ey - cy);
            let v2 = vx * vx + vy * vy;
            if v2 < 1e-300 {
                return; // The endpoint sits on the center.
            }
            let (gx, gy) = (-vy / v2, vx / v2); // ∂φ/∂e; ∂φ/∂c = −∂φ/∂e
            let g = if e_is_a {
                [-gx, -gy, gx, gy, 0.0, 0.0]
            } else {
                [-gx, -gy, 0.0, 0.0, gx, gy]
            };
            (vy.atan2(vx), g)
        };
        let (over, [d_phi, d_start, d_end]) = span_overshoot(phi, x[7], x[8]);
        r[1] = rad * over;
        let row: Vec<f64> = dphi
            .iter()
            .map(|g| rad * d_phi * g)
            .chain([over, rad * d_start, rad * d_end])
            .collect();
        set_row(j, 1, &row);
    }
}
