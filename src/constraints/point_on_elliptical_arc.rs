use nalgebra::DMatrix;

use crate::constraints::ellipse::{D, EllipseFrame, V2, span_overshoot_d};
use crate::constraints::{Constraint, Var, xy};

/// Constrains a point to lie on an elliptical arc's span: on its ellipse,
/// between its start and end angles (counter-clockwise, parametric), as
/// [`PointOnArcConstraint`](crate::constraints::point_on_arc::PointOnArcConstraint)
/// does for an Arc.
///
/// Entities:
///   - `point_id`  – the point p
///   - `center_id` – the ellipse's center c
///   - `focus_id`  – its focus f
///   - `arc_id`    – the elliptical arc (minor radius b, start angle α, end
///     angle β)
///
/// Residuals, with a = sqrt(b² + |f − c|²) and t the parametric angle of
/// p's direction from c (`EllipseFrame::param_angle`):
///   R₀ = |p − f| + |p − (2c − f)| − 2a   (on the ellipse, as `PointOnEllipse`)
///   R₁ = a · overshoot(t, α, β)          (on the span: 0 while t is within
///                                         it, else the angle past the
///                                         nearer end, scaled to a length)
pub struct PointOnEllipticalArcConstraint {
    pub point_id: String,
    pub center_id: String,
    pub focus_id: String,
    pub arc_id: String,
}

impl PointOnEllipticalArcConstraint {
    pub fn new(point_id: String, center_id: String, focus_id: String, arc_id: String) -> Self {
        Self {
            point_id,
            center_id,
            focus_id,
            arc_id,
        }
    }
}

impl Constraint for PointOnEllipticalArcConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [
            &xy(&self.point_id)[..],
            &xy(&self.center_id),
            &xy(&self.focus_id),
            &[
                Var::MinorRadius(&self.arc_id),
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
        // x = [px, py, cx, cy, fx, fy, b, α, β]
        let e = EllipseFrame::<9>::new(x, 2, 4, 6);
        let p = V2::point(x, 0);
        let on = p.sub(e.focus).norm() + p.sub(e.focus2()).norm() - e.major_radius.scale(2.0);
        r[0] = on.v;
        on.write_row(j, 0);

        let t = e.param_angle(p);
        let over = e.major_radius * span_overshoot_d(t, D::var(x, 7), D::var(x, 8));
        r[1] = over.v;
        over.write_row(j, 1);
    }
}
