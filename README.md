# ACS — Angelo's Constraint Solver

A geometric constraint solver written in Rust with WebAssembly bindings for web applications.

![Dragging points in the demo: tangent lines, a line tangent to an arc, and a curved slot](https://raw.githubusercontent.com/angelobartolome/acs/main/docs/demo.gif)

## Overview

ACS solves 2-D geometric constraint systems using a Dog-Leg (trust-region) numerical solver with analytical Jacobians. It ships as a Rust crate, a WASM module callable from JavaScript, and a static library behind a C ABI.

- **Components:** each sketch is split into independent connected components, each solved on its own. A built-in pre-solver skips any component whose constraints already hold, about 5–8× faster on the bundled drag benchmark (`cargo test --release --test presolver_bench -- --nocapture`).
- **Diagnosis:** every solve reports the **Conflicting** constraints (they can't all hold), the **Redundant** ones (they repeat what others already fix), the sketch's remaining **degrees of freedom**, and which entities are **fully constrained**.
- **Drags as soft goals:** constraints marked `temporary` (the cursor's position while dragging) are met as closely as the real constraints allow, and the real ones always hold exactly.
- **One type per relationship:** `distance`, `on`, `tangent`, … cover every combination of entities they make sense for; the variant is inferred from the entities referenced.

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
| Elliptical arcs (an ellipse's center, focus and minor radius, plus start and end points at parametric angles) | ✅ |

### Constraints

ACS names constraints by relationship (its **native vocabulary**, used by the JSON API `acsSolveSketch` and the C ABI `P3DSketch_Solve`). One type covers every combination of entities it makes sense for; the variant is inferred from the kinds of the entities its fields reference. Lines are segments: constraints measure against the segment unless they are **Extension** variants (`extension: true`), which measure against the infinite line through a line's endpoints so sketches authored under infinite-line semantics keep their meaning. `internal: true` selects the inside variant of tangency between circles and arcs, or of a distance to a circle or between circles and arcs.

| Type | Entities | Description |
|------|----------|-------------|
| `coincident` | point, point | Two points overlap |
| `horizontal` / `vertical` | line, or two points | Horizontal / vertical |
| `parallel` / `perpendicular` | line, line | Two lines are parallel / perpendicular |
| `collinear` | line, line | Both lines lie on one infinite line (the segments need not overlap) |
| `normal` | line, circle or arc | The line's Extension passes through the curve's center |
| `angle` | line, line; or arc | Directed angle between two lines; or an arc's sweep, `0 < value < 2π` (radians) |
| `direction` | line, or two points | Direction from +X (radians, counter-clockwise) |
| `distance` | point–point, point–line (`extension?`), point–circle (`internal?`), line–circle (`extension?`), circle–circle, circle–arc, arc–arc (`internal?`), line–line | Point–point distance; point–segment distance (with `extension`, to the line's Extension); the gap from a point, segment (or Extension) or circle to a circle's outside (with `internal`, inside it), and between arcs and circles the same way with the nearest points on each arc's span; two parallel lines `value` apart (both of `b`'s endpoints from `a`'s Extension) |
| `offset` | point, line | Distance from a point to a line's Extension on a given side (Linked Offsets) |
| `on` | point, line (`extension?`) / circle / arc / ellipse / elliptical arc | Point lies on the segment (or Extension), circle, arc's span, ellipse, or elliptical arc's span |
| `midpoint` | `entities`: [point, line], [line, line] (`extension?`) or [point, arc] | A point is the midpoint of a line; or the midpoint of one line lies on another line (or its Extension); or a point is the middle of an arc's span |
| `tangent` | line–circle (`extension?`), line–arc, line–ellipse, line–elliptical arc, arc–elliptical arc (at a shared endpoint), circle–circle, circle–arc, arc–arc (`internal?`) | Tangency on the segment and on each arc's span; circles and arcs touch externally, or one inside the other with `internal: true`; arcs sharing an endpoint are tangent at it |
| `concentric` | circle/arc, circle/arc | Share a center |
| `equal` | line–line, circle/arc–circle/arc, or values | Equal length, equal radius, or `a = b` |
| `radius` | circle or arc | Fixed radius |
| `diameter` | circle or arc | Fixed diameter |
| `length` | line or arc | Fixed length of a line, or of an arc (`radius × sweep`) |
| `difference` | values | `b − a = value` |
| `x` / `y` | point | Pin a point to a coordinate value |
| `mirror` | point, point, line | A point is another mirrored across a line's Extension |
| `rotation` | point, point, center point | A point is another rotated about a center by a given angle (circular array copy) |
| `translation` | point, point, two direction points | A point is another translated `distance × count` along a direction (linear array copy) |
| `ellipse_axis` | ellipse or elliptical arc, point or two points, `which` (`major`/`minor`) | A point is an end of an ellipse's major or minor axis; or two points are its opposite ends |

Values (for `equal` and `difference`) are constants (numbers or sketch Parameters) or entity properties the solver may move (a point's `x`/`y`, a circle's or arc's `radius`, an ellipse's or elliptical arc's `radmin`).

Internally each variant is a `ConstraintType` (`DistancePointLine`, `TangentExtensionCircle`, …) with an analytical Jacobian; see [SPEC.md](SPEC.md).

Sketch **Parameters** (JSON `param`) are named fixed values any scalar field may reference by id; see [USAGE.md](USAGE.md).

---

## Installation

### Rust crate

ACS is published on crates.io as [`acs-solver`](https://crates.io/crates/acs-solver) (the name `acs` is taken); the library itself is `acs`:

```toml
[dependencies]
acs-solver = "0.1"
```

```rust
use acs::{ConstraintSolver, ConstraintType, Point};
```

### WebAssembly package

Download `acs-<version>.tgz` from the [Releases](https://github.com/angelobartolome/acs/releases) page and install it (`npm install ./acs-<version>.tgz`), or build it:

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
solver.add_point(Point::new("p2".into(), 1.0, 1.0, false)); // free

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
println!("p2 = ({}, {})", p2.x, p2.y); // (3.54, 3.54): 5 from p1, along the way it started
println!("{} degrees of freedom left", solver.dof()); // 1: p2 can still circle p1
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

The WASM function takes a JSON string and returns a JSON string. The output `primitives` array mirrors the input with the solved values (`x`, `y`, `radius`, arc angles, an ellipse's `radmin`, an elliptical arc's `radmin` and angles), alongside `status`, `conflicting`, `redundant`, `dof`, `fullyConstrained` and `stats`.

To drag, add constraints marked `temporary: true` that hold the dragged geometry at the cursor, such as `{ type: "x", id: "gx", point: "p", value: 6, temporary: true }`. A point on a line, dragged off it, slides along the line to the closest point. See [USAGE.md](USAGE.md#drags-soft-goals).

`acsConstraintCatalog()` returns the constraint types and fields as JSON. For the full JSON input/output format, constraint type strings, and performance tips see [USAGE.md](USAGE.md).

---

## Building

### Prerequisites

- Rust 1.88+ (edition 2024)
- `wasm-pack` (for WASM builds)
- Xcode (for the iOS libraries and the XCFramework in `scripts/release.sh`)

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
// 0 = understood, non-zero = malformed
int  P3DSketch_Solve(const char *requestJson, char **responseJson);
void P3DSketch_Free(char *p); // releases *responseJson
```

The request and response are the [WASM JSON API](USAGE.md#wasm-json-api) contract, exactly as `acsSolveSketch` answers it. (Before 0.1.7 `P3DSketch_Solve` took a `vocabulary` argument between the request and the response pointer. The symbol name is unchanged, so nothing catches a caller built against the old header at link time: rebuild it against the new one.) `scripts/release.sh` builds macOS arm64, iOS arm64, iOS Simulator and Linux x86_64 archives (`dist/acs-<version>-<platform>.tar.gz`, unpacking to `acs/{include, lib/libp3d_sketch_solver.a, VERSION}`; a Linux program links it with `-lpthread -ldl -lm`), the Apple three as dynamic frameworks in one XCFramework (`dist/acs-<version>-xcframework.zip`, unpacking to `ACS.xcframework`, module `ACS`), and the WASM npm tarball (`dist/acs-<version>.tgz`).

An app on Apple platforms links the XCFramework rather than the static library. Rust's exception personality routine, beside Swift's, C++'s and Objective-C's, is one more than compact unwind can encode in one image; in a framework of its own it no longer counts against the app's three.

```bash
cargo build --release --features c-abi
./scripts/release.sh   # everything, in parallel: the archives, the XCFramework and dist/acs-<version>.tgz
./scripts/release.sh ios-arm64 wasm   # or only some pieces (macos-arm64, ios-arm64, ios-arm64-simulator, linux-x86_64, xcframework, wasm)
```

### Releases

Each release publishes the crate to crates.io (`acs-solver`) and prebuilt artifacts on the [Releases](https://github.com/angelobartolome/acs/releases) page: the static library for macOS arm64, iOS arm64, the iOS Simulator and Linux x86_64 (`acs-<version>-<platform>.tar.gz`), the Apple three as an XCFramework (`acs-<version>-xcframework.zip`), and the WASM npm package (`acs-<version>.tgz`, package `acs-solver`, installable with `npm install ./acs-<version>.tgz`).

To cut a release, set `version` in `Cargo.toml`, merge it, then push a matching tag:

```bash
git tag v0.1.4 && git push origin v0.1.4
```

The [Release workflow](.github/workflows/release.yml) checks that the tag matches the crate version, then runs in parallel jobs: clippy, the tests and `cargo publish --dry-run`; one macOS job per Apple platform (`scripts/release.sh <platform>`) and one for the XCFramework (`scripts/release.sh xcframework`); and on Linux the static library (`scripts/release.sh linux-x86_64`) and the WASM package (`scripts/release.sh wasm`). A last job collects the artifacts, publishes the GitHub Release, then publishes the crate with the `CARGO_REGISTRY_TOKEN` repository secret. Run it manually (Actions → Release → Run workflow) to build the artifacts without publishing.

---

### Demo

`wasm-demo/` is a React sketcher built on the native vocabulary, with example sketches (slots, a curved slot, a patterned polygon, …). It uses the WASM package in `pkg/`, so build that first and rebuild it after any Rust change:

```bash
wasm-pack build --target web --out-dir pkg
cd wasm-demo && npm install && npm run dev
```

---

## Testing

```bash
cargo test
cargo test --features c-abi   # C ABI tests, including a C smoke test (needs cc)
cd wasm-demo && npm test      # demo and examples (vitest)
```

Every constraint's analytical Jacobian is checked against finite differences (`tests/jacobian_fd_test.rs`).

---

## Documentation

| Doc | Contents |
|-----|----------|
| [USAGE.md](USAGE.md) | Full WASM JSON API reference, Rust API reference, pre-solver guide |
| [SPEC.md](SPEC.md) | Residual equations and analytical Jacobians for every constraint |
| [CONTEXT.md](CONTEXT.md) | Glossary: Component, Guide, Extension, Conflicting, Redundant, … |
| [AGENTS.md](AGENTS.md) | Architecture and how to add a constraint |

---

## Used by

ACS solves the sketches in [Part3D](https://part3d.app), a parametric 3D CAD app for iPad and Mac.

---

## License

MIT — see [LICENSE](LICENSE).
