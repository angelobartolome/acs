use std::collections::HashMap;

use crate::{ConstraintGraph, GeometrySystem, Solver, SolverResult};

pub struct DogLegSolver {
    max_iterations: usize,
    tolerance: f64,
    initial_trust_radius: f64,
    max_trust_radius: f64,
    eta1: f64, // Trust region parameter for decrease
    eta2: f64, // Trust region parameter for increase
}

impl DogLegSolver {
    pub fn new() -> Self {
        Self {
            max_iterations: 500, // More iterations
            tolerance: 1e-12,    // Much stricter tolerance
            initial_trust_radius: 0.1,
            max_trust_radius: 1.0,
            eta1: 0.25,
            eta2: 0.75,
        }
    }

    pub fn with_parameters(
        max_iterations: usize,
        tolerance: f64,
        initial_trust_radius: f64,
        max_trust_radius: f64,
    ) -> Self {
        Self {
            max_iterations,
            tolerance,
            initial_trust_radius,
            max_trust_radius,
            eta1: 0.25,
            eta2: 0.75,
        }
    }

    fn compute_residual_vector(
        &self,
        geometry: &GeometrySystem,
        constraint_graph: &ConstraintGraph,
    ) -> Vec<f64> {
        constraint_graph
            .get_all_constraints()
            .iter()
            .map(|constraint| constraint.error(geometry))
            .collect()
    }

    fn compute_jacobian_matrix(
        &self,
        geometry: &GeometrySystem,
        constraint_graph: &ConstraintGraph,
    ) -> HashMap<(usize, usize), f64> {
        // Map from (constraint_index, point_id*2 + coordinate) to derivative value
        // coordinate: 0 for x, 1 for y
        let mut jacobian = HashMap::new();

        for (constraint_idx, constraint) in
            constraint_graph.get_all_constraints().iter().enumerate()
        {
            let jacobian_entries = constraint.jacobian(geometry);
            for (point_id, dx, dy) in jacobian_entries {
                jacobian.insert((constraint_idx, point_id * 2), dx); // x derivative
                jacobian.insert((constraint_idx, point_id * 2 + 1), dy); // y derivative
            }
        }

        jacobian
    }

    fn compute_gradient(
        &self,
        jacobian: &HashMap<(usize, usize), f64>,
        residual: &[f64],
    ) -> HashMap<usize, f64> {
        let mut gradient = HashMap::new();

        for (&(constraint_idx, var_idx), &jac_val) in jacobian {
            if constraint_idx < residual.len() {
                let grad_val = gradient.entry(var_idx).or_insert(0.0);
                *grad_val += jac_val * residual[constraint_idx];
            }
        }

        gradient
    }

    fn compute_newton_step(
        &self,
        jacobian: &HashMap<(usize, usize), f64>,
        residual: &[f64],
    ) -> Result<HashMap<usize, f64>, String> {
        // For simplicity, we'll use a damped Newton step instead of solving the full linear system
        // This is a simplified approach - in practice, you'd want to solve J^T * J * step = -J^T * residual

        let gradient = self.compute_gradient(jacobian, residual);
        let mut step = HashMap::new();

        for (var_idx, grad_val) in gradient {
            step.insert(var_idx, -grad_val * 0.1); // Simple damping factor
        }

        Ok(step)
    }

    fn vector_norm(&self, vector: &HashMap<usize, f64>) -> f64 {
        vector.values().map(|v| v * v).sum::<f64>().sqrt()
    }

    fn compute_cauchy_step(
        &self,
        jacobian: &HashMap<(usize, usize), f64>,
        gradient: &HashMap<usize, f64>,
        trust_radius: f64,
    ) -> HashMap<usize, f64> {
        // Cauchy step: step = -α * gradient where α minimizes the quadratic model along the gradient direction
        let gradient_norm_sq = gradient.values().map(|g| g * g).sum::<f64>();

        if gradient_norm_sq == 0.0 {
            return HashMap::new();
        }

        // Compute J * gradient to get the Hessian-gradient product approximation
        let mut jg = HashMap::new();
        for (&(constraint_idx, var_idx), &jac_val) in jacobian {
            if let Some(&grad_val) = gradient.get(&var_idx) {
                let entry = jg.entry(constraint_idx).or_insert(0.0);
                *entry += jac_val * grad_val;
            }
        }

        let jg_norm_sq = jg.values().map(|v| v * v).sum::<f64>();

        let alpha = if jg_norm_sq > 0.0 {
            gradient_norm_sq / jg_norm_sq
        } else {
            1.0
        };

        let gradient_norm = gradient_norm_sq.sqrt();
        let step_alpha = alpha.min(trust_radius / gradient_norm);

        let mut step = HashMap::new();
        for (var_idx, grad_val) in gradient {
            step.insert(*var_idx, -step_alpha * grad_val);
        }

        step
    }

    fn compute_dogleg_step(
        &self,
        cauchy_step: &HashMap<usize, f64>,
        newton_step: &HashMap<usize, f64>,
        trust_radius: f64,
    ) -> HashMap<usize, f64> {
        let cauchy_norm = self.vector_norm(cauchy_step);

        if cauchy_norm >= trust_radius {
            // Scale Cauchy step to trust region boundary
            let scale = trust_radius / cauchy_norm;
            let mut step = HashMap::new();
            for (var_idx, val) in cauchy_step {
                step.insert(*var_idx, val * scale);
            }
            return step;
        }

        // Compute the dogleg path: cauchy_step + β * (newton_step - cauchy_step)
        // Find β such that ||step|| = trust_radius

        let mut diff = HashMap::new();
        let all_vars: std::collections::HashSet<_> =
            cauchy_step.keys().chain(newton_step.keys()).collect();

        for &var_idx in &all_vars {
            let c_val = cauchy_step.get(var_idx).unwrap_or(&0.0);
            let n_val = newton_step.get(var_idx).unwrap_or(&0.0);
            diff.insert(*var_idx, n_val - c_val);
        }

        let a = self.vector_norm(&diff).powi(2);
        let b = 2.0
            * all_vars
                .iter()
                .map(|&var_idx| {
                    let c_val = cauchy_step.get(var_idx).unwrap_or(&0.0);
                    let d_val = diff.get(var_idx).unwrap_or(&0.0);
                    c_val * d_val
                })
                .sum::<f64>();
        let c = cauchy_norm.powi(2) - trust_radius.powi(2);

        let beta = if a > 1e-15 {
            let discriminant = b * b - 4.0 * a * c;
            if discriminant >= 0.0 {
                (-b + discriminant.sqrt()) / (2.0 * a)
            } else {
                1.0
            }
        } else {
            1.0
        };

        let beta = beta.clamp(0.0, 1.0);

        let mut step = HashMap::new();
        for &var_idx in &all_vars {
            let c_val = cauchy_step.get(var_idx).unwrap_or(&0.0);
            let d_val = diff.get(var_idx).unwrap_or(&0.0);
            step.insert(*var_idx, c_val + beta * d_val);
        }

        step
    }

    fn save_geometry_state(&self, geometry: &GeometrySystem) -> HashMap<usize, (f64, f64)> {
        let mut state = HashMap::new();
        for (id, point) in geometry.get_all_points() {
            state.insert(*id, (point.x, point.y));
        }
        state
    }

    fn restore_geometry_state(
        &self,
        geometry: &mut GeometrySystem,
        state: &HashMap<usize, (f64, f64)>,
    ) {
        for (id, (x, y)) in state {
            if let Some(point) = geometry.get_point_mut(*id) {
                point.x = *x;
                point.y = *y;
            }
        }
    }

    fn apply_step(&self, geometry: &mut GeometrySystem, step: &HashMap<usize, f64>) {
        for (var_idx, delta) in step {
            let point_id = var_idx / 2;
            let coordinate = var_idx % 2;

            if let Some(point) = geometry.get_point_mut(point_id) {
                if coordinate == 0 {
                    point.x += delta;
                } else {
                    point.y += delta;
                }
            }
        }
    }

    fn compute_predicted_reduction(
        &self,
        jacobian: &HashMap<(usize, usize), f64>,
        residual: &[f64],
        step: &HashMap<usize, f64>,
    ) -> f64 {
        // Predicted reduction = -g^T * step - 0.5 * step^T * H * step
        // Approximating H ≈ J^T * J

        let gradient = self.compute_gradient(jacobian, residual);

        // Compute -g^T * step
        let linear_term = gradient
            .iter()
            .map(|(var_idx, grad_val)| {
                let step_val = step.get(var_idx).unwrap_or(&0.0);
                -grad_val * step_val
            })
            .sum::<f64>();

        // Compute step^T * J^T * J * step / 2
        let mut j_step = vec![0.0; residual.len()];
        for (&(constraint_idx, var_idx), &jac_val) in jacobian {
            if let Some(&step_val) = step.get(&var_idx) {
                j_step[constraint_idx] += jac_val * step_val;
            }
        }

        let quadratic_term = 0.5 * j_step.iter().map(|v| v * v).sum::<f64>();

        linear_term + quadratic_term
    }
}

impl Solver for DogLegSolver {
    fn solve(
        &self,
        geometry: &mut GeometrySystem,
        constraint_graph: &ConstraintGraph,
    ) -> Result<SolverResult, String> {
        let initial_error = constraint_graph.total_error(geometry);
        let mut current_error = initial_error;
        let mut trust_radius = self.initial_trust_radius;

        for iteration in 0..self.max_iterations {
            // Check for convergence
            if current_error.sqrt() < self.tolerance {
                return Ok(SolverResult::Converged {
                    iterations: iteration,
                    final_error: current_error,
                    initial_error,
                });
            }

            // Compute residual vector and Jacobian matrix
            let residual = self.compute_residual_vector(geometry, constraint_graph);
            let jacobian = self.compute_jacobian_matrix(geometry, constraint_graph);

            if residual.is_empty() {
                // No constraints to solve
                return Ok(SolverResult::Converged {
                    iterations: iteration,
                    final_error: 0.0,
                    initial_error,
                });
            }

            // Compute gradient: J^T * residual
            let gradient = self.compute_gradient(&jacobian, &residual);

            // Compute Newton step: solve J * step = -residual
            let newton_step = self.compute_newton_step(&jacobian, &residual)?;
            let newton_step_norm = self.vector_norm(&newton_step);

            // Compute Cauchy point step
            let cauchy_step = self.compute_cauchy_step(&jacobian, &gradient, trust_radius);

            // Compute dogleg step
            let step = if newton_step_norm <= trust_radius {
                // Newton step is within trust region
                newton_step
            } else {
                // Use dogleg method to interpolate between Cauchy and Newton steps
                self.compute_dogleg_step(&cauchy_step, &newton_step, trust_radius)
            };

            // Save current state in case we need to reject the step
            let original_points = self.save_geometry_state(geometry);

            // Apply the step
            self.apply_step(geometry, &step);

            // Compute new error
            let new_error = constraint_graph.total_error(geometry);

            // Compute predicted reduction
            let predicted_reduction = self.compute_predicted_reduction(&jacobian, &residual, &step);

            // Compute actual reduction
            let actual_reduction = current_error - new_error;

            // Compute trust region ratio
            let rho = if predicted_reduction.abs() < 1e-15 {
                1.0 // Avoid division by zero
            } else {
                actual_reduction / predicted_reduction
            };

            // Update trust region radius
            if rho > self.eta2 {
                // Good step, increase trust radius
                trust_radius = (trust_radius * 2.0).min(self.max_trust_radius);
            } else if rho < self.eta1 {
                // Poor step, decrease trust radius
                trust_radius *= 0.5;

                // Reject the step
                self.restore_geometry_state(geometry, &original_points);
                continue;
            }
            // If eta1 <= rho <= eta2, keep current trust radius and accept step

            current_error = new_error;
        }

        Ok(SolverResult::MaxIterationsReached {
            iterations: self.max_iterations,
            final_error: current_error,
            initial_error,
        })
    }
}
