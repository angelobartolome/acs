use wasm_bindgen::prelude::wasm_bindgen;

use crate::ConstraintSolver;

#[wasm_bindgen(js_name = ConstraintSolver)]

pub struct WrappedConstraintSolver {
    inner: ConstraintSolver,
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

    pub fn reset(&mut self) -> Result<(), String> {
        self.inner = ConstraintSolver::new();
        Ok(())
    }

    pub fn solve(&mut self) -> Result<String, String> {
        match self.inner.solve() {
            Ok(result) => Ok(format!("{:?}", result)),
            Err(e) => Err(e),
        }
    }

    pub fn print_state(&self) -> String {
        self.inner.get_state_as_string()
    }

    pub fn get_point(&self, id: &str) -> Option<crate::Point> {
        self.inner.get_point(id.to_string()).cloned()
    }
    // Add more methods as needed
}
