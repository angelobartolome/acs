//! ACS: a 2-D geometric constraint solver for CAD sketches.
//!
//! A sketch is geometry (points, lines, circles, arcs, ellipses and
//! elliptical arcs) and constraints between it (coincident, tangent,
//! distance, …): [geometric constraint
//! solving](https://en.wikipedia.org/wiki/Geometric_constraint_solving). ACS moves the geometry until
//! every constraint holds, and reports what it couldn't satisfy
//! (Conflicting), what was unnecessary (Redundant) and how many degrees of
//! freedom are left.
//!
//! Each constraint is a set of residuals with exact partial derivatives.
//! The sketch splits into independent groups ([`component_graph`]), each
//! solved by [Powell's dog leg method](https://en.wikipedia.org/wiki/Powell%27s_dog_leg_method)
//! ([`dogleg_solver`]); drags (temporary constraints) are minimized within
//! the real ones by a trust-region
//! [SQP](https://en.wikipedia.org/wiki/Sequential_quadratic_programming). Degrees of freedom and the
//! Conflicting/Redundant diagnosis come from the
//! [rank](https://en.wikipedia.org/wiki/Rank_(linear_algebra)) and [null
//! space](https://en.wikipedia.org/wiki/Kernel_(linear_algebra)) of each group's Jacobian. Lines are segments
//! and arcs are their spans: a point `on` a line stays between its ends
//! unless the constraint names the line's Extension.
//!
//! There are two ways in:
//!
//! - The **JSON API**, [`sketch_solve::solve_sketch_json`]: one request with
//!   every primitive, one response with the solved values and the
//!   diagnosis. It is what the WebAssembly export `acsSolveSketch` and the C
//!   ABI `P3DSketch_Solve` (feature `c-abi`) run. The constraint types and
//!   their fields are listed by [`constraint_catalog::catalog_json`]
//!   (`acsConstraintCatalog`); `USAGE.md` documents the contract.
//! - The **Rust API**, [`ConstraintSolver`]: add geometry and
//!   [`ConstraintType`]s by ID, then [`ConstraintSolver::solve`].
//!
//! ```
//! use acs::sketch_solve::solve_sketch_json;
//!
//! let request = r#"{ "version": 1, "primitives": [
//!     { "id": "a", "type": "point", "x": 0, "y": 0, "fixed": true },
//!     { "id": "b", "type": "point", "x": 4, "y": 1 },
//!     { "id": "l", "type": "line", "p1_id": "a", "p2_id": "b" },
//!     { "id": "k1", "type": "horizontal", "line": "l" },
//!     { "id": "k2", "type": "length", "curve": "l", "value": 10 }
//! ] }"#;
//! let response: serde_json::Value =
//!     serde_json::from_str(&solve_sketch_json(request).unwrap()).unwrap();
//! assert_eq!(response["status"], "converged");
//! assert_eq!(response["dof"], 0);
//! let b = &response["primitives"][1];
//! assert!((b["x"].as_f64().unwrap() - 10.0).abs() < 1e-9);
//! assert!(b["y"].as_f64().unwrap().abs() < 1e-9);
//! ```
//!
//! The same sketch through the Rust API:
//!
//! ```
//! use acs::{ConstraintSolver, ConstraintType, Line, Point, SolverResult};
//!
//! let mut cs = ConstraintSolver::new();
//! cs.add_point(Point::new("a".into(), 0.0, 0.0, true));
//! cs.add_point(Point::new("b".into(), 4.0, 1.0, false));
//! cs.add_line(Line::new("l".into(), "a".into(), "b".into()));
//! cs.add_constraint(ConstraintType::Horizontal("a".into(), "b".into())).unwrap();
//! cs.add_constraint(ConstraintType::DistancePointPoint("a".into(), "b".into(), 10.0)).unwrap();
//! assert!(matches!(cs.solve().unwrap(), SolverResult::Converged { .. }));
//! let b = cs.get_point("b".into()).unwrap();
//! assert!((b.x - 10.0).abs() < 1e-9 && b.y.abs() < 1e-9);
//! ```

#![warn(missing_docs)]

pub mod constraints;
pub mod geometry;
pub mod var_registry;
pub mod solver;

pub mod dogleg_solver;
pub mod sketch_solve;
pub mod component_graph;
mod sketch_system;
pub mod constraint_catalog;

// The sketch solver C ABI, for the static library.
#[cfg(feature = "c-abi")]
pub mod c_abi;

pub use constraints::*;
pub use dogleg_solver::*;
pub use geometry::*;
pub use var_registry::*;
pub use solver::*;

/// The WebAssembly exports (`wasm-bindgen`).
pub mod bindings {
    pub mod sketch_solve;
}
