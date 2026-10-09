//! The curve of a [`Spline`]: a clamped
//! [B-spline](https://en.wikipedia.org/wiki/B-spline) built from its handle
//! Points, either as its control points or as fit points it passes through.
//!
//! - **Control points** (`n ≥ 4`): a cubic, clamped uniform knots on
//!   `[0, 1]` unless the Spline gives a (clamped) knot vector.
//! - **Fit points** (`interpolated`): a cubic through the `n ≥ 2` points, at
//!   [centripetal](https://en.wikipedia.org/wiki/Centripetal_Catmull%E2%80%93Rom_spline)
//!   parameters `u₀ = 0 < … < uₙ₋₁ = 1` (steps `√|Qᵢ₊₁ − Qᵢ|`, normalized),
//!   knots at the parameters (`[0,0,0,0, u₁ … uₙ₋₂, 1,1,1,1]`) and natural
//!   end conditions (`C″ = 0` at both ends): `n + 2` control points, the
//!   solution of one linear system. The control points are therefore a
//!   function of the fit points, nonlinear through the parameters.
//!
//! Everything is generic over [`Scalar`], which `f64` and [`Dv`] implement:
//! `Dv` carries partials, so the same code that builds and evaluates the
//! curve differentiates it exactly (forward mode, as `constraints/ellipse.rs`
//! does with a fixed size), through the parameters, the knots and the
//! linear solve. [`BSpline`] is the plain `f64` curve.

use std::ops::{Add, Div, Mul, Neg, Sub};

use nalgebra::DMatrix;

use crate::constraints::{Var, xy};
use crate::geometry::Spline;

/// Added (squared) to each squared chord in the centripetal parameters:
/// `Δᵢ = (|Qᵢ₊₁ − Qᵢ|² + ε²)^¼`, so coincident fit points give a tiny step
/// (1e-6) rather than a repeated knot (a singular system), and `Δ` stays
/// differentiable at 0. Its effect is relative `ε²/|d|²`, nil at sketch sizes.
const CHORD_EPS_SQ: f64 = 1e-24;

/// Below this squared length a vector has no direction.
const DEGENERATE_SQ: f64 = 1e-24;

/// The highest degree: a Spline is a cubic.
pub const DEGREE: usize = 3;

/// A number the curve code runs on: `f64`, or [`Dv`] with partials.
pub trait Scalar:
    Clone
    + std::fmt::Debug
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
{
    /// A constant.
    fn cst(v: f64) -> Self;
    /// Its value.
    fn val(&self) -> f64;
    /// √self (0, with zero partials, at or below 0).
    fn sqrt(&self) -> Self;
}

impl Scalar for f64 {
    fn cst(v: f64) -> Self {
        v
    }
    fn val(&self) -> f64 {
        *self
    }
    fn sqrt(&self) -> Self {
        if *self <= 0.0 { 0.0 } else { f64::sqrt(*self) }
    }
}

/// A value with its partials w.r.t. a kernel's local variables (`g[k]` is
/// ∂/∂`x[k]`; missing trailing entries are 0, so a constant has none).
#[derive(Debug, Clone, PartialEq)]
pub struct Dv {
    /// The value.
    pub v: f64,
    /// Its partials.
    pub g: Vec<f64>,
}

impl Dv {
    /// Local variable `i`: value `x[i]`, partial 1 w.r.t. itself.
    pub fn var(x: &[f64], i: usize) -> Self {
        let mut g = vec![0.0; i + 1];
        g[i] = 1.0;
        Dv { v: x[i], g }
    }

    fn zip(&self, o: &Self, f: impl Fn(f64, f64) -> f64) -> Vec<f64> {
        let n = self.g.len().max(o.g.len());
        (0..n)
            .map(|k| f(self.g.get(k).copied().unwrap_or(0.0), o.g.get(k).copied().unwrap_or(0.0)))
            .collect()
    }

    fn map(&self, v: f64, dv: f64) -> Self {
        Dv { v, g: self.g.iter().map(|g| g * dv).collect() }
    }

    /// The value `v` whose partials w.r.t. `inputs` are `partials`, by the
    /// chain rule: brings a result computed in the fixed-size forward mode
    /// of `constraints/ellipse.rs` (`D<N>`) or as a `segment.rs` gradient,
    /// over these inputs, into `Dv`, so the spline kernels reuse those
    /// helpers rather than re-deriving them.
    pub(crate) fn chain<const N: usize>(v: f64, partials: &[f64; N], inputs: &[Dv; N]) -> Dv {
        let len = inputs.iter().map(|d| d.g.len()).max().unwrap_or(0);
        let mut g = vec![0.0; len];
        for (p, input) in partials.iter().zip(inputs) {
            for (gk, ik) in g.iter_mut().zip(&input.g) {
                *gk += p * ik;
            }
        }
        Dv { v, g }
    }

    /// Writes the partials into row `row` of a local Jacobian.
    pub fn write_row(&self, j: &mut DMatrix<f64>, row: usize) {
        for (k, &g) in self.g.iter().enumerate() {
            j[(row, k)] = g;
        }
    }

    /// atan2(self, x), the angle of (x, self); zero partials at the origin.
    pub fn atan2(&self, x: &Self) -> Self {
        let y = self;
        let sq = x.v * x.v + y.v * y.v;
        let v = y.v.atan2(x.v);
        if sq < DEGENERATE_SQ {
            return Dv::cst(v);
        }
        Dv { v, g: y.zip(x, |dy, dx| (x.v * dy - y.v * dx) / sq) }
    }
}

impl Add for Dv {
    type Output = Self;
    fn add(self, o: Self) -> Self {
        Dv { v: self.v + o.v, g: self.zip(&o, |a, b| a + b) }
    }
}

impl Sub for Dv {
    type Output = Self;
    fn sub(self, o: Self) -> Self {
        Dv { v: self.v - o.v, g: self.zip(&o, |a, b| a - b) }
    }
}

impl Neg for Dv {
    type Output = Self;
    fn neg(self) -> Self {
        self.map(-self.v, -1.0)
    }
}

impl Mul for Dv {
    type Output = Self;
    /// (st)' = s't + st'
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn mul(self, o: Self) -> Self {
        Dv { v: self.v * o.v, g: self.zip(&o, |a, b| a * o.v + self.v * b) }
    }
}

impl Div for Dv {
    type Output = Self;
    /// (s/t)' = s'/t − s·t'/t²
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn div(self, o: Self) -> Self {
        let t2 = o.v * o.v;
        Dv { v: self.v / o.v, g: self.zip(&o, |a, b| a / o.v - self.v * b / t2) }
    }
}

impl Scalar for Dv {
    fn cst(v: f64) -> Self {
        Dv { v, g: Vec::new() }
    }
    fn val(&self) -> f64 {
        self.v
    }
    fn sqrt(&self) -> Self {
        if self.v <= 0.0 {
            return Dv::cst(0.0);
        }
        let s = self.v.sqrt();
        self.map(s, 0.5 / s)
    }
}

/// A 2-D vector of [`Scalar`]s.
#[derive(Debug, Clone)]
pub struct P2<S> {
    /// x.
    pub x: S,
    /// y.
    pub y: S,
}

impl<S: Scalar> P2<S> {
    /// The vector (x, y).
    pub fn new(x: S, y: S) -> Self {
        P2 { x, y }
    }
    /// self + o.
    pub fn add(&self, o: &Self) -> Self {
        P2::new(self.x.clone() + o.x.clone(), self.y.clone() + o.y.clone())
    }
    /// self − o.
    pub fn sub(&self, o: &Self) -> Self {
        P2::new(self.x.clone() - o.x.clone(), self.y.clone() - o.y.clone())
    }
    /// k·self.
    pub fn scale(&self, k: &S) -> Self {
        P2::new(self.x.clone() * k.clone(), self.y.clone() * k.clone())
    }
    /// self · o.
    pub fn dot(&self, o: &Self) -> S {
        self.x.clone() * o.x.clone() + self.y.clone() * o.y.clone()
    }
    /// self × o = x·o.y − y·o.x.
    pub fn cross(&self, o: &Self) -> S {
        self.x.clone() * o.y.clone() - self.y.clone() * o.x.clone()
    }
    /// |self| (0, with zero partials, for a near-zero vector).
    pub fn norm(&self) -> S {
        let sq = self.dot(self);
        if sq.val() < DEGENERATE_SQ {
            return S::cst(sq.val().max(0.0).sqrt());
        }
        sq.sqrt()
    }
    /// self / |self|; the constant +X for a near-zero vector.
    pub fn unit(&self) -> Self {
        let len = self.norm();
        if len.val() * len.val() < DEGENERATE_SQ {
            return P2::new(S::cst(1.0), S::cst(0.0));
        }
        P2::new(self.x.clone() / len.clone(), self.y.clone() / len)
    }
    /// self rotated 90° counter-clockwise.
    pub fn rot90(&self) -> Self {
        P2::new(-self.y.clone(), self.x.clone())
    }
    /// The values.
    pub fn val(&self) -> [f64; 2] {
        [self.x.val(), self.y.val()]
    }
}

/// A clamped B-spline over [`Scalar`]s.
#[derive(Debug, Clone)]
pub struct Curve<S> {
    /// Its degree, 3.
    pub degree: usize,
    /// Knot vector, `control.len() + degree + 1` long, clamped.
    pub knots: Vec<S>,
    /// Control points.
    pub control: Vec<P2<S>>,
}

/// Why a spline's handles or knots don't make a curve.
pub fn check(handles: usize, interpolated: bool, knots: Option<&[f64]>) -> Result<(), String> {
    if handles < 2 {
        return Err(format!("a spline needs at least 2 points, got {handles}"));
    }
    // A cubic has at least 4 control points; a fit-point spline has n + 2.
    if !interpolated && handles <= DEGREE {
        return Err(format!(
            "a control-point spline needs at least {} control points (it is a cubic), got {handles}",
            DEGREE + 1
        ));
    }
    let Some(knots) = knots else {
        return Ok(());
    };
    if interpolated {
        return Err("'knots' applies only to a control-point spline (interpolated: false)".into());
    }
    let p = DEGREE;
    if knots.len() != handles + p + 1 {
        return Err(format!(
            "a degree-{p} spline with {handles} control points needs {} knots, got {}",
            handles + p + 1,
            knots.len()
        ));
    }
    if knots.iter().any(|k| !k.is_finite()) || knots.windows(2).any(|w| w[1] < w[0]) {
        return Err("knots must be finite and non-decreasing".into());
    }
    let last = knots.len() - 1;
    if knots[..=p].iter().any(|&k| k != knots[0]) || knots[last - p..].iter().any(|&k| k != knots[last]) {
        return Err(format!("knots must be clamped (the first and last {} equal)", p + 1));
    }
    if knots[0] >= knots[last] {
        return Err("knots span no interval".into());
    }
    if knots[p + 1..last - p].iter().any(|&k| knots.iter().filter(|&&o| o == k).count() > p) {
        return Err(format!("an interior knot repeats more than {p} times"));
    }
    Ok(())
}

impl<S: Scalar> Curve<S> {
    /// The curve with these handles: its fit points when `interpolated`,
    /// else its control points over `knots` (clamped uniform when `None`).
    /// The handles and knots must pass [`check`].
    pub fn new(handles: Vec<P2<S>>, interpolated: bool, knots: Option<&[f64]>) -> Self {
        if interpolated {
            return Self::interpolate(&handles);
        }
        let n = handles.len();
        let degree = DEGREE;
        let knots = match knots {
            Some(k) => k.iter().map(|&k| S::cst(k)).collect(),
            None => {
                let spans = n - degree;
                let mut k = vec![S::cst(0.0); degree + 1];
                k.extend((1..spans).map(|i| S::cst(i as f64 / spans as f64)));
                k.extend(std::iter::repeat_n(S::cst(1.0), degree + 1));
                k
            }
        };
        Curve { degree, knots, control: handles }
    }

    /// The length of its control polygon: a size for weighing an angle
    /// against distances.
    pub fn polygon_length(&self) -> S {
        self.control.windows(2).map(|w| w[1].sub(&w[0]).norm()).fold(S::cst(0.0), |a, b| a + b)
    }

    /// Its parameter range `[first knot, last knot]`.
    pub fn domain(&self) -> (f64, f64) {
        (self.knots[0].val(), self.knots[self.knots.len() - 1].val())
    }

    /// The knot span `i` (`kᵢ ≤ t < kᵢ₊₁`, non-empty) whose polynomial
    /// gives the curve at `t`; the first or last one outside the domain,
    /// whose polynomial continues the curve past its ends.
    fn span(&self, t: f64) -> usize {
        let (p, n) = (self.degree, self.control.len());
        let k = |i: usize| self.knots[i].val();
        if t >= k(n) {
            return n - 1;
        }
        if t <= k(p) {
            return p;
        }
        let (mut lo, mut hi) = (p, n);
        while hi - lo > 1 {
            let mid = (lo + hi) / 2;
            if t < k(mid) { hi = mid } else { lo = mid }
        }
        lo
    }

    /// The `degree + 1` basis functions non-zero on span `i` at `t`, and their
    /// derivatives up to order `nd` (Piegl & Tiller, *The NURBS Book*,
    /// A2.3): `ders[k][j]` is the k-th derivative of basis `i − degree + j`.
    /// Orders above the degree are 0.
    fn basis(&self, i: usize, t: &S, nd: usize) -> Vec<Vec<S>> {
        let p = self.degree;
        let u = &self.knots;
        let zero = || S::cst(0.0);
        let mut ndu = vec![vec![zero(); p + 1]; p + 1];
        ndu[0][0] = S::cst(1.0);
        let mut left = vec![zero(); p + 1];
        let mut right = vec![zero(); p + 1];
        for j in 1..=p {
            left[j] = t.clone() - u[i + 1 - j].clone();
            right[j] = u[i + j].clone() - t.clone();
            let mut saved = zero();
            for r in 0..j {
                ndu[j][r] = right[r + 1].clone() + left[j - r].clone();
                let temp = ndu[r][j - 1].clone() / ndu[j][r].clone();
                ndu[r][j] = saved + right[r + 1].clone() * temp.clone();
                saved = left[j - r].clone() * temp;
            }
            ndu[j][j] = saved;
        }
        let mut ders = vec![vec![zero(); p + 1]; nd + 1];
        for j in 0..=p {
            ders[0][j] = ndu[j][p].clone();
        }
        let n = nd.min(p);
        for r in 0..=p {
            let (mut s1, mut s2) = (0, 1);
            let mut a = vec![vec![zero(); p + 1]; 2];
            a[0][0] = S::cst(1.0);
            for k in 1..=n {
                let mut d = zero();
                let rk = r as isize - k as isize;
                let pk = p - k;
                if r >= k {
                    let rk = rk as usize;
                    a[s2][0] = a[s1][0].clone() / ndu[pk + 1][rk].clone();
                    d = a[s2][0].clone() * ndu[rk][pk].clone();
                }
                let j1 = if rk >= -1 { 1 } else { (-rk) as usize };
                let j2 = if r as isize - 1 <= pk as isize { k - 1 } else { p - r };
                for j in j1..=j2 {
                    let rkj = (rk + j as isize) as usize;
                    a[s2][j] = (a[s1][j].clone() - a[s1][j - 1].clone()) / ndu[pk + 1][rkj].clone();
                    d = d + a[s2][j].clone() * ndu[rkj][pk].clone();
                }
                if r <= pk {
                    a[s2][k] = -a[s1][k - 1].clone() / ndu[pk + 1][r].clone();
                    d = d + a[s2][k].clone() * ndu[r][pk].clone();
                }
                ders[k][r] = d;
                std::mem::swap(&mut s1, &mut s2);
            }
        }
        let mut factor = p as f64;
        for (k, row) in ders.iter_mut().enumerate().skip(1).take(n) {
            for d in row.iter_mut() {
                *d = d.clone() * S::cst(factor);
            }
            factor *= (p - k) as f64;
        }
        ders
    }

    /// The point and its derivatives w.r.t. the parameter up to order `nd`
    /// at `t` (`[C, C′, C″, …]`). Outside the domain the curve continues
    /// along its end tangent, `C(t₁) + (t − t₁)·C′(t₁)` (C¹ at the ends),
    /// so a contact parameter a solve step carries past an end lands on a
    /// gently varying curve while its span row brings it back. (The end
    /// spans' cubics continued instead swing away fast enough to stall
    /// Dog-Leg from a poor start.)
    pub fn eval(&self, t: &S, nd: usize) -> Vec<P2<S>> {
        let (lo, hi) = (&self.knots[0], &self.knots[self.knots.len() - 1]);
        let end = if t.val() < lo.val() {
            Some(lo)
        } else if t.val() > hi.val() {
            Some(hi)
        } else {
            None
        };
        if let Some(end) = end {
            let at = self.eval_span(end, 1);
            let dt = t.clone() - end.clone();
            let mut out = vec![at[0].add(&at[1].scale(&dt)), at[1].clone()];
            out.extend((2..=nd).map(|_| P2::new(S::cst(0.0), S::cst(0.0))));
            out.truncate(nd + 1);
            return out;
        }
        self.eval_span(t, nd)
    }

    /// [`Self::eval`] by the span's polynomial (inside the domain).
    fn eval_span(&self, t: &S, nd: usize) -> Vec<P2<S>> {
        let p = self.degree;
        let i = self.span(t.val());
        let ders = self.basis(i, t, nd);
        ders.iter()
            .map(|row| {
                let mut c = P2::new(S::cst(0.0), S::cst(0.0));
                for (j, n) in row.iter().enumerate() {
                    c = c.add(&self.control[i - p + j].scale(n));
                }
                c
            })
            .collect()
    }

    /// The natural cubic through `q` at centripetal parameters (see the
    /// module docs).
    fn interpolate(q: &[P2<S>]) -> Self {
        let n = q.len();
        let steps: Vec<S> = q
            .windows(2)
            .map(|w| {
                let d = w[1].sub(&w[0]);
                (d.dot(&d) + S::cst(CHORD_EPS_SQ)).sqrt().sqrt()
            })
            .collect();
        let total = steps.iter().cloned().fold(S::cst(0.0), |a, b| a + b);
        let mut u = vec![S::cst(0.0)];
        let mut sum = S::cst(0.0);
        for s in &steps[..n - 2] {
            sum = sum + s.clone();
            u.push(sum.clone() / total.clone());
        }
        u.push(S::cst(1.0));

        let mut knots = vec![S::cst(0.0); DEGREE + 1];
        knots.extend(u[1..n - 1].iter().cloned());
        knots.extend(std::iter::repeat_n(S::cst(1.0), DEGREE + 1));
        let m = n + 2;
        let mut curve = Curve {
            degree: DEGREE,
            knots,
            control: vec![P2::new(S::cst(0.0), S::cst(0.0)); m],
        };

        // Rows: C″(0) = 0, C(uᵢ) = Qᵢ, C″(1) = 0.
        let mut a = vec![vec![S::cst(0.0); m]; m];
        let mut rhs = vec![P2::new(S::cst(0.0), S::cst(0.0)); m];
        let fill = |row: usize, t: &S, order: usize, a: &mut Vec<Vec<S>>| {
            let i = curve.span(t.val());
            let ders = curve.basis(i, t, order);
            for (j, v) in ders[order].iter().enumerate() {
                a[row][i - DEGREE + j] = v.clone();
            }
        };
        fill(0, &u[0], 2, &mut a);
        for (k, uk) in u.iter().enumerate() {
            fill(k + 1, uk, 0, &mut a);
            rhs[k + 1] = q[k].clone();
        }
        fill(m - 1, &u[n - 1], 2, &mut a);
        curve.control = solve(a, rhs);
        curve
    }

    /// Parameters `count` evenly spaced over each non-empty knot span (and
    /// the domain's end), to seed searches for a closest point or contact.
    pub fn samples(&self, per_span: usize) -> Vec<f64> {
        let k: Vec<f64> = self.knots.iter().map(Scalar::val).collect();
        let mut out = Vec::new();
        for w in k.windows(2).filter(|w| w[1] > w[0]) {
            out.extend((0..per_span).map(|s| w[0] + (w[1] - w[0]) * s as f64 / per_span as f64));
        }
        out.push(self.domain().1);
        out
    }
}

/// Solves `a·x = rhs` (2 right-hand sides, as points) by Gaussian
/// elimination with partial pivoting (pivots chosen by value).
fn solve<S: Scalar>(mut a: Vec<Vec<S>>, mut rhs: Vec<P2<S>>) -> Vec<P2<S>> {
    let m = a.len();
    for col in 0..m {
        let pivot = (col..m)
            .max_by(|&i, &j| a[i][col].val().abs().total_cmp(&a[j][col].val().abs()))
            .unwrap();
        a.swap(col, pivot);
        rhs.swap(col, pivot);
        for row in col + 1..m {
            if a[row][col].val() == 0.0 {
                continue;
            }
            let f = a[row][col].clone() / a[col][col].clone();
            let (above, below) = a.split_at_mut(row);
            for (target, pivot) in below[0][col..].iter_mut().zip(&above[col][col..]) {
                *target = target.clone() - f.clone() * pivot.clone();
            }
            rhs[row] = rhs[row].sub(&rhs[col].scale(&f));
        }
    }
    let mut x = vec![P2::new(S::cst(0.0), S::cst(0.0)); m];
    for row in (0..m).rev() {
        let mut s = rhs[row].clone();
        for k in row + 1..m {
            s = s.sub(&x[k].scale(&a[row][k]));
        }
        let inv = S::cst(1.0) / a[row][row].clone();
        x[row] = s.scale(&inv);
    }
    x
}

/// A Spline's curve in plain numbers: degree, knots and control points, as
/// the JSON response's `curve` reports it. Equal data in either handle form
/// gives the same curve: a fit-point spline's `control_points` and `knots`,
/// given back as a control-point spline, describe it exactly.
#[derive(Debug, Clone, PartialEq)]
pub struct BSpline {
    /// Its degree.
    pub degree: usize,
    /// Its clamped knot vector.
    pub knots: Vec<f64>,
    /// Its control points.
    pub control_points: Vec<[f64; 2]>,
}

impl BSpline {
    /// The curve through these fit points (at least 2).
    pub fn from_fit_points(points: &[[f64; 2]]) -> Result<Self, String> {
        Self::new(points, true, None)
    }

    /// The curve of these control points (at least 2) over `knots`
    /// (clamped uniform when `None`).
    pub fn from_control_points(points: &[[f64; 2]], knots: Option<&[f64]>) -> Result<Self, String> {
        Self::new(points, false, knots)
    }

    /// The curve of a Spline's handles (see [`Curve::new`]).
    pub fn new(handles: &[[f64; 2]], interpolated: bool, knots: Option<&[f64]>) -> Result<Self, String> {
        check(handles.len(), interpolated, knots)?;
        let handles = handles.iter().map(|&[x, y]| P2::new(x, y)).collect();
        Ok(Curve::new(handles, interpolated, knots).into())
    }

    /// Its parameter range.
    pub fn domain(&self) -> (f64, f64) {
        (self.knots[0], self.knots[self.knots.len() - 1])
    }

    /// The point at parameter `t`.
    pub fn point(&self, t: f64) -> [f64; 2] {
        self.curve().eval(&t, 0)[0].val()
    }

    /// The derivative w.r.t. the parameter at `t`.
    pub fn derivative(&self, t: f64) -> [f64; 2] {
        self.curve().eval(&t, 1)[1].val()
    }

    fn curve(&self) -> Curve<f64> {
        Curve {
            degree: self.degree,
            knots: self.knots.clone(),
            control: self.control_points.iter().map(|&[x, y]| P2::new(x, y)).collect(),
        }
    }
}

impl From<Curve<f64>> for BSpline {
    fn from(c: Curve<f64>) -> Self {
        BSpline {
            degree: c.degree,
            knots: c.knots,
            control_points: c.control.iter().map(P2::val).collect(),
        }
    }
}

/// The parameter in `[lo, hi]` of the point of `curve` nearest `p`: the best
/// of a sampling, refined by Newton on `(C(t) − p)·C′(t) = 0`.
pub fn closest_param(curve: &Curve<f64>, p: [f64; 2]) -> f64 {
    let (lo, hi) = curve.domain();
    let dist = |t: f64| {
        let c = curve.eval(&t, 0)[0].val();
        (c[0] - p[0]).hypot(c[1] - p[1])
    };
    let mut t = curve
        .samples(16)
        .into_iter()
        .min_by(|&a, &b| dist(a).total_cmp(&dist(b)))
        .unwrap_or(lo);
    for _ in 0..30 {
        let d = curve.eval(&t, 2);
        let (c, c1, c2) = (d[0].val(), d[1].val(), d[2].val());
        let w = [c[0] - p[0], c[1] - p[1]];
        let f = w[0] * c1[0] + w[1] * c1[1];
        let df = c1[0] * c1[0] + c1[1] * c1[1] + w[0] * c2[0] + w[1] * c2[1];
        if df <= 0.0 {
            break;
        }
        let next = (t - f / df).clamp(lo, hi);
        if (next - t).abs() < 1e-15 {
            t = next;
            break;
        }
        t = next;
    }
    t
}

/// How far `t` lies outside `[lo, hi]` (0 inside), with partials: a
/// Spline's span, as an Arc's span overshoot keeps a point on the arc.
pub fn overshoot(t: &Dv, lo: f64, hi: f64) -> Dv {
    if t.v < lo {
        t.clone() - Dv::cst(lo)
    } else if t.v > hi {
        t.clone() - Dv::cst(hi)
    } else {
        Dv::cst(0.0)
    }
}

// ── Kernel plumbing: a Spline's curve from a kernel's local variables ──


/// The x, y of each of `spline`'s handles: what a kernel reads for its curve.
pub(crate) fn handle_vars(spline: &Spline) -> Vec<Var<'_>> {
    spline.points.iter().flat_map(|p| xy(p)).collect()
}

/// The Point whose x and y are local variables `i` and `i + 1`.
pub(crate) fn point_d(x: &[f64], i: usize) -> P2<Dv> {
    P2::new(Dv::var(x, i), Dv::var(x, i + 1))
}

/// `spline`'s curve with partials, its handles at local variables
/// `offset..offset + 2n`.
pub(crate) fn curve_d(spline: &Spline, x: &[f64], offset: usize) -> Curve<Dv> {
    let handles = (0..spline.points.len()).map(|k| point_d(x, offset + 2 * k)).collect();
    Curve::new(handles, spline.interpolated, spline.knots.as_deref())
}

/// `spline`'s curve in plain numbers (see [`curve_d`]).
pub(crate) fn curve_f(spline: &Spline, x: &[f64], offset: usize) -> Curve<f64> {
    let handles = (0..spline.points.len())
        .map(|k| P2::new(x[offset + 2 * k], x[offset + 2 * k + 1]))
        .collect();
    Curve::new(handles, spline.interpolated, spline.knots.as_deref())
}

/// The point and unit tangent of `curve` at `t`, in plain numbers.
pub(crate) fn point_tangent(curve: &Curve<f64>, t: f64) -> ([f64; 2], [f64; 2]) {
    let d = curve.eval(&t, 1);
    (d[0].val(), d[1].unit().val())
}

/// The angle from a curve's unit tangent τ̂ to the unit direction g, modulo
/// π, in (−π/2, π/2]: 0 when g runs along the tangent either way. Its
/// derivative is 1 everywhere but the jump at perpendicular (`sin` of it, or
/// `n̂·g`, has a zero derivative there, a false minimum a solve stalls in).
pub(crate) fn line_angle(g: &P2<Dv>, tau: &P2<Dv>) -> Dv {
    let (along, across) = (tau.dot(g), tau.cross(g));
    // atan2 of the direction folded into the right half-plane.
    if along.v < 0.0 { (-across).atan2(&(-along)) } else { across.atan2(&along) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolates_its_fit_points() {
        let q = [[0.0, 0.0], [1.0, 2.0], [3.0, 1.5], [4.0, -1.0], [6.0, 0.5]];
        let s = BSpline::from_fit_points(&q).unwrap();
        assert_eq!(s.control_points.len(), q.len() + 2);
        let mut hits = 0;
        for &p in &q {
            let t = closest_param(&s.curve(), p);
            let c = s.point(t);
            assert!((c[0] - p[0]).hypot(c[1] - p[1]) < 1e-12, "{c:?} vs {p:?}");
            hits += 1;
        }
        assert_eq!(hits, q.len());
    }

    #[test]
    fn two_fit_points_make_a_straight_line() {
        let s = BSpline::from_fit_points(&[[1.0, 1.0], [5.0, 3.0]]).unwrap();
        for t in [0.1, 0.5, 0.8] {
            let c = s.point(t);
            assert!(((c[0] - 1.0) * 2.0 - (c[1] - 1.0) * 4.0).abs() < 1e-12);
        }
    }

    #[test]
    fn derivatives_match_differences() {
        let s = BSpline::from_fit_points(&[[0.0, 0.0], [1.0, 2.0], [3.0, 1.5], [4.0, -1.0]]).unwrap();
        let c = s.curve();
        for t in [-0.1, 0.05, 0.3, 0.77, 0.999, 1.1] {
            let d = c.eval(&t, 2);
            let h = 1e-6;
            let (a, b) = (c.eval(&(t + h), 1), c.eval(&(t - h), 1));
            for k in 0..2 {
                let fd1 = (a[0].val()[k] - b[0].val()[k]) / (2.0 * h);
                let fd2 = (a[1].val()[k] - b[1].val()[k]) / (2.0 * h);
                assert!((d[1].val()[k] - fd1).abs() < 1e-6, "t {t}");
                assert!((d[2].val()[k] - fd2).abs() < 1e-5, "t {t}");
            }
        }
    }
}
