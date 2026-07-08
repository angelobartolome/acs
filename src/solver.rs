use crate::{
    Constraint, ConstraintType, GeometrySystem, ParametricDogLegSolver, Point, create_constraint,
};
use crate::component_graph::{find_components, is_component_satisfied};
use crate::dogleg_solver::build_param_manager;
use crate::geometry::{Arc as GeoArc, Circle, Line};

pub struct ConstraintGraph {
    constraints: Vec<Box<dyn Constraint>>,
}

impl Default for ConstraintGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl ConstraintGraph {
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
        }
    }

    pub fn get_constraints(&self) -> &[Box<dyn Constraint>] {
        &self.constraints
    }
}

#[derive(Debug)]
pub enum SolverResult {
    Converged {
        iterations: usize,
        final_error: f64,
        initial_error: f64,
    },
    MaxIterationsReached {
        iterations: usize,
        final_error: f64,
        initial_error: f64,
    },
}

pub trait Solver {
    fn solve(
        &self,
        geometry: &mut GeometrySystem,
        constraint_graph: &ConstraintGraph,
    ) -> Result<SolverResult, String>;
}

pub struct ConstraintSolver {
    geometry: GeometrySystem,
    constraint_graph: ConstraintGraph,
    constraint_types: Vec<ConstraintType>,
    solver: ParametricDogLegSolver,
}

impl Default for ConstraintSolver {
    fn default() -> Self {
        Self::new()
    }
}

impl ConstraintSolver {
    pub fn new() -> Self {
        Self {
            geometry: GeometrySystem::new(),
            constraint_graph: ConstraintGraph {
                constraints: Vec::new(),
            },
            constraint_types: Vec::new(),
            solver: ParametricDogLegSolver::new(),
        }
    }

    pub fn set_max_iterations(&mut self, max_iterations: usize) {
        self.solver.set_max_iterations(max_iterations);
    }

    pub fn add_point(&mut self, point: crate::geometry::Point) -> String {
        self.geometry.add_point(point)
    }

    pub fn add_circle(&mut self, circle: crate::geometry::Circle) -> String {
        self.geometry.add_circle(circle)
    }

    pub fn add_line(&mut self, line: Line) -> String {
        self.geometry.add_line(line)
    }

    pub fn add_arc(&mut self, arc: GeoArc) -> String {
        self.geometry.add_arc(arc)
    }

    pub fn add_constraint(&mut self, constraint_type: ConstraintType) -> Result<(), String> {
        self.constraint_types.push(constraint_type.clone());
        self.constraint_graph
            .constraints
            .push(create_constraint(constraint_type)?);
        Ok(())
    }

    pub fn solve(&mut self) -> Result<SolverResult, String> {
        let constraints = self.constraint_graph.get_constraints();

        if constraints.is_empty() {
            return Ok(SolverResult::Converged {
                iterations: 0,
                final_error: 0.0,
                initial_error: 0.0,
            });
        }

        let mut param_manager = build_param_manager(&self.geometry);
        let components = find_components(&self.constraint_types);

        const TOLF: f64 = 1e-10;
        let mut total_iters = 0usize;
        let mut any_max_iter = false;
        let mut combined_initial_error = 0.0f64;
        let mut combined_final_error = 0.0f64;

        for component_indices in &components.components {
            if is_component_satisfied(component_indices, constraints, &param_manager, TOLF) {
                continue;
            }
            let component_constraints: Vec<&dyn Constraint> = component_indices
                .iter()
                .map(|&i| constraints[i].as_ref())
                .collect();
            let result = self.solver.solve_dl(&mut param_manager, &component_constraints);
            match result {
                SolverResult::Converged {
                    iterations,
                    final_error,
                    initial_error,
                } => {
                    total_iters += iterations;
                    combined_final_error = combined_final_error.max(final_error);
                    combined_initial_error = combined_initial_error.max(initial_error);
                }
                SolverResult::MaxIterationsReached {
                    iterations,
                    final_error,
                    initial_error,
                } => {
                    total_iters += iterations;
                    combined_final_error = combined_final_error.max(final_error);
                    combined_initial_error = combined_initial_error.max(initial_error);
                    any_max_iter = true;
                }
            }
        }

        ParametricDogLegSolver::sync_geometry_from_parameters(
            &mut param_manager,
            &mut self.geometry,
        )?;

        if any_max_iter {
            Ok(SolverResult::MaxIterationsReached {
                iterations: total_iters,
                final_error: combined_final_error,
                initial_error: combined_initial_error,
            })
        } else {
            Ok(SolverResult::Converged {
                iterations: total_iters,
                final_error: combined_final_error,
                initial_error: combined_initial_error,
            })
        }
    }

    /// Returns the IDs of all geometry entities (points, circles, arcs) that are
    /// fully constrained (degrees of freedom = 0) at the current configuration.
    ///
    /// DOF is computed per connected component using a rank/null-space analysis of
    /// the analytical Jacobian. A free (non-fixed) parameter is considered *locked*
    /// when no infinitesimal motion in the constraint null space moves it. An entity
    /// is fully constrained when every one of its parameters is locked (fixed
    /// parameters count as locked). Note that a `Line` owns no parameters; callers
    /// should treat a line as fully constrained when both of its endpoints are.
    ///
    /// This should be evaluated at a solved (converged) configuration for the result
    /// to be meaningful.
    pub fn fully_constrained_entity_ids(&self) -> Vec<String> {
        use nalgebra::SymmetricEigen;

        let constraints = self.constraint_graph.get_constraints();
        let param_manager = build_param_manager(&self.geometry);
        let info = param_manager.get_parameter_info();
        let num_params = param_manager.num_parameters();

        // Track, per global parameter column, whether it is locked (0 DOF).
        // Fixed parameters are trivially locked. Free parameters that are never
        // touched by any constraint are movable (not locked).
        let mut locked = vec![false; num_params];
        for (i, pi) in info.iter().enumerate() {
            if pi.is_fixed {
                locked[i] = true;
            }
        }

        // Threshold for treating a singular/eigen value as zero (null space) and
        // for treating a null-space motion component as nonzero. Consistent with
        // the solver's 1e-10 convergence tolerance.
        const ZERO_EIG: f64 = 1e-9;
        const MOTION_EPS: f64 = 1e-7;

        let fixed_mask: Vec<bool> = info.iter().map(|pi| pi.is_fixed).collect();
        let components = find_components(&self.constraint_types);

        for component_indices in &components.components {
            let component_constraints: Vec<&dyn Constraint> = component_indices
                .iter()
                .map(|&i| constraints[i].as_ref())
                .collect();
            if component_constraints.is_empty() {
                continue;
            }

            let (_res, jac) = ParametricDogLegSolver::build_system(
                &param_manager,
                &component_constraints,
                &fixed_mask,
            );

            // Columns (parameters) actually referenced by this component's
            // constraints. Only these can be locked by this component; every
            // other column is left untouched (motion is free elsewhere).
            let jtj = jac.transpose() * &jac; // num_params x num_params, symmetric PSD
            let eig = SymmetricEigen::new(jtj);

            // Accumulate squared motion available in the null space for each column.
            let mut null_motion_sq = vec![0.0f64; num_params];
            for (k, &lambda) in eig.eigenvalues.iter().enumerate() {
                if lambda.abs() <= ZERO_EIG {
                    let v = eig.eigenvectors.column(k);
                    for r in 0..num_params {
                        null_motion_sq[r] += v[r] * v[r];
                    }
                }
            }

            // A free column touched by this component is locked if it has no
            // motion in the null space.
            for pi in info.iter() {
                if pi.is_fixed {
                    continue;
                }
                let col = pi.global_index;
                // Is this parameter's column referenced by the component Jacobian?
                let touched = jac.column(col).iter().any(|&x| x.abs() > 0.0);
                if touched && null_motion_sq[col].sqrt() <= MOTION_EPS {
                    locked[col] = true;
                }
            }
        }

        // An entity is fully constrained iff all of its parameter columns are locked.
        let mut per_entity: std::collections::BTreeMap<String, bool> = std::collections::BTreeMap::new();
        for pi in info.iter() {
            let entry = per_entity.entry(pi.entity_id.clone()).or_insert(true);
            *entry = *entry && locked[pi.global_index];
        }

        per_entity
            .into_iter()
            .filter_map(|(id, all_locked)| if all_locked { Some(id) } else { None })
            .collect()
    }

    pub fn get_point(&self, id: String) -> Option<&Point> {
        self.geometry.get_point(&id)
    }

    pub fn get_circle(&self, id: String) -> Option<&Circle> {
        self.geometry.get_circle(&id)
    }

    pub fn get_arc(&self, id: String) -> Option<&GeoArc> {
        self.geometry.get_arc(&id)
    }

    pub fn print_state(&self) {
        println!("Geometry System State:");
        for (id, point) in self.geometry.get_all_points() {
            println!("Point ID: {}, Position: ({}, {})", id, point.x, point.y);
        }
        for (id, line) in self.geometry.get_all_lines() {
            println!("Line ID: {}, Start: {}, End: {}", id, line.start, line.end);
        }
    }

    pub fn get_state_as_string(&self) -> String {
        let mut state = String::new();
        state.push_str("Geometry System State:\n");
        for (id, point) in self.geometry.get_all_points() {
            state.push_str(&format!(
                "Point ID: {}, Position: ({}, {})\n",
                id, point.x, point.y
            ));
        }
        for (id, line) in self.geometry.get_all_lines() {
            state.push_str(&format!(
                "Line ID: {}, Start: {}, End: {}\n",
                id, line.start, line.end
            ));
        }
        state
    }
}
