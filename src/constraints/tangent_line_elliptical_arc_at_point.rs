use nalgebra::DMatrix;

use crate::constraints::ellipse::{D, EllipseFrame, V2};
use crate::constraints::{ArcEnd, Constraint, Var, xy};

/// A Line tangent to an elliptical arc at one of the arc's endpoints, which
/// is also the Line's endpoint: the Line runs along the ellipse's tangent
/// there.
///
/// Used instead of the segment-and-span form (`TangentLineEllipticalArc`)
/// when a line endpoint *is* an elliptical arc endpoint, as `TangentAtPoint`
/// is for an Arc: through a point already on the curve that form only
/// changes quadratically, which makes diagnosis report spurious Redundant
/// constraints. The angle at the point changes linearly.
///
/// Entities:
///   - `point_id`  – the shared point p (a Line endpoint and an arc endpoint)
///   - `other_id`  – the Line's other endpoint o
///   - `center_id`, `focus_id` – the ellipse's center and focus
///   - `arc_id`    – the elliptical arc (minor radius b, and the angle θ of
///     the end p is: its start angle or its end angle)
///
/// Residual, with τ(θ) = −a·sin θ·u + b·cos θ·n the ellipse's tangent at θ:
///   R = (o − p)/|o − p| × τ/|τ|   (the sine of the angle between them)
pub struct TangentLineEllipticalArcAtPointConstraint {
    pub point_id: String,
    pub other_id: String,
    pub center_id: String,
    pub focus_id: String,
    pub arc_id: String,
    pub end: ArcEnd,
}

impl TangentLineEllipticalArcAtPointConstraint {
    pub fn new(
        point_id: String,
        other_id: String,
        center_id: String,
        focus_id: String,
        arc_id: String,
        end: ArcEnd,
    ) -> Self {
        Self {
            point_id,
            other_id,
            center_id,
            focus_id,
            arc_id,
            end,
        }
    }
}

impl Constraint for TangentLineEllipticalArcAtPointConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        let angle = match self.end {
            ArcEnd::Start => Var::StartAngle(&self.arc_id),
            ArcEnd::End => Var::EndAngle(&self.arc_id),
        };
        [
            &xy(&self.point_id)[..],
            &xy(&self.other_id),
            &xy(&self.center_id),
            &xy(&self.focus_id),
            &[Var::MinorRadius(&self.arc_id), angle],
        ]
        .concat()
    }

    fn num_residuals(&self) -> usize {
        1
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        // x = [px, py, ox, oy, cx, cy, fx, fy, b, θ]
        let e = EllipseFrame::<10>::new(x, 4, 6, 8);
        let line = V2::point(x, 2).sub(V2::point(x, 0)).unit();
        let tangent = e.tangent_at(D::var(x, 9)).unit();
        let res = line.cross(tangent);
        r[0] = res.v;
        res.write_row(j, 0);
    }
}
