use nalgebra::DMatrix;

use crate::constraints::{Constraint, Var, set_row, xy};

/// Two arcs (`TangentArcs`) tangent at a Point they share, one endpoint of
/// each: their radii there are collinear, the centers on opposite sides of
/// the point (external) or, when `internal`, on the same side.
///
/// Used instead of the distance form (`TangentCurvesConstraint`) when the
/// arcs share an endpoint. Through a point already on both circles (the
/// arcs' implicit rules), the distance between centers can only fall short
/// of r₁ + r₂ (or exceed |r₁ − r₂|), so it changes quadratically as one arc
/// turns about the point and its Jacobian row vanishes at tangency (spurious
/// Redundant, `dof` too high). The angle between the radii changes linearly.
/// The shared point lies on both spans (it's an endpoint of each).
///
/// Entities:
///   - `point_id`   – the shared point p
///   - `center1_id` – the first arc's center c₁
///   - `center2_id` – the second arc's center c₂
///
/// Let v₁ = p − c₁, v₂ = p − c₂ and θ = atan2(v₁ × v₂, v₁ · v₂), the angle
/// from v₁ to v₂.
///
/// Residual: θ (internal) or θ − π wrapped to (−π, π] (external):
/// atan2(−v₁ × v₂, −v₁ · v₂). Either way ∂θ/∂v₁ = (v₁y, −v₁x)/|v₁|² and
/// ∂θ/∂v₂ = (−v₂y, v₂x)/|v₂|². Its only zero is the chosen side, so a
/// solve can't flip the join. For a degenerate radius (|v₁| or |v₂| ≈ 0)
/// the cross product v₁ × v₂ is used.
pub struct TangentArcsAtPointConstraint {
    pub point_id: String,
    pub center1_id: String,
    pub center2_id: String,
    pub internal: bool,
}

impl TangentArcsAtPointConstraint {
    pub fn new(point_id: String, center1_id: String, center2_id: String, internal: bool) -> Self {
        Self {
            point_id,
            center1_id,
            center2_id,
            internal,
        }
    }
}

impl Constraint for TangentArcsAtPointConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [xy(&self.point_id), xy(&self.center1_id), xy(&self.center2_id)].concat()
    }

    fn num_residuals(&self) -> usize {
        1
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        // x = [px, py, c1x, c1y, c2x, c2y]
        let (v1x, v1y) = (x[0] - x[2], x[1] - x[3]);
        let (v2x, v2y) = (x[0] - x[4], x[1] - x[5]);
        let cross = v1x * v2y - v1y * v2x;
        let dot = v1x * v2x + v1y * v2y;
        let (n1, n2) = (v1x * v1x + v1y * v1y, v2x * v2x + v2y * v2y);

        // Partials w.r.t. v₁ and v₂; p moves both (+), c₁ only v₁ (−), c₂
        // only v₂ (−).
        let (g1, g2) = if n1 < 1e-24 || n2 < 1e-24 {
            r[0] = cross;
            ([v2y, -v2x], [-v1y, v1x])
        } else {
            let sign = if self.internal { 1.0 } else { -1.0 };
            r[0] = (sign * cross).atan2(sign * dot);
            ([v1y / n1, -v1x / n1], [-v2y / n2, v2x / n2])
        };
        set_row(
            j,
            0,
            &[g1[0] + g2[0], g1[1] + g2[1], -g1[0], -g1[1], -g2[0], -g2[1]],
        );
    }
}
