use nalgebra::DMatrix;

use crate::constraints::ellipse::{D, EllipseFrame, V2};
use crate::constraints::{ArcEnd, Constraint, Var, xy};

/// An Arc tangent to an elliptical arc at an endpoint they share: the Arc's
/// radius there is normal to the ellipse (perpendicular to its tangent).
/// The only Arc–elliptical-arc tangency (the catalog rejects a pair with no
/// shared endpoint).
///
/// Entities:
///   - `point_id`   – the shared point p (an endpoint of both)
///   - `arc_center_id` – the Arc's center k
///   - `center_id`, `focus_id` – the ellipse's center and focus
///   - `arc_id`     – the elliptical arc (minor radius b, and the angle θ of
///     the end p is: its start angle or its end angle)
///
/// Residual, with τ(θ) = −a·sin θ·u + b·cos θ·n the ellipse's tangent at θ:
///   R = (p − k)/|p − k| · τ/|τ|   (the cosine of the angle between the
///                                  Arc's radius and the tangent)
pub struct TangentArcEllipticalArcAtPointConstraint {
    pub point_id: String,
    pub arc_center_id: String,
    pub center_id: String,
    pub focus_id: String,
    pub arc_id: String,
    pub end: ArcEnd,
}

impl TangentArcEllipticalArcAtPointConstraint {
    pub fn new(
        point_id: String,
        arc_center_id: String,
        center_id: String,
        focus_id: String,
        arc_id: String,
        end: ArcEnd,
    ) -> Self {
        Self {
            point_id,
            arc_center_id,
            center_id,
            focus_id,
            arc_id,
            end,
        }
    }
}

impl Constraint for TangentArcEllipticalArcAtPointConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        let angle = match self.end {
            ArcEnd::Start => Var::StartAngle(&self.arc_id),
            ArcEnd::End => Var::EndAngle(&self.arc_id),
        };
        [
            &xy(&self.point_id)[..],
            &xy(&self.arc_center_id),
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
        // x = [px, py, kx, ky, cx, cy, fx, fy, b, θ]
        let e = EllipseFrame::<10>::new(x, 4, 6, 8);
        let radius = V2::point(x, 0).sub(V2::point(x, 2)).unit();
        let tangent = e.tangent_at(D::var(x, 9)).unit();
        let res = radius.dot(tangent);
        r[0] = res.v;
        res.write_row(j, 0);
    }
}
