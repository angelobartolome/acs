use crate::geometry::GeometrySystem;
use crate::{ConstraintGraph, Solver, SolverResult};
use std::collections::HashMap;

pub struct NewtonRaphsonSolver {
    max_iterations: usize,
    tolerance: f64,
    damping_factor: f64,
}

impl NewtonRaphsonSolver {
    pub fn new() -> Self {
        Self {
            max_iterations: 500,
            tolerance: 1e-9,
            damping_factor: 0.8,
        }
    }

    pub fn with_parameters(max_iterations: usize, tolerance: f64, damping_factor: f64) -> Self {
        Self {
            max_iterations,
            tolerance,
            damping_factor,
        }
    }
}

impl Solver for NewtonRaphsonSolver {
    fn solve(
        &self,
        geometry: &mut GeometrySystem,
        constraint_graph: &ConstraintGraph,
    ) -> Result<SolverResult, String> {
        let mut iteration = 0;
        let initial_error = constraint_graph.total_error(geometry);

        while iteration < self.max_iterations {
            let total_error = constraint_graph.total_error(geometry);

            if total_error < self.tolerance {
                return Ok(SolverResult::Converged {
                    iterations: iteration,
                    final_error: total_error,
                    initial_error,
                });
            }

            // Build Jacobian matrix and error vector
            let mut jacobian: HashMap<usize, (f64, f64)> = HashMap::new(); // point_id -> (dx_sum, dy_sum)

            for constraint in constraint_graph.get_all_constraints() {
                let error = constraint.error(geometry);
                let jacobian_entries = constraint.jacobian(geometry);

                for (point_id, dx, dy) in jacobian_entries {
                    let entry = jacobian.entry(point_id).or_insert((0.0, 0.0));
                    entry.0 += error * dx;
                    entry.1 += error * dy;
                }
            }

            // Apply corrections with damping
            let mut max_correction = 0f64;
            for (point_id, (dx, dy)) in jacobian {
                let correction_x = -self.damping_factor * dx;
                let correction_y = -self.damping_factor * dy;

                max_correction = max_correction.max(correction_x.abs().max(correction_y.abs()));

                if let Some(point) = geometry.get_point_mut(point_id) {
                    point.x += correction_x;
                    point.y += correction_y;
                }
            }

            // Check for convergence based on correction magnitude
            if max_correction < self.tolerance {
                return Ok(SolverResult::Converged {
                    iterations: iteration + 1,
                    final_error: constraint_graph.total_error(geometry),
                    initial_error,
                });
            }

            iteration += 1;
        }

        Ok(SolverResult::MaxIterationsReached {
            iterations: self.max_iterations,
            final_error: constraint_graph.total_error(geometry),
            initial_error,
        })
    }
}
