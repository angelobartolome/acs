use wasm_bindgen::prelude::wasm_bindgen;
use serde_json;

use crate::ConstraintSolver;
use crate::bindings::types::{
    PrimitiveJson, ConstraintJson, SolverRequest, SolverResponse, SolverResultJson,
};

#[wasm_bindgen(js_name = ConstraintSolver)]

pub struct WrappedConstraintSolver {
    inner: ConstraintSolver,
}

impl Default for WrappedConstraintSolver {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen(js_class = ConstraintSolver)]
impl WrappedConstraintSolver {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: ConstraintSolver::new(),
        }
    }

    // TODO: Those methods are during development, they will change to be more generic
    pub fn add_point(&mut self, point: &crate::Point) -> String {
        self.inner.add_point(point.clone())
    }

    pub fn add_circle(&mut self, circle: &crate::Circle) -> String {
        self.inner.add_circle(circle.clone())
    }

    pub fn add_vertical_constraint(
        &mut self,
        point_a_id: String,
        point_b_id: String,
    ) -> Result<(), String> {
        self.inner
            .add_constraint(crate::ConstraintType::Vertical(point_a_id, point_b_id))
            .map_err(|e| e.to_string())
    }

    pub fn add_horizontal_constraint(
        &mut self,
        point_a_id: String,
        point_b_id: String,
    ) -> Result<(), String> {
        self.inner
            .add_constraint(crate::ConstraintType::Horizontal(point_a_id, point_b_id))
            .map_err(|e| e.to_string())
    }

    pub fn add_parallel_constraint(
        &mut self,
        point_a_id: String,
        point_b_id: String,
        point_c_id: String,
        point_d_id: String,
    ) -> Result<(), String> {
        self.inner
            .add_constraint(crate::ConstraintType::Parallel(
                point_a_id, point_b_id, point_c_id, point_d_id,
            ))
            .map_err(|e| e.to_string())
    }

    pub fn add_point_on_line_constraint(
        &mut self,
        point_id: String,
        point_line_a_id: String,
        point_line_b_id: String,
    ) -> Result<(), String> {
        self.inner
            .add_constraint(crate::ConstraintType::PointOnLine(
                point_id,
                point_line_a_id,
                point_line_b_id,
            ))
            .map_err(|e| e.to_string())
    }

    pub fn add_equal_radius_constraint(
        &mut self,
        circle1_id: String,
        circle2_id: String,
    ) -> Result<(), String> {
        self.inner
            .add_constraint(crate::ConstraintType::EqualRadius(circle1_id, circle2_id))
            .map_err(|e| e.to_string())
    }

    pub fn reset(&mut self) -> Result<(), String> {
        self.inner = ConstraintSolver::new();
        Ok(())
    }

    pub fn solve(&mut self) -> Result<String, String> {
        match self.inner.solve() {
            Ok(result) => Ok(format!("{result:?}")),
            Err(e) => Err(e),
        }
    }

    pub fn print_state(&self) -> String {
        self.inner.get_state_as_string()
    }

    pub fn get_point(&self, id: &str) -> Option<crate::Point> {
        self.inner.get_point(id.to_string()).cloned()
    }

    pub fn get_circle(&self, id: &str) -> Option<crate::Circle> {
        self.inner.get_circle(id.to_string()).cloned()
    }

    // JSON-based methods for generic frontend interface
    pub fn add_primitives_json(&mut self, json: String) -> Result<String, String> {
        let primitives: Vec<PrimitiveJson> = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse primitives JSON: {e}"))?;

        for primitive in primitives {
            match primitive {
                PrimitiveJson::Point { id, x, y, fixed } => {
                    self.inner.add_point(crate::Point { id, x, y, fixed });
                }
                PrimitiveJson::Circle {
                    id,
                    center,
                    radius,
                    fixed,
                } => {
                    self.inner.add_circle(crate::Circle {
                        id,
                        center,
                        radius,
                        fixed,
                    });
                }
                PrimitiveJson::Line { id, start, end } => {
                    self.inner.add_line(crate::Line { id, start, end });
                }
                PrimitiveJson::Arc {
                    id,
                    center,
                    radius,
                    start_angle,
                    end_angle,
                    fixed,
                } => {
                    self.inner.add_arc(crate::Arc {
                        id,
                        center,
                        radius,
                        start_angle,
                        end_angle,
                        fixed,
                    });
                }
            }
        }

        Ok("Primitives added successfully".to_string())
    }

    pub fn add_constraints_json(&mut self, json: String) -> Result<String, String> {
        let constraints: Vec<ConstraintJson> = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse constraints JSON: {e}"))?;

        for constraint_json in constraints {
            let constraint_type: crate::ConstraintType = constraint_json
                .try_into()
                .map_err(|e| format!("Failed to convert constraint: {e}"))?;
            self.inner
                .add_constraint(constraint_type)
                .map_err(|e| format!("Failed to add constraint: {e}"))?;
        }

        Ok("Constraints added successfully".to_string())
    }

    pub fn solve_and_get_state_json(&mut self) -> Result<String, String> {
        // Solve first
        let solver_result = self.inner.solve().map_err(|e| e.to_string())?;

        // Collect all primitives
        let mut primitives = Vec::new();

        // Add all points
        for point in self.inner.get_all_points().values() {
            primitives.push(PrimitiveJson::from(point.clone()));
        }

        // Add all circles
        for circle in self.inner.get_all_circles().values() {
            primitives.push(PrimitiveJson::from(circle.clone()));
        }

        // Add all lines
        for line in self.inner.get_all_lines().values() {
            primitives.push(PrimitiveJson::from(line.clone()));
        }

        // Add all arcs
        for arc in self.inner.get_all_arcs().values() {
            primitives.push(PrimitiveJson::from(arc.clone()));
        }

        let response = SolverResponse {
            primitives,
            result: SolverResultJson::from(solver_result),
        };

        serde_json::to_string(&response)
            .map_err(|e| format!("Failed to serialize response: {e}"))
    }

    pub fn solve_from_json(&mut self, json: String) -> Result<String, String> {
        let request: SolverRequest = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse request JSON: {e}"))?;

        // Reset solver
        self.inner = ConstraintSolver::new();

        // Add all primitives
        for primitive in request.primitives {
            match primitive {
                PrimitiveJson::Point { id, x, y, fixed } => {
                    self.inner.add_point(crate::Point { id, x, y, fixed });
                }
                PrimitiveJson::Circle {
                    id,
                    center,
                    radius,
                    fixed,
                } => {
                    self.inner.add_circle(crate::Circle {
                        id,
                        center,
                        radius,
                        fixed,
                    });
                }
                PrimitiveJson::Line { id, start, end } => {
                    self.inner.add_line(crate::Line { id, start, end });
                }
                PrimitiveJson::Arc {
                    id,
                    center,
                    radius,
                    start_angle,
                    end_angle,
                    fixed,
                } => {
                    self.inner.add_arc(crate::Arc {
                        id,
                        center,
                        radius,
                        start_angle,
                        end_angle,
                        fixed,
                    });
                }
            }
        }

        // Add all constraints
        for constraint_json in request.constraints {
            let constraint_type: crate::ConstraintType = constraint_json
                .try_into()
                .map_err(|e| format!("Failed to convert constraint: {e}"))?;
            self.inner
                .add_constraint(constraint_type)
                .map_err(|e| format!("Failed to add constraint: {e}"))?;
        }

        // Solve
        let solver_result = self.inner.solve().map_err(|e| e.to_string())?;

        // Collect all primitives
        let mut primitives = Vec::new();

        // Add all points
        for point in self.inner.get_all_points().values() {
            primitives.push(PrimitiveJson::from(point.clone()));
        }

        // Add all circles
        for circle in self.inner.get_all_circles().values() {
            primitives.push(PrimitiveJson::from(circle.clone()));
        }

        // Add all lines
        for line in self.inner.get_all_lines().values() {
            primitives.push(PrimitiveJson::from(line.clone()));
        }

        // Add all arcs
        for arc in self.inner.get_all_arcs().values() {
            primitives.push(PrimitiveJson::from(arc.clone()));
        }

        let response = SolverResponse {
            primitives,
            result: SolverResultJson::from(solver_result),
        };

        serde_json::to_string(&response)
            .map_err(|e| format!("Failed to serialize response: {e}"))
    }
    // Add more methods as needed
}
