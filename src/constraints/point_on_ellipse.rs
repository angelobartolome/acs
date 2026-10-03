use nalgebra::DMatrix;

use crate::constraints::ellipse::{EllipseFrame, V2};
use crate::constraints::{Constraint, Var, xy};

/// Constrains a point to lie on an ellipse.
///
/// Entities:
///   - `point_id`   – the point p
///   - `center_id`  – the ellipse's center c
///   - `focus_id`   – the ellipse's focus f
///   - `ellipse_id` – the ellipse (minor radius b)
///
/// Residual (as GCS's `ConstraintPointOnEllipse`): the sum of the distances
/// to the two foci is the major diameter,
///   |p − f| + |p − (2c − f)| − 2a = 0,   a = sqrt(b² + |f − c|²).
pub struct PointOnEllipseConstraint {
    pub point_id: String,
    pub center_id: String,
    pub focus_id: String,
    pub ellipse_id: String,
}

impl PointOnEllipseConstraint {
    pub fn new(point_id: String, center_id: String, focus_id: String, ellipse_id: String) -> Self {
        Self {
            point_id,
            center_id,
            focus_id,
            ellipse_id,
        }
    }
}

impl Constraint for PointOnEllipseConstraint {
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
        1
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        // x = [px, py, cx, cy, fx, fy, b]
        let e = EllipseFrame::<7>::new(x, 2, 4, 6);
        let p = V2::point(x, 0);
        let res = p.sub(e.focus).norm() + p.sub(e.focus2()).norm() - e.major_radius.scale(2.0);
        r[0] = res.v;
        res.write_row(j, 0);
    }
}
