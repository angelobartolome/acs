// geometry.rs
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line {
    pub start: usize, // Point ID
    pub end: usize,   // Point ID
}

impl Line {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Debug)]
pub struct GeometrySystem {
    points: HashMap<usize, Point>,
    lines: HashMap<usize, Line>,
    next_point_id: usize,
    next_line_id: usize,
}

impl GeometrySystem {
    pub fn new() -> Self {
        Self {
            points: HashMap::new(),
            lines: HashMap::new(),
            next_point_id: 0,
            next_line_id: 0,
        }
    }

    pub fn add_point(&mut self, point: Point) -> usize {
        let id = self.next_point_id;
        self.points.insert(id, point);
        self.next_point_id += 1;
        id
    }

    pub fn add_line(&mut self, start_point: usize, end_point: usize) -> Result<usize, String> {
        if !self.points.contains_key(&start_point) || !self.points.contains_key(&end_point) {
            return Err("Invalid point IDs".to_string());
        }

        let id = self.next_line_id;
        let line = Line::new(start_point, end_point);
        self.lines.insert(id, line);
        self.next_line_id += 1;
        Ok(id)
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
