//! [`ConstraintSolver`], the Rust API: geometry and constraints by string
//! ID, solved in place.

use crate::constraints::{geometry_value, reads};
use crate::geometry::{Arc as GeoArc, Circle, CurveParam, Ellipse, EllipticalArc, Line, Spline};
use crate::sketch_system::{Role, SketchSystem};
pub use crate::sketch_system::Diagnosis;
use crate::{
    Constraint, ConstraintType, GeometrySystem, ParametricDogLegSolver, Point, check_vars,
    create_constraint,
};

/// How a solve ended. Errors are the largest residual (L-inf) over the real
/// (non-temporary) constraints; iterations are summed over Components.
#[derive(Debug)]
pub enum SolverResult {
    /// Every real constraint holds: the largest residual is below
    /// `TOLF` (1e-10).
    Converged {
        /// Iterations taken.
        iterations: usize,
        /// Largest residual at the end.
        final_error: f64,
        /// Largest residual at the start.
        initial_error: f64,
    },
    /// Some real constraint doesn't hold: the iterations ran out, or the
    /// solve stalled in a local minimum (the sketch may be conflicting).
    MaxIterationsReached {
        /// Iterations taken.
        iterations: usize,
        /// Largest residual at the end.
        final_error: f64,
        /// Largest residual at the start.
        initial_error: f64,
    },
}

/// A sketch's geometry and constraints, by string ID: add [`Point`]s and
/// the entities referencing them, then constraints ([`ConstraintType`]),
/// then [`Self::solve`], which moves the geometry in place; read the
/// results back with the getters, [`Self::dof`] and [`Self::diagnose`].
/// The JSON API ([`crate::sketch_solve::solve_sketch_json`]) runs on it.
pub struct ConstraintSolver {
    geometry: GeometrySystem,
    constraints: Vec<Box<dyn Constraint>>,
    /// Parallel to `constraints`: how each takes part in the solve.
    roles: Vec<Role>,
    solver: ParametricDogLegSolver,
}

impl Default for ConstraintSolver {
    fn default() -> Self {
        Self::new()
    }
}

impl ConstraintSolver {
    /// An empty sketch.
    pub fn new() -> Self {
        Self {
            geometry: GeometrySystem::new(),
            constraints: Vec::new(),
            roles: Vec::new(),
            solver: ParametricDogLegSolver::new(),
        }
    }

    /// Caps the iterations of each Component's solve (default 100).
    pub fn set_max_iterations(&mut self, max_iterations: usize) {
        self.solver.set_max_iterations(max_iterations);
    }

    /// Adds a point and returns its ID.
    pub fn add_point(&mut self, point: crate::geometry::Point) -> String {
        self.geometry.add_point(point)
    }

    /// Adds a circle (its center point first) and returns its ID.
    pub fn add_circle(&mut self, circle: crate::geometry::Circle) -> String {
        self.geometry.add_circle(circle)
    }

    /// Adds a line (its endpoints first) and returns its ID.
    pub fn add_line(&mut self, line: Line) -> String {
        self.geometry.add_line(line)
    }

    /// Adds an arc (its center and endpoints first) and returns its ID. The
    /// solver keeps the endpoints on it.
    pub fn add_arc(&mut self, arc: GeoArc) -> String {
        self.geometry.add_arc(arc)
    }

    /// Adds an ellipse (its center and focus first) and returns its ID.
    pub fn add_ellipse(&mut self, ellipse: Ellipse) -> String {
        self.geometry.add_ellipse(ellipse)
    }

    /// Adds an elliptical arc (its center, focus and endpoints first) and
    /// returns its ID. The solver keeps the endpoints on it.
    pub fn add_elliptical_arc(&mut self, arc: EllipticalArc) -> String {
        self.geometry.add_elliptical_arc(arc)
    }

    /// Adds a spline (its handle Points first) and returns its ID.
    pub fn add_spline(&mut self, spline: Spline) -> String {
        self.geometry.add_spline(spline)
    }

    /// Adds a constraint. Every entity it references must already be added,
    /// with the right kind (a point where a point is expected, and so on).
    /// Constraints are numbered in the order they're added (temporary ones
    /// included); [`Diagnosis`] reports them by that index. A constraint that
    /// owns curve parameters (`on`/`tangent` with a Spline) names each by an
    /// ID no entity has; this creates them, starting at the contact the
    /// constraint finds nearest in the current geometry.
    pub fn add_constraint(&mut self, constraint_type: ConstraintType) -> Result<(), String> {
        self.push_constraint(constraint_type, false)
    }

    /// Adds a temporary constraint, such as one holding a dragged point: a
    /// soft goal, met as closely as the real constraints allow without ever
    /// breaking them. [`Self::solve`]'s result is about the real constraints
    /// only; temporaries are also left out of [`Self::dof`],
    /// [`Self::fully_constrained_entity_ids`] and [`Self::diagnose`].
    pub fn add_temporary_constraint(&mut self, constraint_type: ConstraintType) -> Result<(), String> {
        self.push_constraint(constraint_type, true)
    }

    fn push_constraint(&mut self, constraint_type: ConstraintType, temporary: bool) -> Result<(), String> {
        let constraint = create_constraint(constraint_type)?;
        let curve_params: Vec<String> = constraint.curve_params().iter().map(|s| s.to_string()).collect();
        for (k, id) in curve_params.iter().enumerate() {
            if self.geometry.has_id(id) || curve_params[..k].contains(id) {
                return Err(format!("curve parameter '{id}' needs an ID no entity has"));
            }
        }
        for id in &curve_params {
            self.geometry.add_curve_param(CurveParam { id: id.clone(), value: 0.0 });
        }
        if let Err(e) = check_vars(constraint.as_ref(), &self.geometry) {
            for id in &curve_params {
                self.geometry.remove_curve_param(id);
            }
            return Err(e);
        }
        if !curve_params.is_empty() {
            let x: Vec<f64> = reads(constraint.as_ref())
                .iter()
                .map(|v| geometry_value(v, &self.geometry).unwrap_or(0.0))
                .collect();
            for (id, value) in curve_params.into_iter().zip(constraint.init_curve_params(&x)) {
                self.geometry.add_curve_param(CurveParam { id, value });
            }
        }
        self.constraints.push(constraint);
        self.roles.push(if temporary {
            Role::Temporary
        } else {
            Role::Real
        });
        Ok(())
    }

    fn system(&self) -> SketchSystem<'_> {
        SketchSystem::new(&self.geometry, &self.constraints, &self.roles)
    }

    /// Solves every Component of the sketch that isn't already solved and
    /// writes the results back into the geometry.
    pub fn solve(&mut self) -> Result<SolverResult, String> {
        let mut system = SketchSystem::new(&self.geometry, &self.constraints, &self.roles);
        let result = system.solve(&self.solver);
        system.write_back(&mut self.geometry)?;
        Ok(result)
    }

    /// IDs of the Points, Circles, Arcs and Ellipses that are fully constrained (zero
    /// degrees of freedom) at the current configuration, from a rank analysis
    /// of each Component's Jacobian. Fixed variables count as locked. A Line
    /// owns no variables; callers treat it as fully constrained when both
    /// endpoints are. Meaningful only at a solved configuration.
    pub fn fully_constrained_entity_ids(&self) -> Vec<String> {
        self.system().fully_constrained_entity_ids()
    }

    /// Degrees of freedom the constraints leave at the current configuration:
    /// free variables minus the rank of each Component's Jacobian.
    pub fn dof(&self) -> usize {
        self.system().dof()
    }

    /// Conflicting and Redundant constraints at the current configuration
    /// (call it after [`Self::solve`]), by constraint index. Temporary
    /// constraints are never reported. The rule is documented on
    /// `SketchSystem::diagnose`: dependent constraints carrying residual in
    /// an unsolved Component are Conflicting; in a Solved Component, a
    /// minimal set of dependent constraints, latest first, is Redundant.
    pub fn diagnose(&self) -> Diagnosis {
        self.system().diagnose()
    }

    /// The point with this ID.
    pub fn get_point(&self, id: String) -> Option<&Point> {
        self.geometry.get_point(&id)
    }

    /// The circle with this ID.
    pub fn get_circle(&self, id: String) -> Option<&Circle> {
        self.geometry.get_circle(&id)
    }

    /// The arc with this ID.
    pub fn get_arc(&self, id: String) -> Option<&GeoArc> {
        self.geometry.get_arc(&id)
    }

    /// The ellipse with this ID.
    pub fn get_ellipse(&self, id: String) -> Option<&Ellipse> {
        self.geometry.get_ellipse(&id)
    }

    /// The elliptical arc with this ID.
    pub fn get_elliptical_arc(&self, id: String) -> Option<&EllipticalArc> {
        self.geometry.get_elliptical_arc(&id)
    }

    /// IDs of the curve parameters constraint `index` owns, in its order
    /// (none for a constraint that owns none, or an index past the last).
    pub fn constraint_curve_params(&self, index: usize) -> Vec<String> {
        self.constraints
            .get(index)
            .map(|c| c.curve_params().into_iter().map(String::from).collect())
            .unwrap_or_default()
    }

    /// The current value of the curve parameter with this ID.
    pub fn curve_param(&self, id: &str) -> Option<f64> {
        self.geometry.get_curve_param(id).map(|p| p.value)
    }

    /// Sets a curve parameter's value, such as the one a previous solve
    /// reported, to start the next solve from that contact instead of the
    /// nearest one. Fails for an unknown ID.
    pub fn set_curve_param(&mut self, id: &str, value: f64) -> Result<(), String> {
        if self.geometry.get_curve_param(id).is_none() {
            return Err(format!("'{id}' is not a curve parameter"));
        }
        self.geometry.add_curve_param(CurveParam { id: id.to_string(), value });
        Ok(())
    }

    /// The spline with this ID.
    pub fn get_spline(&self, id: String) -> Option<&Spline> {
        self.geometry.get_spline(&id)
    }

    /// The curve of the spline with this ID, from its handles' current
    /// positions.
    pub fn spline_curve(&self, id: &str) -> Option<crate::spline::BSpline> {
        let s = self.geometry.get_spline(id)?;
        let handles: Option<Vec<[f64; 2]>> = s
            .points
            .iter()
            .map(|p| self.geometry.get_point(p).map(|p| [p.x, p.y]))
            .collect();
        crate::spline::BSpline::new(&handles?, s.interpolated, s.knots.as_deref()).ok()
    }

    /// Prints the points and lines (for debugging).
    pub fn print_state(&self) {
        println!("Geometry System State:");
        for (id, point) in self.geometry.get_all_points() {
            println!("Point ID: {}, Position: ({}, {})", id, point.x, point.y);
        }
        for (id, line) in self.geometry.get_all_lines() {
            println!("Line ID: {}, Start: {}, End: {}", id, line.start, line.end);
        }
    }

    /// The points and lines as text (for debugging).
    pub fn get_state_as_string(&self) -> String {
        let mut state = String::new();
        state.push_str("Geometry System State:\n");
        for (id, point) in self.geometry.get_all_points() {
            state.push_str(&format!(
                "Point ID: {}, Position: ({}, {})\n",
                id, point.x, point.y
            ));
        }
        for (id, line) in self.geometry.get_all_lines() {
            state.push_str(&format!(
                "Line ID: {}, Start: {}, End: {}\n",
                id, line.start, line.end
            ));
        }
        state
    }
}
