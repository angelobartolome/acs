//! [`EllipticalArcRulesConstraint`]: ties an elliptical arc's start and end
//! Points to its ellipse and angles, as
//! [`ArcRulesConstraint`](crate::constraints::arc_rules::ArcRulesConstraint)
//! does for an Arc.

use nalgebra::DMatrix;

use crate::constraints::ellipse::{D, EllipseFrame, V2};
use crate::constraints::{Constraint, Var, xy};

/// Ties an elliptical arc's start and end Points to its ellipse and angles,
/// as [`ArcRulesConstraint`](crate::constraints::arc_rules::ArcRulesConstraint)
/// does for an Arc.
///
/// Every EllipticalArc gets these rules implicitly (`SketchSystem` adds one
/// per elliptical arc); they are never a sketch constraint of their own.
///
/// Entities:
///   - `center_id` – the ellipse's center c
///   - `focus_id`  – its focus f (the major axis points from c to f)
///   - `start_id`  – the start point s
///   - `end_id`    – the end point e
///   - `arc_id`    – the elliptical arc (minor radius b, start angle α, end
///     angle β; parametric)
///
/// With P(t) = c + a·cos t·u + b·sin t·n (the point at parametric angle t,
/// a = sqrt(b² + |f − c|²), u = (f − c)/|f − c|, n = rot90(u)):
///   R₀,₁ = s − P(α)     R₂,₃ = e − P(β)
pub struct EllipticalArcRulesConstraint {
    /// The ellipse's center c.
    pub center_id: String,
    /// Its focus f (the major axis points from c to f).
    pub focus_id: String,
    /// The start point s.
    pub start_id: String,
    /// The end point e.
    pub end_id: String,
    /// The elliptical arc (minor radius b, start angle α, end.
    pub arc_id: String,
}

impl EllipticalArcRulesConstraint {
    /// The constraint over these entities, by ID (see the fields).
    pub fn new(
        center_id: String,
        focus_id: String,
        start_id: String,
        end_id: String,
        arc_id: String,
    ) -> Self {
        Self {
            center_id,
            focus_id,
            start_id,
            end_id,
            arc_id,
        }
    }
}

impl Constraint for EllipticalArcRulesConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [
            &xy(&self.center_id)[..],
            &xy(&self.focus_id),
            &xy(&self.start_id),
            &xy(&self.end_id),
            &[
                Var::MinorRadius(&self.arc_id),
                Var::StartAngle(&self.arc_id),
                Var::EndAngle(&self.arc_id),
            ],
        ]
        .concat()
    }

    fn num_residuals(&self) -> usize {
        4
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        // x = [cx, cy, fx, fy, sx, sy, ex, ey, b, α, β]
        let e = EllipseFrame::<11>::new(x, 0, 2, 8);
        let start = V2::point(x, 4).sub(e.point_at(D::var(x, 9)));
        let end = V2::point(x, 6).sub(e.point_at(D::var(x, 10)));
        for (row, res) in [start.x, start.y, end.x, end.y].into_iter().enumerate() {
            r[row] = res.v;
            res.write_row(j, row);
        }
    }
}
