//! Sketch geometry: [`Point`]s, and the [`Line`]s, [`Circle`]s, [`Arc`]s,
//! [`Ellipse`]s and [`EllipticalArc`]s that reference them by ID, held in a
//! [`GeometrySystem`]. Each entity's own numbers (a point's coordinates, a
//! radius, angles) are solver variables ([`VarEntity`]) unless `fixed`.

use crate::var_registry::VarEntity;
use std::collections::HashMap;
use wasm_bindgen::prelude::wasm_bindgen;

/// A point, the only geometry with coordinates of its own: every other
/// entity references Points by ID (a line its endpoints, a circle its center).
#[derive(Debug, Clone, PartialEq)]
#[wasm_bindgen(getter_with_clone)]
pub struct Point {
    /// Unique ID; other geometry and constraints reference the point by it.
    pub id: String,
    /// x coordinate.
    pub x: f64,
    /// y coordinate.
    pub y: f64,
    /// Whether the solver holds the coordinates.
    pub fixed: bool,
}

#[wasm_bindgen]
impl Point {
    /// The entity with these values (IDs for the Points it references).
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

/// A Line: the segment between two Points (never infinite; constraints
/// that measure against the infinite line name its Extension). It owns no
/// variables.
#[derive(Debug, Clone, PartialEq)]
#[wasm_bindgen(getter_with_clone)]
pub struct Line {
    /// Unique ID.
    pub id: String,
    /// ID of the start Point (JSON `p1_id`).
    pub start: String,
    /// ID of the end Point (JSON `p2_id`).
    pub end: String,
}

#[wasm_bindgen]
impl Line {
    /// The entity with these values (IDs for the Points it references).
    #[wasm_bindgen(constructor)]
    pub fn new(id: String, start: String, end: String) -> Self {
        Self { id, start, end }
    }
}

/// A circle: a center Point and its radius, which it owns as a variable.
#[derive(Debug, Clone, PartialEq)]
#[wasm_bindgen(getter_with_clone)]
pub struct Circle {
    /// Unique ID.
    pub id: String,
    /// ID of the center Point.
    pub center: String,
    /// Radius.
    pub radius: f64,
    /// Whether the solver holds the radius (not the center, a Point of its own).
    pub fixed: bool,
}

#[wasm_bindgen]
impl Circle {
    /// The entity with these values (IDs for the Points it references).
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
/// rules (an `ArcRulesConstraint` over these Points) for every arc.
#[derive(Debug, Clone, PartialEq)]
#[wasm_bindgen(getter_with_clone)]
pub struct Arc {
    /// Unique ID.
    pub id: String,
    /// ID of the center Point.
    pub center: String,
    /// Start Point ID; always at `start_angle`.
    pub start: String,
    /// End Point ID; always at `end_angle`.
    pub end: String,
    /// Radius.
    pub radius: f64,
    /// Angle of the start Point around the center, in radians.
    pub start_angle: f64,
    /// Angle of the end Point, in radians; the arc sweeps counter-clockwise from `start_angle`.
    pub end_angle: f64,
    /// Whether the solver holds the radius and angles (not the Points).
    pub fixed: bool,
}

#[wasm_bindgen]
impl Arc {
    /// The entity with these values (IDs for the Points it references).
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

/// An [Ellipse](https://en.wikipedia.org/wiki/Ellipse): a center Point, a focus Point (`focus1`, which sets the
/// major axis direction) and its minor radius `radmin`, which it owns as a
/// Var. Its major radius is derived: `a = sqrt(radmin² + |focus1 − center|²)`.
#[derive(Debug, Clone, PartialEq)]
#[wasm_bindgen(getter_with_clone)]
pub struct Ellipse {
    /// Unique ID.
    pub id: String,
    /// ID of the center Point.
    pub center: String,
    /// ID of a focus Point; center → focus is the major axis.
    pub focus1: String,
    /// Minor radius.
    pub radmin: f64,
    /// Whether the solver holds `radmin` (not the Points).
    pub fixed: bool,
}

#[wasm_bindgen]
impl Ellipse {
    /// The entity with these values (IDs for the Points it references).
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

/// An arc of an Ellipse: the Ellipse's center Point, focus Point
/// (`focus1`) and minor radius `radmin`, plus start and end Points and
/// angles. The angles are the ellipse's own parametric angle `t` (its
/// [eccentric anomaly](https://en.wikipedia.org/wiki/Eccentric_anomaly), not the polar angle), measured
/// from its major axis (center → focus1) towards its minor axis
/// (`rot90`): the point at `t` is `c + a·cos t·u + b·sin t·n`. It sweeps
/// counter-clockwise from `start_angle` to `end_angle`, as an `Arc` does
/// (sweep `(end − start) mod 2π`, 0 = a full turn). It owns `radmin` and
/// both angles as Vars; `SketchSystem` adds implicit rules (an
/// `EllipticalArcRulesConstraint`) holding the endpoints at their angles.
#[derive(Debug, Clone, PartialEq)]
#[wasm_bindgen(getter_with_clone)]
pub struct EllipticalArc {
    /// Unique ID.
    pub id: String,
    /// ID of its ellipse's center Point.
    pub center: String,
    /// ID of a focus Point; center → focus is the major axis.
    pub focus1: String,
    /// Start Point ID; always at `start_angle`.
    pub start: String,
    /// End Point ID; always at `end_angle`.
    pub end: String,
    /// Minor radius.
    pub radmin: f64,
    /// Parametric angle of the start Point, in radians.
    pub start_angle: f64,
    /// Parametric angle of the end Point, in radians; the arc sweeps counter-clockwise from `start_angle`.
    pub end_angle: f64,
    /// Whether the solver holds `radmin` and the angles (not the Points).
    pub fixed: bool,
}

#[wasm_bindgen]
impl EllipticalArc {
    /// The entity with these values (IDs for the Points it references).
    #[wasm_bindgen(constructor)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        center: String,
        focus1: String,
        start: String,
        end: String,
        radmin: f64,
        start_angle: f64,
        end_angle: f64,
        fixed: bool,
    ) -> Self {
        Self {
            id,
            center,
            focus1,
            start,
            end,
            radmin,
            start_angle,
            end_angle,
            fixed,
        }
    }
}

/// Values in `Var` order: `MinorRadius` (0), `StartAngle` (1), `EndAngle`
/// (2), so Ellipse kernels read its `radmin` and Arc-style kernels its
/// angles.
impl VarEntity for EllipticalArc {
    fn values(&self) -> Vec<f64> {
        vec![self.radmin, self.start_angle, self.end_angle]
    }

    fn set_values(&mut self, params: &[f64]) -> Result<(), String> {
        if params.len() != 3 {
            return Err(format!(
                "EllipticalArc requires exactly 3 values, got {}",
                params.len()
            ));
        }
        self.radmin = params[0];
        self.start_angle = params[1];
        self.end_angle = params[2];
        Ok(())
    }

    fn var_names(&self) -> Vec<String> {
        vec![
            format!("{}.radmin", self.id),
            format!("{}.start_angle", self.id),
            format!("{}.end_angle", self.id),
        ]
    }

    fn is_var_fixed(&self, index: usize) -> bool {
        match index {
            0..=2 => self.fixed,
            _ => true,
        }
    }
}

/// Every entity of a sketch, by ID.
#[derive(Debug)]
pub struct GeometrySystem {
    points: HashMap<String, Point>,
    lines: HashMap<String, Line>,
    circles: HashMap<String, Circle>,
    arcs: HashMap<String, Arc>,
    ellipses: HashMap<String, Ellipse>,
    elliptical_arcs: HashMap<String, EllipticalArc>,
}

impl Default for GeometrySystem {
    fn default() -> Self {
        Self::new()
    }
}

impl GeometrySystem {
    /// An empty sketch.
    pub fn new() -> Self {
        Self {
            points: HashMap::new(),
            lines: HashMap::new(),
            circles: HashMap::new(),
            arcs: HashMap::new(),
            ellipses: HashMap::new(),
            elliptical_arcs: HashMap::new(),
        }
    }

    /// Adds a point (replacing one with the
    /// same ID) and returns its ID.
    pub fn add_point(&mut self, point: Point) -> String {
        let id = point.id.clone();
        self.points.insert(id.clone(), point);
        id
    }

    /// Adds a line (replacing one with the
    /// same ID) and returns its ID.
    pub fn add_line(&mut self, line: Line) -> String {
        let id = line.id.clone();
        self.lines.insert(id.clone(), line);
        id
    }

    /// The point with this ID.
    pub fn get_point(&self, id: &str) -> Option<&Point> {
        self.points.get(id)
    }



    /// All points, by ID.
    pub fn get_all_points(&self) -> &HashMap<String, Point> {
        &self.points
    }

    /// All points, by ID, to update in place.
    pub fn get_all_points_mut(&mut self) -> &mut HashMap<String, Point> {
        &mut self.points
    }

    /// All lines, by ID.
    pub fn get_all_lines(&self) -> &HashMap<String, Line> {
        &self.lines
    }

    /// Adds a circle (replacing one with the
    /// same ID) and returns its ID.
    pub fn add_circle(&mut self, circle: Circle) -> String {
        let id = circle.id.clone();
        self.circles.insert(id.clone(), circle);
        id
    }

    /// The circle with this ID.
    pub fn get_circle(&self, id: &str) -> Option<&Circle> {
        self.circles.get(id)
    }


    /// All circles, by ID.
    pub fn get_all_circles(&self) -> &HashMap<String, Circle> {
        &self.circles
    }

    /// All circles, by ID, to update in place.
    pub fn get_all_circles_mut(&mut self) -> &mut HashMap<String, Circle> {
        &mut self.circles
    }

    /// Adds an arc (replacing one with the
    /// same ID) and returns its ID.
    pub fn add_arc(&mut self, arc: Arc) -> String {
        let id = arc.id.clone();
        self.arcs.insert(id.clone(), arc);
        id
    }

    /// The arc with this ID.
    pub fn get_arc(&self, id: &str) -> Option<&Arc> {
        self.arcs.get(id)
    }


    /// All arcs, by ID.
    pub fn get_all_arcs(&self) -> &HashMap<String, Arc> {
        &self.arcs
    }

    /// All arcs, by ID, to update in place.
    pub fn get_all_arcs_mut(&mut self) -> &mut HashMap<String, Arc> {
        &mut self.arcs
    }

    /// Adds an ellipse (replacing one with the
    /// same ID) and returns its ID.
    pub fn add_ellipse(&mut self, ellipse: Ellipse) -> String {
        let id = ellipse.id.clone();
        self.ellipses.insert(id.clone(), ellipse);
        id
    }

    /// The ellipse with this ID.
    pub fn get_ellipse(&self, id: &str) -> Option<&Ellipse> {
        self.ellipses.get(id)
    }

    /// All ellipses, by ID.
    pub fn get_all_ellipses(&self) -> &HashMap<String, Ellipse> {
        &self.ellipses
    }

    /// All ellipses, by ID, to update in place.
    pub fn get_all_ellipses_mut(&mut self) -> &mut HashMap<String, Ellipse> {
        &mut self.ellipses
    }

    /// Adds an elliptical arc (replacing one with the
    /// same ID) and returns its ID.
    pub fn add_elliptical_arc(&mut self, arc: EllipticalArc) -> String {
        let id = arc.id.clone();
        self.elliptical_arcs.insert(id.clone(), arc);
        id
    }

    /// The elliptical arc with this ID.
    pub fn get_elliptical_arc(&self, id: &str) -> Option<&EllipticalArc> {
        self.elliptical_arcs.get(id)
    }

    /// All elliptical arcs, by ID.
    pub fn get_all_elliptical_arcs(&self) -> &HashMap<String, EllipticalArc> {
        &self.elliptical_arcs
    }

    /// All elliptical arcs, by ID, to update in place.
    pub fn get_all_elliptical_arcs_mut(&mut self) -> &mut HashMap<String, EllipticalArc> {
        &mut self.elliptical_arcs
    }




}
