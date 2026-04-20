#![allow(non_snake_case)]

use nalgebra::{DMatrix, DVector};

use crate::{ParameterManager, constraints::Constraint};

/// Constrains a line to be tangent to a circle.
///
/// The perpendicular distance from the circle's center to the infinite line
/// through `pa`–`pb` equals the circle's radius.
///
/// Entities:
///   - `line_pa_id`       – first point of the line
///   - `line_pb_id`       – second point of the line
///   - `circle_center_id` – center point of the circle
///   - `circle_id`        – circle entity (radius at param 0)
///
/// Let dx = bx−ax, dy = by−ay, L² = dx²+dy²
/// area = dy·(cx−ax) − dx·(cy−ay)
/// Residual: area² − r²·L² = 0
pub struct TangentLineCircleConstraint {
    pub line_pa_id: String,
    pub line_pb_id: String,
    pub circle_center_id: String,
    pub circle_id: String,
}

impl TangentLineCircleConstraint {
    pub fn new(
        line_pa_id: String,
        line_pb_id: String,
        circle_center_id: String,
        circle_id: String,
    ) -> Self {
        Self {
            line_pa_id,
            line_pb_id,
            circle_center_id,
            circle_id,
        }
    }
}

impl Constraint for TangentLineCircleConstraint {
    fn num_residuals(&self) -> usize {
        1
    }

    fn residual(&self, pm: &ParameterManager) -> DVector<f64> {
        let p = pm.get_parameters();
        let ax = p[pm.get_global_index(&self.line_pa_id, 0).expect("pa.x")];
        let ay = p[pm.get_global_index(&self.line_pa_id, 1).expect("pa.y")];
        let bx = p[pm.get_global_index(&self.line_pb_id, 0).expect("pb.x")];
        let by = p[pm.get_global_index(&self.line_pb_id, 1).expect("pb.y")];
        let cx = p[pm.get_global_index(&self.circle_center_id, 0).expect("center.x")];
        let cy = p[pm.get_global_index(&self.circle_center_id, 1).expect("center.y")];
        let r = p[pm.get_global_index(&self.circle_id, 0).expect("circle.r")];

        let dx = bx - ax;
        let dy = by - ay;
        let l2 = dx * dx + dy * dy;
        let area = dy * (cx - ax) - dx * (cy - ay);

        DVector::from(vec![area * area - r * r * l2])
    }

    fn jacobian(&self, pm: &ParameterManager) -> DMatrix<f64> {
        let n = pm.num_parameters();
        let mut J = DMatrix::<f64>::zeros(1, n);

        let p = pm.get_parameters();
        let i_ax = pm.get_global_index(&self.line_pa_id, 0).expect("pa.x");
        let i_ay = pm.get_global_index(&self.line_pa_id, 1).expect("pa.y");
        let i_bx = pm.get_global_index(&self.line_pb_id, 0).expect("pb.x");
        let i_by = pm.get_global_index(&self.line_pb_id, 1).expect("pb.y");
        let i_cx = pm.get_global_index(&self.circle_center_id, 0).expect("center.x");
        let i_cy = pm.get_global_index(&self.circle_center_id, 1).expect("center.y");
        let i_r = pm.get_global_index(&self.circle_id, 0).expect("circle.r");

        let ax = p[i_ax];
        let ay = p[i_ay];
        let bx = p[i_bx];
        let by = p[i_by];
        let cx = p[i_cx];
        let cy = p[i_cy];
        let r = p[i_r];

        let dx = bx - ax;
        let dy = by - ay;
        let l2 = dx * dx + dy * dy;
        let area = dy * (cx - ax) - dx * (cy - ay);

        // R = area² − r²·L²
        // ∂R/∂param = 2·area·(∂area/∂param) − r²·(∂L²/∂param)
        //
        // ∂area/∂ax = −dy, ∂area/∂ay = dx, ∂area/∂bx = dy, ∂area/∂by = −dx
        // ∂area/∂cx = dy,  ∂area/∂cy = −dx
        // ∂L²/∂ax = −2dx, ∂L²/∂ay = −2dy, ∂L²/∂bx = 2dx, ∂L²/∂by = 2dy

        J[(0, i_ax)] = 2.0 * area * (-dy) - r * r * (-2.0 * dx);
        J[(0, i_ay)] = 2.0 * area * dx - r * r * (-2.0 * dy);
        J[(0, i_bx)] = 2.0 * area * dy - r * r * (2.0 * dx);
        J[(0, i_by)] = 2.0 * area * (-dx) - r * r * (2.0 * dy);
        J[(0, i_cx)] = 2.0 * area * dy;
        J[(0, i_cy)] = 2.0 * area * (-dx);
        J[(0, i_r)] = -2.0 * r * l2;

        J
    }
}
