# ACS Frontend Usage Guide

## Overview

ACS exposes two integration paths:

| Path | Use when |
|------|----------|
| **WASM JSON API** (`acsSolveSketch`) | Browser / JS frontend — send a JSON description of the sketch in ACS's native constraint vocabulary, receive updated positions back |
| **C ABI** (`P3DSketch_Solve`) | Native code linking the static library — the same JSON contract (see *C ABI* below) |
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

`acsSolveSketch` takes a JSON string and returns a JSON string, in ACS's **native vocabulary** (below). The `P3DSketch_Solve` C ABI has the same contract (envelope, geometry, Parameters, constraints, response): a request and its response are identical whichever way ACS is called.

| Export | |
|---|---|
| `acsSolveSketch(request)` | solves a sketch |
| `acsConstraintCatalog()` | the constraint catalog |
| `P3DSketch_Solve` (C ABI, `--features c-abi`) | solves a sketch |

In Rust: `sketch_solve::solve_sketch_json(request)`.

### Input format

```json
{
  "version": 1,
  "primitives": [ ...geometry..., ...constraints... ],
  "maxIterations": 100
}
```

`version` must be `1`. `maxIterations` is optional (default 100). Extra primitive fields (`isReference`, `isConstruction`, `group`) are accepted and passed through; `isReference: true` also holds the entity fixed (see below). A constraint with `"temporary": true` (such as one holding a dragged point) is a soft goal: the other (real) constraints hold exactly, and temporary ones are met as closely as those allow (a point on a Line dragged off it stays on the Line, at the closest point to the cursor; a fully constrained point doesn't move). `status` is about the real constraints only, so a temporary goal that can't be met never makes a solve `failed`; temporary constraints are also left out of `conflicting`, `redundant`, `dof` and `fullyConstrained`.

All primitives live in one flat array — geometry first, then constraints, though order within each group does not matter.

#### Geometry primitives

```json
{ "type": "point",  "id": "p1", "x": 0.0, "y": 0.0, "fixed": false }
{ "type": "line",   "id": "l1", "p1_id": "p1", "p2_id": "p2" }
{ "type": "circle", "id": "c1", "c_id": "center1", "radius": 5.0, "fixed": false }
{ "type": "arc",    "id": "a1", "c_id": "center1", "start_id": "s1", "end_id": "e1",
                    "radius": 5.0, "start_angle": 0.0, "end_angle": 1.5708, "fixed": false }
{ "type": "ellipse", "id": "e1", "c_id": "center1", "focus1_id": "focus1", "radmin": 3.0, "fixed": false }
```

- A **line** has no parameters of its own — it is fully defined by its two endpoint points.
- A **circle** stores only its radius; the center position is a separate `point` referenced by `c_id`.
- An **arc** references three points: its center `c_id` and its endpoints `start_id` and `end_id`, all required. It sweeps counter-clockwise from `start_angle` to `end_angle` (radians); the sweep is `(end_angle − start_angle) mod 2π`, a full turn when that is 0. Its `radius`, `start_angle` and `end_angle` are solver variables. Its endpoints always lie on it (at `radius` from the center, at `start_angle` and `end_angle`) without any constraint, so lines that share an arc's endpoint stay attached as the arc's center, radius or angles change, and vice versa. A free arc has 5 degrees of freedom (center, radius, two angles); its endpoints are determined.
- An **ellipse** references two points, its center `c_id` and a focus `focus1_id` (which sets the major axis direction), and owns its minor radius `radmin` as a solver variable. The major radius is derived, `a = √(radmin² + |focus1 − center|²)`, and the other focus is `2·center − focus1`. A free ellipse has 5 degrees of freedom (center, focus, `radmin`). Its axis endpoints are separate points held by `ellipse_axis`.
- An **elliptical arc** (`elliptical_arc`) is an arc of an ellipse: an ellipse's center `c_id`, focus `focus1_id` and `radmin`, plus endpoints `start_id` and `end_id` (all required) and `start_angle`/`end_angle`. Its angles are the ellipse's own *parametric* angle `t`, measured from the major axis (center → `focus1_id`) towards the minor axis (that direction turned 90° counter-clockwise): the point at `t` is `center + a·cos t·u + radmin·sin t·n`, `u` the unit major direction, `n` the unit minor one. (For a point on the ellipse, `t = atan2(y'/radmin, x'/a)` in the axes' frame; it is the polar angle only on a circle.) It sweeps counter-clockwise from `start_angle` to `end_angle`, like an arc (`(end_angle − start_angle) mod 2π`, a full turn at 0). It owns `radmin`, `start_angle` and `end_angle` as solver variables, and its endpoints always lie on it at their angles, as an arc's do. A free elliptical arc has 7 degrees of freedom (center, focus, `radmin`, two angles). A projected circle or ellipse edge seen at an angle is an elliptical arc: send its center, a focus on its major axis, its minor radius and its endpoints with their parametric angles.
- `fixed: true` stops the solver from changing that entity's own parameters: a point's `x`/`y`, a circle's `radius`, an arc's `radius`/`start_angle`/`end_angle`, an ellipse's `radmin`, an elliptical arc's `radmin`/`start_angle`/`end_angle`. A circle, arc or ellipse's center (and an arc's endpoints, an ellipse's focus) are separate points with their own `fixed` flag, so a fixed circle whose center point is free can still move.
- `isReference: true` (Reference Geometry) implies `fixed: true`: the solver never moves a reference point, nor a reference circle's radius, a reference arc's radius and angles, a reference ellipse's `radmin` or a reference elliptical arc's `radmin` and angles, whatever is constrained to them. Like `fixed`, it covers only the entity's own values, so Reference Geometry marks its points `isReference` too.

#### Parameters

```json
{ "type": "param", "id": "offset1", "name": "Offset1", "value": 4.0 }
```

A **Parameter** is a named value constraints share by id, such as a Linked Offset's distance. The solver holds it fixed and returns the primitive unchanged; to change every constraint that uses it, change `value` and solve again. Only `value` is read (`name`, `group` and anything else pass through).

#### Scalar fields and property references

- A **scalar** field (`value`, `side`, `angle`, `distance`, `count`, …) takes a number, a Parameter id (`"value": "offset1"`) or a numeric string. A string that is neither a Parameter id nor a number is rejected as an unknown Parameter.
- A **value** field (the operands of `difference`, and of `equal` over values) also takes a property reference, `{ "entity": "c1", "property": "radius" }`, naming a value the solver may move: a point's `x` or `y`, a circle's or arc's `radius`, or an ellipse's or elliptical arc's `radmin`. Property references in plain scalar fields are rejected.

#### Constraint primitives (native vocabulary)

Every constraint primitive needs a unique `id` and a `type` string. ACS names constraints by relationship, with role-named fields; one type covers every combination of entities it makes sense for, and the **variant is inferred from the kinds of the entities the fields reference**. For example `distance` with `b` a point is a point–point distance, with `b` a line a point–segment distance. `extension: true` selects a variant measured against the line's Extension (the infinite line through its endpoints) instead of the segment, and `internal: true` the inside variant of a distance to a circle.

| Type | Fields (kinds) | Meaning |
|---|---|---|
| `coincident` | `a`, `b`: point | Two points overlap |
| `horizontal` / `vertical` | `line`: line, or `a`, `b`: point | The line, or the direction `a → b`, is horizontal / vertical |
| `parallel` | `a`, `b`: line | Two lines are parallel |
| `perpendicular` | `a`, `b`: line | Two lines are perpendicular |
| `collinear` | `a`, `b`: line | Both of `b`'s endpoints lie on `a`'s Extension: the lines lie on one infinite line, with or without a gap between the segments. One constraint (Redundant or Conflicting as a unit); `a`, `b` in either order |
| `normal` | `line`: line; `curve`: circle or arc | The line's Extension passes through the curve's center. Direction only: the line need not reach the curve, nor an arc's span |
| `angle` | `a`, `b`: line; `value` | Directed angle in radians from `a` to `b` |
| `angle` | `arc`: arc; `value` | The arc's sweep (counter-clockwise from its start angle to its end angle) in radians. Only `0 < value < 2π` is accepted; anything else is rejected (`constraint k1: arc sweep 7 is not between 0 and 2π`). The residual is the sweep unwrapped near `value`, so the end angle may cross the start (the 0/2π seam) without a jump |
| `direction` | `line`: line, or `a`, `b`: point; `value` | Direction of the line (`p1 → p2`), or of `a → b`, is `value` radians counter-clockwise from +X (not `value + π`) |
| `distance` | `a`: point; `b`: point or line; `value`; `extension?` | Point–point distance; point–segment distance (to the nearest endpoint when past an end); with `extension: true`, perpendicular distance to the line's Extension |
| `distance` | `a`: point or line, `b`: circle; `value`; `internal?` (point), `extension?` (line) | Gap to the circle, signed: point–circle `\|p − c\| − r` (negative inside); with `internal: true`, `r − \|p − c\|` (negative outside). Line–circle: distance from the center to the segment minus `r` (`extension: true`: to the Extension) |
| `distance` | `a`, `b`: circle; `value`; `internal?` | Gap between the circles, signed: `\|c1 − c2\| − r1 − r2` (negative when they overlap); with `internal: true`, the inside gap `\|r1 − r2\| − \|c1 − c2\|` between the smaller circle and the inside of the larger (either may be `a`) |
| `distance` | `a`, `b`: circle–arc or arc–arc; `value`; `internal?` | The same gap, measured between the arcs' circles along the line of centers, with the nearest points on each arc's span (as for `tangent`, which is a gap of 0) |
| `distance` | `a`: point or line, `b`: arc; `value`; `internal?` (point), `extension?` (line) | The point–circle or line–circle gap to the arc's circle, signed the same way, with the nearest point on the arc's span: the ray from the arc's center through the point (or the line's nearest point: the foot of the perpendicular, the nearer endpoint past an end of the segment, always the foot with `extension: true`) must cross the span. A point or line off the span isn't measured to an arc endpoint; the solve brings the ray onto the span (moving the point, the line or the arc, whichever is free) |
| `distance` | `a`, `b`: line; `value` | Both of `b`'s endpoints are `value` from `a`'s Extension, on one side (the side `b`'s midpoint is on): `b` is parallel to `a`, `value` away. An explicit `parallel` on the pair is then `redundant`. `value` must be positive (two lines at distance 0 are `collinear`) |
| `offset` | `point`: point, `line`: line, `value`, `side` | Point is `value` from the line's Extension on `side`: `1` left of `p1 → p2`, `-1` right (any negative number counts as `-1`). Always the Extension (a Linked Offset's endpoints routinely stick out past its source line) |
| `on` | `point`: point; `curve`: line, circle, arc, ellipse or elliptical arc; `extension?` (lines) | Point lies on the segment (closest point clamped to it), on the line's Extension (`extension: true`), on the circle, on the arc's span, or on the ellipse; with an elliptical arc, on its ellipse within its span (between its parametric start and end angles) |
| `midpoint` | `entities`: [point, line], [line, line] or [point, arc]; `extension?` ([line, line]) | `[point, line]`: the point is the line's midpoint. `[line, line]`: the first line's midpoint lies on the second, on its segment (or its Extension with `extension: true`). `[point, arc]`: the point is the middle of the arc's span, on its circle halfway (by angle) from its start to its end counter-clockwise; where the end angle passes the start (a full turn ↔ none) the middle flips to the opposite side. The variant is inferred from the entities' kinds; the array must have exactly two ids |
| `tangent` | `a`, `b`: line–circle, line–arc, line–ellipse, line–elliptical arc, arc–elliptical arc, circle–circle, circle–arc or arc–arc; `extension?` (line–circle); `internal?` (circles and arcs) | Line segment tangent to the circle with the tangency point on the segment (`extension: true`: anywhere along the Extension); line segment tangent to the arc, touching both the segment and the arc's span (when the line ends at one of the arc's endpoints, as in a slot, tangency is the angle at that shared point: the line is perpendicular to the radius there); line segment tangent to the ellipse with the tangency point on the segment. Two circles or arcs touch externally (centers `r1 + r2` apart) or, with `internal: true`, one inside the other (centers `\|r1 − r2\|` apart); the tangency point lies on each arc's span. Two arcs sharing an endpoint are tangent at it: their radii there are collinear, the centers on opposite sides of the point (external) or the same side (`internal: true`). When a line endpoint is held on the circle, arc, ellipse or elliptical arc by `on` (a tangency snap), the line is tangent at that endpoint: the endpoint is the tangent point exactly. A line is tangent to an elliptical arc touching the segment and the arc's span; when the line ends at one of the elliptical arc's endpoints it runs along the ellipse's tangent there. An arc and an elliptical arc are tangent only at an endpoint they share (the arc's radius there is normal to the ellipse); without one the constraint is rejected (`an arc and an elliptical arc are tangent only at an endpoint they share`) |
| `concentric` | `a`, `b`: circle or arc | Share a center |
| `equal` | `a`, `b`: line–line, circle/arc–circle/arc, or values | Equal length; equal radius; `a = b` (value fields) |
| `radius` | `curve`: circle or arc; `value` | Fixed radius |
| `diameter` | `curve`: circle or arc; `value` | Fixed diameter, `2r = value` (the constraint keeps the number given) |
| `length` | `curve`: line or arc; `value` | A line's length; an arc's length `r · sweep`, sweep `(end_angle − start_angle) mod 2π` (a full turn when 0) |
| `difference` | `a`, `b`, `value` (value fields) | `b − a = value`, e.g. two radii a Parameter apart |
| `x` / `y` | `point`: point; `value` | Point pinned to an x / y coordinate |
| `horizontal_distance` / `vertical_distance` | `a`, `b`: point, or `line`: line; `value` | Signed distance along one axis: `b.x − a.x = value` (vertical: `b.y − a.y`); for a line, `p2 − p1`. A negative `value` puts `b` left of (below) `a`; send the sign the geometry has when the dimension is created. Fixes one axis only (with a point–point `distance` too, `b` is fully placed). `a` and `b` are ordered (swapping them flips the sign) |
| `mirror` | `source`, `image`: point; `axis`: line | `image` is `source` mirrored across the axis line's Extension (a mirror axis reflects points beside or beyond its ends too) |
| `rotation` | `source`, `copy`, `center`: point; `angle` | `copy` is `source` rotated `angle` radians counter-clockwise about `center` (one circular array copy) |
| `translation` | `source`, `copy`, `from`, `to`: point; `distance`, `count` | `copy` is `source` moved `distance × count` along the unit direction `from → to` (copy `count` of a linear array; swap `from`/`to` to flip it) |
| `ellipse_axis` | `ellipse`: ellipse or elliptical arc (its ellipse), `point`: point, `which`: `"major"` or `"minor"` | Point is an endpoint of the ellipse's major axis (`center ± a·u`, u the unit direction center → focus) or minor axis (`center ± radmin·rot90(u)`): either end, the one it is nearer. Two of these on one axis may both settle on the same end |
| `ellipse_axis` | `ellipse`: ellipse or elliptical arc (its ellipse), `a`, `b`: point, `which`: `"major"` or `"minor"` | `a` and `b` are the two opposite ends of the major or minor axis, held symmetric about the center along the axis a diameter apart; which point takes which end isn't fixed, so each keeps the end it is nearest |

Notes:

- For `distance`, `tangent`, `concentric` and `equal`, `a` and `b` may come in either order (`distance` from a line to a point is the same constraint).
- `mirror`'s `axis`, `rotation`'s `center` and `translation`'s `from`/`to` are Guides. Dragging the source or the copy moves the other and leaves the Guide where it is; moving the Guide (a drag, or other constraints on it) carries the copy along. Other real constraints on the copies move a free Guide as they need: Horizontal on one side of a patterned polygon turns it about a center that moves with it, and a mirror whose source and image are both fixed turns a free axis onto their perpendicular bisector. A free axis/center/direction keeps its own degrees of freedom in `dof`, and its copies aren't `fullyConstrained`.
- A combination of kinds no variant takes is rejected naming the constraint, the kinds it got and the ones it accepts, e.g. `constraint k1: unsupported combination for 'distance': got a: point, b: ellipse; expected (a: point, b: point) or (a: point, b: line) or (a: point, b: line, extension) or (a: point, b: circle) or (a: point, b: circle, internal) or (a: point, b: arc) or (a: point, b: arc, internal) or (a: line, b: circle) or (a: line, b: circle, extension) or (a: line, b: arc) or (a: line, b: arc, extension) or (a: circle, b: circle) or (a: circle, b: circle, internal) or (a: circle, b: arc) or (a: circle, b: arc, internal) or (a: arc, b: arc) or (a: arc, b: arc, internal) or (a: line, b: line)`. So is `extension: true` on a variant that has no Extension form (`on` with a circle, `tangent` with an arc, `coincident`, …), and `internal: true` on one without an inside form (anything but `distance` point–circle, point–arc, circle–circle, circle–arc and arc–arc).
- `internal` (tangency between circles and arcs; point–circle, point–arc, circle–circle, circle–arc and arc–arc `distance`) is never inferred: the solver keeps whichever side the constraint names, so a drag can't turn a fillet inside out or pull a point out through its circle. An editor sets it when the constraint is created, from the geometry (`internal: true` when one curve lies inside the other). `internal: true` with a line is rejected: `constraint k: field 'internal' applies only between a circle or arc and a point, circle or arc; got a: line, b: arc`. `extension` and `internal` must be `true` or `false` when present.
- Circle and arc fields resolve their center point from the circle's or arc's `c_id` (and an arc field its endpoints from `start_id`/`end_id`). `on` with a circle puts the point anywhere on the circle; with an arc, only on its span.
- Ellipse fields resolve the ellipse's center and focus points from its `c_id` and `focus1_id`. Tangency to an ellipse is to a Line's segment, as with a circle: the tangency point (where the line from one focus to the other focus's mirror image in the line crosses it) must lie on the segment. There is no Extension variant for ellipses.
- The machine-readable list is available from `acsConstraintCatalog()`: JSON `[{ "type", "fields": [{ "name", "index"?, "kind" }], "extension"?, "internal"? }]`, one row per variant (so `distance` has eighteen rows), kinds `point`/`line`/`circle`/`arc`/`ellipse`/`elliptical_arc`/`scalar`/`value`/`axis` (`axis`: the string `"major"` or `"minor"`). A field with an `index` is that element of the array field `name` (`midpoint`'s `entities`). `extension` is present on rows that take the flag: `false` on the segment variant, `true` on the Extension variant. `internal` likewise: `false` on the outside variant, `true` on the inside one (tangency circle–circle, circle–arc and arc–arc; `distance` point–circle, point–arc, circle–circle, circle–arc and arc–arc). (`midpoint {point, line}` and `midpoint_on`, deprecated in 0.1.6, were removed in 0.1.7: use `midpoint` with `entities`.)
- The request is rejected (see *Rejected requests* below) when it has the wrong `version`, no `primitives` array, a primitive without `type` or `id`, an unknown type, a missing field, a constraint that references a missing or wrong-kind entity, or an unsupported combination. Nothing is solved; the error names the problem.
- Only `point`, `line`, `circle`, `arc` and `ellipse` are geometry, and `param` is a Parameter. Any other `type` is looked up as a constraint.

#### Drags (soft goals)

Mark a constraint `temporary: true` to make it a soft goal: in each component the real constraints hold exactly, and the temporary ones are met as closely as the real ones allow (least squares). A drag sends the sketch with goals holding the dragged geometry at the cursor, one solve per mouse move:

```json
{ "type": "point", "id": "p", "x": 2, "y": 0 },
{ "type": "on", "id": "k", "point": "p", "curve": "l" },
{ "type": "x", "id": "gx", "point": "p", "value": 6, "temporary": true },
{ "type": "y", "id": "gy", "point": "p", "value": 3, "temporary": true }
```

With `l` running along y = 0, `p` slides along the line to (6, 0), the closest it can get to the cursor. A goal that can't be met never makes a solve `failed` and is never `conflicting` or `redundant`; `status`, `dof` and `fullyConstrained` consider only the real constraints. A free Guide (a mirror's axis, an array's center or direction) may move in a drag like any free geometry, so every copy drags alike; pin it to keep it put.

### Output format

```json
{
  "version": 1,
  "status": "converged",
  "primitives": [ ...same array, same order, with updated x/y/radius/angles/radmin... ],
  "conflicting": [],
  "redundant": [],
  "dof": 0,
  "fullyConstrained": ["p1", "p2", "line1"],
  "stats": { "iterations": 3, "initialError": 1600.0, "finalError": 1.2e-12 }
}
```

| Field | Values |
|---|---|
| `version` | `1` |
| `status` | `"converged"` when every constraint holds within 1e-10, `"failed"` otherwise; `"invalid"` only on rejected requests |
| `primitives` | The request's primitives in the same order, with solved values; unknown fields pass through and Parameters come back unchanged |
| `conflicting` | IDs of Conflicting constraints, in request order. Only in a component that isn't solved: every constraint in a dependency (rows of the Jacobian that combine to zero) that carries residual, so both sides of a conflict are listed |
| `redundant` | IDs of Redundant constraints, in request order. Only in a solved component: a minimal set of dependent constraints whose removal changes neither the solution nor `dof`, chosen latest first (of two duplicates, the later one) |
| `dof` | Degrees of freedom the constraints leave: free values minus the rank of each component's Jacobian |

| `fullyConstrained` | *ACS extension.* IDs of entities with zero degrees of freedom; a line is included when both its endpoints are, an ellipse when its `radmin` and its center and focus points are, an elliptical arc when its `radmin`, angles, center, focus and endpoints are. Only populated on a converged solve; `[]` otherwise |
| `stats` | *ACS extension.* `iterations` (summed across components), `initialError`, `finalError` (max across components) |

`conflicting` and `redundant` are diagnosed per component at the solved position, so a sketch that only disagrees at its starting position reports nothing. Temporary constraints are never listed and their rows are left out of the diagnosis. A constraint between fixed geometry only is conflicting if it doesn't hold and redundant if it does.

A failed solve is a normal response, not an error.

### Rejected requests

A request ACS can't read gets `{ "version": 1, "status": "invalid", "error": "..." }` and nothing is solved. When the problem is a constraint, the response also has `constraintId`, and the error reads like `constraint k1: missing field 'value'`, `constraint k1: unknown type 'made_up'`, `constraint k1: 'ghost' is not a point` or `constraint k1: unsupported combination for 'distance': got a: point, b: ellipse; expected (a: point, b: point) or (a: point, b: line) or (a: point, b: line, extension) or (a: point, b: circle) or (a: point, b: circle, internal) or (a: point, b: arc) or (a: point, b: arc, internal) or (a: line, b: circle) or (a: line, b: circle, extension) or (a: line, b: arc) or (a: line, b: arc, extension) or (a: circle, b: circle) or (a: circle, b: circle, internal) or (a: circle, b: arc) or (a: circle, b: arc, internal) or (a: arc, b: arc) or (a: arc, b: arc, internal) or (a: line, b: line)`. Other rejections include a wrong `version` or `maxIterations`, a duplicate id, geometry referencing a missing point (`line l1: 'p9' is not a point`), a Parameter without a numeric `value` (`param d: missing field 'value'`), an unknown Parameter id (`constraint k1: field 'value': unknown Parameter 'd9'`), and a bad property reference (`constraint k1: field 'a': circle 'c1' has no property 'diameter'`, `... 'ghost' is not an entity`).

### C ABI

The same contract is exported to C from the static library built with `--features c-abi` (header: `include/p3d_sketch_solver.h`). `P3DSketch_Solve(request, &response)` answers exactly as `acsSolveSketch`. (Before 0.1.7 it took a `vocabulary` argument between the request and the response pointer. The symbol name is unchanged, so nothing catches a caller built against the old header at link time: rebuild it against the new one.) It returns `0` with the response for an understood request (whatever the solve `status`) and non-zero with the `invalid` response above for a malformed one. A null or non-UTF-8 request, or an internal panic, is also answered with an `invalid` response, never a crash. `*response` is always set and must be released with `P3DSketch_Free`. No state is kept between calls.

### Full example: a constrained square

```js
const result = JSON.parse(acsSolveSketch(JSON.stringify({
  version: 1,
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
let result = JSON.parse(acsSolveSketch(JSON.stringify({ version: 1, primitives: sketchState })));
// store solved positions back
sketchState = result.primitives;

// user drags a free line endpoint — update only those two points in sketchState
sketchState = sketchState.map(p => {
  if (p.id === 'line_end') return { ...p, x: newX, y: newY };
  return p;                            // square points keep their solved values
});

// re-solve — the square component is already satisfied, only the line is solved
result = JSON.parse(acsSolveSketch(JSON.stringify({ version: 1, primitives: sketchState })));
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
ConstraintType::Collinear(a_p1, a_p2, b_p1, b_p2)  // b's endpoints on a's Extension

// Point on geometry
ConstraintType::PointOnLine(point_id, line_p1_id, line_p2_id)  // point on segment (clamped)
ConstraintType::PointOnCircle(point_id, circle_center_id, circle_id)

// Circle / arc
ConstraintType::FixedRadius(circle_or_arc_id, radius)
ConstraintType::EqualRadius(circle1_id, circle2_id)
ConstraintType::Concentric(center1_point_id, center2_point_id)
ConstraintType::TangentLineCircle(line_p1_id, line_p2_id, circle_center_id, circle_id)
ConstraintType::Tangent(c1_center_id, c1_id, c2_center_id, c2_id)  // circle-circle
ConstraintType::Diameter(circle_or_arc_id, diameter)
ConstraintType::DistancePointCircle(point_id, circle_center_id, circle_id, gap, internal)
ConstraintType::DistanceLineCircle(line_p1_id, line_p2_id, circle_center_id, circle_id, gap)
ConstraintType::DistanceCircleCircle(c1_center_id, c1_id, c2_center_id, c2_id, gap, internal)
ConstraintType::DistanceCircleArc(circle_center_id, circle_id, arc_center_id, arc_id, gap, internal)  // nearest point on the arc's span
ConstraintType::DistanceArcs(arc1_center_id, arc1_id, arc2_center_id, arc2_id, gap, internal)  // nearest points on both spans
ConstraintType::DistancePointArc(point_id, arc_center_id, arc_id, gap, internal)  // nearest point on the arc's span
ConstraintType::DistanceLineArc(line_p1_id, line_p2_id, arc_center_id, arc_id, gap)  // segment; nearest point on the span
ConstraintType::DistanceExtensionArc(line_p1_id, line_p2_id, arc_center_id, arc_id, gap)  // Extension; foot on the span

// Arcs (Arc::new(id, center_id, start_id, end_id, radius, start_angle, end_angle, fixed))
ConstraintType::PointOnArc(point_id, arc_center_id, arc_id)  // on the arc's span
ConstraintType::MidpointOfArc(point_id, arc_center_id, arc_id)  // the middle of the arc's span
ConstraintType::TangentLineArc(line_p1_id, line_p2_id, arc_center_id, arc_id)
ConstraintType::TangentAtPoint(shared_point_id, other_line_end_id, arc_center_id)  // tangent where line and arc share a point
ConstraintType::TangentCirclesInternal(c1_center_id, c1_id, c2_center_id, c2_id)  // one circle inside the other
ConstraintType::TangentCircleArc(circle_center_id, circle_id, arc_center_id, arc_id, internal)
ConstraintType::TangentArcs(arc1_center_id, arc1_id, arc2_center_id, arc2_id, internal)
ConstraintType::TangentArcsAtPoint(shared_point_id, arc1_center_id, arc2_center_id, internal)  // arcs sharing an endpoint
ConstraintType::ArcLength(arc_id, length)  // radius × sweep
ConstraintType::ArcSweep(arc_id, sweep)    // 0 < sweep < 2π, else add_constraint fails

// Midpoints
ConstraintType::Midpoint(midpoint_id, endpoint_a_id, endpoint_b_id)
ConstraintType::MidpointOfLineOnLine(l1p1, l1p2, l2p1, l2p2)

// Extension variants: measure against the infinite line through a line's endpoints
ConstraintType::PointOnExtension(point_id, line_p1_id, line_p2_id)
ConstraintType::DistancePointExtension(point_id, line_p1_id, line_p2_id, distance)
ConstraintType::TangentExtensionCircle(line_p1_id, line_p2_id, circle_center_id, circle_id)
ConstraintType::MidpointOfLineOnExtension(l1p1, l1p2, l2p1, l2p2)
ConstraintType::SignedDistancePointExtension(point_id, line_p1_id, line_p2_id, distance, side)
ConstraintType::DistanceExtensionCircle(line_p1_id, line_p2_id, circle_center_id, circle_id, gap)
ConstraintType::DistanceLineLine(a_p1_id, a_p2_id, b_p1_id, b_p2_id, distance)  // b's ends from a's Extension

// Relations between scalars: each Operand is Operand::Const(f64) or an entity
// property the solver may move: Operand::X(point_id), Operand::Y(point_id),
// Operand::Radius(circle_or_arc_id)
ConstraintType::Difference(param1, param2, difference)  // param2 - param1 = difference
ConstraintType::Equal(param1, param2)

// Arrays and mirror
ConstraintType::PointPointAngle(p1_id, p2_id, angle)                   // direction
ConstraintType::MirrorPointExtension(pa_id, pb_id, axis_p1_id, axis_p2_id) // mirror
ConstraintType::CircularInstance(p0_id, pk_id, center_id, angle)        // rotation
ConstraintType::LinearInstance(p0_id, pk_id, dir_p1_id, dir_p2_id, base_distance, n) // translation
```

---

## Common mistakes

**Sending zero/default positions on every call.** If your frontend stores the sketch as "constraints only" and reconstructs geometry at `(0, 0)` each time, the pre-solver cannot skip anything — every component looks unsolved. Store and forward the solved positions.

**Using the same ID for unrelated entities.** IDs are the edges in the constraint graph. If a point ID is accidentally reused across two figures, those figures merge into one component and the pre-solver can no longer skip either independently.

**Marking a circle's center as `fixed: true` but not the circle itself.** The radius is a separate parameter on the circle entity. Fix both if you want a fully pinned circle: `fixed: true` on the center point, and a `radius` (`FixedRadius`) constraint for the radius.

**Expecting `status: "failed"` for under-constrained sketches.** An under-constrained sketch has many valid solutions; the solver picks one close to the initial positions and returns `"converged"` (check `dof` to see how much freedom is left). `"failed"` means the solver could not satisfy the constraints at all (over-constrained or conflicting).
