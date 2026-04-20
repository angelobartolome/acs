#![allow(non_snake_case)]

use nalgebra::{DMatrix, DVector};

use crate::{ParameterManager, constraints::Constraint};

/// Constrains a point to lie on the circumference of a circle.
///
/// Entities:
///   - `point_id`        – the free point (x, y)
///   - `circle_center_id`– the circle's center point (cx, cy)
///   - `circle_id`       – the circle entity (radius r at param index 0)
///
/// Residual: (px − cx)² + (py − cy)² − r² = 0
pub struct PointOnCircleConstraint {
    pub point_id: String,
    pub circle_center_id: String,
    pub circle_id: String,
}

impl PointOnCircleConstraint {
    pub fn new(point_id: String, circle_center_id: String, circle_id: String) -> Self {
        Self {
            point_id,
            circle_center_id,
            circle_id,
        }
    }
}

impl Constraint for PointOnCircleConstraint {
    fn num_residuals(&self) -> usize {
        1
    }

    fn residual(&self, pm: &ParameterManager) -> DVector<f64> {
        let p = pm.get_parameters();
        let px = p[pm.get_global_index(&self.point_id, 0).expect("point.x")];
        let py = p[pm.get_global_index(&self.point_id, 1).expect("point.y")];
        let cx = p[pm.get_global_index(&self.circle_center_id, 0).expect("center.x")];
        let cy = p[pm.get_global_index(&self.circle_center_id, 1).expect("center.y")];
        let r = p[pm.get_global_index(&self.circle_id, 0).expect("circle.r")];

        let dx = px - cx;
        let dy = py - cy;
        DVector::from(vec![dx * dx + dy * dy - r * r])
    }

    fn jacobian(&self, pm: &ParameterManager) -> DMatrix<f64> {
        let n = pm.num_parameters();
        let mut J = DMatrix::<f64>::zeros(1, n);

        let p = pm.get_parameters();
        let i_px = pm.get_global_index(&self.point_id, 0).expect("point.x");
        let i_py = pm.get_global_index(&self.point_id, 1).expect("point.y");
        let i_cx = pm.get_global_index(&self.circle_center_id, 0).expect("center.x");
        let i_cy = pm.get_global_index(&self.circle_center_id, 1).expect("center.y");
        let i_r = pm.get_global_index(&self.circle_id, 0).expect("circle.r");

        let dx = p[i_px] - p[i_cx];
        let dy = p[i_py] - p[i_cy];
        let r = p[i_r];

        J[(0, i_px)] = 2.0 * dx;
        J[(0, i_py)] = 2.0 * dy;
        J[(0, i_cx)] = -2.0 * dx;
        J[(0, i_cy)] = -2.0 * dy;
        J[(0, i_r)] = -2.0 * r;

        J
    }
}
