//! Shared geometry for constraints on an Ellipse.
//!
//! An Ellipse is a center Point `c`, a focus Point `f` and its minor radius
//! `b` (`radmin`). Everything else is derived: the focal distance
//! `k = |f − c|`, the major radius `a = sqrt(b² + k²)`, the unit major
//! direction `u = (f − c) / k`, the unit minor direction `n = rot90(u)` and
//! the second focus `2c − f`.
//!
//! The quantities are computed in forward mode, like GCS's `DeriVector2`:
//! each [`D`] carries its value and its exact partials w.r.t. a kernel's `N`
//! local variables, and every operation applies the chain rule. A kernel's
//! Jacobian row is then the partials of its residual, analytical rather than
//! numeric (`tests/jacobian_fd_test.rs` checks them).

use std::ops::{Add, Div, Mul, Neg, Sub};

use nalgebra::DMatrix;

use crate::constraints::arc_span::span_overshoot;

/// Below this squared length a vector has no direction (and a length with no
/// usable partials).
const DEGENERATE_SQ: f64 = 1e-24;

/// A value with its partials w.r.t. `N` local variables.
#[derive(Debug, Clone, Copy)]
pub(crate) struct D<const N: usize> {
    pub v: f64,
    pub g: [f64; N],
}

impl<const N: usize> D<N> {
    /// Local variable `i`: value `x[i]`, partial 1 w.r.t. itself.
    pub fn var(x: &[f64], i: usize) -> Self {
        let mut g = [0.0; N];
        g[i] = 1.0;
        D { v: x[i], g }
    }

    /// A constant: all partials 0.
    pub fn cst(v: f64) -> Self {
        D { v, g: [0.0; N] }
    }

    fn map(self, v: f64, dv: f64) -> Self {
        D {
            v,
            g: self.g.map(|g| dv * g),
        }
    }

    /// √self; 0 with zero partials at (or below) 0.
    pub fn sqrt(self) -> Self {
        if self.v <= 0.0 {
            return D::cst(0.0);
        }
        let s = self.v.sqrt();
        self.map(s, 0.5 / s)
    }

    pub fn scale(self, k: f64) -> Self {
        self.map(k * self.v, k)
    }

    pub fn sin(self) -> Self {
        self.map(self.v.sin(), self.v.cos())
    }

    pub fn cos(self) -> Self {
        self.map(self.v.cos(), -self.v.sin())
    }

    /// atan2(self, x), the angle of (x, self): d = (x·dy − y·dx)/(x² + y²);
    /// zero partials at the origin, which has no angle.
    pub fn atan2(self, x: Self) -> Self {
        let y = self;
        let sq = x.v * x.v + y.v * y.v;
        let v = y.v.atan2(x.v);
        if sq < DEGENERATE_SQ {
            return D::cst(v);
        }
        let mut g = [0.0; N];
        for (k, g) in g.iter_mut().enumerate() {
            *g = (x.v * y.g[k] - y.v * x.g[k]) / sq;
        }
        D { v, g }
    }

    /// Writes the partials into row `row` of a local Jacobian.
    pub fn write_row(&self, j: &mut DMatrix<f64>, row: usize) {
        for (k, &g) in self.g.iter().enumerate() {
            j[(row, k)] = g;
        }
    }
}

impl<const N: usize> Add for D<N> {
    type Output = Self;
    fn add(self, o: Self) -> Self {
        let mut g = self.g;
        for (a, b) in g.iter_mut().zip(o.g) {
            *a += b;
        }
        D { v: self.v + o.v, g }
    }
}

impl<const N: usize> Sub for D<N> {
    type Output = Self;
    fn sub(self, o: Self) -> Self {
        self + (-o)
    }
}

impl<const N: usize> Neg for D<N> {
    type Output = Self;
    fn neg(self) -> Self {
        self.scale(-1.0)
    }
}

impl<const N: usize> Mul for D<N> {
    type Output = Self;
    /// (st)' = s't + st'
    fn mul(self, o: Self) -> Self {
        let mut g = [0.0; N];
        for (k, g) in g.iter_mut().enumerate() {
            *g = self.g[k] * o.v + self.v * o.g[k];
        }
        D { v: self.v * o.v, g }
    }
}

impl<const N: usize> Div for D<N> {
    type Output = Self;
    /// (s/t)' = s'/t − s·t'/t²
    fn div(self, o: Self) -> Self {
        let mut g = [0.0; N];
        for (k, g) in g.iter_mut().enumerate() {
            *g = self.g[k] / o.v - self.v * o.g[k] / (o.v * o.v);
        }
        D { v: self.v / o.v, g }
    }
}

/// A 2-D vector of [`D`]s.
#[derive(Debug, Clone, Copy)]
pub(crate) struct V2<const N: usize> {
    pub x: D<N>,
    pub y: D<N>,
}

impl<const N: usize> V2<N> {
    /// The Point whose x and y are local variables `i` and `i + 1`.
    pub fn point(x: &[f64], i: usize) -> Self {
        V2 {
            x: D::var(x, i),
            y: D::var(x, i + 1),
        }
    }

    pub fn add(self, o: Self) -> Self {
        V2 {
            x: self.x + o.x,
            y: self.y + o.y,
        }
    }

    pub fn sub(self, o: Self) -> Self {
        V2 {
            x: self.x - o.x,
            y: self.y - o.y,
        }
    }

    pub fn scale(self, k: D<N>) -> Self {
        V2 {
            x: self.x * k,
            y: self.y * k,
        }
    }

    pub fn dot(self, o: Self) -> D<N> {
        self.x * o.x + self.y * o.y
    }

    /// self × o = self.x·o.y − self.y·o.x
    pub fn cross(self, o: Self) -> D<N> {
        self.x * o.y - self.y * o.x
    }

    /// |self|; 0 with zero partials for a (near-)zero vector.
    pub fn norm(self) -> D<N> {
        let sq = self.dot(self);
        if sq.v < DEGENERATE_SQ {
            return D::cst(sq.v.max(0.0).sqrt());
        }
        sq.sqrt()
    }

    /// self / |self|; the constant +X direction for a (near-)zero vector,
    /// which has none.
    pub fn unit(self) -> Self {
        let len = self.norm();
        if len.v * len.v < DEGENERATE_SQ {
            return V2 {
                x: D::cst(1.0),
                y: D::cst(0.0),
            };
        }
        V2 {
            x: self.x / len,
            y: self.y / len,
        }
    }

    /// self rotated 90° counter-clockwise.
    pub fn rot90(self) -> Self {
        V2 {
            x: -self.y,
            y: self.x,
        }
    }
}

/// An Ellipse read from a kernel's local variables, with its derived
/// quantities.
pub(crate) struct EllipseFrame<const N: usize> {
    pub center: V2<N>,
    pub focus: V2<N>,
    /// Minor radius `b`.
    pub minor_radius: D<N>,
    /// `a = sqrt(b² + |f − c|²)`.
    pub major_radius: D<N>,
    /// Unit vector from the center towards the focus (the minor axis is
    /// its `rot90`).
    pub major_dir: V2<N>,
}

impl<const N: usize> EllipseFrame<N> {
    /// The ellipse whose center is at local variables `center, center + 1`,
    /// focus at `focus, focus + 1` and minor radius at `radmin`.
    pub fn new(x: &[f64], center: usize, focus: usize, radmin: usize) -> Self {
        let c = V2::point(x, center);
        let f = V2::point(x, focus);
        let b = D::var(x, radmin);
        let e = f.sub(c);
        EllipseFrame {
            center: c,
            focus: f,
            minor_radius: b,
            major_radius: (b * b + e.dot(e)).sqrt(),
            major_dir: e.unit(),
        }
    }

    /// The other focus, `2c − f`.
    pub fn focus2(&self) -> V2<N> {
        self.center.add(self.center).sub(self.focus)
    }

    /// Unit direction of the minor axis, `rot90` of the major one.
    pub fn minor_dir(&self) -> V2<N> {
        self.major_dir.rot90()
    }

    /// The point at parametric angle `t`: `c + a·cos t·u + b·sin t·n`.
    pub fn point_at(&self, t: D<N>) -> V2<N> {
        let along = self.major_dir.scale(self.major_radius * t.cos());
        let across = self.minor_dir().scale(self.minor_radius * t.sin());
        self.center.add(along).add(across)
    }

    /// The tangent at parametric angle `t` (the derivative of
    /// [`Self::point_at`], counter-clockwise): `−a·sin t·u + b·cos t·n`.
    pub fn tangent_at(&self, t: D<N>) -> V2<N> {
        let along = self.major_dir.scale(-(self.major_radius * t.sin()));
        let across = self.minor_dir().scale(self.minor_radius * t.cos());
        along.add(across)
    }

    /// The parametric angle of `p`'s direction from the center: the `t`
    /// with `p − c` along `(a·cos t, b·sin t)` in the axes' frame,
    /// `atan2(a·(d·n), b·(d·u))` with `d = p − c` (scaling both arguments by
    /// `a·b > 0` keeps a degenerate `b` finite). For a point on the ellipse,
    /// the `t` [`Self::point_at`] puts it at.
    pub fn param_angle(&self, p: V2<N>) -> D<N> {
        let d = p.sub(self.center);
        let across = self.major_radius * d.dot(self.minor_dir());
        let along = self.minor_radius * d.dot(self.major_dir);
        across.atan2(along)
    }
}

/// [`span_overshoot`] in forward mode: how far the angle `t` falls outside
/// the span from `start` to `end` (0 within it), with its partials.
pub(crate) fn span_overshoot_d<const N: usize>(t: D<N>, start: D<N>, end: D<N>) -> D<N> {
    let (v, [dt, ds, de]) = span_overshoot(t.v, start.v, end.v);
    let mut g = [0.0; N];
    for (k, g) in g.iter_mut().enumerate() {
        *g = dt * t.g[k] + ds * start.g[k] + de * end.g[k];
    }
    D { v, g }
}
