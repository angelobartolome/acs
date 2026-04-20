# ACS — Angelo's Constraint Solver

A geometric constraint solver written in Rust with WebAssembly bindings for web applications.

![Demo of Horizontal and Vertical Constraints](/docs/demo.gif)

## Overview

ACS solves 2-D geometric constraint systems using a Dog-Leg (trust-region) numerical solver with analytical Jacobians. It ships as both a Rust crate and a WASM module callable from JavaScript.

A built-in pre-solver partitions each sketch into independent connected components and skips any component whose constraints are already satisfied — giving a 5–37× speedup on typical interactive edits.

---

## Features

### Geometric Primitives

| Primitive | Status |
|-----------|--------|
| Points | ✅ |
| Lines (two-point) | ✅ |
| Circles | ✅ |
| Arcs | ✅ |

### Constraints

| Constraint | Description |
|------------|-------------|
| `Horizontal` | Two points share the same Y coordinate |
| `Vertical` | Two points share the same X coordinate |
| `Parallel` | Two lines are parallel |
| `Perpendicular` | Two lines meet at 90° |
| `Angle` | Directed angle between two lines equals a target (radians) |
| `Coincident` | Two points overlap |
| `EqualX` / `EqualY` | Pin a point to a coordinate value |
| `PointOnLine` | Point lies on an infinite line |
| `DistancePointPoint` | Euclidean distance between two points |
| `DistancePointLine` | Perpendicular distance from a point to a line |
| `EqualLength` | Two line segments have the same length |
| `Midpoint` | A point is the midpoint of a segment |
| `Symmetric` | Two points are mirror images across an axis line |
| `FixedRadius` | Circle or arc has a fixed radius |
| `EqualRadius` | Two circles share the same radius |
| `Concentric` | Two circles share the same center |
| `PointOnCircle` | Point lies on a circle's circumference |
| `Tangent` | Two circles are externally tangent |
| `TangentLineCircle` | A line is tangent to a circle |

---

## Installation

> This library is not yet published to crates.io or npm. Build from source or reference it via a local path.

### Rust crate

```toml
[dependencies]
acs = { path = "../path/to/acs" }
```

### WebAssembly package

```bash
wasm-pack build --target web --out-dir pkg
```

---

## Quick Start

### Rust API

```rust
use acs::{ConstraintSolver, ConstraintType, Point, SolverResult};

let mut solver = ConstraintSolver::new();

solver.add_point(Point::new("p1".into(), 0.0, 0.0, true));  // fixed
solver.add_point(Point::new("p2".into(), 3.0, 4.0, false)); // free

solver.add_constraint(ConstraintType::DistancePointPoint(
    "p1".into(), "p2".into(), 5.0,
)).unwrap();

match solver.solve().unwrap() {
    SolverResult::Converged { iterations, final_error, .. } => {
        println!("converged in {iterations} iters, error={final_error:.2e}");
    }
    SolverResult::MaxIterationsReached { final_error, .. } => {
        println!("did not converge, error={final_error:.2e}");
    }
}

let p2 = solver.get_point("p2".into()).unwrap();
println!("p2 = ({}, {})", p2.x, p2.y);
```

### WASM / JavaScript API

```js
import init, { acsSolveSketch } from './pkg/acs.js';
await init();

const result = JSON.parse(acsSolveSketch(JSON.stringify({
  primitives: [
    { type: "point", id: "tl", x: 0,  y: 10, fixed: false },
    { type: "point", id: "tr", x: 10, y: 10, fixed: false },
    { type: "point", id: "bl", x: 0,  y: 0,  fixed: false },
    { type: "point", id: "br", x: 10, y: 0,  fixed: false },

    { type: "line", id: "top",    p1_id: "tl", p2_id: "tr" },
    { type: "line", id: "bottom", p1_id: "bl", p2_id: "br" },
    { type: "line", id: "left",   p1_id: "tl", p2_id: "bl" },
    { type: "line", id: "right",  p1_id: "tr", p2_id: "br" },

    { type: "coordinate_x", id: "cx_tl", p_id: "tl", x: 0  },
    { type: "coordinate_y", id: "cy_tl", p_id: "tl", y: 10 },
    { type: "p2p_distance",  id: "w", p1_id: "tl", p2_id: "tr", distance: 10 },
    { type: "p2p_distance",  id: "h", p1_id: "tl", p2_id: "bl", distance: 10 },
    { type: "horizontal_l",  id: "h_top",    l_id: "top"    },
    { type: "horizontal_l",  id: "h_bottom", l_id: "bottom" },
    { type: "vertical_l",    id: "v_left",   l_id: "left"   },
    { type: "vertical_l",    id: "v_right",  l_id: "right"  },
  ]
})));

if (result.ok) {
  const pts = Object.fromEntries(
    result.primitives
      .filter(p => p.type === 'point')
      .map(p => [p.id, { x: p.x, y: p.y }])
  );
  console.log(pts.tl, pts.tr, pts.bl, pts.br);
}
```

The WASM function takes a JSON string and returns a JSON string. The output `primitives` array mirrors the input with geometric parameters (`x`, `y`, `radius`) updated to the solved values.

For full JSON input/output format, constraint type strings, and performance tips see [USAGE.md](USAGE.md).

---

## Building

### Prerequisites

- Rust 1.70+
- `wasm-pack` (for WASM builds)

### Rust library

```bash
cargo build --release
```

### WebAssembly

```bash
wasm-pack build --target web --out-dir pkg
```

---

## Testing

```bash
cargo test
```

---

## Documentation

| Doc | Contents |
|-----|----------|
| [USAGE.md](USAGE.md) | Full WASM JSON API reference, Rust API reference, pre-solver guide |
| [SPEC.md](SPEC.md) | Residual equations and analytical Jacobians for every constraint |

---

## License

MIT — see [LICENSE](LICENSE).
