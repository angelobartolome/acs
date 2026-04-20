#![allow(non_snake_case)]

use nalgebra::{DMatrix, DVector};

use crate::{ParameterManager, constraints::Constraint};

/// Midpoint of segment (l1a, l1b) lies on the infinite line through (l2a, l2b).
///
/// R = (mx − x₂ₐ)(y₂ᵦ − y₂ₐ) − (my − y₂ₐ)(x₂ᵦ − x₂ₐ) = 0
/// with mx = (x₁ₐ + x₁ᵦ)/2, my = (y₁ₐ + y₁ᵦ)/2.
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
    fn num_residuals(&self) -> usize {
        1
    }

    fn residual(&self, pm: &ParameterManager) -> DVector<f64> {
        let p = pm.get_parameters();
        let x1a = p[pm.get_global_index(&self.l1_a, 0).expect("l1a.x")];
        let y1a = p[pm.get_global_index(&self.l1_a, 1).expect("l1a.y")];
        let x1b = p[pm.get_global_index(&self.l1_b, 0).expect("l1b.x")];
        let y1b = p[pm.get_global_index(&self.l1_b, 1).expect("l1b.y")];
        let x2a = p[pm.get_global_index(&self.l2_a, 0).expect("l2a.x")];
        let y2a = p[pm.get_global_index(&self.l2_a, 1).expect("l2a.y")];
        let x2b = p[pm.get_global_index(&self.l2_b, 0).expect("l2b.x")];
        let y2b = p[pm.get_global_index(&self.l2_b, 1).expect("l2b.y")];

        let mx = (x1a + x1b) / 2.0;
        let my = (y1a + y1b) / 2.0;
        let dx2 = x2b - x2a;
        let dy2 = y2b - y2a;

        let r = (mx - x2a) * dy2 - (my - y2a) * dx2;
        DVector::from(vec![r])
    }

    fn jacobian(&self, pm: &ParameterManager) -> DMatrix<f64> {
        let n = pm.num_parameters();
        let mut j = DMatrix::<f64>::zeros(1, n);

        let p = pm.get_parameters();
        let x2a = p[pm.get_global_index(&self.l2_a, 0).expect("l2a.x")];
        let y2a = p[pm.get_global_index(&self.l2_a, 1).expect("l2a.y")];
        let x2b = p[pm.get_global_index(&self.l2_b, 0).expect("l2b.x")];
        let y2b = p[pm.get_global_index(&self.l2_b, 1).expect("l2b.y")];

        let dx2 = x2b - x2a;
        let dy2 = y2b - y2a;

        let i_x1a = pm.get_global_index(&self.l1_a, 0).expect("l1a.x");
        let i_y1a = pm.get_global_index(&self.l1_a, 1).expect("l1a.y");
        let i_x1b = pm.get_global_index(&self.l1_b, 0).expect("l1b.x");
        let i_y1b = pm.get_global_index(&self.l1_b, 1).expect("l1b.y");
        let i_x2a = pm.get_global_index(&self.l2_a, 0).expect("l2a.x");
        let i_y2a = pm.get_global_index(&self.l2_a, 1).expect("l2a.y");
        let i_x2b = pm.get_global_index(&self.l2_b, 0).expect("l2b.x");
        let i_y2b = pm.get_global_index(&self.l2_b, 1).expect("l2b.y");

        let x1a = p[i_x1a];
        let y1a = p[i_y1a];
        let x1b = p[i_x1b];
        let y1b = p[i_y1b];

        let mx = (x1a + x1b) / 2.0;
        let my = (y1a + y1b) / 2.0;

        j[(0, i_x1a)] = dy2 / 2.0;
        j[(0, i_y1a)] = -dx2 / 2.0;
        j[(0, i_x1b)] = dy2 / 2.0;
        j[(0, i_y1b)] = -dx2 / 2.0;

        j[(0, i_x2a)] = -dy2 + (my - y2a);
        j[(0, i_y2a)] = -(mx - x2a);
        j[(0, i_x2b)] = -(my - y2a);
        j[(0, i_y2b)] = mx - x2a;

        j
    }
}
