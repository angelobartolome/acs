use crate::parameter_system::ParametricEntity;
use std::collections::HashMap;
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Debug, Clone, PartialEq)]
#[wasm_bindgen(getter_with_clone)]
pub struct Point {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub fixed: bool,
}

#[wasm_bindgen]
impl Point {
    #[wasm_bindgen(constructor)]
    pub fn new(id: String, x: f64, y: f64, fixed: bool) -> Self {
        Self { id, x, y, fixed }
    }
}

impl ParametricEntity for Point {
    fn get_parameters(&self) -> Vec<f64> {
        vec![self.x, self.y]
    }

    fn set_parameters(&mut self, params: &[f64]) -> Result<(), String> {
        if params.len() != 2 {
            return Err(format!(
                "Point requires exactly 2 parameters, got {}",
                params.len()
            ));
        }
        self.x = params[0];
        self.y = params[1];
        Ok(())
    }

    fn parameter_names(&self) -> Vec<String> {
        vec![format!("{}.x", self.id), format!("{}.y", self.id)]
    }

    fn is_parameter_fixed(&self, param_index: usize) -> bool {
        match param_index {
            0 | 1 => self.fixed, // Both x and y are fixed if the point is fixed
            _ => true,           // Invalid parameter indices are considered fixed
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[wasm_bindgen(getter_with_clone)]
pub struct Line {
    pub id: String,
    pub start: String, // Point ID
    pub end: String,   // Point ID
}

#[wasm_bindgen]
impl Line {
    #[wasm_bindgen(constructor)]
    pub fn new(id: String, start: String, end: String) -> Self {
        Self { id, start, end }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[wasm_bindgen(getter_with_clone)]
pub struct Circle {
    pub id: String,
    pub center: String, // Point ID
    pub radius: f64,
    pub fixed: bool,
}

#[wasm_bindgen]
impl Circle {
    #[wasm_bindgen(constructor)]
    pub fn new(id: String, center: String, radius: f64, fixed: bool) -> Self {
        Self {
            id,
            center,
            radius,
            fixed,
        }
    }
}

impl ParametricEntity for Circle {
    fn get_parameters(&self) -> Vec<f64> {
        vec![self.radius]
    }

    fn set_parameters(&mut self, params: &[f64]) -> Result<(), String> {
        if params.len() != 1 {
            return Err(format!(
                "Circle requires exactly 1 parameter, got {}",
                params.len()
            ));
        }
        self.radius = params[0];
        Ok(())
    }

    fn parameter_names(&self) -> Vec<String> {
        vec![format!("{}.radius", self.id)]
    }

    fn is_parameter_fixed(&self, param_index: usize) -> bool {
        match param_index {
            0 => self.fixed, // Radius is fixed if the circle is fixed
            _ => true,       // Invalid parameter indices are considered fixed
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[wasm_bindgen(getter_with_clone)]
pub struct Arc {
    pub id: String,
    pub center: String, // Point ID
    pub radius: f64,
    pub start_angle: f64, // in radians
    pub end_angle: f64,   // in radians
    pub fixed: bool,
}

#[wasm_bindgen]
impl Arc {
    #[wasm_bindgen(constructor)]
    pub fn new(
        id: String,
        center: String,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
        fixed: bool,
    ) -> Self {
        Self {
            id,
            center,
            radius,
            start_angle,
            end_angle,
            fixed,
        }
    }
}

impl ParametricEntity for Arc {
    fn get_parameters(&self) -> Vec<f64> {
        vec![self.radius, self.start_angle, self.end_angle]
    }

    fn set_parameters(&mut self, params: &[f64]) -> Result<(), String> {
        if params.len() != 3 {
            return Err(format!(
                "Arc requires exactly 3 parameters, got {}",
                params.len()
            ));
        }
        self.radius = params[0];
        self.start_angle = params[1];
        self.end_angle = params[2];
        Ok(())
    }

    fn parameter_names(&self) -> Vec<String> {
        vec![
            format!("{}.radius", self.id),
            format!("{}.start_angle", self.id),
            format!("{}.end_angle", self.id),
        ]
    }

    fn is_parameter_fixed(&self, param_index: usize) -> bool {
        match param_index {
            0..=2 => self.fixed, // All parameters are fixed if the arc is fixed
            _ => true,           // Invalid parameter indices are considered fixed
        }
    }
}

#[derive(Debug)]
pub struct GeometrySystem {
    points: HashMap<String, Point>,
    lines: HashMap<String, Line>,
    circles: HashMap<String, Circle>,
    arcs: HashMap<String, Arc>,
}

impl Default for GeometrySystem {
    fn default() -> Self {
        Self::new()
    }
}

impl GeometrySystem {
    pub fn new() -> Self {
        Self {
            points: HashMap::new(),
            lines: HashMap::new(),
            circles: HashMap::new(),
            arcs: HashMap::new(),
        }
    }

    pub fn add_point(&mut self, point: Point) -> String {
        let id = point.id.clone();
        self.points.insert(id.clone(), point);
        id
    }

    pub fn add_line(&mut self, line: Line) -> String {
        let id = line.id.clone();
        self.lines.insert(id.clone(), line);
        id
    }

    pub fn get_point(&self, id: &str) -> Option<&Point> {
        self.points.get(id)
    }

    pub fn get_point_mut(&mut self, id: &str) -> Option<&mut Point> {
        self.points.get_mut(id)
    }

    pub fn get_line(&self, id: &str) -> Option<&Line> {
        self.lines.get(id)
    }

    pub fn get_all_points(&self) -> &HashMap<String, Point> {
        &self.points
    }

    pub fn get_all_points_mut(&mut self) -> &mut HashMap<String, Point> {
        &mut self.points
    }

    pub fn get_all_lines(&self) -> &HashMap<String, Line> {
        &self.lines
    }

    pub fn add_circle(&mut self, circle: Circle) -> String {
        let id = circle.id.clone();
        self.circles.insert(id.clone(), circle);
        id
    }

    pub fn get_circle(&self, id: &str) -> Option<&Circle> {
        self.circles.get(id)
    }

    pub fn get_circle_mut(&mut self, id: &str) -> Option<&mut Circle> {
        self.circles.get_mut(id)
    }

    pub fn get_all_circles(&self) -> &HashMap<String, Circle> {
        &self.circles
    }

    pub fn get_all_circles_mut(&mut self) -> &mut HashMap<String, Circle> {
        &mut self.circles
    }

    pub fn add_arc(&mut self, arc: Arc) -> String {
        let id = arc.id.clone();
        self.arcs.insert(id.clone(), arc);
        id
    }

    pub fn get_arc(&self, id: &str) -> Option<&Arc> {
        self.arcs.get(id)
    }

    pub fn get_arc_mut(&mut self, id: &str) -> Option<&mut Arc> {
        self.arcs.get_mut(id)
    }

    pub fn get_all_arcs(&self) -> &HashMap<String, Arc> {
        &self.arcs
    }

    pub fn get_all_arcs_mut(&mut self) -> &mut HashMap<String, Arc> {
        &mut self.arcs
    }

    pub fn update_point(&mut self, id: &str, point: Point) -> Result<(), String> {
        if !self.points.contains_key(id) {
            return Err("Point not found".to_string());
        }
        self.points.insert(id.to_string(), point);
        Ok(())
    }

    pub fn update_circle(&mut self, id: &str, circle: Circle) -> Result<(), String> {
        if !self.circles.contains_key(id) {
            return Err("Circle not found".to_string());
        }
        self.circles.insert(id.to_string(), circle);
        Ok(())
    }

    pub fn update_arc(&mut self, id: &str, arc: Arc) -> Result<(), String> {
        if !self.arcs.contains_key(id) {
            return Err("Arc not found".to_string());
        }
        self.arcs.insert(id.to_string(), arc);
        Ok(())
    }
}
