use crate::geometry::{Arc as GeoArc, Circle, Ellipse, EllipticalArc, Line};
use crate::sketch_system::{Role, SketchSystem};
pub use crate::sketch_system::Diagnosis;
use crate::{
    Constraint, ConstraintType, GeometrySystem, ParametricDogLegSolver, Point, check_vars,
    create_constraint,
};

#[derive(Debug)]
pub enum SolverResult {
    Converged {
        iterations: usize,
        final_error: f64,
        initial_error: f64,
    },
    MaxIterationsReached {
        iterations: usize,
        final_error: f64,
        initial_error: f64,
    },
}

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
    pub fn new() -> Self {
        Self {
            geometry: GeometrySystem::new(),
            constraints: Vec::new(),
            roles: Vec::new(),
            solver: ParametricDogLegSolver::new(),
        }
    }

    pub fn set_max_iterations(&mut self, max_iterations: usize) {
        self.solver.set_max_iterations(max_iterations);
    }

    pub fn add_point(&mut self, point: crate::geometry::Point) -> String {
        self.geometry.add_point(point)
    }

    pub fn add_circle(&mut self, circle: crate::geometry::Circle) -> String {
        self.geometry.add_circle(circle)
    }

    pub fn add_line(&mut self, line: Line) -> String {
        self.geometry.add_line(line)
    }

    pub fn add_arc(&mut self, arc: GeoArc) -> String {
        self.geometry.add_arc(arc)
    }

    pub fn add_ellipse(&mut self, ellipse: Ellipse) -> String {
        self.geometry.add_ellipse(ellipse)
    }

    pub fn add_elliptical_arc(&mut self, arc: EllipticalArc) -> String {
        self.geometry.add_elliptical_arc(arc)
    }

    /// Adds a constraint. Every entity it references must already be added,
    /// with the right kind (a point where a point is expected, and so on).
    /// Constraints are numbered in the order they're added (temporary ones
    /// included); [`Diagnosis`] reports them by that index.
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
        check_vars(constraint.as_ref(), &self.geometry)?;
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

    pub fn get_point(&self, id: String) -> Option<&Point> {
        self.geometry.get_point(&id)
    }

    pub fn get_circle(&self, id: String) -> Option<&Circle> {
        self.geometry.get_circle(&id)
    }

    pub fn get_arc(&self, id: String) -> Option<&GeoArc> {
        self.geometry.get_arc(&id)
    }

    pub fn get_ellipse(&self, id: String) -> Option<&Ellipse> {
        self.geometry.get_ellipse(&id)
    }

    pub fn get_elliptical_arc(&self, id: String) -> Option<&EllipticalArc> {
        self.geometry.get_elliptical_arc(&id)
    }

    pub fn print_state(&self) {
        println!("Geometry System State:");
        for (id, point) in self.geometry.get_all_points() {
            println!("Point ID: {}, Position: ({}, {})", id, point.x, point.y);
        }
        for (id, line) in self.geometry.get_all_lines() {
            println!("Line ID: {}, Start: {}, End: {}", id, line.start, line.end);
        }
    }

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
