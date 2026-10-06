use nalgebra::DMatrix;

use crate::constraints::ellipse::{D, EllipseFrame, V2};
use crate::constraints::{Constraint, Var, xy};

/// Constrains a Line (the segment between its endpoints) to be tangent to an
/// ellipse, touching it on the segment, as a Line is tangent to a circle.
///
/// Entities:
///   - `line_pa_id`, `line_pb_id` – the Line's endpoints a, b
///   - `center_id`                – the ellipse's center c
///   - `focus_id`                 – the ellipse's focus f₁ (f₂ = 2c − f₁)
///   - `ellipse_id`               – the ellipse (minor radius b)
///
/// With g = (b − a)/|b − a|, ν = rot90(g), and for each focus its signed
/// distance sᵢ = ν·(fᵢ − a) from the line and its position σᵢ = g·(fᵢ − a)
/// along it:
///   R₀ = |c − f₁ + s₁·ν| − a_maj
///        (half of |f₂ − f₁'| − 2a_maj, f₁' = f₁ mirrored in the line: the
///        tangency condition GCS's `ConstraintEllipseTangentLine` uses)
///   R₁ = overshoot of σ_T = (s₁σ₂ + s₂σ₁)/(s₁ + s₂) past [0, |b − a|]
///        (σ_T is the tangency point's position along the line: where the
///        segment f₂ f₁' crosses it; 0 while it's on the segment)
pub struct TangentLineEllipseConstraint {
    pub line_pa_id: String,
    pub line_pb_id: String,
    pub center_id: String,
    pub focus_id: String,
    pub ellipse_id: String,
}

impl TangentLineEllipseConstraint {
    pub fn new(
        line_pa_id: String,
        line_pb_id: String,
        center_id: String,
        focus_id: String,
        ellipse_id: String,
    ) -> Self {
        Self {
            line_pa_id,
            line_pb_id,
            center_id,
            focus_id,
            ellipse_id,
        }
    }
}

/// Below this, the line (nearly) passes through the center and has no
/// tangency point.
const NO_TANGENCY: f64 = 1e-12;

impl Constraint for TangentLineEllipseConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [
            &xy(&self.center_id)[..],
            &xy(&self.focus_id),
            &xy(&self.line_pa_id),
            &xy(&self.line_pb_id),
            &[Var::MinorRadius(&self.ellipse_id)],
        ]
        .concat()
    }

    fn num_residuals(&self) -> usize {
        2
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        // x = [cx, cy, fx, fy, ax, ay, bx, by, b]
        let e = EllipseFrame::<9>::new(x, 0, 2, 8);
        let t = line_ellipse_tangency(&e, V2::point(x, 4), V2::point(x, 6));
        r[0] = t.tangency.v;
        t.tangency.write_row(j, 0);
        r[1] = t.overshoot.v;
        t.overshoot.write_row(j, 1);
    }
}

/// A Line from `a` to `b` against ellipse `e`, as
/// [`TangentLineEllipseConstraint`] measures it.
pub(crate) struct LineEllipseTangency<const N: usize> {
    /// R₀: 0 when the line's Extension is tangent to the ellipse.
    pub tangency: D<N>,
    /// R₁: how far the tangency point lies off the segment (0 on it).
    pub overshoot: D<N>,
    /// The tangency point (where the line through the far focus and the
    /// mirror of the near one crosses the line), unless the line passes
    /// (nearly) through the center or the segment has no length.
    pub point: Option<V2<N>>,
}

pub(crate) fn line_ellipse_tangency<const N: usize>(
    e: &EllipseFrame<N>,
    a: V2<N>,
    b: V2<N>,
) -> LineEllipseTangency<N> {
    let d = b.sub(a);
    let g = d.unit();
    let nu = g.rot90();
    let (f1, f2) = (e.focus, e.focus2());
    let s1 = nu.dot(f1.sub(a));

    let h = e.center.sub(f1).add(nu.scale(s1));
    let tangency = h.norm() - e.major_radius;

    let s2 = nu.dot(f2.sub(a));
    let (sigma1, sigma2) = (g.dot(f1.sub(a)), g.dot(f2.sub(a)));
    let sum = s1 + s2;
    let len = d.norm();
    // A degenerate segment has no direction to overshoot along.
    if sum.v.abs() < NO_TANGENCY || len.v < NO_TANGENCY {
        return LineEllipseTangency {
            tangency,
            overshoot: D::cst(0.0),
            point: None,
        };
    }
    let sigma = (s1 * sigma2 + s2 * sigma1) / sum;
    let overshoot = if sigma.v < 0.0 {
        sigma
    } else if sigma.v > len.v {
        sigma - len
    } else {
        D::cst(0.0)
    };
    LineEllipseTangency {
        tangency,
        overshoot,
        point: Some(a.add(g.scale(sigma))),
    }
}
