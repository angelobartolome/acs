#![allow(non_snake_case)]

use nalgebra::{DMatrix, DVector};

use crate::{
    Constraint, ConstraintGraph, EntityType, GeometrySystem, ParameterManager, Solver, SolverResult,
};

/// Selects how the Gauss-Newton step is computed inside the Dogleg solver.
/// Matches the `dogLegGaussStep` option in FreeCAD's GCS.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GaussStepMethod {
    /// Direct solve `J h = -f` via full-pivot LU (default; avoids forming J^T J).
    FullPivLU,
    /// Minimum-norm solution `J^T (J J^T)^{-1} (-f)` via full-pivot LU.
    LeastNormFullPivLU,
    /// Minimum-norm solution via Cholesky of `J J^T` (falls back to LU if J J^T is rank-deficient).
    LeastNormLdlt,
}

pub struct ParametricDogLegSolver {
    max_iterations: usize,
    /// Convergence tolerance on the L-infinity norm of the residual vector.
    tolf: f64,
    /// Convergence tolerance on the L-infinity norm of the gradient.
    tolg: f64,
    /// Convergence tolerance on the trust radius relative to the parameter norm.
    tolx: f64,
    gauss_step: GaussStepMethod,
}

pub(crate) fn build_param_manager(geometry: &GeometrySystem) -> ParameterManager {
    let mut pm = ParameterManager::new();

    let mut point_ids: Vec<String> = geometry.get_all_points().keys().cloned().collect();
    point_ids.sort();
    for id in point_ids {
        let point = geometry.get_point(&id).expect("point");
        pm.register_entity(id, EntityType::Point, point);
    }

    let mut circle_ids: Vec<String> = geometry.get_all_circles().keys().cloned().collect();
    circle_ids.sort();
    for id in circle_ids {
        let circle = geometry.get_circle(&id).expect("circle");
        pm.register_entity(id, EntityType::Circle, circle);
    }

    let mut arc_ids: Vec<String> = geometry.get_all_arcs().keys().cloned().collect();
    arc_ids.sort();
    for id in arc_ids {
        let arc = geometry.get_arc(&id).expect("arc");
        pm.register_entity(id, EntityType::Arc, arc);
    }

    pm
}

impl Default for ParametricDogLegSolver {
    fn default() -> Self {
        Self::new()
    }
}

impl ParametricDogLegSolver {
    pub fn new() -> Self {
        Self {
            max_iterations: 100,
            tolf: 1e-10,
            tolg: 1e-80,
            tolx: 1e-80,
            gauss_step: GaussStepMethod::FullPivLU,
        }
    }

    pub fn set_max_iterations(&mut self, max_iterations: usize) {
        self.max_iterations = max_iterations.max(1);
    }

    pub fn set_gauss_step(&mut self, method: GaussStepMethod) {
        self.gauss_step = method;
    }

    pub fn solve_parametric(
        &self,
        geometry: &mut GeometrySystem,
        constraint_graph: &ConstraintGraph,
    ) -> Result<SolverResult, String> {
        let mut param_manager = build_param_manager(geometry);
        let constraints = constraint_graph.get_constraints();
        let refs: Vec<&dyn Constraint> = constraints.iter().map(|c| c.as_ref()).collect();
        let result = self.solve_dl(&mut param_manager, &refs);
        Self::sync_geometry_from_parameters(&mut param_manager, geometry)?;
        Ok(result)
    }

    pub(crate) fn solve_dl(
        &self,
        param_manager: &mut ParameterManager,
        constraints: &[&dyn Constraint],
    ) -> SolverResult {
        if param_manager.num_parameters() == 0 {
            return SolverResult::Converged {
                iterations: 0,
                final_error: 0.0,
                initial_error: 0.0,
            };
        }

        let fixed_mask: Vec<bool> = param_manager
            .get_parameter_info()
            .iter()
            .map(|info| info.is_fixed)
            .collect();

        let (mut fx, mut jac) = Self::build_system(param_manager, constraints, &fixed_mask);
        let mut err = 0.5 * fx.norm_squared();
        let initial_error = fx.norm();

        // Divergence limit is fixed from the initial error (matches GCS.cpp).
        let diverging_lim = 1e6 * err + 1e12;

        let mut g = jac.transpose() * (-&fx);
        let mut g_inf = g.amax();
        let mut fx_inf = fx.amax();

        let mut delta = 0.1_f64;
        let mut nu = 2.0_f64;
        let mut reduce = 0i32;
        let mut iter = 0usize;
        let mut stop = 0u8;

        while stop == 0 {
            // --- Convergence / divergence checks (matches GCS::solve_DL order) ---
            if fx_inf <= self.tolf {
                stop = 1; // exact solution found
                break;
            }
            if g_inf <= self.tolg {
                stop = 2; // gradient vanished
                break;
            }
            let x_norm = {
                let p = param_manager.get_parameters();
                p.iter().map(|v| v * v).sum::<f64>().sqrt()
            };
            if delta <= self.tolx * (self.tolx + x_norm) {
                stop = 2; // trust radius collapsed
                break;
            }
            if iter >= self.max_iterations {
                stop = 4;
                break;
            }
            if err > diverging_lim || err.is_nan() {
                stop = 6; // diverging or NaN
                break;
            }

            // --- Cauchy (steepest-descent) step: h_sd = alpha * g ---
            // When J*g underflows (near-zero gradient at near-solution), fall back to h_sd = 0
            // so the dogleg path degrades to a scaled GN step instead of stopping early.
            let jg_norm_sq = (&jac * &g).norm_squared();
            let (alpha, h_sd) = if jg_norm_sq > 1e-300 {
                let a = g.norm_squared() / jg_norm_sq;
                (a, a * &g)
            } else {
                (0.0, DVector::zeros(g.len()))
            };

            // --- Gauss-Newton step: solve J h = -f ---
            let h_gn = match self.compute_gn_step(&jac, &fx) {
                Some(h) => h,
                None => {
                    stop = 6;
                    break;
                }
            };

            // Bail out if the linear solve produced garbage.
            let rel_error = (&jac * &h_gn + &fx).norm() / fx.norm();
            if rel_error > 1e15 {
                break;
            }

            // --- Dogleg step selection ---
            let h_dl = if h_gn.norm() < delta {
                // GN step fits inside the trust region.
                if h_gn.norm() <= self.tolx * (self.tolx + x_norm) {
                    stop = 5; // step negligible relative to parameters
                    break;
                }
                h_gn.clone()
            } else if alpha * g.norm() >= delta {
                // Cauchy step already reaches the trust region boundary.
                (delta / (alpha * g.norm())) * &h_sd
            } else {
                // Interpolate along the dogleg path between h_sd and h_gn.
                let b = &h_gn - &h_sd;
                let bb = b.norm_squared();
                let gb = h_sd.dot(&b).abs();
                // c = delta^2 - ||h_sd||^2, rewritten for numerical stability
                let c = (delta + h_sd.norm()) * (delta - h_sd.norm());

                // Two-case formula avoids catastrophic cancellation when gb ≈ 0.
                let beta = if gb > 0.0 {
                    c / (gb + (gb * gb + c * bb).sqrt())
                } else if bb > 1e-300 {
                    ((gb * gb + c * bb).sqrt() - gb) / bb
                } else {
                    1.0 // h_gn ≈ h_sd; the whole segment collapses to a point
                };
                &h_sd + beta * &b
            };

            // --- Apply trial step ---
            let old_params: Vec<f64> = param_manager.get_parameters().to_vec();
            for (i, &v) in h_dl.iter().enumerate() {
                if !fixed_mask[i] {
                    let _ = param_manager.set_parameter(i, old_params[i] + v);
                }
            }

            let (fx_new, jac_new) = Self::build_system(param_manager, constraints, &fixed_mask);
            let err_new = 0.5 * fx_new.norm_squared();

            // Predicted reduction in the linear model vs actual reduction.
            let dl_model = err - 0.5 * (&fx + &jac * &h_dl).norm_squared();
            let df_actual = err - err_new;

            // Accept step only if both actual and predicted reductions are positive.
            let rho = if df_actual > 0.0 && dl_model > 0.0 {
                fx = fx_new;
                jac = jac_new;
                err = err_new;
                g = jac.transpose() * (-&fx);
                g_inf = g.amax();
                fx_inf = fx.amax();
                dl_model / df_actual
            } else {
                // Reject: revert parameters; fx/jac remain valid for old_params.
                for (i, &v) in old_params.iter().enumerate() {
                    if !fixed_mask[i] {
                        let _ = param_manager.set_parameter(i, v);
                    }
                }
                -1.0
            };

            // --- Trust-region update (matches GCS::solve_DL nu/reduce logic) ---
            if (rho - 1.0).abs() < 0.2 && h_dl.norm() > delta / 3.0 && reduce <= 0 {
                delta *= 3.0;
                nu = 2.0;
                reduce = 0;
            } else if rho < 0.25 {
                delta /= nu;
                nu *= 2.0; // exponential backoff on repeated failures
                reduce = 2;
            } else {
                reduce -= 1;
            }

            iter += 1;
        }

        let final_error = fx.norm();
        match stop {
            1 | 2 | 5 => SolverResult::Converged {
                iterations: iter,
                final_error,
                initial_error,
            },
            _ => SolverResult::MaxIterationsReached {
                iterations: iter,
                final_error,
                initial_error,
            },
        }
    }

    /// Computes the Gauss-Newton step using the selected linear-solve method.
    /// Returns `None` only if every fallback fails.
    fn compute_gn_step(&self, jac: &DMatrix<f64>, fx: &DVector<f64>) -> Option<DVector<f64>> {
        let neg_fx = -fx;
        let svd_fallback = || jac.clone().svd(true, true).solve(&neg_fx, 1e-12).ok();

        match self.gauss_step {
            GaussStepMethod::FullPivLU => {
                // Direct solve J h = -f; avoids forming the normal equations J^T J.
                // nalgebra's full_piv_lu panics on non-square matrices, so fall back to SVD
                // for rectangular systems (under/overdetermined).
                if jac.nrows() == jac.ncols() {
                    jac.clone().full_piv_lu().solve(&neg_fx).or_else(svd_fallback)
                } else {
                    svd_fallback()
                }
            }
            GaussStepMethod::LeastNormFullPivLU => {
                // Minimum-norm solution: h = J^T (J J^T)^{-1} (-f)
                let jt = jac.transpose();
                let jjt = jac * &jt;
                jjt.full_piv_lu()
                    .solve(&neg_fx)
                    .map(|y| &jt * y)
                    .or_else(svd_fallback)
            }
            GaussStepMethod::LeastNormLdlt => {
                // Minimum-norm via Cholesky of J J^T (PSD); fall back to LU if rank-deficient.
                let jt = jac.transpose();
                let jjt = jac * &jt;
                jjt.clone()
                    .cholesky()
                    .map(|chol| &jt * chol.solve(&neg_fx))
                    .or_else(|| jjt.full_piv_lu().solve(&neg_fx).map(|y| &jt * y))
                    .or_else(svd_fallback)
            }
        }
    }

    fn build_system(
        param_manager: &ParameterManager,
        constraints: &[&dyn Constraint],
        fixed_mask: &[bool],
    ) -> (DVector<f64>, DMatrix<f64>) {
        let total_residuals: usize = constraints.iter().map(|c| c.num_residuals()).sum();
        let total_vars = param_manager.num_parameters();

        let mut residuals = DVector::<f64>::zeros(total_residuals);
        let mut jacobian = DMatrix::<f64>::zeros(total_residuals, total_vars);

        let mut row_offset = 0;
        for c in constraints {
            let r = c.residual(param_manager);
            let j = c.jacobian(param_manager);
            residuals.rows_mut(row_offset, r.len()).copy_from(&r);
            jacobian.rows_mut(row_offset, j.nrows()).copy_from(&j);
            row_offset += r.len();
        }

        for (i, &is_fixed) in fixed_mask.iter().enumerate() {
            if is_fixed {
                jacobian.column_mut(i).fill(0.0);
            }
        }

        (residuals, jacobian)
    }

    pub(crate) fn sync_geometry_from_parameters(
        param_manager: &mut ParameterManager,
        geometry: &mut GeometrySystem,
    ) -> Result<(), String> {
        for (id, point) in geometry.get_all_points_mut() {
            param_manager.update_entity_parameters(id, point)?;
        }
        for (id, circle) in geometry.get_all_circles_mut() {
            param_manager.update_entity_parameters(id, circle)?;
        }
        for (id, arc) in geometry.get_all_arcs_mut() {
            param_manager.update_entity_parameters(id, arc)?;
        }
        Ok(())
    }
}

impl Solver for ParametricDogLegSolver {
    fn solve(
        &self,
        geometry: &mut GeometrySystem,
        constraint_graph: &ConstraintGraph,
    ) -> Result<SolverResult, String> {
        self.solve_parametric(geometry, constraint_graph)
    }
}
