//! [`TangentLineEllipseAtPointConstraint`]: a Line tangent to an ellipse (or
//! an elliptical arc's ellipse) at its endpoint p, a point held on the curve
//! by another constraint.

use nalgebra::DMatrix;

use crate::constraints::ellipse::{EllipseFrame, V2};
use crate::constraints::{Constraint, Var, xy};

/// A Line tangent to an ellipse (or an elliptical arc's ellipse) at its
/// endpoint p, a point held on the curve by another constraint: the Line
/// runs along the ellipse's tangent at p's parametric angle.
///
/// Built by `constraint_catalog::tangent_at_held_endpoints` in place of
/// `TangentLineEllipse` / `TangentLineEllipticalArc` when a real `on` holds
/// a Line endpoint on that same curve, as `TangentAtPoint` is for a circle
/// or an arc: through a point already on the curve, the line–ellipse
/// tangency only changes quadratically as p slides along the Line, so p
/// drifts from the tangent point and diagnosis reports spurious Redundant
/// constraints. The angle at p changes linearly. The `on` keeps p on the
/// curve (and an elliptical arc's span), so both forms have the same
/// solutions.
///
/// Entities:
///   - `point_id`   – the held Line endpoint p
///   - `other_id`   – the Line's other endpoint o
///   - `center_id`, `focus_id` – the ellipse's center c and focus f
///   - `ellipse_id` – the ellipse or elliptical arc (minor radius b; an
///     elliptical arc's `MinorRadius` Var is an Ellipse's)
///
/// Residual, with t(p) the parametric angle of p's direction from c
/// (`EllipseFrame::param_angle`) and τ(t) = −a·sin t·u + b·cos t·n the
/// ellipse's tangent there (`EllipseFrame::tangent_at`):
///   R = (o − p)/|o − p| × τ(t(p))/|τ(t(p))|   (the sine of the angle
///                                              between them)
pub struct TangentLineEllipseAtPointConstraint {
    /// The held Line endpoint p.
    pub point_id: String,
    /// The Line's other endpoint o.
    pub other_id: String,
    /// The ellipse's center c and focus f.
    pub center_id: String,
    /// The ellipse's center c and focus f.
    pub focus_id: String,
    /// The ellipse or elliptical arc (minor radius b; an.
    pub ellipse_id: String,
}

impl TangentLineEllipseAtPointConstraint {
    /// The constraint over these entities, by ID (see the fields).
    pub fn new(
        point_id: String,
        other_id: String,
        center_id: String,
        focus_id: String,
        ellipse_id: String,
    ) -> Self {
        Self {
            point_id,
            other_id,
            center_id,
            focus_id,
            ellipse_id,
        }
    }
}

impl Constraint for TangentLineEllipseAtPointConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [
            &xy(&self.point_id)[..],
            &xy(&self.other_id),
            &xy(&self.center_id),
            &xy(&self.focus_id),
            &[Var::MinorRadius(&self.ellipse_id)],
        ]
        .concat()
    }

    fn num_residuals(&self) -> usize {
        1
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        // x = [px, py, ox, oy, cx, cy, fx, fy, b]
        let e = EllipseFrame::<9>::new(x, 4, 6, 8);
        let p = V2::point(x, 0);
        let line = V2::point(x, 2).sub(p).unit();
        let tangent = e.tangent_at(e.param_angle(p)).unit();
        let res = line.cross(tangent);
        r[0] = res.v;
        res.write_row(j, 0);
    }
}
