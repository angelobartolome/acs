//! [`EllipseDiameterConstraint`]: constrains two points to be the two
//! endpoints of an ellipse's major or minor axis.

use nalgebra::DMatrix;

use crate::constraints::ellipse::{D, EllipseFrame, V2};
use crate::constraints::{Constraint, EllipseAxis, Var, xy};

/// Constrains two points to be the two endpoints of an ellipse's major or
/// minor axis: native two-point `ellipse_axis`, which an ellipse tool adds.
///
/// Entities:
///   - `p1_id`, `p2_id` – the two points
///   - `center_id`      – the ellipse's center c
///   - `focus_id`       – the ellipse's focus f
///   - `ellipse_id`     – the ellipse (minor radius b)
///
/// With u the unit major direction (c → f), a the major radius, m the
/// midpoint of p1 p2 and h = (p2 − p1) / 2:
///   R₀, R₁ = m − c                 (the points are symmetric about c)
///   R₂ = u × h (major) or u · h (minor)   (along the axis)
///   R₃ = |h| − a (major) or |h| − b (minor)
/// So {p1, p2} = {c + a·u, c − a·u} (major) or {c + b·n, c − b·n} (minor),
/// n = rot90(u). Unlike GCS, which assigns each point an end once (by which
/// is nearer when the constraint is added), the pair is unordered, so the
/// points keep whichever ends they're nearest.
pub struct EllipseDiameterConstraint {
    /// The two points.
    pub p1_id: String,
    /// The two points.
    pub p2_id: String,
    /// The ellipse's center c.
    pub center_id: String,
    /// The ellipse's focus f.
    pub focus_id: String,
    /// The ellipse (minor radius b).
    pub ellipse_id: String,
    /// Which axis the points are the ends of.
    pub axis: EllipseAxis,
}

impl EllipseDiameterConstraint {
    /// The constraint over these entities, by ID (see the fields).
    pub fn new(
        p1_id: String,
        p2_id: String,
        center_id: String,
        focus_id: String,
        ellipse_id: String,
        axis: EllipseAxis,
    ) -> Self {
        Self {
            p1_id,
            p2_id,
            center_id,
            focus_id,
            ellipse_id,
            axis,
        }
    }
}

impl Constraint for EllipseDiameterConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [
            &xy(&self.p1_id)[..],
            &xy(&self.p2_id),
            &xy(&self.center_id),
            &xy(&self.focus_id),
            &[Var::MinorRadius(&self.ellipse_id)],
        ]
        .concat()
    }

    fn num_residuals(&self) -> usize {
        4
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        // x = [p1x, p1y, p2x, p2y, cx, cy, fx, fy, b]
        let e = EllipseFrame::<9>::new(x, 4, 6, 8);
        let (p1, p2) = (V2::point(x, 0), V2::point(x, 2));
        let half = D::cst(0.5);
        let m = p1.add(p2).scale(half).sub(e.center);
        let h = p2.sub(p1).scale(half);
        let (off_axis, radius) = match self.axis {
            EllipseAxis::Major => (e.major_dir.cross(h), e.major_radius),
            EllipseAxis::Minor => (e.major_dir.dot(h), e.minor_radius),
        };
        let along = h.norm() - radius;
        for (row, res) in [m.x, m.y, off_axis, along].into_iter().enumerate() {
            r[row] = res.v;
            res.write_row(j, row);
        }
    }
}
