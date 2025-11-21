use nalgebra::{DMatrix, DVector};

use crate::{
    Constraint, ConstraintGraph, EntityType, GeometrySystem, ParameterManager, Solver, SolverResult,
};

pub struct ParametricDogLegSolver {
    max_iterations: usize,
    tolerance: f64,
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
            tolerance: 1e-6,
        }
    }

    pub fn solve_parametric(
        &self,
        geometry: &mut GeometrySystem,
        constraint_graph: &ConstraintGraph,
    ) -> Result<SolverResult, String> {
        // Build parameter manager from geometry
        let mut param_manager = ParameterManager::new();

        // Register all points
        for (id, point) in geometry.get_all_points() {
            param_manager.register_entity(id.clone(), EntityType::Point, point);
        }

        // Register all circles
        for (id, circle) in geometry.get_all_circles() {
            param_manager.register_entity(id.clone(), EntityType::Circle, circle);
        }

        // Register all arcs
        for (id, arc) in geometry.get_all_arcs() {
            param_manager.register_entity(id.clone(), EntityType::Arc, arc);
        }

        let constraints = constraint_graph.get_constraints();

        let result = Self::solve_constraints_parametric(
            &mut param_manager,
            constraints,
            self.max_iterations,
            self.tolerance,
        );

        // Update geometry with final parameter values
        Self::sync_geometry_from_parameters(&mut param_manager, geometry)?;

        Ok(result)
    }

    fn solve_constraints_parametric(
        param_manager: &mut ParameterManager,
        constraints: &[Box<dyn Constraint>],
        max_iter: usize,
        tolerance: f64,
    ) -> SolverResult {
        let mut trust_radius = 1.0;
        let mut prev_residual_norm = f64::INFINITY;
        let mut stagnation_count = 0;

        for iter in 0..max_iter {
            let (residuals, _) = Self::build_system_parametric(param_manager, constraints);
            let residual_norm = residuals.norm();

            if residual_norm < tolerance {
                println!(
                    "Converged after {} iterations with residual norm: {:.2e}",
                    iter + 1,
                    residual_norm
                );

                return SolverResult::Converged {
                    initial_error: prev_residual_norm,
                    final_error: residual_norm,
                    iterations: iter + 1,
                };
            }

            // Check for stagnation
            if (residual_norm - prev_residual_norm).abs() < 1e-12 {
                stagnation_count += 1;
                if stagnation_count > 5 {
                    return SolverResult::MaxIterationsReached {
                        initial_error: prev_residual_norm,
                        iterations: iter + 1,
                        final_error: residual_norm,
                    };
                }
            } else {
                stagnation_count = 0;
            }
            prev_residual_norm = residual_norm;

            let rho = Self::dog_leg_step_parametric(param_manager, constraints, trust_radius);

            // Update trust radius based on step quality
            if rho > 0.75 {
                trust_radius = (trust_radius * 2.0).min(10.0);
            } else if rho < 0.25 {
                trust_radius *= 0.5;
            }

            // Ensure trust radius doesn't get too small
            if trust_radius < 1e-8 {
                return SolverResult::MaxIterationsReached {
                    initial_error: prev_residual_norm,
                    iterations: iter + 1,
                    final_error: residual_norm,
                };
            }
        }

        // If we exit the loop without converging, return final status
        let (final_residuals, _) = Self::build_system_parametric(param_manager, constraints);
        let final_residual_norm = final_residuals.norm();
        if final_residual_norm >= tolerance {
            return SolverResult::MaxIterationsReached {
                initial_error: prev_residual_norm,
                iterations: max_iter,
                final_error: final_residual_norm,
            };
        }

        SolverResult::Converged {
            initial_error: prev_residual_norm,
            final_error: final_residual_norm,
            iterations: max_iter,
        }
    }

    fn build_system_parametric(
        param_manager: &ParameterManager,
        constraints: &[Box<dyn Constraint>],
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

        (residuals, jacobian)
    }

    fn dog_leg_step_parametric(
        param_manager: &mut ParameterManager,
        constraints: &[Box<dyn Constraint>],
        trust_radius: f64,
    ) -> f64 {
        let (residuals, jacobian) = Self::build_system_parametric(param_manager, constraints);

        if residuals.norm() < 1e-14 {
            return 1.0; // Already at solution
        }

        // Gauss-Newton step
        let jt = jacobian.transpose();
        let jtj = &jt * &jacobian;
        let jtr = &jt * &residuals;

        // Clone jtj and jtr for later use
        let jtj_clone = jtj.clone();
        let jtr_clone = jtr.clone();

        // Use pseudo-inverse for robustness
        let gn_step = match jtj.try_inverse() {
            Some(inv) => -inv * &jtr,
            None => {
                // Use SVD-based pseudo-inverse for rank-deficient cases
                let svd = jtj_clone.svd(true, true);
                -svd.pseudo_inverse(1e-12)
                    .expect("SVD pseudo-inverse computation failed: matrix is too rank-deficient")
                    * &jtr
            }
        };

        // Gradient step
        let gradient = jtr_clone;
        let grad_step = if gradient.norm() > 1e-14 {
            let alpha = gradient.dot(&gradient) / (&jacobian * &gradient).norm_squared();
            -alpha * &gradient
        } else {
            DVector::zeros(gradient.len())
        };

        // Compute dog leg step
        let gn_norm = gn_step.norm();
        let grad_norm = grad_step.norm();

        let step = if gn_norm <= trust_radius {
            gn_step
        } else if grad_norm >= trust_radius {
            (trust_radius / grad_norm) * grad_step
        } else {
            // Dog leg interpolation
            let beta_num = trust_radius * trust_radius - grad_norm * grad_norm;
            let diff = &gn_step - &grad_step;
            let beta_denom = diff.dot(&diff);
            let beta = if beta_denom > 1e-14 {
                (-grad_step.dot(&diff)
                    + (grad_step.dot(&diff).powi(2) + beta_denom * beta_num).sqrt())
                    / beta_denom
            } else {
                0.0
            };
            grad_step + beta * diff
        };

        // Apply step to parameters (only non-fixed ones)
        let old_params = param_manager.get_parameters().to_vec();
        let old_residual_norm = residuals.norm();

        // Apply step (collect fixed status to avoid borrowing issues)
        let fixed_params: Vec<bool> = param_manager
            .get_parameter_info()
            .iter()
            .map(|info| info.is_fixed)
            .collect();

        for (i, &step_val) in step.iter().enumerate() {
            if i < fixed_params.len() && !fixed_params[i] {
                let new_val = old_params[i] + step_val;
                let _ = param_manager.set_parameter(i, new_val);
            }
        }

        // Compute new residuals
        let (new_residuals, _) = Self::build_system_parametric(param_manager, constraints);
        let new_residual_norm = new_residuals.norm();

        // Compute step quality
        let jt_jacobian = &jt * &jacobian;
        let predicted_reduction = old_residual_norm.powi(2)
            - (old_residual_norm.powi(2)
                + 2.0 * gradient.dot(&step)
                + step.dot(&(&jt_jacobian * &step)));
        let actual_reduction = old_residual_norm.powi(2) - new_residual_norm.powi(2);

        let rho = if predicted_reduction.abs() < 1e-14 {
            1.0 // Perfect step when prediction is negligible
        } else {
            actual_reduction / predicted_reduction
        };

        // If step quality is poor, revert the step
        if rho < 0.1 {
            // Revert parameters
            for (i, &old_val) in old_params.iter().enumerate() {
                if i < fixed_params.len() && !fixed_params[i] {
                    let _ = param_manager.set_parameter(i, old_val);
                }
            }
        }

        rho
    }

    fn sync_geometry_from_parameters(
        param_manager: &mut ParameterManager,
        geometry: &mut GeometrySystem,
    ) -> Result<(), String> {
        // Update points
        for (id, point) in geometry.get_all_points_mut() {
            param_manager.update_entity_parameters(id, point)?;
        }

        // Update circles
        for (id, circle) in geometry.get_all_circles_mut() {
            param_manager.update_entity_parameters(id, circle)?;
        }

        // Update arcs
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
