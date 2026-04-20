#![allow(non_snake_case)]

use nalgebra::{DMatrix, DVector};

use crate::{ParameterManager, constraints::Constraint};

/// Constrains the directed angle from line L1 to line L2 to a specific value.
///
/// Entities: four points (p1,p2,p3,p4) defining L1 = p1→p2 and L2 = p3→p4.
/// Target angle θ in radians.
///
/// Let d1 = (dx1, dy1) = (x2−x1, y2−y1)
///     d2 = (dx2, dy2) = (x4−x3, y4−y3)
///     dot   = dx1·dx2 + dy1·dy2
///     cross = dx1·dy2 − dy1·dx2
///
/// Residual: dot·sin(θ) − cross·cos(θ) = 0
///
/// This encodes atan2(cross, dot) = θ, i.e. the angle from d1 to d2 is θ.
pub struct AngleConstraint {
    pub p1: String,
    pub p2: String,
    pub p3: String,
    pub p4: String,
    pub angle: f64,
}

impl AngleConstraint {
    pub fn new(p1: String, p2: String, p3: String, p4: String, angle: f64) -> Self {
        Self {
            p1,
            p2,
            p3,
            p4,
            angle,
        }
    }
}

impl Constraint for AngleConstraint {
    fn num_residuals(&self) -> usize {
        1
    }

    fn residual(&self, pm: &ParameterManager) -> DVector<f64> {
        let p = pm.get_parameters();
        let x1 = p[pm.get_global_index(&self.p1, 0).expect("p1.x")];
        let y1 = p[pm.get_global_index(&self.p1, 1).expect("p1.y")];
        let x2 = p[pm.get_global_index(&self.p2, 0).expect("p2.x")];
        let y2 = p[pm.get_global_index(&self.p2, 1).expect("p2.y")];
        let x3 = p[pm.get_global_index(&self.p3, 0).expect("p3.x")];
        let y3 = p[pm.get_global_index(&self.p3, 1).expect("p3.y")];
        let x4 = p[pm.get_global_index(&self.p4, 0).expect("p4.x")];
        let y4 = p[pm.get_global_index(&self.p4, 1).expect("p4.y")];

        let dx1 = x2 - x1;
        let dy1 = y2 - y1;
        let dx2 = x4 - x3;
        let dy2 = y4 - y3;

        let dot = dx1 * dx2 + dy1 * dy2;
        let cross = dx1 * dy2 - dy1 * dx2;

        let sin_t = self.angle.sin();
        let cos_t = self.angle.cos();

        DVector::from(vec![dot * sin_t - cross * cos_t])
    }

    fn jacobian(&self, pm: &ParameterManager) -> DMatrix<f64> {
        let n = pm.num_parameters();
        let mut J = DMatrix::<f64>::zeros(1, n);

        let p = pm.get_parameters();
        let i1x = pm.get_global_index(&self.p1, 0).expect("p1.x");
        let i1y = pm.get_global_index(&self.p1, 1).expect("p1.y");
        let i2x = pm.get_global_index(&self.p2, 0).expect("p2.x");
        let i2y = pm.get_global_index(&self.p2, 1).expect("p2.y");
        let i3x = pm.get_global_index(&self.p3, 0).expect("p3.x");
        let i3y = pm.get_global_index(&self.p3, 1).expect("p3.y");
        let i4x = pm.get_global_index(&self.p4, 0).expect("p4.x");
        let i4y = pm.get_global_index(&self.p4, 1).expect("p4.y");

        let dx1 = p[i2x] - p[i1x];
        let dy1 = p[i2y] - p[i1y];
        let dx2 = p[i4x] - p[i3x];
        let dy2 = p[i4y] - p[i3y];

        let sin_t = self.angle.sin();
        let cos_t = self.angle.cos();

        // R = dot·sin_t − cross·cos_t
        // ∂R/∂param = (∂dot/∂param)·sin_t − (∂cross/∂param)·cos_t
        //
        // For line 1 (dx1, dy1 depend on x1,y1,x2,y2):
        //   ∂dot/∂x1 = −dx2,  ∂cross/∂x1 = −dy2
        //   ∂dot/∂y1 = −dy2,  ∂cross/∂y1 =  dx2
        //   ∂dot/∂x2 =  dx2,  ∂cross/∂x2 =  dy2
        //   ∂dot/∂y2 =  dy2,  ∂cross/∂y2 = −dx2
        //
        // For line 2 (dx2, dy2 depend on x3,y3,x4,y4):
        //   ∂dot/∂x3 = −dx1,  ∂cross/∂x3 =  dy1
        //   ∂dot/∂y3 = −dy1,  ∂cross/∂y3 = −dx1
        //   ∂dot/∂x4 =  dx1,  ∂cross/∂x4 = −dy1
        //   ∂dot/∂y4 =  dy1,  ∂cross/∂y4 =  dx1

        J[(0, i1x)] = (-dx2) * sin_t - (-dy2) * cos_t;
        J[(0, i1y)] = (-dy2) * sin_t - (dx2) * cos_t;
        J[(0, i2x)] = (dx2) * sin_t - (dy2) * cos_t;
        J[(0, i2y)] = (dy2) * sin_t - (-dx2) * cos_t;
        J[(0, i3x)] = (-dx1) * sin_t - (dy1) * cos_t;
        J[(0, i3y)] = (-dy1) * sin_t - (-dx1) * cos_t;
        J[(0, i4x)] = (dx1) * sin_t - (-dy1) * cos_t;
        J[(0, i4y)] = (dy1) * sin_t - (dx1) * cos_t;

        J
    }
}
