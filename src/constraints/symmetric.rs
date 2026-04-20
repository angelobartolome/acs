#![allow(non_snake_case)]

use nalgebra::{DMatrix, DVector};

use crate::{ParameterManager, constraints::Constraint};

/// Constrains two points to be symmetric (mirror images) across a line.
///
/// Entities:
///   - `p_id`       – first point p
///   - `q_id`       – second point q (mirror of p)
///   - `axis_pa_id` – first point of the symmetry axis
///   - `axis_pb_id` – second point of the symmetry axis
///
/// Conditions encoded as 2 residuals:
///
/// 1. The midpoint M = ((px+qx)/2, (py+qy)/2) lies on the axis line.
/// 2. The vector p→q is perpendicular to the axis.
///
/// Let dx = bx−ax, dy = by−ay (axis direction),
///     epx = (px+qx)/2 − ax,  epy = (py+qy)/2 − ay  (shifted midpoint)
///
/// R₀ = epy·dx − epx·dy = 0   (collinearity)
/// R₁ = (qx−px)·dx + (qy−py)·dy = 0   (perpendicularity)
pub struct SymmetricConstraint {
    pub p_id: String,
    pub q_id: String,
    pub axis_pa_id: String,
    pub axis_pb_id: String,
}

impl SymmetricConstraint {
    pub fn new(
        p_id: String,
        q_id: String,
        axis_pa_id: String,
        axis_pb_id: String,
    ) -> Self {
        Self {
            p_id,
            q_id,
            axis_pa_id,
            axis_pb_id,
        }
    }
}

impl Constraint for SymmetricConstraint {
    fn num_residuals(&self) -> usize {
        2
    }

    fn residual(&self, pm: &ParameterManager) -> DVector<f64> {
        let params = pm.get_parameters();
        let px = params[pm.get_global_index(&self.p_id, 0).expect("p.x")];
        let py = params[pm.get_global_index(&self.p_id, 1).expect("p.y")];
        let qx = params[pm.get_global_index(&self.q_id, 0).expect("q.x")];
        let qy = params[pm.get_global_index(&self.q_id, 1).expect("q.y")];
        let ax = params[pm.get_global_index(&self.axis_pa_id, 0).expect("axis_a.x")];
        let ay = params[pm.get_global_index(&self.axis_pa_id, 1).expect("axis_a.y")];
        let bx = params[pm.get_global_index(&self.axis_pb_id, 0).expect("axis_b.x")];
        let by = params[pm.get_global_index(&self.axis_pb_id, 1).expect("axis_b.y")];

        let dx = bx - ax;
        let dy = by - ay;
        let epx = (px + qx) / 2.0 - ax;
        let epy = (py + qy) / 2.0 - ay;

        let r0 = epy * dx - epx * dy;
        let r1 = (qx - px) * dx + (qy - py) * dy;

        DVector::from(vec![r0, r1])
    }

    fn jacobian(&self, pm: &ParameterManager) -> DMatrix<f64> {
        let n = pm.num_parameters();
        let mut J = DMatrix::<f64>::zeros(2, n);

        let params = pm.get_parameters();
        let i_px = pm.get_global_index(&self.p_id, 0).expect("p.x");
        let i_py = pm.get_global_index(&self.p_id, 1).expect("p.y");
        let i_qx = pm.get_global_index(&self.q_id, 0).expect("q.x");
        let i_qy = pm.get_global_index(&self.q_id, 1).expect("q.y");
        let i_ax = pm.get_global_index(&self.axis_pa_id, 0).expect("axis_a.x");
        let i_ay = pm.get_global_index(&self.axis_pa_id, 1).expect("axis_a.y");
        let i_bx = pm.get_global_index(&self.axis_pb_id, 0).expect("axis_b.x");
        let i_by = pm.get_global_index(&self.axis_pb_id, 1).expect("axis_b.y");

        let px = params[i_px];
        let py = params[i_py];
        let qx = params[i_qx];
        let qy = params[i_qy];
        let ax = params[i_ax];
        let ay = params[i_ay];
        let bx = params[i_bx];
        let by = params[i_by];

        let dx = bx - ax;
        let dy = by - ay;
        let epx = (px + qx) / 2.0 - ax;
        let epy = (py + qy) / 2.0 - ay;
        let vx = qx - px;
        let vy = qy - py;

        // --- R₀ = epy·dx − epx·dy ---
        // ∂R₀/∂px = (∂epy/∂px)·dx − (∂epx/∂px)·dy = 0·dx − ½·dy = −½·dy
        J[(0, i_px)] = -0.5 * dy;
        // ∂R₀/∂py = ½·dx
        J[(0, i_py)] = 0.5 * dx;
        // ∂R₀/∂qx = −½·dy
        J[(0, i_qx)] = -0.5 * dy;
        // ∂R₀/∂qy = ½·dx
        J[(0, i_qy)] = 0.5 * dx;
        // ∂R₀/∂ax: ∂epy/∂ax=0, ∂dx/∂ax=−1, ∂epx/∂ax=−1, ∂dy/∂ax=0
        //        = epy·(−1) − (−1)·dy = −epy + dy
        J[(0, i_ax)] = -epy + dy;
        // ∂R₀/∂ay: ∂epy/∂ay=−1, ∂dx/∂ay=0, ∂epx/∂ay=0, ∂dy/∂ay=−1
        //        = (−1)·dx − epx·(−1) = −dx + epx
        J[(0, i_ay)] = -dx + epx;
        // ∂R₀/∂bx: ∂dx/∂bx=1  → epy·1 = epy
        J[(0, i_bx)] = epy;
        // ∂R₀/∂by: ∂dy/∂by=1  → −epx·1 = −epx
        J[(0, i_by)] = -epx;

        // --- R₁ = vx·dx + vy·dy ---
        // ∂R₁/∂px = −dx
        J[(1, i_px)] = -dx;
        // ∂R₁/∂py = −dy
        J[(1, i_py)] = -dy;
        // ∂R₁/∂qx = dx
        J[(1, i_qx)] = dx;
        // ∂R₁/∂qy = dy
        J[(1, i_qy)] = dy;
        // ∂R₁/∂ax: ∂dx/∂ax=−1 → vx·(−1) = −vx
        J[(1, i_ax)] = -vx;
        // ∂R₁/∂ay: ∂dy/∂ay=−1 → vy·(−1) = −vy
        J[(1, i_ay)] = -vy;
        // ∂R₁/∂bx: ∂dx/∂bx=1 → vx
        J[(1, i_bx)] = vx;
        // ∂R₁/∂by: ∂dy/∂by=1 → vy
        J[(1, i_by)] = vy;

        J
    }
}
