use std::collections::HashMap;
use wasm_bindgen::prelude::wasm_bindgen;
#[derive(Debug, Clone, Copy, PartialEq)]
#[wasm_bindgen]
pub struct Point {
    pub id: usize,
    pub x: f64,
    pub y: f64,
}

#[wasm_bindgen]
impl Point {
    #[wasm_bindgen(constructor)]
    pub fn new(id: usize, x: f64, y: f64) -> Self {
        Self { id, x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[wasm_bindgen]
pub struct Line {
    pub id: usize,
    pub start: usize, // Point ID
    pub end: usize,   // Point ID
}

#[wasm_bindgen]
impl Line {
    #[wasm_bindgen(constructor)]
    pub fn new(id: usize, start: usize, end: usize) -> Self {
        Self { id, start, end }
    }
}

#[derive(Debug)]
pub struct GeometrySystem {
    points: HashMap<usize, Point>,
    lines: HashMap<usize, Line>,
}

impl GeometrySystem {
    pub fn new() -> Self {
        Self {
            points: HashMap::new(),
            lines: HashMap::new(),
        }
    }

    pub fn add_point(&mut self, point: Point) -> usize {
        self.points.insert(point.id, point);
        point.id
    }

    pub fn add_line(&mut self, line: Line) -> usize {
        self.lines.insert(line.id, line);
        line.id
    }

    pub fn get_point(&self, id: usize) -> Option<&Point> {
        self.points.get(&id)
    }

    pub fn get_point_mut(&mut self, id: usize) -> Option<&mut Point> {
        self.points.get_mut(&id)
    }

    pub fn get_line(&self, id: usize) -> Option<&Line> {
        self.lines.get(&id)
    }

    pub fn get_all_points(&self) -> &HashMap<usize, Point> {
        &self.points
    }

    pub fn get_all_lines(&self) -> &HashMap<usize, Line> {
        &self.lines
    }

    pub fn update_point(&mut self, id: usize, point: Point) -> Result<(), String> {
        if !self.points.contains_key(&id) {
            return Err("Point not found".to_string());
        }
        self.points.insert(id, point);
        Ok(())
    }
}
