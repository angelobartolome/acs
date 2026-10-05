use nalgebra::DMatrix;

use crate::constraints::{Constraint, Var, set_row, xy};

/// Ties an arc's start and end Points to its center, radius and angles, so
/// other geometry (lines sharing an endpoint) follows the arc and vice versa.
///
/// Every Arc gets these rules implicitly (`SketchSystem` adds one per arc);
/// they are never a sketch constraint of their own.
///
/// Entities:
///   - `center_id` – the arc's center point (cx, cy)
///   - `start_id`  – the arc's start point (sx, sy)
///   - `end_id`    – the arc's end point (ex, ey)
///   - `arc_id`    – the arc (radius r, start angle α, end angle β)
///
/// Residuals:
///   R₀ = sx − cx − r·cos α     R₁ = sy − cy − r·sin α
///   R₂ = ex − cx − r·cos β     R₃ = ey − cy − r·sin β
pub struct ArcRulesConstraint {
    pub center_id: String,
    pub start_id: String,
    pub end_id: String,
    pub arc_id: String,
}

impl ArcRulesConstraint {
    pub fn new(center_id: String, start_id: String, end_id: String, arc_id: String) -> Self {
        Self {
            center_id,
            start_id,
            end_id,
            arc_id,
        }
    }
}

impl Constraint for ArcRulesConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [
            &xy(&self.center_id)[..],
            &xy(&self.start_id),
            &xy(&self.end_id),
            &[
                Var::Radius(&self.arc_id),
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
        // x = [cx, cy, sx, sy, ex, ey, r, α, β]
        let (cx, cy, rad) = (x[0], x[1], x[6]);
        let (ca, sa) = (x[7].cos(), x[7].sin());
        let (cb, sb) = (x[8].cos(), x[8].sin());

        r[0] = x[2] - cx - rad * ca;
        set_row(j, 0, &[-1.0, 0.0, 1.0, 0.0, 0.0, 0.0, -ca, rad * sa, 0.0]);
        r[1] = x[3] - cy - rad * sa;
        set_row(j, 1, &[0.0, -1.0, 0.0, 1.0, 0.0, 0.0, -sa, -rad * ca, 0.0]);
        r[2] = x[4] - cx - rad * cb;
        set_row(j, 2, &[-1.0, 0.0, 0.0, 0.0, 1.0, 0.0, -cb, 0.0, rad * sb]);
        r[3] = x[5] - cy - rad * sb;
        set_row(j, 3, &[0.0, -1.0, 0.0, 0.0, 0.0, 1.0, -sb, 0.0, -rad * cb]);
    }
}
