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
    pub fn add_point(&mut self, point: &crate::Point) -> usize {
        self.inner.add_point(*point)
    }

    pub fn add_line(&mut self, line: &crate::Line) -> usize {
        self.inner.add_line(*line)
    }

    pub fn add_vertical_constraint(&mut self, line_id: usize) -> Result<(), String> {
        self.inner
            .add_constraint(crate::ConstraintType::Vertical(line_id))
    }

    pub fn add_horizontal_constraint(&mut self, line_id: usize) -> Result<(), String> {
        self.inner
            .add_constraint(crate::ConstraintType::Horizontal(line_id))
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

    pub fn get_point(&self, id: usize) -> Option<crate::Point> {
        self.inner.get_point(id).cloned()
    }
    // Add more methods as needed
}
