//! [`TangentSplineAtPointConstraint`]: a Line, an Arc or another Spline is
//! tangent to a Spline at a point known to be on both.

use nalgebra::DMatrix;

use crate::constraints::{Constraint, SplineAt, Var, xy};
use crate::geometry::Spline;
use crate::spline::{Curve, Dv, P2, Scalar, curve_d, handle_vars, line_angle, point_d};

/// The other side of a tangency at a known point.
pub enum Side {
    /// A Line.
    Line {
        /// Its endpoint on the Spline.
        point: String,
        /// Its other endpoint.
        other: String,
    },
    /// An Arc, by the radius to its endpoint on the Spline.
    ArcRadius {
        /// Its endpoint on the Spline.
        point: String,
        /// Its center Point.
        center: String,
    },
    /// Another Spline.
    Spline {
        /// The Spline.
        spline: Spline,
        /// Where on it the contact is.
        at: SplineAt,
    },
}

/// Tangency where the contact point is already known to be on the Spline:
/// a Line or Arc endpoint that *is* the Spline's first or last handle (on
/// the curve at its start or end), a Line endpoint an `on` holds on the
/// Spline (at that `on`'s curve parameter), or two Splines sharing an end.
/// As for `TangentAtPoint` (Line–arc), the distance form of tangency is
/// degenerate through a point already on the curve (its row vanishes at the
/// solution); the angle here changes linearly. With τ̂ = C′(t)/|C′(t)| the
/// Spline's unit tangent at `at` and ∠(u, τ̂) the angle from τ̂ to the unit
/// direction u modulo π, in (−π/2, π/2] (0 when they are parallel either
/// way; `crate::spline::line_angle`):
///
/// - Line p→o: R = ∠(ĝ, τ̂), ĝ = (o − p)/|o − p|
/// - Arc with center k: R = ∠(rot90(r̂), τ̂), r̂ = (p − k)/|p − k| (the
///   arc's tangent at p along the curve's)
/// - Spline D at `at₂`: R = ∠(τ̂_D, τ̂)
///
/// It owns no parameter: `at` is a domain end (a constant) or another
/// constraint's [`Var::CurveParam`], which it reads.
///
/// Entities:
///   - `spline` – the Spline, curve C (its handles are the variables)
///   - `at`     – where on C the contact is: its start, its end, or a curve
///     parameter
///   - `side`   – the other side: a Line's endpoint p on C and its other
///     endpoint o; an Arc's endpoint p on C and its center k; or another
///     Spline D and where on D the contact is
pub struct TangentSplineAtPointConstraint {
    /// The other side.
    pub side: Side,
    /// The Spline.
    pub spline: Spline,
    /// Where on it the contact is.
    pub at: SplineAt,
}

impl TangentSplineAtPointConstraint {
    /// The constraint over these entities, by ID (see the fields).
    pub fn new(side: Side, spline: Spline, at: SplineAt) -> Self {
        Self { side, spline, at }
    }
}

/// `spline`'s handles, then its curve parameter if `at` is one.
fn spline_vars<'a>(spline: &'a Spline, at: &'a SplineAt) -> Vec<Var<'a>> {
    let mut v = handle_vars(spline);
    if let SplineAt::CurveParam(t) = at {
        v.push(Var::CurveParam(t));
    }
    v
}

/// The unit tangent of `spline` at `at`, its handles (and parameter) from
/// local variable `offset`; and the index after them.
fn tangent_at(spline: &Spline, at: &SplineAt, x: &[f64], offset: usize) -> (P2<Dv>, usize) {
    let curve: Curve<Dv> = curve_d(spline, x, offset);
    let (lo, hi) = curve.domain();
    let next = offset + 2 * spline.points.len();
    let (t, next) = match at {
        SplineAt::Start => (Dv::cst(lo), next),
        SplineAt::End => (Dv::cst(hi), next),
        SplineAt::CurveParam(_) => (Dv::var(x, next), next + 1),
    };
    (curve.eval(&t, 1)[1].unit(), next)
}

impl Constraint for TangentSplineAtPointConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        // x = [handles (2n), (t), the other side's variables]
        let mut v = spline_vars(&self.spline, &self.at);
        match &self.side {
            Side::Line { point, other } => v.extend(xy(point).into_iter().chain(xy(other))),
            Side::ArcRadius { point, center } => v.extend(xy(point).into_iter().chain(xy(center))),
            Side::Spline { spline, at } => v.extend(spline_vars(spline, at)),
        }
        v
    }

    fn num_residuals(&self) -> usize {
        1
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        let (tau, o) = tangent_at(&self.spline, &self.at, x, 0);
        let res = match &self.side {
            Side::Line { .. } => line_angle(&point_d(x, o + 2).sub(&point_d(x, o)).unit(), &tau),
            // The arc's tangent at p is its radius turned a quarter turn.
            Side::ArcRadius { .. } => line_angle(&point_d(x, o).sub(&point_d(x, o + 2)).unit().rot90(), &tau),
            Side::Spline { spline, at } => line_angle(&tangent_at(spline, at, x, o).0, &tau),
        };
        r[0] = res.v;
        res.write_row(j, 0);
    }
}
