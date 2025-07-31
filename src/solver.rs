use crate::{Constraint, ConstraintType, DogLegSolver, GeometrySystem, Point, create_constraint};

pub struct ConstraintGraph {
    constraints: Vec<Box<dyn Constraint>>,
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
    solver: Box<dyn Solver>,
}

impl ConstraintSolver {
    pub fn new() -> Self {
        Self {
            geometry: GeometrySystem::new(),
            constraint_graph: ConstraintGraph {
                constraints: Vec::new(),
            },
            solver: Box::new(DogLegSolver::new()),
        }
    }

    pub fn add_point(&mut self, point: crate::geometry::Point) -> String {
        self.geometry.add_point(point)
    }

    pub fn add_line(&mut self, line: crate::geometry::Line) -> String {
        self.geometry.add_line(line)
    }

    pub fn add_constraint(&mut self, constraint_type: ConstraintType) -> Result<(), String> {
        self.constraint_graph
            .constraints
            .push(create_constraint(constraint_type)?);
        Ok(())
    }

    pub fn solve(&mut self) -> Result<SolverResult, String> {
        self.solver
            .solve(&mut self.geometry, &self.constraint_graph)
    }

    pub fn get_point(&self, id: String) -> Option<&Point> {
        self.geometry.get_point(&id)
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
