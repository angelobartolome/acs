use nalgebra::DMatrix;

use crate::constraints::segment::line_signed_distance;
use crate::constraints::{Constraint, Var, xy};

/// Two Lines lie on one infinite line: both endpoints of Line B lie on Line
/// A's Extension. The segments need not overlap.
///
/// Residuals: the signed perpendicular distance from each of B's endpoints
/// to A's Extension (see `segment::line_signed_distance`). When B shares an
/// endpoint with A, that residual is identically 0 (a zero Jacobian row),
/// and the constraint still removes one degree of freedom as a unit.
pub struct CollinearConstraint {
    pub a1: String, // ID of Line A's first point
    pub a2: String, // ID of Line A's second point
    pub b1: String, // ID of Line B's first point
    pub b2: String, // ID of Line B's second point
}

impl CollinearConstraint {
    pub fn new(a1: String, a2: String, b1: String, b2: String) -> Self {
        Self { a1, a2, b1, b2 }
    }
}

impl Constraint for CollinearConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [xy(&self.a1), xy(&self.a2), xy(&self.b1), xy(&self.b2)].concat()
    }

    fn num_residuals(&self) -> usize {
        2
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        let (ax, ay, bx, by) = (x[0], x[1], x[2], x[3]);
        for (row, p) in [4, 6].into_iter().enumerate() {
            // g is w.r.t. (px, py, ax, ay, bx, by).
            let (d, g) = line_signed_distance(x[p], x[p + 1], ax, ay, bx, by);
            r[row] = d;
            for k in 0..4 {
                j[(row, k)] = g[k + 2];
            }
            j[(row, p)] = g[0];
            j[(row, p + 1)] = g[1];
        }
    }
}
