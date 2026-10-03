use nalgebra::DMatrix;

use crate::constraints::ellipse::{EllipseFrame, V2};
use crate::constraints::{Constraint, EllipseAxis, Var, xy};

/// Constrains a point to be an endpoint (either one) of an ellipse's major or
/// minor axis.
///
/// Entities:
///   - `point_id`   – the point p
///   - `center_id`  – the ellipse's center c
///   - `focus_id`   – the ellipse's focus f
///   - `ellipse_id` – the ellipse (minor radius b)
///
/// With u the unit major direction (c → f), n = rot90(u), a the major radius
/// and d = p − c:
///   Major: R₀ = u × d (p is on the major axis),  R₁ = |d| − a
///   Minor: R₀ = u · d (p is on the minor axis),  R₁ = |d| − b
/// Together they hold p at c ± a·u (major) or c ± b·n (minor), whichever end
/// it is nearer: the solver keeps it at the end it starts by.
pub struct EllipseAxisPointConstraint {
    pub point_id: String,
    pub center_id: String,
    pub focus_id: String,
    pub ellipse_id: String,
    pub axis: EllipseAxis,
}

impl EllipseAxisPointConstraint {
    pub fn new(
        point_id: String,
        center_id: String,
        focus_id: String,
        ellipse_id: String,
        axis: EllipseAxis,
    ) -> Self {
        Self {
            point_id,
            center_id,
            focus_id,
            ellipse_id,
            axis,
        }
    }
}

impl Constraint for EllipseAxisPointConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [
            &xy(&self.point_id)[..],
            &xy(&self.center_id),
            &xy(&self.focus_id),
            &[Var::MinorRadius(&self.ellipse_id)],
        ]
        .concat()
    }

    fn num_residuals(&self) -> usize {
        2
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        // x = [px, py, cx, cy, fx, fy, b]
        let e = EllipseFrame::<7>::new(x, 2, 4, 6);
        let d = V2::point(x, 0).sub(e.center);
        let (off_axis, radius) = match self.axis {
            EllipseAxis::Major => (e.major_dir.cross(d), e.major_radius),
            EllipseAxis::Minor => (e.major_dir.dot(d), e.minor_radius),
        };
        let along = d.norm() - radius;
        for (row, res) in [off_axis, along].into_iter().enumerate() {
            r[row] = res.v;
            res.write_row(j, row);
        }
    }
}
