//! [`PointOnSplineConstraint`]: a point lies on a Spline.

use nalgebra::DMatrix;

use crate::constraints::{Constraint, Var, xy};
use crate::geometry::Spline;
use crate::spline::{Dv, closest_param, curve_d, curve_f, handle_vars, overshoot, point_d};

/// Constrains a point to lie on a Spline (between its ends), at a curve
/// parameter `t` this constraint owns.
///
/// Entities:
///   - `point_id` – the point p
///   - `spline`   – the Spline, curve C (its handles are the variables)
///   - `param_id` – the curve parameter t (a [`Var::Param`] it owns)
///
/// Residuals:
///   R₀,₁ = p − C(t)
///   R₂   = how far t lies outside the curve's domain [t₀, t₁] (0 inside,
///          like an Arc's span row)
///
/// Two equations and one more unknown: a point on a free Spline loses one
/// degree of freedom. Partials are exact (`crate::spline`'s forward mode,
/// through a fit-point Spline's parameters, knots and linear solve);
/// ∂C/∂t = C′(t). `t` starts at the point's closest point on the curve.
pub struct PointOnSplineConstraint {
    /// The point p.
    pub point_id: String,
    /// The Spline.
    pub spline: Spline,
    /// Its curve parameter t, owned by this constraint.
    pub param_id: String,
}

impl PointOnSplineConstraint {
    /// The constraint over these entities, by ID (see the fields).
    pub fn new(point_id: String, spline: Spline, param_id: String) -> Self {
        Self { point_id, spline, param_id }
    }
}

impl Constraint for PointOnSplineConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        // x = [handles (2n), px, py, t]
        let mut v = handle_vars(&self.spline);
        v.extend(xy(&self.point_id));
        v.push(Var::Param(&self.param_id));
        v
    }

    fn params(&self) -> Vec<&str> {
        vec![&self.param_id]
    }

    fn init_params(&self, x: &[f64]) -> Vec<f64> {
        let h = 2 * self.spline.points.len();
        vec![closest_param(&curve_f(&self.spline, x, 0), [x[h], x[h + 1]])]
    }

    fn num_residuals(&self) -> usize {
        3
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        let h = 2 * self.spline.points.len();
        let curve = curve_d(&self.spline, x, 0);
        let t = Dv::var(x, h + 2);
        let c = &curve.eval(&t, 0)[0];
        let d = point_d(x, h).sub(c);
        let (lo, hi) = curve.domain();
        for (row, v) in [d.x, d.y, overshoot(&t, lo, hi)].into_iter().enumerate() {
            r[row] = v.v;
            v.write_row(j, row);
        }
    }
}
