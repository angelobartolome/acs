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

// WebAssembly bindings
pub mod bindings {
    pub mod sketch_solve;
}
