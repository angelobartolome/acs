//! [`TangentSplineConstraint`]: a Line, circle, arc, ellipse or another
//! Spline touches a Spline tangentially, at a contact point whose curve
//! parameter(s) the constraint owns.

use nalgebra::DMatrix;

use crate::constraints::arc_span::span_overshoot;
use crate::constraints::{Constraint, Var, xy};
use crate::geometry::Spline;
use crate::spline::{
    Curve, Dv, P2, Scalar, curve_d, curve_f, handle_vars, line_angle, overshoot, point_d,
    point_tangent,
};

/// What a Spline is tangent to, by ID.
pub enum TangentTo {
    /// A Line (the segment), by its endpoints a, b.
    Line(String, String),
    /// A circle, by its center Point and its own ID.
    Circle(String, String),
    /// An arc, by its center Point and its own ID; the contact is on its span.
    Arc(String, String),
    /// An ellipse, by its center and focus Points and its own ID.
    Ellipse(String, String, String),
    /// Another Spline, with the curve parameter of the contact on it (also
    /// owned by this constraint).
    Spline(Spline, String),
}

/// Tangency between a Spline (curve C, contact parameter t) and another
/// curve, touching at q = C(t) with unit tangent τ̂ = C′(t)/|C′(t)| and unit
/// normal n̂ = rot90(τ̂). ∠(u, τ̂) is the angle from τ̂ to the unit direction
/// u modulo π, in (−π/2, π/2] (0 when parallel either way;
/// `crate::spline::line_angle`), and S the length of the Spline's control
/// polygon, which weighs an angle against distances.
///
/// - Line a→b (g = (b − a)/|b − a|, σ = g·(q − a), f = a + σ·g the line's
///   point nearest q): R₀ = n̂·(f − q) (the line through q's tangent),
///   R₁ = S·∠(g, τ̂) (the line along it), R₂ = how far σ lies outside
///   [0, |b − a|] (q on the segment, as `TangentLineCircle`'s segment row).
/// - Circle (center c, radius r, v = q − c): R₀ = |v| − r, R₁ = v·τ̂ (q is
///   the foot of c on the curve, so the radius is normal to it).
/// - Arc: the circle's rows, plus R = r·overshoot(atan2(v), α, β) (q on the
///   arc's span, as `PointOnArc`).
/// - Ellipse (center c, focus f, f₂ = 2c − f, major radius A):
///   R₀ = |q − f| + |q − f₂| − 2A (q on it, as `PointOnEllipse`),
///   R₁ = S·∠(rot90(m̂), τ̂) with m = (q − f)/|q − f| + (q − f₂)/|q − f₂|
///   (the ellipse's normal at q bisects the focal directions; its tangent is
///   m̂ turned a quarter).
/// - Spline D (contact parameter s, w = D(s) − q): R₀ = n̂·w, R₁ = τ̂·w
///   (D(s) on q's normal at zero height: the same point), R₂ = S·∠(τ̂_D, τ̂).
///
/// and last, one span row per Spline: how far its parameter lies outside its
/// domain (0 inside). Each form has one equation more than the parameters it
/// owns, so it removes one degree of freedom. Neither side of the touch is
/// chosen: a circle can touch the curve from either side, wherever the
/// contact parameter starts (the best of a sampling of the curve).
///
/// The rows are chosen to converge from poor starts: each gap is measured
/// along the Spline's normal or at a foot point, so it barely changes as
/// the contact slides (sliding along a tangent can't fake progress), and
/// each angle is the angle itself, whose derivative never vanishes (the
/// sine of it has a false minimum at perpendicular). Partials are exact,
/// `crate::spline`'s forward mode.
pub struct TangentSplineConstraint {
    /// The other curve.
    pub to: TangentTo,
    /// The Spline.
    pub spline: Spline,
    /// Its contact parameter t, owned by this constraint.
    pub param_id: String,
}

impl TangentSplineConstraint {
    /// The constraint over these entities, by ID (see the fields).
    pub fn new(to: TangentTo, spline: Spline, param_id: String) -> Self {
        Self { to, spline, param_id }
    }

    /// Local index of the other curve's first variable (after the
    /// Spline's handles and t).
    fn other(&self) -> usize {
        2 * self.spline.points.len() + 1
    }
}

/// The other curve's own span row on an arc: r·overshoot(atan2(v), α, β),
/// with α, β at local variables `alpha`, `alpha + 1`.
fn arc_span_row(v: &P2<Dv>, r: &Dv, x: &[f64], alpha: usize) -> Dv {
    let phi = v.y.atan2(&v.x);
    let (o, [d_phi, d_start, d_end]) = span_overshoot(phi.v, x[alpha], x[alpha + 1]);
    let along = phi.clone() * Dv::cst(d_phi)
        + Dv::var(x, alpha) * Dv::cst(d_start)
        + Dv::var(x, alpha + 1) * Dv::cst(d_end);
    // Value o (the variables' values cancel), partials of `along`.
    let o = Dv { v: o, g: along.g };
    o * r.clone()
}

/// An ellipse (center, focus, minor radius b) at q: its on-ellipse residual
/// |q − f| + |q − f₂| − 2A and its unit normal there.
fn ellipse_at<S: Scalar>(q: &P2<S>, center: P2<S>, focus: P2<S>, b: S) -> (S, P2<S>) {
    let e = focus.sub(&center);
    let major = (b.clone() * b + e.dot(&e)).sqrt();
    let focus2 = center.add(&center).sub(&focus);
    let (w1, w2) = (q.sub(&focus), q.sub(&focus2));
    let on = w1.norm() + w2.norm() - major * S::cst(2.0);
    (on, w1.unit().add(&w2.unit()).unit())
}

/// How far a position σ along a segment of length `len` lies outside it.
fn segment_overshoot(sigma: Dv, len: Dv) -> Dv {
    if sigma.v < 0.0 {
        sigma
    } else if sigma.v > len.v {
        sigma - len
    } else {
        Dv::cst(0.0)
    }
}

impl Constraint for TangentSplineConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        // x = [handles (2n), t, the other curve's variables]
        let mut v = handle_vars(&self.spline);
        v.push(Var::CurveParam(&self.param_id));
        match &self.to {
            TangentTo::Line(a, b) => v.extend(xy(a).into_iter().chain(xy(b))),
            TangentTo::Circle(c, id) => {
                v.extend(xy(c));
                v.push(Var::Radius(id));
            }
            TangentTo::Arc(c, id) => {
                v.extend(xy(c));
                v.extend([Var::Radius(id), Var::StartAngle(id), Var::EndAngle(id)]);
            }
            TangentTo::Ellipse(c, f, id) => {
                v.extend(xy(c).into_iter().chain(xy(f)));
                v.push(Var::MinorRadius(id));
            }
            TangentTo::Spline(s, t) => {
                v.extend(handle_vars(s));
                v.push(Var::CurveParam(t));
            }
        }
        v
    }

    fn curve_params(&self) -> Vec<&str> {
        match &self.to {
            TangentTo::Spline(_, t) => vec![&self.param_id, t],
            _ => vec![&self.param_id],
        }
    }

    fn init_curve_params(&self, x: &[f64]) -> Vec<f64> {
        let seed = self.seed_params(x);
        self.refine_params(x, seed)
    }

    fn num_residuals(&self) -> usize {
        match self.to {
            TangentTo::Line(..) | TangentTo::Arc(..) => 4,
            TangentTo::Circle(..) | TangentTo::Ellipse(..) => 3,
            TangentTo::Spline(..) => 5,
        }
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        self.eval_rows(x, r, j);
    }
}

impl TangentSplineConstraint {
    /// Local indices of its parameters, in [`Constraint::curve_params`] order.
    fn param_indices(&self) -> Vec<usize> {
        let o = self.other();
        match &self.to {
            TangentTo::Spline(s, _) => vec![o - 1, o + 2 * s.points.len()],
            _ => vec![o - 1],
        }
    }

    /// Gauss–Newton on the parameters alone (the geometry held), from
    /// `seed`, each kept in its curve's domain: in a sketch that already
    /// holds, they land on the contact exactly (so the pre-solver skips it),
    /// and otherwise as close as the geometry allows.
    fn refine_params(&self, x: &[f64], seed: Vec<f64>) -> Vec<f64> {
        let idx = self.param_indices();
        let mut x = x.to_vec();
        for (&i, &v) in idx.iter().zip(&seed) {
            x[i] = v;
        }
        let domains: Vec<(f64, f64)> = match &self.to {
            TangentTo::Spline(s, _) => vec![
                curve_f(&self.spline, &x, 0).domain(),
                curve_f(s, &x, self.other()).domain(),
            ],
            _ => vec![curve_f(&self.spline, &x, 0).domain()],
        };
        let n = self.num_residuals();
        for _ in 0..30 {
            let mut r = vec![0.0; n];
            let mut j = DMatrix::zeros(n, x.len());
            self.eval_rows(&x, &mut r, &mut j);
            let jp = j.select_columns(idx.iter());
            let Some(h) = jp.clone().svd(true, true).solve(&-nalgebra::DVector::from(r), 1e-14).ok() else {
                break;
            };
            for (k, &i) in idx.iter().enumerate() {
                x[i] = (x[i] + h[k]).clamp(domains[k].0, domains[k].1);
            }
            if h.amax() < 1e-15 {
                break;
            }
        }
        idx.iter().map(|&i| x[i]).collect()
    }

    /// The best of a sampling of the curve(s) for the contact.
    fn seed_params(&self, x: &[f64]) -> Vec<f64> {
        let curve = curve_f(&self.spline, x, 0);
        let o = self.other();
        let pt = |i: usize| P2::new(x[i], x[i + 1]);
        let samples = curve.samples(16);
        let best = |score: &dyn Fn([f64; 2], [f64; 2]) -> f64| {
            samples
                .iter()
                .copied()
                .map(|t| {
                    let (q, tau) = point_tangent(&curve, t);
                    (t, score(q, tau))
                })
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .map_or(0.0, |(t, _)| t)
        };
        match &self.to {
            TangentTo::Line(..) => {
                let (a, b) = (pt(o), pt(o + 2));
                let d = b.sub(&a);
                let len = d.norm();
                let g = d.unit();
                vec![best(&|q, tau| {
                    let w = P2::new(q[0], q[1]).sub(&a);
                    let along = g.dot(&w).clamp(0.0, len);
                    let off = w.sub(&g.scale(&along)).norm();
                    off + len * g.cross(&P2::new(tau[0], tau[1])).abs()
                })]
            }
            TangentTo::Circle(..) | TangentTo::Arc(..) => {
                let (c, r) = (pt(o), x[o + 2]);
                let arc = matches!(self.to, TangentTo::Arc(..));
                vec![best(&|q, tau| {
                    let v = P2::new(q[0], q[1]).sub(&c);
                    let mut s = (v.norm() - r).abs() + r * v.unit().dot(&P2::new(tau[0], tau[1])).abs();
                    if arc {
                        s += r * span_overshoot(v.y.atan2(v.x), x[o + 3], x[o + 4]).0.abs();
                    }
                    s
                })]
            }
            TangentTo::Ellipse(..) => {
                let (c, f, b) = (pt(o), pt(o + 2), x[o + 4]);
                let size = (b * b + f.sub(&c).dot(&f.sub(&c))).sqrt();
                vec![best(&|q, tau| {
                    let (on, n) = ellipse_at(&P2::new(q[0], q[1]), c.clone(), f.clone(), b);
                    on.abs() + size * n.dot(&P2::new(tau[0], tau[1])).abs()
                })]
            }
            TangentTo::Spline(s, _) => {
                let other = curve_f(s, x, o);
                let ours: Vec<_> = samples.iter().map(|&t| (t, point_tangent(&curve, t))).collect();
                let theirs: Vec<_> = other.samples(16).into_iter().map(|t| (t, point_tangent(&other, t))).collect();
                let size = chord(&curve).max(chord(&other)).max(1e-9);
                let mut best = (f64::INFINITY, 0.0, 0.0);
                for (t, (q1, t1)) in &ours {
                    for (u, (q2, t2)) in &theirs {
                        let gap = (q1[0] - q2[0]).hypot(q1[1] - q2[1]);
                        let score = gap + size * (t1[0] * t2[1] - t1[1] * t2[0]).abs();
                        if score < best.0 {
                            best = (score, *t, *u);
                        }
                    }
                }
                vec![best.1, best.2]
            }
        }
    }

    /// The residuals and partials (see the type's docs).
    fn eval_rows(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        let o = self.other();
        let curve = curve_d(&self.spline, x, 0);
        let t = Dv::var(x, o - 1);
        let d = curve.eval(&t, 1);
        let (q, tau) = (&d[0], d[1].unit());
        let (lo, hi) = curve.domain();
        let span = overshoot(&t, lo, hi);
        let rows: Vec<Dv> = match &self.to {
            TangentTo::Line(..) => {
                let (a, b) = (point_d(x, o), point_d(x, o + 2));
                let ab = b.sub(&a);
                let g = ab.unit();
                let w = q.sub(&a);
                let n = tau.rot90();
                let foot = a.add(&g.scale(&g.dot(&w)));
                vec![
                    n.dot(&foot.sub(q)),
                    line_angle(&g, &tau) * size_d(&curve),
                    segment_overshoot(g.dot(&w), ab.norm()),
                    span,
                ]
            }
            TangentTo::Circle(..) | TangentTo::Arc(..) => {
                let (c, rad) = (point_d(x, o), Dv::var(x, o + 2));
                let v = q.sub(&c);
                let mut rows = vec![v.norm() - rad.clone(), v.dot(&tau), span];
                if matches!(self.to, TangentTo::Arc(..)) {
                    rows.push(arc_span_row(&v, &rad, x, o + 3));
                }
                rows
            }
            TangentTo::Ellipse(..) => {
                let (on, n) = ellipse_at(q, point_d(x, o), point_d(x, o + 2), Dv::var(x, o + 4));
                vec![on, line_angle(&n.rot90(), &tau) * size_d(&curve), span]
            }
            TangentTo::Spline(s, _) => {
                let other = curve_d(s, x, o);
                let u = Dv::var(x, o + 2 * s.points.len());
                let e = other.eval(&u, 1);
                let gap = e[0].sub(q);
                let (lo2, hi2) = other.domain();
                vec![
                    tau.rot90().dot(&gap),
                    tau.dot(&gap),
                    line_angle(&e[1].unit(), &tau) * size_d(&curve),
                    span,
                    overshoot(&u, lo2, hi2),
                ]
            }
        };
        for (row, v) in rows.into_iter().enumerate() {
            r[row] = v.v;
            v.write_row(j, row);
        }
    }
}

/// The length of a curve's control polygon, with partials: a size for
/// weighing an angle against distances.
fn size_d(curve: &Curve<Dv>) -> Dv {
    curve.control.windows(2).map(|w| w[1].sub(&w[0]).norm()).fold(Dv::cst(0.0), |a, b| a + b)
}

/// The length of a curve's control polygon: a size for weighing angles
/// against distances.
fn chord(curve: &Curve<f64>) -> f64 {
    curve.control.windows(2).map(|w| w[1].sub(&w[0]).norm()).sum()
}
