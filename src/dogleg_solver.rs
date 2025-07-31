use std::collections::HashMap;

use nalgebra::{DMatrix, DVector};

use crate::{Constraint, Point, Solver};
pub struct DogLegSolver {
    max_iterations: usize,
    tolerance: f64,
}

impl Solver for DogLegSolver {
    fn solve(
        &self,
        geometry: &mut crate::GeometrySystem,
        constraint_graph: &crate::ConstraintGraph,
    ) -> Result<crate::SolverResult, String> {
        let points = geometry.get_all_points_mut();
        let constraints = constraint_graph.get_constraints();

        return Ok(DogLegSolver::solve_constraints(
            points,
            constraints,
            self.max_iterations,
            self.tolerance,
        ));
    }
}

impl DogLegSolver {
    pub fn new() -> Self {
        Self {
            max_iterations: 100,
            tolerance: 1e-6,
        }
    }

    pub fn solve_constraints(
        points: &mut HashMap<String, Point>,
        constraints: &[Box<dyn Constraint>],
        max_iter: usize,
        tolerance: f64,
    ) -> crate::SolverResult {
        let mut trust_radius = 1.0;
        let mut prev_residual_norm = f64::INFINITY;
        let mut stagnation_count = 0;

        // Build index map (string ID -> variable index)
        let mut id_to_index = HashMap::new();
        let mut index = 0;
        for key in points.keys() {
            id_to_index.insert(key.clone(), index);
            index += 1;
        }

        for iter in 0..max_iter {
            let (residuals, _) = DogLegSolver::build_system(points, constraints, &id_to_index);
            let residual_norm = residuals.norm();

            if residual_norm < tolerance {
                println!(
                    "Converged after {} iterations with residual norm: {:.2e}",
                    iter + 1,
                    residual_norm
                );

                return crate::SolverResult::Converged {
                    initial_error: prev_residual_norm,
                    final_error: residual_norm,
                    iterations: iter + 1,
                };
            }

            // Check for stagnation
            if (residual_norm - prev_residual_norm).abs() < 1e-12 {
                stagnation_count += 1;
                if stagnation_count > 5 {
                    return crate::SolverResult::MaxIterationsReached {
                        initial_error: prev_residual_norm,
                        iterations: iter + 1,
                        final_error: residual_norm,
                    };
                }
            } else {
                stagnation_count = 0;
            }
            prev_residual_norm = residual_norm;

            let rho = DogLegSolver::dog_leg_step(points, constraints, &id_to_index, trust_radius);

            // Update trust radius based on step quality
            if rho > 0.75 {
                trust_radius = (trust_radius * 2.0).min(10.0);
            } else if rho < 0.25 {
                trust_radius *= 0.5;
            }

            // Ensure trust radius doesn't get too small
            if trust_radius < 1e-8 {
                return crate::SolverResult::MaxIterationsReached {
                    initial_error: prev_residual_norm,
                    iterations: iter + 1,
                    final_error: residual_norm,
                };
            }
        }

        // If we exit the loop without converging, print final status
        let (final_residuals, _) = DogLegSolver::build_system(points, constraints, &id_to_index);
        let final_residual_norm = final_residuals.norm();
        if final_residual_norm >= tolerance {
            return crate::SolverResult::MaxIterationsReached {
                initial_error: prev_residual_norm,
                iterations: max_iter,
                final_error: final_residual_norm,
            };
        }

        crate::SolverResult::Converged {
            initial_error: prev_residual_norm,
            final_error: final_residual_norm,
            iterations: max_iter,
        }
    }

    pub fn build_system(
        points: &HashMap<String, Point>,
        constraints: &[Box<dyn Constraint>],
        id_to_index: &HashMap<String, usize>,
    ) -> (DVector<f64>, DMatrix<f64>) {
        let total_residuals: usize = constraints.iter().map(|c| c.num_residuals()).sum();
        let total_vars = points.len() * 2;

        let mut residuals = DVector::<f64>::zeros(total_residuals);
        let mut jacobian = DMatrix::<f64>::zeros(total_residuals, total_vars);

        let mut row_offset = 0;
        for c in constraints {
            let r = c.residual(points);
            let j = c.jacobian(points, &id_to_index);

            residuals.rows_mut(row_offset, r.len()).copy_from(&r);
            jacobian.rows_mut(row_offset, j.nrows()).copy_from(&j);

            row_offset += r.len();
        }

        (residuals, jacobian)
    }

    fn dog_leg_step(
        points: &mut HashMap<String, Point>,
        constraints: &[Box<dyn Constraint>],
        id_to_index: &HashMap<String, usize>,
        trust_radius: f64,
    ) -> f64 {
        let (residuals, jacobian) = DogLegSolver::build_system(points, constraints, id_to_index);
        let initial_cost = 0.5 * residuals.norm_squared();

        // g = J^T r
        let g = jacobian.transpose() * &residuals;

        // B = J^T J
        let hessian = jacobian.transpose() * jacobian.clone();

        // p_b = solve(B, -g) using pseudo-inverse for singular matrices
        let p_b = match hessian.clone().lu().solve(&(-&g)) {
            Some(solution) => solution,
            None => {
                // Matrix is singular, use pseudo-inverse approach

                let svd = hessian.clone().svd(true, true);
                let tolerance = 1e-12;
                let mut s_inv = DVector::zeros(svd.singular_values.len());
                for (i, &s) in svd.singular_values.iter().enumerate() {
                    if s > tolerance {
                        s_inv[i] = 1.0 / s;
                    }
                }
                let u = svd.u.unwrap();
                let vt = svd.v_t.unwrap();
                let s_inv_mat = DMatrix::from_diagonal(&s_inv);
                vt.transpose() * s_inv_mat * u.transpose() * (-&g)
            }
        };

        // pU = -alpha * g
        let g_tg = g.dot(&g);
        let bg = &hessian * &g;
        let alpha = if g.dot(&bg) > 1e-12 {
            g_tg / g.dot(&bg)
        } else {
            0.0
        };
        let p_u = -&g * alpha;

        let step = if p_b.norm() <= trust_radius {
            p_b
        } else if p_u.norm() >= trust_radius {
            p_u.normalize() * trust_radius
        } else {
            let diff = &p_b - &p_u;
            let tau = DogLegSolver::solve_tau(&p_u, &diff, trust_radius);
            &p_u + diff * tau
        };

        // Store original positions
        let original_points = points.clone();

        // Apply step only to non-fixed points
        for (id, idx) in id_to_index {
            let point = points.get_mut(id).expect("Point not found");
            if !point.fixed {
                point.x += step[*idx * 2];
                point.y += step[*idx * 2 + 1];
            }
        }

        // Calculate new cost
        let (new_residuals, _) = DogLegSolver::build_system(points, constraints, id_to_index);
        let new_cost = 0.5 * new_residuals.norm_squared();

        // Calculate actual vs predicted reduction
        let actual_reduction = initial_cost - new_cost;
        let predicted_reduction = -g.dot(&step) - 0.5 * step.dot(&(&hessian * &step));

        let rho = if predicted_reduction.abs() > 1e-12 {
            actual_reduction / predicted_reduction
        } else {
            0.0
        };

        // If step made things worse, revert
        if rho < 0.0 {
            for (id, _idx) in id_to_index {
                let point = points.get_mut(id).expect("Point not found");
                *point = original_points[id].clone();
            }

            // for (i, point) in points.iter_mut() {
            //     *point = original_points[i].clone();
            // }
        }

        rho
    }

    fn solve_tau(p_u: &DVector<f64>, diff: &DVector<f64>, trust_radius: f64) -> f64 {
        let a = diff.dot(diff);
        let b = 2.0 * p_u.dot(diff);
        let c = p_u.dot(p_u) - trust_radius * trust_radius;
        let disc = b * b - 4.0 * a * c;

        if disc < 0.0 {
            return 0.0;
        }
        let t1 = (-b + disc.sqrt()) / (2.0 * a);
        let t2 = (-b - disc.sqrt()) / (2.0 * a);
        if t1 >= 0.0 && t1 <= 1.0 { t1 } else { t2 }
    }
}
