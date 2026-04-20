#![allow(non_snake_case)]

use nalgebra::{DMatrix, DVector};

use crate::{ParameterManager, constraints::Constraint};

/// Constrains the perpendicular distance from a point to an infinite line.
///
/// Entities:
///   - `point_id`   – the free point p
///   - `line_pa_id` – first point of the line (a)
///   - `line_pb_id` – second point of the line (b)
///   - `distance`   – target perpendicular distance d
///
/// Let dx = bx−ax, dy = by−ay, L² = dx²+dy²
/// area = dy·(px−ax) − dx·(py−ay)   (signed area × 2 of triangle p,a,b)
/// Residual: area² − d²·L² = 0
pub struct DistancePointLineConstraint {
    pub point_id: String,
    pub line_pa_id: String,
    pub line_pb_id: String,
    pub distance: f64,
}

impl DistancePointLineConstraint {
    pub fn new(
        point_id: String,
        line_pa_id: String,
        line_pb_id: String,
        distance: f64,
    ) -> Self {
        Self {
            point_id,
            line_pa_id,
            line_pb_id,
            distance,
        }
    }
}

impl Constraint for DistancePointLineConstraint {
    fn num_residuals(&self) -> usize {
        1
    }

    fn residual(&self, pm: &ParameterManager) -> DVector<f64> {
        let p = pm.get_parameters();
        let px = p[pm.get_global_index(&self.point_id, 0).expect("point.x")];
        let py = p[pm.get_global_index(&self.point_id, 1).expect("point.y")];
        let ax = p[pm.get_global_index(&self.line_pa_id, 0).expect("pa.x")];
        let ay = p[pm.get_global_index(&self.line_pa_id, 1).expect("pa.y")];
        let bx = p[pm.get_global_index(&self.line_pb_id, 0).expect("pb.x")];
        let by = p[pm.get_global_index(&self.line_pb_id, 1).expect("pb.y")];

        let dx = bx - ax;
        let dy = by - ay;
        let l2 = dx * dx + dy * dy;
        let area = dy * (px - ax) - dx * (py - ay);
        let d2 = self.distance * self.distance;

        DVector::from(vec![area * area - d2 * l2])
    }

    fn jacobian(&self, pm: &ParameterManager) -> DMatrix<f64> {
        let n = pm.num_parameters();
        let mut J = DMatrix::<f64>::zeros(1, n);

        let p = pm.get_parameters();
        let i_px = pm.get_global_index(&self.point_id, 0).expect("point.x");
        let i_py = pm.get_global_index(&self.point_id, 1).expect("point.y");
        let i_ax = pm.get_global_index(&self.line_pa_id, 0).expect("pa.x");
        let i_ay = pm.get_global_index(&self.line_pa_id, 1).expect("pa.y");
        let i_bx = pm.get_global_index(&self.line_pb_id, 0).expect("pb.x");
        let i_by = pm.get_global_index(&self.line_pb_id, 1).expect("pb.y");

        let px = p[i_px];
        let py = p[i_py];
        let ax = p[i_ax];
        let ay = p[i_ay];
        let bx = p[i_bx];
        let by = p[i_by];

        let dx = bx - ax;
        let dy = by - ay;
        let area = dy * (px - ax) - dx * (py - ay);
        let d2 = self.distance * self.distance;

        // R = area² − d²·L²
        // ∂R/∂param = 2·area·(∂area/∂param) − d²·(∂L²/∂param)
        //
        // ∂area/∂px = dy,  ∂area/∂py = −dx
        // ∂area/∂ax = −dy, ∂area/∂ay = dx, ∂area/∂bx = dy, ∂area/∂by = −dx
        // ∂L²/∂ax  = −2dx, ∂L²/∂ay = −2dy, ∂L²/∂bx = 2dx, ∂L²/∂by = 2dy

        J[(0, i_px)] = 2.0 * area * dy;
        J[(0, i_py)] = 2.0 * area * (-dx);
        J[(0, i_ax)] = 2.0 * area * (-dy) - d2 * (-2.0 * dx);
        J[(0, i_ay)] = 2.0 * area * dx - d2 * (-2.0 * dy);
        J[(0, i_bx)] = 2.0 * area * dy - d2 * (2.0 * dx);
        J[(0, i_by)] = 2.0 * area * (-dx) - d2 * (2.0 * dy);

        J
    }
}
