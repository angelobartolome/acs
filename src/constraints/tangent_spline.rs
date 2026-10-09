//! [`TangentSplineConstraint`]: a Line, circle, arc, ellipse or another
//! Spline touches a Spline tangentially, at a contact point whose curve
//! parameter(s) the constraint owns.

use nalgebra::{DMatrix, DVector};

use crate::constraints::arc_span::span_overshoot;
use crate::constraints::ellipse::{D, EllipseFrame, V2};
use crate::constraints::segment::foot_overshoot;
use crate::constraints::{Constraint, Var, xy};
use crate::geometry::Spline;
use crate::spline::{
    Curve, Dv, P2, Scalar, curve_d, curve_f, handle_vars, line_angle, overshoot, point_d,
    point_tangent,
};

/// What a Spline is tangent to.
pub enum TangentTo {
    /// A Line (the segment).
    Line(LineSide),
    /// A circle.
    Circle(CircleSide),
    /// An arc; the contact is on its span.
    Arc(ArcSide),
    /// An ellipse.
    Ellipse(EllipseSide),
    /// Another Spline.
    Spline(SplineSide),
}

impl TangentTo {
    /// Its part in the tangency.
    fn partner(&self) -> &dyn Partner {
        match self {
            TangentTo::Line(p) => p,
            TangentTo::Circle(p) => p,
            TangentTo::Arc(p) => p,
            TangentTo::Ellipse(p) => p,
            TangentTo::Spline(p) => p,
        }
    }
}

/// A Line, by ID.
pub struct LineSide {
    /// Its start Point a.
    pub a: String,
    /// Its end Point b.
    pub b: String,
}

/// A circle, by ID.
pub struct CircleSide {
    /// Its center Point.
    pub center: String,
    /// The circle (its radius).
    pub circle: String,
}

/// An arc, by ID.
pub struct ArcSide {
    /// Its center Point.
    pub center: String,
    /// The arc (its radius and angles).
    pub arc: String,
}

/// An ellipse, by ID.
pub struct EllipseSide {
    /// Its center Point.
    pub center: String,
    /// Its focus Point.
    pub focus: String,
    /// The ellipse (its minor radius).
    pub ellipse: String,
}

/// Another Spline, by ID.
pub struct SplineSide {
    /// The Spline.
    pub spline: Spline,
    /// The contact's curve parameter on it (also owned by the constraint).
    pub param_id: String,
}

/// Tangency between a Spline (curve C, contact parameter t) and another
/// curve.
///
/// Entities:
///   - `spline`   – the Spline, curve C (its handles are the variables)
///   - `param_id` – the contact's curve parameter t on C (a
///     [`Var::CurveParam`] it owns)
///   - `to`       – the other curve: a Line's endpoints a, b; a circle's or
///     arc's center c and the circle or arc (radius r, an arc's angles α,
///     β); an ellipse's center c, focus f and the ellipse (minor radius b);
///     or another Spline D with the contact's curve parameter s on it (also
///     owned)
///
/// They touch at q = C(t) with unit tangent τ̂ = C′(t)/|C′(t)| and unit
/// normal n̂ = rot90(τ̂). ∠(u, τ̂) is the angle from τ̂ to the unit direction
/// u modulo π, in (−π/2, π/2] (0 when parallel either way;
/// `crate::spline::line_angle`), and S the length of the Spline's control
/// polygon (`Curve::polygon_length`), which weighs an angle against
/// distances.
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
///   (D(s) on q's normal at zero height: the same point), R₂ = S·∠(τ̂_D, τ̂),
///   R₃ = D's span row for s.
///
/// and last, C's span row: how far t lies outside C's domain (0 inside). Each form has one equation more than the parameters it
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

/// The contact on the Spline, with partials: what every [`Partner`]'s rows
/// are measured from.
struct Contact<'a> {
    /// The local variables.
    x: &'a [f64],
    /// Local index of the other curve's first variable.
    first: usize,
    /// The Spline's curve.
    curve: &'a Curve<Dv>,
    /// The contact point q = C(t).
    q: &'a P2<Dv>,
    /// The unit tangent τ̂ there.
    tau: &'a P2<Dv>,
}

/// The Spline's curve in plain numbers and a sampling of it, to seed the
/// contact parameter(s).
struct Seeding<'a> {
    /// The local variables.
    x: &'a [f64],
    /// Local index of the other curve's first variable.
    first: usize,
    /// The Spline's curve.
    curve: &'a Curve<f64>,
    /// Parameters sampled along it.
    samples: Vec<f64>,
}

impl Seeding<'_> {
    /// The Point at local variables `i`, `i + 1`.
    fn point(&self, i: usize) -> P2<f64> {
        P2::new(self.x[i], self.x[i + 1])
    }

    /// The sampled t with the lowest `score(q, τ̂)`.
    fn best(&self, score: impl Fn(P2<f64>, P2<f64>) -> f64) -> f64 {
        self.samples
            .iter()
            .map(|&t| {
                let (q, tau) = point_tangent(self.curve, t);
                (t, score(P2::new(q[0], q[1]), P2::new(tau[0], tau[1])))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map_or(0.0, |(t, _)| t)
    }
}

/// The other curve's part in a tangency: the variables it reads (after the
/// Spline's handles and t), a curve parameter of its own if it is a Spline,
/// its rows and how to seed the contact.
trait Partner {
    /// Its variables.
    fn vars(&self) -> Vec<Var<'_>>;
    /// Its rows, measured from the contact.
    fn rows(&self, c: &Contact) -> Vec<Dv>;
    /// How many rows.
    fn num_rows(&self) -> usize;
    /// Starting values of the contact parameters: t, then its own.
    fn seed(&self, s: &Seeding) -> Vec<f64>;
    /// The curve parameter it owns, if it is a Spline.
    fn own_param(&self) -> Option<&str> {
        None
    }
    /// That parameter's local index and domain, given the local variables
    /// and its first variable's index.
    fn own_slot(&self, _x: &[f64], _first: usize) -> Option<(usize, (f64, f64))> {
        None
    }
}

impl Partner for LineSide {
    fn vars(&self) -> Vec<Var<'_>> {
        xy(&self.a).into_iter().chain(xy(&self.b)).collect()
    }

    fn num_rows(&self) -> usize {
        3
    }

    fn rows(&self, c: &Contact) -> Vec<Dv> {
        let (a, b) = (point_d(c.x, c.first), point_d(c.x, c.first + 2));
        let g = b.sub(&a).unit();
        let foot = a.add(&g.scale(&g.dot(&c.q.sub(&a))));
        vec![
            c.tau.rot90().dot(&foot.sub(c.q)),
            line_angle(&g, c.tau) * c.curve.polygon_length(),
            segment_overshoot(c.q, &a, &b),
        ]
    }

    fn seed(&self, s: &Seeding) -> Vec<f64> {
        let (a, b) = (s.point(s.first), s.point(s.first + 2));
        let d = b.sub(&a);
        let (len, g) = (d.norm(), d.unit());
        vec![s.best(|q, tau| {
            let w = q.sub(&a);
            let along = g.dot(&w).clamp(0.0, len);
            w.sub(&g.scale(&along)).norm() + len * g.cross(&tau).abs()
        })]
    }
}

/// A circle's rows (center c, radius r, v = q − c): |v| − r and v·τ̂.
fn circle_rows(c: &Contact) -> (Vec<Dv>, P2<Dv>, Dv) {
    let (center, radius) = (point_d(c.x, c.first), Dv::var(c.x, c.first + 2));
    let v = c.q.sub(&center);
    (vec![v.norm() - radius.clone(), v.dot(c.tau)], v, radius)
}

/// A circle's seed score: how far q is from the circle, and the radius from
/// being normal to the curve there.
fn circle_score(s: &Seeding, q: &P2<f64>, tau: &P2<f64>) -> (f64, P2<f64>) {
    let (center, radius) = (s.point(s.first), s.x[s.first + 2]);
    let v = q.sub(&center);
    ((v.norm() - radius).abs() + radius * v.unit().dot(tau).abs(), v)
}

impl Partner for CircleSide {
    fn vars(&self) -> Vec<Var<'_>> {
        let mut v = xy(&self.center).to_vec();
        v.push(Var::Radius(&self.circle));
        v
    }

    fn num_rows(&self) -> usize {
        2
    }

    fn rows(&self, c: &Contact) -> Vec<Dv> {
        circle_rows(c).0
    }

    fn seed(&self, s: &Seeding) -> Vec<f64> {
        vec![s.best(|q, tau| circle_score(s, &q, &tau).0)]
    }
}

impl Partner for ArcSide {
    fn vars(&self) -> Vec<Var<'_>> {
        let mut v = xy(&self.center).to_vec();
        v.extend([Var::Radius(&self.arc), Var::StartAngle(&self.arc), Var::EndAngle(&self.arc)]);
        v
    }

    fn num_rows(&self) -> usize {
        3
    }

    fn rows(&self, c: &Contact) -> Vec<Dv> {
        let (mut rows, v, radius) = circle_rows(c);
        rows.push(arc_span_row(&v, &radius, c.x, c.first + 3));
        rows
    }

    fn seed(&self, s: &Seeding) -> Vec<f64> {
        let (alpha, beta, radius) = (s.x[s.first + 3], s.x[s.first + 4], s.x[s.first + 2]);
        vec![s.best(|q, tau| {
            let (score, v) = circle_score(s, &q, &tau);
            score + radius * span_overshoot(v.y.atan2(v.x), alpha, beta).0.abs()
        })]
    }
}

impl Partner for EllipseSide {
    fn vars(&self) -> Vec<Var<'_>> {
        let mut v: Vec<Var> = xy(&self.center).into_iter().chain(xy(&self.focus)).collect();
        v.push(Var::MinorRadius(&self.ellipse));
        v
    }

    fn num_rows(&self) -> usize {
        2
    }

    fn rows(&self, c: &Contact) -> Vec<Dv> {
        let (center, focus) = (point_d(c.x, c.first), point_d(c.x, c.first + 2));
        let (on, n) = ellipse_at(c.q, &center, &focus, &Dv::var(c.x, c.first + 4));
        vec![on, line_angle(&n.rot90(), c.tau) * c.curve.polygon_length()]
    }

    fn seed(&self, s: &Seeding) -> Vec<f64> {
        let cst = |p: P2<f64>| P2::new(Dv::cst(p.x), Dv::cst(p.y));
        let (center, focus, b) = (s.point(s.first), s.point(s.first + 2), s.x[s.first + 4]);
        let major_radius = (b * b + focus.sub(&center).dot(&focus.sub(&center))).sqrt();
        let (center, focus, b) = (cst(center), cst(focus), Dv::cst(b));
        vec![s.best(|q, tau| {
            let (on, n) = ellipse_at(&cst(q), &center, &focus, &b);
            on.v.abs() + major_radius * (n.x.v * tau.x + n.y.v * tau.y).abs()
        })]
    }
}

impl Partner for SplineSide {
    fn vars(&self) -> Vec<Var<'_>> {
        let mut v = handle_vars(&self.spline);
        v.push(Var::CurveParam(&self.param_id));
        v
    }

    fn num_rows(&self) -> usize {
        4
    }

    fn rows(&self, c: &Contact) -> Vec<Dv> {
        let other = curve_d(&self.spline, c.x, c.first);
        let s = Dv::var(c.x, c.first + 2 * self.spline.points.len());
        let at = other.eval(&s, 1);
        let gap = at[0].sub(c.q);
        let (lo, hi) = other.domain();
        vec![
            c.tau.rot90().dot(&gap),
            c.tau.dot(&gap),
            line_angle(&at[1].unit(), c.tau) * c.curve.polygon_length(),
            overshoot(&s, lo, hi),
        ]
    }

    fn seed(&self, s: &Seeding) -> Vec<f64> {
        let other = curve_f(&self.spline, s.x, s.first);
        let ours: Vec<_> = s.samples.iter().map(|&t| (t, point_tangent(s.curve, t))).collect();
        let theirs: Vec<_> = other.samples(16).into_iter().map(|u| (u, point_tangent(&other, u))).collect();
        let size = s.curve.polygon_length().max(other.polygon_length()).max(1e-9);
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

    fn own_param(&self) -> Option<&str> {
        Some(&self.param_id)
    }

    fn own_slot(&self, x: &[f64], first: usize) -> Option<(usize, (f64, f64))> {
        let index = first + 2 * self.spline.points.len();
        Some((index, curve_f(&self.spline, x, first).domain()))
    }
}

impl TangentSplineConstraint {
    /// The constraint over these entities, by ID (see the fields).
    pub fn new(to: TangentTo, spline: Spline, param_id: String) -> Self {
        Self { to, spline, param_id }
    }

    /// Local index of the other curve's first variable (after the
    /// Spline's handles and t).
    fn other_offset(&self) -> usize {
        2 * self.spline.points.len() + 1
    }

    /// Local index, and domain, of each curve parameter it owns, in
    /// [`Constraint::curve_params`] order.
    fn param_slots(&self, x: &[f64]) -> Vec<(usize, (f64, f64))> {
        let first = self.other_offset();
        let mut slots = vec![(first - 1, curve_f(&self.spline, x, 0).domain())];
        slots.extend(self.to.partner().own_slot(x, first));
        slots
    }

    /// Gauss–Newton on the parameters alone (the geometry held), from
    /// `seed`, each kept in its curve's domain: in a sketch that already
    /// holds, they land on the contact exactly (so the pre-solver skips it),
    /// and otherwise as close as the geometry allows.
    fn refine_params(&self, x: &[f64], seed: Vec<f64>) -> Vec<f64> {
        let mut x = x.to_vec();
        let slots = self.param_slots(&x);
        for (&(i, _), &v) in slots.iter().zip(&seed) {
            x[i] = v;
        }
        let idx: Vec<usize> = slots.iter().map(|&(i, _)| i).collect();
        let n = self.num_residuals();
        for _ in 0..30 {
            let mut r = vec![0.0; n];
            let mut j = DMatrix::zeros(n, x.len());
            self.eval(&x, &mut r, &mut j);
            let jp = j.select_columns(idx.iter());
            let Some(h) = jp.svd(true, true).solve(&-DVector::from(r), 1e-14).ok() else {
                break;
            };
            for (k, &(i, (lo, hi))) in slots.iter().enumerate() {
                x[i] = (x[i] + h[k]).clamp(lo, hi);
            }
            if h.amax() < 1e-15 {
                break;
            }
        }
        idx.iter().map(|&i| x[i]).collect()
    }
}

/// The other curve's own span row on an arc: r·overshoot(atan2(v), α, β),
/// with α, β at local variables `alpha`, `alpha + 1`.
fn arc_span_row(v: &P2<Dv>, r: &Dv, x: &[f64], alpha: usize) -> Dv {
    let phi = v.y.atan2(&v.x);
    let (outside, [d_phi, d_start, d_end]) = span_overshoot(phi.v, x[alpha], x[alpha + 1]);
    let along = phi.clone() * Dv::cst(d_phi)
        + Dv::var(x, alpha) * Dv::cst(d_start)
        + Dv::var(x, alpha + 1) * Dv::cst(d_end);
    // Value `outside` (the variables' values cancel), partials of `along`.
    let outside = Dv { v: outside, g: along.g };
    outside * r.clone()
}

/// An ellipse (center, focus, minor radius b) at q: its on-ellipse residual
/// |q − f| + |q − f₂| − 2A and its unit normal there, by [`EllipseFrame`]
/// over the seven inputs, chained into `Dv`.
fn ellipse_at(q: &P2<Dv>, center: &P2<Dv>, focus: &P2<Dv>, b: &Dv) -> (Dv, P2<Dv>) {
    let inputs = [
        q.x.clone(),
        q.y.clone(),
        center.x.clone(),
        center.y.clone(),
        focus.x.clone(),
        focus.y.clone(),
        b.clone(),
    ];
    let x = inputs.clone().map(|d| d.v);
    let e = EllipseFrame::<7>::new(&x, 2, 4, 6);
    let p = V2::point(&x, 0);
    let chain = |d: D<7>| Dv::chain(d.v, &d.g, &inputs);
    let n = e.normal_at(p);
    (chain(e.focal_residual(p)), P2::new(chain(n.x), chain(n.y)))
}

/// How far the foot of q on the Line a→b falls outside the segment, by
/// `segment.rs`'s [`foot_overshoot`] over the six inputs, chained into `Dv`.
fn segment_overshoot(q: &P2<Dv>, a: &P2<Dv>, b: &P2<Dv>) -> Dv {
    let inputs = [q.x.clone(), q.y.clone(), a.x.clone(), a.y.clone(), b.x.clone(), b.y.clone()];
    let [qx, qy, ax, ay, bx, by] = inputs.clone().map(|d| d.v);
    let (v, g) = foot_overshoot(qx, qy, ax, ay, bx, by);
    Dv::chain(v, &g, &inputs)
}

impl Constraint for TangentSplineConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        // x = [handles (2n), t, the other curve's variables]
        let mut v = handle_vars(&self.spline);
        v.push(Var::CurveParam(&self.param_id));
        v.extend(self.to.partner().vars());
        v
    }

    fn curve_params(&self) -> Vec<&str> {
        let mut params = vec![self.param_id.as_str()];
        params.extend(self.to.partner().own_param());
        params
    }

    fn init_curve_params(&self, x: &[f64]) -> Vec<f64> {
        let curve = curve_f(&self.spline, x, 0);
        let seeding = Seeding { x, first: self.other_offset(), samples: curve.samples(16), curve: &curve };
        let seed = self.to.partner().seed(&seeding);
        self.refine_params(x, seed)
    }

    fn num_residuals(&self) -> usize {
        self.to.partner().num_rows() + 1
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        let first = self.other_offset();
        let curve = curve_d(&self.spline, x, 0);
        let t = Dv::var(x, first - 1);
        let d = curve.eval(&t, 1);
        let tau = d[1].unit();
        let contact = Contact { x, first, curve: &curve, q: &d[0], tau: &tau };
        let mut rows = self.to.partner().rows(&contact);
        let (lo, hi) = curve.domain();
        rows.push(overshoot(&t, lo, hi));
        for (row, v) in rows.into_iter().enumerate() {
            r[row] = v.v;
            v.write_row(j, row);
        }
    }
}
