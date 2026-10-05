use nalgebra::DMatrix;

use crate::constraints::arc_span::span_overshoot;
use crate::constraints::{Constraint, Var};

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
/// a solve can't turn one into the other. With a `gap` (`with_gap`), the
/// curves hold that distance apart instead of touching, measured the same
/// way (`distance` between a circle and an arc, or two arcs): outside,
/// d = r₁ + r₂ + gap; inside, d = |r₁ − r₂| − gap. Tangency is a gap of 0.
///
/// Let w = c_b − c_a and d = |w|. The tangency point lies on the line of
/// centers: from c_a in direction +w, from c_b in direction −w (external);
/// inside, both in direction s·w with s = sign(r_a − r_b) (from the larger
/// circle's center through the smaller's).
///
/// Residuals:
///   R₀ = d − (r_a + r_b) − gap           (external)
///   R₀ = d − s·(r_a − r_b) + gap         (internal)
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
    pub gap: f64,
}

impl TangentCurvesConstraint {
    pub fn new(a: TangentCurve, b: TangentCurve, internal: bool) -> Self {
        Self { a, b, internal, gap: 0.0 }
    }

    /// The curves held `gap` apart instead of touching.
    pub fn with_gap(self, gap: f64) -> Self {
        Self { gap, ..self }
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
            r[0] = d - s * (ra - rb) + self.gap;
            (s, (-s, s))
        } else {
            r[0] = d - (ra + rb) - self.gap;
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
