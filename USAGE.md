# ACS Frontend Usage Guide

## Overview

ACS exposes two integration paths:

| Path | Use when |
|------|----------|
| **WASM JSON API** (`acsSolveSketch`) | Browser / JS frontend — send a JSON description of the sketch, receive updated positions back |
| **Rust native API** (`ConstraintSolver`) | Server-side Rust or embedding ACS as a Rust crate |

Both paths run the same dogleg solver with the same pre-solver optimization. The sections below cover each path and explain how to get the most out of the pre-solver.

---

## WASM JSON API

### Loading the module

```js
import init, { acsSolveSketch } from './pkg/acs.js';
await init();
```

### Calling the solver

```js
const response = JSON.parse(acsSolveSketch(JSON.stringify(sketchInput)));
```

`acsSolveSketch` takes a JSON string and returns a JSON string.

### Input format

```json
{
  "primitives": [ ...geometry..., ...constraints... ],
  "max_iterations": 100
}
```

`max_iterations` is optional (default 100). The array can also be passed bare (without the wrapper object).

All primitives live in one flat array — geometry first, then constraints, though order within each group does not matter.

#### Geometry primitives

```json
{ "type": "point",  "id": "p1", "x": 0.0, "y": 0.0, "fixed": false }
{ "type": "line",   "id": "l1", "p1_id": "p1", "p2_id": "p2" }
{ "type": "circle", "id": "c1", "c_id": "center1", "radius": 5.0, "fixed": false }
{ "type": "arc",    "id": "a1", "c_id": "center1", "radius": 5.0,
                    "start_angle": 0.0, "end_angle": 1.5708, "fixed": false }
```

- A **line** has no parameters of its own — it is fully defined by its two endpoint points.
- A **circle** stores only its radius; the center position is a separate `point` referenced by `c_id`.
- `fixed: true` pins an entity; the solver never moves it.

#### Constraint primitives

Every constraint primitive needs a unique `id` and a `type` string. The full list of supported types:

| Type string | Parameters | Meaning |
|---|---|---|
| `horizontal_pp` | `p1_id`, `p2_id` | Two points on same horizontal line |
| `vertical_pp` | `p1_id`, `p2_id` | Two points on same vertical line |
| `horizontal_l` | `l_id` | Line is horizontal |
| `vertical_l` | `l_id` | Line is vertical |
| `parallel` | `l1_id`, `l2_id` | Two lines are parallel |
| `perpendicular_ll` | `l1_id`, `l2_id` | Two lines are perpendicular |
| `perpendicular_pppp` | `l1p1_id`, `l1p2_id`, `l2p1_id`, `l2p2_id` | Same, expressed as four points |
| `p2p_coincident` | `p1_id`, `p2_id` | Two points overlap |
| `point_on_line_pl` | `p_id`, `l_id` | Point lies on a line |
| `point_on_line_ppp` | `p_id`, `lp1_id`, `lp2_id` | Same, line expressed as two points |
| `point_on_circle` | `p_id`, `c_id` | Point lies on circle circumference |
| `p2p_distance` | `p1_id`, `p2_id`, `distance` | Fixed Euclidean distance |
| `p2l_distance` | `p_id`, `l_id`, `distance` | Fixed perpendicular distance from point to line |
| `l2l_angle_ll` | `l1_id`, `l2_id`, `angle` | Directed angle in radians from L1 to L2 |
| `l2l_angle_pppp` | `l1p1_id`, `l1p2_id`, `l2p1_id`, `l2p2_id`, `angle` | Same, four-point form |
| `equal_length` | `l1_id`, `l2_id` | Two segments have equal length |
| `equal_radius_cc` | `c1_id`, `c2_id` | Two circles share radius |
| `circle_radius` | `c_id`, `radius` | Circle has a fixed radius |
| `arc_radius` | `a_id`, `radius` | Arc has a fixed radius |
| `tangent_lc` | `l_id`, `c_id` | Line is tangent to circle |
| `coordinate_x` | `p_id`, `x` | Point pinned to an x-coordinate |
| `coordinate_y` | `p_id`, `y` | Point pinned to a y-coordinate |
| `midpoint_on_line_ll` | `l1_id`, `l2_id` | Midpoint of L1 lies on L2 |
| `midpoint_on_line_pppp` | `l1p1_id`, `l1p2_id`, `l2p1_id`, `l2p2_id` | Same, four-point form |
| `p2p_symmetric_ppl` | `p1_id`, `p2_id`, `l_id` | Two points symmetric about a line |
| `p2p_symmetric_ppp` | `p1_id`, `p2_id`, `p_id` | Two points symmetric about a point |

### Output format

```json
{
  "ok": true,
  "status": "converged",
  "solveStatus": 1,
  "primitives": [ ...same array with updated x/y/radius/angles... ],
  "skipped_constraint_ids": [],
  "conflicting_constraint_ids": [],
  "error": null
}
```

| Field | Values |
|---|---|
| `ok` | `true` only when `status` is `"converged"` |
| `status` | `"converged"` or `"failed"` |
| `solveStatus` | `1` = converged, `2` = failed |
| `primitives` | The original array with geometric parameters updated in-place |
| `skipped_constraint_ids` | Constraints the solver could not interpret (unsupported type, missing fields) |
| `error` | `null` on success; error message string on failure |

### Full example: a constrained square

```js
const result = JSON.parse(acsSolveSketch(JSON.stringify({
  primitives: [
    // Points
    { type: "point", id: "tl", x: 0,  y: 10, fixed: false },
    { type: "point", id: "tr", x: 10, y: 10, fixed: false },
    { type: "point", id: "bl", x: 0,  y: 0,  fixed: false },
    { type: "point", id: "br", x: 10, y: 0,  fixed: false },

    // Lines
    { type: "line", id: "top",    p1_id: "tl", p2_id: "tr" },
    { type: "line", id: "bottom", p1_id: "bl", p2_id: "br" },
    { type: "line", id: "left",   p1_id: "tl", p2_id: "bl" },
    { type: "line", id: "right",  p1_id: "tr", p2_id: "br" },

    // Constraints: pin top-left corner, fix size to 10×10
    { type: "coordinate_x", id: "cx_tl", p_id: "tl", x: 0  },
    { type: "coordinate_y", id: "cy_tl", p_id: "tl", y: 10 },
    { type: "p2p_distance",  id: "w",    p1_id: "tl", p2_id: "tr", distance: 10 },
    { type: "p2p_distance",  id: "h",    p1_id: "tl", p2_id: "bl", distance: 10 },
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
  // pts.tl, pts.tr, pts.bl, pts.br now hold solved positions
}
```

---

## Taking advantage of the pre-solver

### What the pre-solver does

Before running any numerical iteration, ACS partitions the sketch into **connected components** — groups of entities linked by shared constraints. A square with 4 corners and 8 constraints forms one component; an unrelated free line forms another.

Any component whose constraints are already satisfied (residuals ≤ 1e-10) is **skipped entirely** — no Jacobian assembly, no iteration. Only the components that actually need work are passed to the dogleg solver.

### The rule: pass current positions in every call

The pre-solver makes decisions based on the positions you send. If a component is already solved and you send its current (correct) positions, it is skipped. If you always send default/zeroed positions, every component looks unsolved on every call.

**Always store the solver output and feed it back as input on the next call.**

```js
// sketch state lives in your app
let sketchState = buildInitialSketch();

// initial solve
let result = JSON.parse(acsSolveSketch(JSON.stringify({ primitives: sketchState })));
// store solved positions back
sketchState = result.primitives;

// user drags a free line endpoint — update only those two points in sketchState
sketchState = sketchState.map(p => {
  if (p.id === 'line_end') return { ...p, x: newX, y: newY };
  return p;                            // square points keep their solved values
});

// re-solve — the square component is already satisfied, only the line is solved
result = JSON.parse(acsSolveSketch(JSON.stringify({ primitives: sketchState })));
sketchState = result.primitives;
```

The square's 8 constraints have residuals of ~0 because the positions are already correct — the pre-solver detects this immediately and skips the entire component. Only the single line constraint runs through the dogleg. This is what drives the ~5–37× speedup measured in the benchmark.

### What counts as a component

Two constraints are in the same component if they share at least one entity ID. Entities connect transitively: if constraint A touches point `p2` and constraint B also touches `p2`, A and B (and all their other entities) are in the same component.

A fully-constrained square has all four points wired together → one component. A free line whose endpoints are not referenced by any square constraint → separate component. Dragging the line only re-solves the line's component.

### Breaking a large sketch into independent regions

If your sketch has multiple independent figures (two squares, a circle and a polygon), make sure they share no entity IDs and no constraints that cross between them. Each figure becomes its own component and is solved independently. If one figure is fully constrained and the other is not, only the second one runs.

Avoid connecting unrelated figures with "bridge" constraints unless the geometry actually requires it — every bridge merges two components, forcing both to be re-solved together on every change.

---

## Rust native API

Use this when embedding ACS in a Rust application or running server-side solving.

```rust
use acs::{ConstraintSolver, ConstraintType, Point, SolverResult};
use acs::geometry::{Circle, Line};

let mut solver = ConstraintSolver::new();

// Add geometry
solver.add_point(Point::new("p1".into(), 0.0, 0.0, true));   // fixed
solver.add_point(Point::new("p2".into(), 3.0, 4.0, false));  // free

// Add constraints
solver.add_constraint(ConstraintType::DistancePointPoint(
    "p1".into(), "p2".into(), 5.0,
)).unwrap();

// Solve
match solver.solve().unwrap() {
    SolverResult::Converged { iterations, final_error, .. } => {
        println!("converged in {iterations} iters, error={final_error:.2e}");
    }
    SolverResult::MaxIterationsReached { final_error, .. } => {
        println!("did not converge, error={final_error:.2e}");
    }
}

// Read results
let p2 = solver.get_point("p2".into()).unwrap();
println!("p2 = ({}, {})", p2.x, p2.y);  // distance 5 from origin
```

### Reusing the solver across interactions

For an interactive application, keep one `ConstraintSolver` alive for the lifetime of the sketch instead of rebuilding it on every change. Update the dragged point's position, then call `solve()` again:

```rust
// initial setup
let mut solver = ConstraintSolver::new();
// ... add_point / add_line / add_constraint calls ...
solver.solve().unwrap();

// user moves a point → update in your own data model, rebuild if needed
// Currently ConstraintSolver does not expose a mutable get_point_mut(),
// so for drag interactions use the JSON API or rebuild with updated positions.
```

> The JSON API is currently the recommended path for drag interactions because it handles position updates cleanly. The Rust API is ideal for batch solving or when constraints change between solves.

### Configuration

```rust
solver.set_max_iterations(200);  // default 100
```

The solver converges when the L-infinity norm of all residuals drops below **1e-10**. This threshold is also what the pre-solver uses to determine whether a component is already satisfied.

---

## Constraint reference (Rust API)

All constraints in the Rust API use `ConstraintType` variants. String arguments are entity IDs; scalar arguments are `f64`.

```rust
// Alignment
ConstraintType::Horizontal(p1_id, p2_id)
ConstraintType::Vertical(p1_id, p2_id)

// Coincidence / position
ConstraintType::Coincident(p1_id, p2_id)
ConstraintType::EqualX(point_id, x_value)
ConstraintType::EqualY(point_id, y_value)

// Distance
ConstraintType::DistancePointPoint(p1_id, p2_id, distance)
ConstraintType::DistancePointLine(point_id, line_p1_id, line_p2_id, distance)

// Line relationships
ConstraintType::Parallel(l1p1, l1p2, l2p1, l2p2)
ConstraintType::Perpendicular(l1p1, l1p2, l2p1, l2p2)
ConstraintType::Angle(l1p1, l1p2, l2p1, l2p2, radians)
ConstraintType::EqualLength(l1p1, l1p2, l2p1, l2p2)

// Point on geometry
ConstraintType::PointOnLine(point_id, line_p1_id, line_p2_id)
ConstraintType::PointOnCircle(point_id, circle_center_id, circle_id)

// Circle / arc
ConstraintType::FixedRadius(circle_or_arc_id, radius)
ConstraintType::EqualRadius(circle1_id, circle2_id)
ConstraintType::Concentric(center1_point_id, center2_point_id)
ConstraintType::TangentLineCircle(line_p1_id, line_p2_id, circle_center_id, circle_id)
ConstraintType::Tangent(c1_center_id, c1_id, c2_center_id, c2_id)  // circle-circle

// Symmetry
ConstraintType::Symmetric(p_id, q_id, axis_p1_id, axis_p2_id)
ConstraintType::Midpoint(midpoint_id, endpoint_a_id, endpoint_b_id)
ConstraintType::MidpointOfLineOnLine(l1p1, l1p2, l2p1, l2p2)
```

---

## Common mistakes

**Sending zero/default positions on every call.** If your frontend stores the sketch as "constraints only" and reconstructs geometry at `(0, 0)` each time, the pre-solver cannot skip anything — every component looks unsolved. Store and forward the solved positions.

**Using the same ID for unrelated entities.** IDs are the edges in the constraint graph. If a point ID is accidentally reused across two figures, those figures merge into one component and the pre-solver can no longer skip either independently.

**Marking a circle's center as `fixed: true` but not the circle itself.** The radius is a separate parameter on the circle entity. Fix both if you want a fully pinned circle: `fixed: true` on the center point, and a `circle_radius` / `FixedRadius` constraint for the radius.

**Expecting `ok: false` for under-constrained sketches.** An under-constrained sketch has many valid solutions; the solver picks one close to the initial positions and returns `ok: true`. `ok: false` means the solver could not satisfy the constraints at all (over-constrained or conflicting).
