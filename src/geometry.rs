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

#[derive(Debug)]
pub struct GeometrySystem {
    points: HashMap<String, Point>,
    lines: HashMap<String, Line>,
}

impl GeometrySystem {
    pub fn new() -> Self {
        Self {
            points: HashMap::new(),
            lines: HashMap::new(),
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

    pub fn update_point(&mut self, id: &str, point: Point) -> Result<(), String> {
        if !self.points.contains_key(id) {
            return Err("Point not found".to_string());
        }
        self.points.insert(id.to_string(), point);
        Ok(())
    }
}
