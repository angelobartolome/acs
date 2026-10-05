use nalgebra::DMatrix;

use crate::constraints::arc_span::span_overshoot;
use crate::constraints::{Constraint, Var, set_row, xy};

/// One side of a curve–curve tangency: a Circle, or an Arc whose span the
/// tangency point must lie on.
pub struct TangentCurve {
    pub center_id: String,
    pub id: String,
    pub arc: bool,
}

impl TangentCurve {
    pub fn circle(center_id: String, id: String) -> Self {
        Self { center_id, id, arc: false }
    }

    pub fn arc(center_id: String, id: String) -> Self {
        Self { center_id, id, arc: true }
    }

    fn vars(&self) -> Vec<Var<'_>> {
        let mut v = vec![Var::X(&self.center_id), Var::Y(&self.center_id), Var::Radius(&self.id)];
        if self.arc {
            v.extend([Var::StartAngle(&self.id), Var::EndAngle(&self.id)]);
        }
        v
    }
}

/// Tangency between two circles or arcs (`a` and `b`), touching externally
/// (distance between centers = r₁ + r₂) or, when `internal`, inside
/// (= |r₁ − r₂|). The flag is the caller's choice and is never inferred, so
/// a solve can't turn one into the other.
///
/// Let w = c_b − c_a and d = |w|. The tangency point lies on the line of
/// centers: from c_a in direction +w, from c_b in direction −w (external);
/// inside, both in direction s·w with s = sign(r_a − r_b) (from the larger
/// circle's center through the smaller's).
///
/// Residuals:
///   R₀ = d − (r_a + r_b)                 (external)
///   R₀ = d − s·(r_a − r_b)               (internal)
///   and per Arc side k, in order a then b:
///   R  = r_k · overshoot(φ_k, α_k, β_k)  (the tangency point lies on the
///                                         arc's span; φ_k is its direction
///                                         from c_k, see `span_overshoot`)
///
/// ∂φ/∂w = (−w_y, w_x)/d² for either direction ±w.
pub struct TangentCurvesConstraint {
    pub a: TangentCurve,
    pub b: TangentCurve,
    pub internal: bool,
}

impl TangentCurvesConstraint {
    pub fn new(a: TangentCurve, b: TangentCurve, internal: bool) -> Self {
        Self { a, b, internal }
    }
}

impl Constraint for TangentCurvesConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [self.a.vars(), self.b.vars()].concat()
    }

    fn num_residuals(&self) -> usize {
        1 + self.a.arc as usize + self.b.arc as usize
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        // x = [cax, cay, ra, (αa, βa,) cbx, cby, rb, (αb, βb)]
        let ob = if self.a.arc { 5 } else { 3 };
        let (ra, rb) = (x[2], x[ob + 2]);
        let (wx, wy) = (x[ob] - x[0], x[ob + 1] - x[1]);
        let d = wx.hypot(wy);

        // s: which way the tangency point lies from each center along w.
        let (s, (dra, drb)) = if self.internal {
            let s = if ra >= rb { 1.0 } else { -1.0 };
            r[0] = d - s * (ra - rb);
            (s, (-s, s))
        } else {
            r[0] = d - (ra + rb);
            (1.0, (-1.0, -1.0))
        };
        j[(0, 2)] = dra;
        j[(0, ob + 2)] = drb;
        if d < 1e-12 {
            return; // Coincident centers: no line of centers.
        }
        let (ux, uy) = (wx / d, wy / d);
        j[(0, 0)] = -ux;
        j[(0, 1)] = -uy;
        j[(0, ob)] = ux;
        j[(0, ob + 1)] = uy;

        // ∂φ/∂(c_a) = (w_y, −w_x)/d², ∂φ/∂(c_b) = −∂φ/∂(c_a).
        let (gx, gy) = (wy / (d * d), -wx / (d * d));
        // Direction of the tangency point from each center: a along s·w,
        // b along −w (external) or s·w (internal).
        let sb = if self.internal { s } else { -1.0 };
        let mut row = 1;
        for (curve, offset, sign) in [(&self.a, 0, s), (&self.b, ob, sb)] {
            if !curve.arc {
                continue;
            }
            let rad = x[offset + 2];
            let phi = (sign * wy).atan2(sign * wx);
            let (o, [d_phi, d_start, d_end]) = span_overshoot(phi, x[offset + 3], x[offset + 4]);
            r[row] = rad * o;
            let k = rad * d_phi;
            j[(row, 0)] = k * gx;
            j[(row, 1)] = k * gy;
            j[(row, ob)] = -k * gx;
            j[(row, ob + 1)] = -k * gy;
            j[(row, offset + 2)] = o;
            j[(row, offset + 3)] = rad * d_start;
            j[(row, offset + 4)] = rad * d_end;
            row += 1;
        }
    }
}

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
