use crate::var_registry::VarEntity;
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

impl VarEntity for Point {
    fn values(&self) -> Vec<f64> {
        vec![self.x, self.y]
    }

    fn set_values(&mut self, params: &[f64]) -> Result<(), String> {
        if params.len() != 2 {
            return Err(format!(
                "Point requires exactly 2 values, got {}",
                params.len()
            ));
        }
        self.x = params[0];
        self.y = params[1];
        Ok(())
    }

    fn var_names(&self) -> Vec<String> {
        vec![format!("{}.x", self.id), format!("{}.y", self.id)]
    }

    fn is_var_fixed(&self, index: usize) -> bool {
        match index {
            0 | 1 => self.fixed, // Both x and y are fixed if the point is fixed
            _ => true,           // Invalid variable indices are considered fixed
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

impl VarEntity for Circle {
    fn values(&self) -> Vec<f64> {
        vec![self.radius]
    }

    fn set_values(&mut self, params: &[f64]) -> Result<(), String> {
        if params.len() != 1 {
            return Err(format!(
                "Circle requires exactly 1 values, got {}",
                params.len()
            ));
        }
        self.radius = params[0];
        Ok(())
    }

    fn var_names(&self) -> Vec<String> {
        vec![format!("{}.radius", self.id)]
    }

    fn is_var_fixed(&self, index: usize) -> bool {
        match index {
            0 => self.fixed, // Radius is fixed if the circle is fixed
            _ => true,       // Invalid variable indices are considered fixed
        }
    }
}

/// An Arc: a span of a circle sweeping counter-clockwise from
/// `start_angle` to `end_angle`, referencing its center, start and end
/// Points. The endpoints always sit on it: the solver adds the arc's own
/// rules (an `ArcRules` over these Points) for every arc.
#[derive(Debug, Clone, PartialEq)]
#[wasm_bindgen(getter_with_clone)]
pub struct Arc {
    pub id: String,
    pub center: String, // Point ID
    /// Start Point ID; always at `start_angle`.
    pub start: String,
    /// End Point ID; always at `end_angle`.
    pub end: String,
    pub radius: f64,
    pub start_angle: f64, // in radians
    pub end_angle: f64,   // in radians
    pub fixed: bool,
}

#[wasm_bindgen]
impl Arc {
    #[wasm_bindgen(constructor)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        center: String,
        start: String,
        end: String,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
        fixed: bool,
    ) -> Self {
        Self {
            id,
            center,
            start,
            end,
            radius,
            start_angle,
            end_angle,
            fixed,
        }
    }
}

impl VarEntity for Arc {
    fn values(&self) -> Vec<f64> {
        vec![self.radius, self.start_angle, self.end_angle]
    }

    fn set_values(&mut self, params: &[f64]) -> Result<(), String> {
        if params.len() != 3 {
            return Err(format!(
                "Arc requires exactly 3 values, got {}",
                params.len()
            ));
        }
        self.radius = params[0];
        self.start_angle = params[1];
        self.end_angle = params[2];
        Ok(())
    }

    fn var_names(&self) -> Vec<String> {
        vec![
            format!("{}.radius", self.id),
            format!("{}.start_angle", self.id),
            format!("{}.end_angle", self.id),
        ]
    }

    fn is_var_fixed(&self, index: usize) -> bool {
        match index {
            0..=2 => self.fixed, // All variables are fixed if the arc is fixed
            _ => true,               // Invalid variable indices are considered fixed
        }
    }
}

/// An Ellipse: a center Point, a focus Point (`focus1`, which sets the
/// major axis direction) and its minor radius `radmin`, which it owns as a
/// Var. Its major radius is derived: `a = sqrt(radmin² + |focus1 − center|²)`.
#[derive(Debug, Clone, PartialEq)]
#[wasm_bindgen(getter_with_clone)]
pub struct Ellipse {
    pub id: String,
    pub center: String, // Point ID
    pub focus1: String, // Point ID
    pub radmin: f64,
    pub fixed: bool,
}

#[wasm_bindgen]
impl Ellipse {
    #[wasm_bindgen(constructor)]
    pub fn new(id: String, center: String, focus1: String, radmin: f64, fixed: bool) -> Self {
        Self {
            id,
            center,
            focus1,
            radmin,
            fixed,
        }
    }
}

impl VarEntity for Ellipse {
    fn values(&self) -> Vec<f64> {
        vec![self.radmin]
    }

    fn set_values(&mut self, params: &[f64]) -> Result<(), String> {
        if params.len() != 1 {
            return Err(format!(
                "Ellipse requires exactly 1 value, got {}",
                params.len()
            ));
        }
        self.radmin = params[0];
        Ok(())
    }

    fn var_names(&self) -> Vec<String> {
        vec![format!("{}.radmin", self.id)]
    }

    fn is_var_fixed(&self, index: usize) -> bool {
        match index {
            0 => self.fixed, // The minor radius is fixed if the ellipse is fixed
            _ => true,
        }
    }
}

#[derive(Debug)]
pub struct GeometrySystem {
    points: HashMap<String, Point>,
    lines: HashMap<String, Line>,
    circles: HashMap<String, Circle>,
    arcs: HashMap<String, Arc>,
    ellipses: HashMap<String, Ellipse>,
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
            ellipses: HashMap::new(),
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


    pub fn get_all_arcs(&self) -> &HashMap<String, Arc> {
        &self.arcs
    }

    pub fn get_all_arcs_mut(&mut self) -> &mut HashMap<String, Arc> {
        &mut self.arcs
    }

    pub fn add_ellipse(&mut self, ellipse: Ellipse) -> String {
        let id = ellipse.id.clone();
        self.ellipses.insert(id.clone(), ellipse);
        id
    }

    pub fn get_ellipse(&self, id: &str) -> Option<&Ellipse> {
        self.ellipses.get(id)
    }

    pub fn get_all_ellipses(&self) -> &HashMap<String, Ellipse> {
        &self.ellipses
    }

    pub fn get_all_ellipses_mut(&mut self) -> &mut HashMap<String, Ellipse> {
        &mut self.ellipses
    }




}
