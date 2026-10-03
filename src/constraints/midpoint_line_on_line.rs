use nalgebra::DMatrix;

use crate::constraints::segment::segment_signed_distance;
use crate::constraints::{Constraint, Var, set_row, xy};

/// The midpoint of Line L1 (l1a, l1b) lies on Line L2 (the segment l2a–l2b).
///
/// R = segment_signed_distance(m, l2a, l2b), with m = (l1a + l1b) / 2.
pub struct MidpointOfLineOnLineConstraint {
    pub l1_a: String,
    pub l1_b: String,
    pub l2_a: String,
    pub l2_b: String,
}

impl MidpointOfLineOnLineConstraint {
    pub fn new(l1_a: String, l1_b: String, l2_a: String, l2_b: String) -> Self {
        Self {
            l1_a,
            l1_b,
            l2_a,
            l2_b,
        }
    }
}

impl Constraint for MidpointOfLineOnLineConstraint {
    fn vars(&self) -> Vec<Var<'_>> {
        [xy(&self.l1_a), xy(&self.l1_b), xy(&self.l2_a), xy(&self.l2_b)].concat()
    }

    fn num_residuals(&self) -> usize {
        1
    }

    fn eval(&self, x: &[f64], r: &mut [f64], j: &mut DMatrix<f64>) {
        let (mx, my) = ((x[0] + x[2]) / 2.0, (x[1] + x[3]) / 2.0);
        let (d, g) = segment_signed_distance(mx, my, x[4], x[5], x[6], x[7]);
        r[0] = d;
        // ∂m/∂l1a = ∂m/∂l1b = ½, so each L1 endpoint gets half the midpoint's
        // partial. L2's endpoints take their partials unchanged.
        let (hx, hy) = (0.5 * g[0], 0.5 * g[1]);
        set_row(j, 0, &[hx, hy, hx, hy, g[2], g[3], g[4], g[5]]);
    }
}
