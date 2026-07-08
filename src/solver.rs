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
