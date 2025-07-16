// solver.rs
use crate::DogLegSolver;
use crate::constraints::base::{Constraint, ConstraintType, create_constraint};
use crate::geometry::GeometrySystem;
use std::collections::HashMap;

pub struct ConstraintGraph {
    constraints: Vec<Box<dyn Constraint>>,
    point_constraint_map: HashMap<usize, Vec<usize>>, // point_id -> constraint_indices
}

impl ConstraintGraph {
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
            point_constraint_map: HashMap::new(),
        }
    }

    pub fn add_constraint(
        &mut self,
        constraint_type: ConstraintType,
        geometry: &GeometrySystem,
    ) -> Result<(), String> {
        let constraint = create_constraint(constraint_type, geometry)?;
        let constraint_index = self.constraints.len();

        // Update point-constraint mapping
        if let Some(line_id) = self.get_line_id_from_constraint(&constraint) {
            if let Some(line) = geometry.get_line(line_id) {
                self.point_constraint_map
                    .entry(line.start)
                    .or_insert_with(Vec::new)
                    .push(constraint_index);
                self.point_constraint_map
                    .entry(line.end)
                    .or_insert_with(Vec::new)
                    .push(constraint_index);
            }
        }

        self.constraints.push(constraint);
        Ok(())
    }

    fn get_line_id_from_constraint(&self, constraint: &Box<dyn Constraint>) -> Option<usize> {
        // This is a bit hacky, but we need to extract line_id from constraint
        // In a real implementation, we'd have a better way to handle this
        match constraint.constraint_type() {
            "Vertical" | "Horizontal" => {
                // For now, we'll need to maintain this mapping elsewhere
                // or redesign the constraint interface
                None
            }
            _ => None,
        }
    }

    pub fn get_constraints_for_point(&self, point_id: usize) -> Vec<&Box<dyn Constraint>> {
        if let Some(constraint_indices) = self.point_constraint_map.get(&point_id) {
            constraint_indices
                .iter()
                .filter_map(|&index| self.constraints.get(index))
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn total_error(&self, geometry: &GeometrySystem) -> f64 {
        self.constraints
            .iter()
            .map(|constraint| {
                let error = constraint.error(geometry);
                error * error // Squared error
            })
            .sum()
    }

    pub fn get_all_constraints(&self) -> &Vec<Box<dyn Constraint>> {
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
            constraint_graph: ConstraintGraph::new(),
            solver: Box::new(DogLegSolver::new()),
        }
    }

    pub fn add_point(&mut self, point: crate::geometry::Point) -> usize {
        self.geometry.add_point(point)
    }

    pub fn add_line(&mut self, line: crate::geometry::Line) -> usize {
        self.geometry.add_line(line)
    }

    pub fn add_constraint(&mut self, constraint_type: ConstraintType) -> Result<(), String> {
        self.constraint_graph
            .add_constraint(constraint_type, &self.geometry)
    }

    pub fn solve(&mut self) -> Result<SolverResult, String> {
        self.solver
            .solve(&mut self.geometry, &self.constraint_graph)
    }

    pub fn move_point(
        &mut self,
        point_id: usize,
        new_position: crate::geometry::Point,
    ) -> Result<SolverResult, String> {
        self.geometry.update_point(point_id, new_position)?;
        self.solve()
    }

    pub fn get_point(&self, id: usize) -> Option<&crate::geometry::Point> {
        self.geometry.get_point(id)
    }

    pub fn get_line(&self, id: usize) -> Option<&crate::geometry::Line> {
        self.geometry.get_line(id)
    }

    pub fn get_constraint_error(&self) -> f64 {
        self.constraint_graph.total_error(&self.geometry)
    }

    pub fn print_state(&self) {
        println!("Points:");
        for (id, point) in self.geometry.get_all_points() {
            println!("  {}: ({:.6}, {:.6})", id, point.x, point.y);
        }

        println!("Lines:");
        for (id, line) in self.geometry.get_all_lines() {
            println!("  {}: {} -> {}", id, line.start, line.end);
        }

        println!("Total constraint error: {:.9}", self.get_constraint_error());
    }

    pub fn get_state_as_string(&self) -> String {
        let mut state = String::new();
        state.push_str("Points:\n");
        for (id, point) in self.geometry.get_all_points() {
            state.push_str(&format!("  {}: ({:.6}, {:.6})\n", id, point.x, point.y));
        }

        state.push_str("Lines:\n");
        for (id, line) in self.geometry.get_all_lines() {
            state.push_str(&format!("  {}: {} -> {}\n", id, line.start, line.end));
        }

        state.push_str(&format!(
            "Total constraint error: {:.9}\n",
            self.get_constraint_error()
        ));

        state
    }
}
