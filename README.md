# ACS - Constraint Solver

A geometric constraint solver written in Rust with WebAssembly bindings for web applications.
This is me learning Rust, thus Angelo's Constraint Solver.
This project is a work in progress and is not yet feature-complete. Contributions are welcome!

## Overview

ACS is a library for solving geometric constraint systems.

![Demo of Horizontal and Vertical Constraints](/docs/demo.gif)

## Features

- **Geometric Primitives**:
  - [x] Points :white_check_mark:
  - [x] Lines :warning: (Removed temporarily, in favor of Point-based constraints)
  - [ ] Arcs
  - [x] Circles :white_check_mark:
- **Constraint Types**:
  - [x] Vertical constraints (force lines to be vertical) :white_check_mark:
  - [x] Horizontal constraints (force lines to be horizontal) :white_check_mark:
  - [x] Parallel constraints (force two lines to be parallel) :white_check_mark:
  - [x] Equal Y and X constraints (force points to have equal Y and X coordinates) :white_check_mark:
  - [x] Coincident constraints (force two points to be coincident) :white_check_mark:
  - [x] Point on line constraints (force a point to lie on a line) :white_check_mark:
  - [x] Equal Radius constraints (force circles to have equal radius) :white_check_mark:
  - [ ] Perpendicular constraints (force two lines to be perpendicular)
  - [ ] Angle constraints (force lines to form a specific angle)
  - [ ] Dimension constraints (force lines/points to have specific lengths or distances)
- **Solvers**:
  - [x] - Dog-Leg solver :white_check_mark:
- **WebAssembly Support**: Compile to WASM for use in web applications :white_check_mark:

## Installation

> [!NOTE]
> This library is not yet published to crates.io or npm. You can build it from source or use the local path in your project.

### As a Rust Crate

Add this to your `Cargo.toml`:

```toml
[dependencies]
acs = { path = "../path/to/acs" }
```

### WebAssembly Package

> [!IMPORTANT]
> The WebAssembly package is not yet published to npm. You can build it from source, check the [Building](#building) section for details.

## Quick Start

### Rust Usage

```rust
use acs::{ConstraintSolver, ConstraintType, Line, Point, SolverResult};

fn main() {
    // Create a new constraint solver
    let mut solver = ConstraintSolver::new();

    // Add points
    let p1 = Point::new(String::from("p1"), 0.0, 0.0, false);
    let p2 = Point::new(String::from("p2"), 1.0, 1.0, false);

    solver.add_point(p1);
    solver.add_point(p2);

    // Add a vertical constraint to the line
    solver
        .add_constraint(ConstraintType::Vertical("p1".into(), "p2".into()))
        .expect("Failed to add constraint");

    // Solve the constraint system
    let result = solver.solve().expect("Failed to solve constraint system");

    match result {
        SolverResult::Converged { iterations, final_error, .. } => {
            println!("Converged in {} iterations with error {}", iterations, final_error);
        }
        SolverResult::MaxIterationsReached { iterations, final_error, .. } => {
            println!("Max iterations ({}) reached with error {}", iterations, final_error);
        }
    }

    // Get the final positions
    let start = solver.get_point("p1".into()).expect("Point p1 should exist");
    let end = solver.get_point("p2".into()).expect("Point p2 should exist");
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
cargo test --test vertical_test

# Run with output
cargo test -- --nocapture
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
