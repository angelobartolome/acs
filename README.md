# ACS - Angelo's Constraint Solver (or Algorithmic Constraint Solver if you prefer)

A geometric constraint solver written in Rust with WebAssembly bindings for web applications.

## Overview

ACS (Algorithmic Constraint Solver) is a library for solving geometric constraint systems.

## Features

- **Geometric Primitives**:
  - [x] Points :white_check_mark:
  - [x] Lines :white_check_mark:
  - [ ] Arcs
  - [ ] Circles
- **Constraint Types**:
  - [x] Vertical constraints (force lines to be vertical) :white_check_mark:
  - [x] Horizontal constraints (force lines to be horizontal) :white_check_mark:
  - [x] Parallel constraints (force two lines to be parallel) :white_check_mark:
  - [ ] Perpendicular constraints (force two lines to be perpendicular)
  - [ ] Angle constraints (force lines to form a specific angle)
  - [ ] Dimension constraints (force lines/points to have specific lengths or distances)
- **Solvers**:
  - [ ] - Dog-Leg solver (combines Gauss-Newton and gradient descent) **WIP** :warning:
- **WebAssembly Support**: Compile to WASM for use in web applications **WIP** :warning:

## Installation

> [!NOTE]
> This library is not yet published to crates.io or npm. You can build it from source or use the local path in your project.

### As a Rust Crate

````
Add this to your `Cargo.toml`:

```toml
[dependencies]
acs = { path = "../path/to/acs" }
````

### WebAssembly Package

> [!IMPORTANT]
> The WebAssembly package is not yet published to npm. You can build it from source.

## Quick Start

### Rust Usage

```rust
use acs::{ConstraintSolver, ConstraintType, Point, SolverResult};

fn main() {
    // Create a new constraint solver
    let mut solver = ConstraintSolver::new();

    // Add points
    let p1 = solver.add_point(Point::new(0.0, 0.0));
    let p2 = solver.add_point(Point::new(1.0, 1.0));

    // Create a line between the points
    let line = solver.add_line(p1, p2).unwrap();

    // Add a vertical constraint to the line
    solver.add_constraint(ConstraintType::Vertical(line)).unwrap();

    // Solve the constraint system
    let result = solver.solve().unwrap();

    match result {
        SolverResult::Converged { iterations, final_error, .. } => {
            println!("Converged in {} iterations with error {}", iterations, final_error);
        }
        SolverResult::MaxIterationsReached { iterations, final_error, .. } => {
            println!("Max iterations ({}) reached with error {}", iterations, final_error);
        }
    }

    // Get the final positions
    let start = solver.get_point(p1).unwrap();
    let end = solver.get_point(p2).unwrap();
    println!("Line endpoints: ({}, {}) to ({}, {})", start.x, start.y, end.x, end.y);
}
```

## Building

### Prerequisites

- Rust 1.70 or later
- `wasm-pack` for WebAssembly builds

### Building the Rust Library

```bash
cargo build --release
```

### Building for WebAssembly

```bash
wasm-pack build --target web --out-dir pkg
```

## Testing

The project includes comprehensive tests:

```bash
# Run all tests
cargo test

# Run specific test
cargo test vertical_test

# Run with output
cargo test -- --nocapture
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
