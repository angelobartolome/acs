//! [`TangentLineEllipticalArcConstraint`]: a Line (the segment) tangent to an
//! elliptical arc, touching it on the segment and on the arc's span.

use nalgebra::DMatrix;

use crate::constraints::ellipse::{D, EllipseFrame, V2, span_overshoot_d};
use crate::constraints::tangent_line_ellipse::line_ellipse_tangency;
use crate::constraints::{Constraint, Var, xy};

/// Constrains a Line (the segment) to be tangent to an elliptical arc,
/// touching it on the segment and on the arc's span, as
/// [`TangentLineArcConstraint`](crate::constraints::tangent_line_arc::TangentLineArcConstraint)
/// does for an Arc.
///
/// Entities:
///   - `line_pa_id`, `line_pb_id` – the Line's endpoints
///   - `center_id`, `focus_id`    – the ellipse's center c and focus f
///   - `arc_id`                   – the elliptical arc (minor radius b,
///     start angle α, end angle β)
///
/// Residuals, R₀ and R₁ as `TangentLineEllipse` (tangency, and the tangency
/// point T on the segment), and
///   R₂ = a · overshoot(t_T, α, β)
/// with t_T the parametric angle of T (0 while T is within the span).
pub struct TangentLineEllipticalArcConstraint {
    /// The Line's endpoints.
    pub line_pa_id: String,
    /// The Line's endpoints.
    pub line_pb_id: String,
    /// The ellipse's center c and focus f.
    pub center_id: String,
    /// The ellipse's center c and focus f.
    pub focus_id: String,
    /// The elliptical arc (minor radius b,.
    pub arc_id: String,
}

impl TangentLineEllipticalArcConstraint {
    /// The constraint over these entities, by ID (see the fields).
    pub fn new(
        line_pa_id: String,
        line_pb_id: String,
        center_id: String,
        focus_id: String,
        arc_id: String,
    ) -> Self {
        Self {
            line_pa_id,
            line_pb_id,
            center_id,
            focus_id,
            arc_id,
        }
    }
}

impl Constraint for TangentLineEllipticalArcConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [
            &xy(&self.center_id)[..],
            &xy(&self.focus_id),
            &xy(&self.line_pa_id),
            &xy(&self.line_pb_id),
            &[
                Var::MinorRadius(&self.arc_id),
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
        // x = [cx, cy, fx, fy, ax, ay, bx, by, b, α, β]
        let e = EllipseFrame::<11>::new(x, 0, 2, 8);
        let t = line_ellipse_tangency(&e, V2::point(x, 4), V2::point(x, 6));
        let span = match t.point {
            Some(p) => e.major_radius * span_overshoot_d(e.param_angle(p), D::var(x, 9), D::var(x, 10)),
            None => D::cst(0.0),
        };
        for (row, res) in [t.tangency, t.overshoot, span].into_iter().enumerate() {
            r[row] = res.v;
            res.write_row(j, row);
        }
    }
}
