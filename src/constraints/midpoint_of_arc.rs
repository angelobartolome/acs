//! [`MidpointOfArcConstraint`]: constrains a point to be the midpoint of an
//! arc's span.

use nalgebra::DMatrix;

use crate::constraints::arc_length::arc_sweep;
use crate::constraints::{Constraint, Var, set_row, xy};

/// Constrains a point to be the midpoint of an arc's span: on its circle,
/// halfway (by angle) from its start angle to its end angle,
/// counter-clockwise.
///
/// Entities:
///   - `point_id`  – the point (px, py)
///   - `center_id` – the arc's center point (cx, cy)
///   - `arc_id`    – the arc (radius r, start angle α, end angle β)
///
/// With sweep = (β − α) mod 2π in (0, 2π] (0 = a full turn, as for
/// `ArcLength`) and m = α + sweep / 2:
///   R₀ = px − (cx + r · cos m)
///   R₁ = py − (cy + r · sin m)
///
/// m = (α + β) / 2 + kπ for an integer k fixed between wraps, so
/// ∂m/∂α = ∂m/∂β = ½. The wrap: when the sweep crosses 0/2π (the end angle
/// passes the start), m jumps by π and the midpoint flips to the opposite
/// side of the circle, as the arc itself jumps between a full turn and
/// none. Everywhere else the residual is smooth; unlike `(α + β) / 2`, m is
/// on the span even when it crosses ±π.
pub struct MidpointOfArcConstraint {
    /// The point (px, py).
    pub point_id: String,
    /// The arc's center point (cx, cy).
    pub center_id: String,
    /// The arc (radius r, start angle α, end angle β).
    pub arc_id: String,
}

impl MidpointOfArcConstraint {
    /// The constraint over these entities, by ID (see the fields).
    pub fn new(point_id: String, center_id: String, arc_id: String) -> Self {
        Self {
            point_id,
            center_id,
            arc_id,
        }
    }
}

impl Constraint for MidpointOfArcConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [
            &xy(&self.point_id)[..],
            &xy(&self.center_id),
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
        // x = [px, py, cx, cy, r, α, β]
        let rad = x[4];
        let m = x[5] + arc_sweep(x[5], x[6]) / 2.0;
        let (sin, cos) = m.sin_cos();
        r[0] = x[0] - (x[2] + rad * cos);
        r[1] = x[1] - (x[3] + rad * sin);
        // ∂(r cos m)/∂m = −r sin m and ∂(r sin m)/∂m = r cos m, each times
        // ∂m/∂α = ∂m/∂β = ½.
        let (hs, hc) = (0.5 * rad * sin, 0.5 * rad * cos);
        set_row(j, 0, &[1.0, 0.0, -1.0, 0.0, -cos, hs, hs]);
        set_row(j, 1, &[0.0, 1.0, 0.0, -1.0, -sin, -hc, -hc]);
    }
}
