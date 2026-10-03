use nalgebra::DMatrix;

use crate::constraints::segment::segment_signed_distance;
use crate::constraints::{Constraint, Var, set_row, xy};

/// Constrains a point to lie on a Line (the segment between its endpoints).
///
/// Residual: the signed distance from the point to the closest point of the
/// segment (see `segment::segment_signed_distance`).
pub struct PointOnLineConstraint {
    pub p1: String,       // ID of the point to constrain
    pub p_line_a: String, // ID of the line's point A
    pub p_line_b: String, // ID of the line's point B
}

impl PointOnLineConstraint {
    pub fn new(p1: String, p_line_a: String, p_line_b: String) -> Self {
        Self {
            p1,
            p_line_a,
            p_line_b,
        }
    }
}

impl Constraint for PointOnLineConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [xy(&self.p1), xy(&self.p_line_a), xy(&self.p_line_b)].concat()
    }

    fn num_residuals(&self) -> usize {
        1
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        let (d, g) = segment_signed_distance(x[0], x[1], x[2], x[3], x[4], x[5]);
        r[0] = d;
        set_row(j, 0, &g);
    }
}
