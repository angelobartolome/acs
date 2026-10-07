//! [`PointOnExtensionConstraint`]: constrains a point to lie on a Line's
//! Extension (the infinite line through its endpoints), so it may sit beyond
//! either end.

use nalgebra::DMatrix;

use crate::constraints::segment::line_signed_distance;
use crate::constraints::{Constraint, Var, set_row, xy};

/// Constrains a point to lie on a Line's Extension (the infinite line through
/// its endpoints), so it may sit beyond either end. The Extension variant of
/// `PointOnLineConstraint`.
///
/// Residual: the signed perpendicular distance from the point to the
/// Extension (see `segment::line_signed_distance`).
pub struct PointOnExtensionConstraint {
    /// The point.
    pub p1: String,
    /// The Line's start.
    pub p_line_a: String,
    /// The Line's end.
    pub p_line_b: String,
}

impl PointOnExtensionConstraint {
    /// The constraint over these entities, by ID (see the fields).
    pub fn new(p1: String, p_line_a: String, p_line_b: String) -> Self {
        Self {
            p1,
            p_line_a,
            p_line_b,
        }
    }
}

impl Constraint for PointOnExtensionConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [xy(&self.p1), xy(&self.p_line_a), xy(&self.p_line_b)].concat()
    }

    fn num_residuals(&self) -> usize {
        1
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        let (d, g) = line_signed_distance(x[0], x[1], x[2], x[3], x[4], x[5]);
        r[0] = d;
        set_row(j, 0, &g);
    }
}
