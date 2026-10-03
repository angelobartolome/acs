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
| Arcs (center, start and end points) | ✅ |
| Ellipses (center and focus points, minor radius) | ✅ |

### Constraints

ACS names constraints by relationship (its **native vocabulary**, used by the JSON API `acsSolveSketch`). One type covers every combination of entities it makes sense for; the variant is inferred from the kinds of the entities its fields reference. Lines are segments: constraints measure against the segment unless they are **Extension** variants (`extension: true`), which measure against the infinite line through a line's endpoints so sketches authored under infinite-line semantics, such as PlaneGCS's, keep their meaning.

| Type | Entities | Description |
|------|----------|-------------|
| `coincident` | point, point | Two points overlap |
| `horizontal` / `vertical` | line, or two points | Horizontal / vertical |
| `parallel` / `perpendicular` | line, line | Two lines are parallel / perpendicular |
| `angle` | line, line | Directed angle between two lines (radians) |
| `direction` | line, or two points | Direction from +X (radians, counter-clockwise) |
| `distance` | point, point or line (`extension?`) | Point–point distance; point–segment distance; with `extension`, distance to the line's Extension |
| `offset` | point, line | Distance from a point to a line's Extension on a given side (Linked Offsets) |
| `on` | point, line (`extension?`) / circle / arc / ellipse | Point lies on the segment (or Extension), circle, arc's span, or ellipse |
| `midpoint` | point, line | A point is the midpoint of a line |
| `midpoint_on` | line, line (`extension?`) | The midpoint of one line lies on another line (or its Extension) |
| `tangent` | line–circle (`extension?`), line–arc, circle–circle, line–ellipse | Tangency on the segment (and arc span); circles touch externally |
| `concentric` | circle/arc, circle/arc | Share a center |
| `equal` | line–line, circle/arc–circle/arc, or values | Equal length, equal radius, or `a = b` |
| `radius` | circle or arc | Fixed radius |
| `difference` | values | `b − a = value` |
| `x` / `y` | point | Pin a point to a coordinate value |
| `mirror` | point, point, line | A point is another mirrored across a line's Extension |
| `rotation` | point, point, center point | A point is another rotated about a center by a given angle (circular array copy) |
| `translation` | point, point, two direction points | A point is another translated `distance × count` along a direction (linear array copy) |
| `ellipse_axis` | ellipse, point, `which` (`major`/`minor`) | A point is an end of an ellipse's major or minor axis |

Values (for `equal` and `difference`) are constants (numbers or sketch Parameters) or entity properties the solver may move (a point's `x`/`y`, a circle's or arc's `radius`, an ellipse's `radmin`).

**PlaneGCS dialect.** Clients written against FreeCAD GCS send its type names (`p2p_distance`, `horizontal_l`, `tangent_lc`, …). Its entry points, the C ABI's `P3DSketch_Solve` and the WASM export `acsSolveSketchPlaneGcs`, accept those types unchanged and map each onto the same internal constraints; see [USAGE.md](USAGE.md#planegcs-dialect) for the mapping.

Internally each variant is a `ConstraintType` (`DistancePointLine`, `TangentExtensionCircle`, …) with an analytical Jacobian; see [SPEC.md](SPEC.md).

Sketch **Parameters** (JSON `param`) are named fixed values any scalar field may reference by id; see [USAGE.md](USAGE.md).

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
  version: 1,
  primitives: [
    { type: "point", id: "tl", x: 0,  y: 10, fixed: false },
    { type: "point", id: "tr", x: 10, y: 10, fixed: false },
    { type: "point", id: "bl", x: 0,  y: 0,  fixed: false },
    { type: "point", id: "br", x: 10, y: 0,  fixed: false },

    { type: "line", id: "top",    p1_id: "tl", p2_id: "tr" },
    { type: "line", id: "bottom", p1_id: "bl", p2_id: "br" },
    { type: "line", id: "left",   p1_id: "tl", p2_id: "bl" },
    { type: "line", id: "right",  p1_id: "tr", p2_id: "br" },

    { type: "x",          id: "cx_tl",    point: "tl", value: 0  },
    { type: "y",          id: "cy_tl",    point: "tl", value: 10 },
    { type: "distance",   id: "w",        a: "tl", b: "tr", value: 10 },
    { type: "distance",   id: "h",        a: "tl", b: "bl", value: 10 },
    { type: "horizontal", id: "h_top",    line: "top"    },
    { type: "horizontal", id: "h_bottom", line: "bottom" },
    { type: "vertical",   id: "v_left",   line: "left"   },
    { type: "vertical",   id: "v_right",  line: "right"  },
  ]
})));

if (result.status === 'converged') {
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

### C ABI (static library)

With the `c-abi` feature, the static library (`libacs.a`) exports the sketch solver C ABI, declared in [`include/p3d_sketch_solver.h`](include/p3d_sketch_solver.h):

```c
int  P3DSketch_Solve(const char *requestJson, char **responseJson); // 0 = understood, non-zero = malformed
void P3DSketch_Free(char *p);                                        // releases *responseJson
```

The request and response are the [WASM JSON API](USAGE.md#wasm-json-api) contract, with constraints in the [PlaneGCS dialect](USAGE.md#planegcs-dialect) (as `acsSolveSketchPlaneGcs` in WASM). `scripts/release-sketch-solver.sh` builds macOS arm64, iOS arm64 and iOS Simulator archives (`dist/sketch-solver-<version>-<platform>.tar.gz`, unpacking to `sketch-solver/{include, lib/libp3d_sketch_solver.a, VERSION}`) and the WASM npm tarball (`dist/acs-<version>.tgz`).

```bash
cargo build --release --features c-abi
./scripts/release-sketch-solver.sh   # SKETCH_SOLVER_VERSION=1.2.0 to override the version
```

---

## Testing

```bash
cargo test
cargo test --features c-abi   # C ABI tests, including a C smoke test (needs cc)
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
