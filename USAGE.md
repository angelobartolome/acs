# ACS Frontend Usage Guide

## Overview

ACS exposes two integration paths:

| Path | Use when |
|------|----------|
| **WASM JSON API** (`acsSolveSketch`) | Browser / JS frontend — send a JSON description of the sketch in ACS's native constraint vocabulary, receive updated positions back |
| **PlaneGCS dialect** (`acsSolveSketchPlaneGcs`, C ABI `P3DSketch_Solve`) | The same contract, with constraints named as FreeCAD's GCS names them (see *PlaneGCS dialect* below) |
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

`acsSolveSketch` takes a JSON string and returns a JSON string. It uses the same contract (envelope, geometry, Parameters, response) as the `P3DSketch_Solve` C ABI; the two differ only in how constraints are named. `acsSolveSketch` speaks ACS's **native vocabulary** (below); `P3DSketch_Solve` and `acsSolveSketchPlaneGcs` speak the **PlaneGCS dialect**. A request and its response are otherwise identical whichever way ACS is called.

| Export | Vocabulary |
|---|---|
| `acsSolveSketch(request)` | native |
| `acsConstraintCatalog()` | native catalog |
| `acsSolveSketchPlaneGcs(request)` | PlaneGCS dialect |
| `acsPlaneGcsConstraintCatalog()` | PlaneGCS dialect catalog |
| `P3DSketch_Solve` (C ABI, `--features c-abi`) | PlaneGCS dialect |

In Rust: `sketch_solve::solve_sketch_json` (native), `solve_planegcs_sketch_json` (dialect), or `solve_sketch_json_in(Vocabulary, request)`.

### Input format

```json
{
  "version": 1,
  "primitives": [ ...geometry..., ...constraints... ],
  "maxIterations": 100,
  "vocabulary": "native"
}
```

`version` must be `1`. `maxIterations` is optional (default 100). `vocabulary` is optional: `"native"` or `"planegcs"` chooses the constraint vocabulary, overriding the entry point's default (`acsSolveSketch` defaults to native; `acsSolveSketchPlaneGcs` and the C ABI to the dialect); any other value is rejected. Extra primitive fields (`isReference`, `isConstruction`, `group`) are accepted and passed through; `isReference: true` also holds the entity fixed (see below). A constraint with `"temporary": true` (such as one holding a dragged point) is a soft goal: the other (real) constraints hold exactly, and temporary ones are met as closely as those allow (a point on a Line dragged off it stays on the Line, at the closest point to the cursor; a fully constrained point doesn't move). `status` is about the real constraints only, so a temporary goal that can't be met never makes a solve `failed`; temporary constraints are also left out of `conflicting`, `redundant`, `dof` and `fullyConstrained`.

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
- An **arc** references three points: its center `c_id` and its endpoints `start_id` and `end_id`, all required. It sweeps counter-clockwise from `start_angle` to `end_angle` (radians); the sweep is `(end_angle − start_angle) mod 2π`, a full turn when that is 0. Its `radius`, `start_angle` and `end_angle` are solver variables. Its endpoints always lie on it (at `radius` from the center, at `start_angle` and `end_angle`) without any constraint, so lines that share an arc's endpoint stay attached as the arc's center, radius or angles change, and vice versa. A free arc has 5 degrees of freedom (center, radius, two angles); its endpoints are determined. An `arc_rules` constraint (which the PlaneGCS dialect sends for every arc) is accepted and adds nothing: it is never reported `redundant` and doesn't change `dof`.
- An **ellipse** references two points, its center `c_id` and a focus `focus1_id` (which sets the major axis direction), and owns its minor radius `radmin` as a solver variable. The major radius is derived, `a = √(radmin² + |focus1 − center|²)`, and the other focus is `2·center − focus1`. A free ellipse has 5 degrees of freedom (center, focus, `radmin`). Its axis endpoints are separate points held by `ellipse_axis` (dialect: the internal-alignment diameters).
- `fixed: true` stops the solver from changing that entity's own parameters: a point's `x`/`y`, a circle's `radius`, an arc's `radius`/`start_angle`/`end_angle`, an ellipse's `radmin`. A circle, arc or ellipse's center (and an arc's endpoints, an ellipse's focus) are separate points with their own `fixed` flag, so a fixed circle whose center point is free can still move.
- `isReference: true` (Reference Geometry) implies `fixed: true`: the solver never moves a reference point, nor a reference circle's radius, a reference arc's radius and angles or a reference ellipse's `radmin`, whatever is constrained to them. Like `fixed`, it covers only the entity's own values, so Reference Geometry marks its points `isReference` too.

#### Parameters

```json
{ "type": "param", "id": "offset1", "name": "Offset1", "value": 4.0 }
```

A **Parameter** is a named value constraints share by id, such as a Linked Offset's distance. The solver holds it fixed and returns the primitive unchanged; to change every constraint that uses it, change `value` and solve again. Only `value` is read (`name`, `group` and anything else pass through).

#### Scalar fields and property references

- A **scalar** field (`value`, `side`, `angle`, `distance`, `count`, …) takes a number, a Parameter id (`"value": "offset1"`) or a numeric string. A string that is neither a Parameter id nor a number is rejected as an unknown Parameter.
- A **value** field (the operands of `difference`, and of `equal` over values) also takes a property reference, `{ "entity": "c1", "property": "radius" }`, naming a value the solver may move: a point's `x` or `y`, a circle's or arc's `radius`, or an ellipse's `radmin`. Property references in plain scalar fields are rejected. (The PlaneGCS dialect spells it `{ "o_id": "c1", "prop": "radius" }`.)

#### Constraint primitives (native vocabulary)

Every constraint primitive needs a unique `id` and a `type` string. ACS names constraints by relationship, with role-named fields; one type covers every combination of entities it makes sense for, and the **variant is inferred from the kinds of the entities the fields reference**. For example `distance` with `b` a point is a point–point distance, with `b` a line a point–segment distance. `extension: true` selects a variant measured against the line's Extension (the infinite line through its endpoints) instead of the segment.

| Type | Fields (kinds) | Meaning |
|---|---|---|
| `coincident` | `a`, `b`: point | Two points overlap |
| `horizontal` / `vertical` | `line`: line, or `a`, `b`: point | The line, or the direction `a → b`, is horizontal / vertical |
| `parallel` | `a`, `b`: line | Two lines are parallel |
| `perpendicular` | `a`, `b`: line | Two lines are perpendicular |
| `angle` | `a`, `b`: line; `value` | Directed angle in radians from `a` to `b` |
| `direction` | `line`: line, or `a`, `b`: point; `value` | Direction of the line (`p1 → p2`), or of `a → b`, is `value` radians counter-clockwise from +X (not `value + π`) |
| `distance` | `a`: point; `b`: point or line; `value`; `extension?` | Point–point distance; point–segment distance (to the nearest endpoint when past an end); with `extension: true`, perpendicular distance to the line's Extension |
| `offset` | `point`: point, `line`: line, `value`, `side` | Point is `value` from the line's Extension on `side`: `1` left of `p1 → p2`, `-1` right (any negative number counts as `-1`). Always the Extension (a Linked Offset's endpoints routinely stick out past its source line) |
| `on` | `point`: point; `curve`: line, circle, arc or ellipse; `extension?` (lines) | Point lies on the segment (closest point clamped to it), on the line's Extension (`extension: true`), on the circle, on the arc's span, or on the ellipse |
| `midpoint` | `point`: point, `line`: line | Point is the line's midpoint |
| `midpoint_on` | `line`: line, `on`: line; `extension?` | Midpoint of `line` lies on segment `on` (or its Extension) |
| `tangent` | `a`, `b`: line–circle, line–arc, circle–circle or line–ellipse; `extension?` (line–circle) | Line segment tangent to the circle with the tangency point on the segment (`extension: true`: anywhere along the Extension); line segment tangent to the arc, touching both the segment and the arc's span (when the line ends at one of the arc's endpoints, as in a slot, tangency is the angle at that shared point: the line is perpendicular to the radius there); two circles touching externally (centers `r1 + r2` apart); line segment tangent to the ellipse with the tangency point on the segment |
| `concentric` | `a`, `b`: circle or arc | Share a center |
| `equal` | `a`, `b`: line–line, circle/arc–circle/arc, or values | Equal length; equal radius; `a = b` (value fields) |
| `radius` | `curve`: circle or arc; `value` | Fixed radius |
| `difference` | `a`, `b`, `value` (value fields) | `b − a = value`, e.g. two radii a Parameter apart |
| `x` / `y` | `point`: point; `value` | Point pinned to an x / y coordinate |
| `mirror` | `source`, `image`: point; `axis`: line | `image` is `source` mirrored across the axis line's Extension (a mirror axis reflects points beside or beyond its ends too) |
| `rotation` | `source`, `copy`, `center`: point; `angle` | `copy` is `source` rotated `angle` radians counter-clockwise about `center` (one circular array copy) |
| `translation` | `source`, `copy`, `from`, `to`: point; `distance`, `count` | `copy` is `source` moved `distance × count` along the unit direction `from → to` (copy `count` of a linear array; swap `from`/`to` to flip it) |
| `ellipse_axis` | `ellipse`: ellipse, `point`: point, `which`: `"major"` or `"minor"` | Point is an endpoint of the ellipse's major axis (`center ± a·u`, u the unit direction center → focus) or minor axis (`center ± radmin·rot90(u)`): either end, the one it is nearer. Two points on opposite ends of one axis need one `ellipse_axis` each (dialect: one diameter alignment for both) |

Notes:

- For `distance`, `tangent`, `concentric` and `equal`, `a` and `b` may come in either order (`distance` from a line to a point is the same constraint).
- `mirror`'s `axis`, `rotation`'s `center` and `translation`'s `from`/`to` (dialect: `mirror_point_ppl`'s axis, `circular_instance`'s `center_id`, `linear_instance`'s `dirP1_id`/`dirP2_id`) are Guides. Dragging the source or the copy moves the other and leaves the Guide where it is; moving the Guide (a drag, or other constraints on it) carries the copy along. Other real constraints on the copies move a free Guide as they need: Horizontal on one side of a patterned polygon turns it about a center that moves with it, and a mirror whose source and image are both fixed turns a free axis onto their perpendicular bisector. A free axis/center/direction keeps its own degrees of freedom in `dof`, and its copies aren't `fullyConstrained`.
- A combination of kinds no variant takes is rejected naming the constraint, the kinds it got and the ones it accepts, e.g. `constraint k1: unsupported combination for 'distance': got a: point, b: circle; expected (a: point, b: point) or (a: point, b: line) or (a: point, b: line, extension)`. So is `extension: true` on a variant that has no Extension form (`on` with a circle, `tangent` with an arc, `coincident`, …).
- Circle and arc fields resolve their center point from the circle's or arc's `c_id` (and an arc field its endpoints from `start_id`/`end_id`). `on` with a circle puts the point anywhere on the circle; with an arc, only on its span.
- Ellipse fields resolve the ellipse's center and focus points from its `c_id` and `focus1_id`. Tangency to an ellipse is to a Line's segment, as with a circle: the tangency point (where the line from one focus to the other focus's mirror image in the line crosses it) must lie on the segment. There is no Extension variant for ellipses.
- The machine-readable list is available from `acsConstraintCatalog()`: JSON `[{ "type", "fields": [{ "name", "kind" }], "extension"? }]`, one row per variant (so `distance` has three rows), kinds `point`/`line`/`circle`/`arc`/`ellipse`/`scalar`/`value`/`axis` (`axis`: the string `"major"` or `"minor"`). `extension` is present on rows that take the flag: `false` on the segment variant, `true` on the Extension variant.
- The request is rejected (see *Rejected requests* below) when it has the wrong `version`, no `primitives` array, a primitive without `type` or `id`, an unknown type, a missing field, a constraint that references a missing or wrong-kind entity, or an unsupported combination. Nothing is solved; the error names the problem.
- Only `point`, `line`, `circle`, `arc` and `ellipse` are geometry, and `param` is a Parameter. Any other `type` is looked up as a constraint.

#### PlaneGCS dialect

The dialect names constraints as FreeCAD's GCS names them, one type per combination of entities. `P3DSketch_Solve` and `acsSolveSketchPlaneGcs` accept exactly these types and fields (and only these: a native type name is an unknown type there, and vice versa), mapping each straight onto ACS's internal constraints; the envelope, geometry, Parameters, rejections and response are the same as above. Property references are `{ "o_id", "prop" }`. Constraints with a `c_id` field also accept an explicit `center_id` override, and `point_on_circle` with an arc's id in `c_id` means the arc's full circle. `acsPlaneGcsConstraintCatalog()` lists the dialect in the same shape as the native catalog.

| Dialect type | Fields | Native equivalent |
|---|---|---|
| `p2p_coincident` | `p1_id`, `p2_id` | `coincident` |
| `horizontal_pp` / `vertical_pp` | `p1_id`, `p2_id` | `horizontal` / `vertical` with `a`, `b` |
| `horizontal_l` / `vertical_l` | `l_id` | `horizontal` / `vertical` with `line` |
| `parallel` | `l1_id`, `l2_id` | `parallel` |
| `perpendicular_ll` | `l1_id`, `l2_id` | `perpendicular` |
| `perpendicular_pppp` | `l1p1_id`, `l1p2_id`, `l2p1_id`, `l2p2_id` | `perpendicular` (lines as point pairs; dialect only) |
| `l2l_angle_ll` | `l1_id`, `l2_id`, `angle` | `angle` |
| `l2l_angle_pppp` | `l1p1_id`, `l1p2_id`, `l2p1_id`, `l2p2_id`, `angle` | `angle` (dialect only) |
| `p2p_angle` | `p1_id`, `p2_id`, `angle` | `direction` with `a`, `b` |
| `p2p_distance` | `p1_id`, `p2_id`, `distance` | `distance`, point–point |
| `p2l_distance` | `p_id`, `l_id`, `distance` | `distance`, point–line |
| `p2l_extension_distance` | `p_id`, `l_id`, `distance` | `distance`, point–line, `extension: true` |
| `p2l_signed_distance` | `p_id`, `l_id`, `distance`, `side` | `offset` |
| `point_on_line_pl` | `p_id`, `l_id` | `on`, line |
| `point_on_line_ppp` | `p_id`, `lp1_id`, `lp2_id` | `on`, line (dialect only) |
| `point_on_extension_pl` | `p_id`, `l_id` | `on`, line, `extension: true` |
| `point_on_circle` | `p_id`, `c_id` | `on`, circle |
| `point_on_arc` | `p_id`, `a_id` | `on`, arc |
| `p2p_symmetric_ppp` | `p1_id`, `p2_id`, `p_id` | `midpoint` (`p_id` is the midpoint of `p1_id`, `p2_id`) |
| `p2p_symmetric_ppl` | `p1_id`, `p2_id`, `l_id` | `mirror` (`p2_id` is `p1_id` mirrored across the line's Extension) |
| `mirror_point_ppl` | `pA_id`, `pB_id`, `axis_id` | `mirror` |
| `midpoint_on_line_ll` | `l1_id`, `l2_id` | `midpoint_on` |
| `midpoint_on_line_pppp` | `l1p1_id`, `l1p2_id`, `l2p1_id`, `l2p2_id` | `midpoint_on` (dialect only) |
| `midpoint_on_extension_ll` | `l1_id`, `l2_id` | `midpoint_on`, `extension: true` |
| `tangent_lc` | `l_id`, `c_id` | `tangent`, line–circle |
| `tangent_extension_lc` | `l_id`, `c_id` | `tangent`, line–circle, `extension: true` |
| `tangent_la` | `l_id`, `a_id` | `tangent`, line–arc |
| `tangent_cc` | `c1_id`, `c2_id` | `tangent`, circle–circle |
| `concentric_cc` | `c1_id`, `c2_id` | `concentric` |
| `equal_length` | `l1_id`, `l2_id` | `equal`, lines |
| `equal_radius_cc` | `c1_id`, `c2_id` | `equal`, circles |
| `equal_radius_aa` | `a1_id`, `a2_id` | `equal`, arcs |
| `equal` | `param1`, `param2` (value fields) | `equal`, values |
| `difference` | `param1`, `param2`, `difference` (value fields) | `difference` (`param2 − param1 = difference`) |
| `circle_radius` | `c_id`, `radius` | `radius`, circle |
| `arc_radius` | `a_id`, `radius` | `radius`, arc |
| `coordinate_x` / `coordinate_y` | `p_id`, `x` / `y` | `x` / `y` |
| `circular_instance` | `p0_id`, `pk_id`, `center_id`, `angle` | `rotation` |
| `linear_instance` | `p0_id`, `pk_id`, `dirP1_id`, `dirP2_id`, `base_distance`, `N` | `translation` |
| `arc_rules` | `a_id` | accepted and adds nothing: every arc already keeps its endpoints on itself |

In the dialect a `param` may come without an `id`, as GCS keys Parameters by `name` (GCS-style clients send `{type: "param", name, value}`): its `name` is then its key, and constraints reference it by that name. Every other primitive needs an `id`.
| `point_on_ellipse` | `p_id`, `e_id` | `on`, ellipse |
| `tangent_le` | `l_id`, `e_id` | `tangent`, line–ellipse (tangency point on the segment, like `tangent_lc`) |
| `internal_alignment_ellipse_major_diameter` | `e_id`, `p1_id`, `p2_id` | `p1_id` and `p2_id` are the two ends of the major axis (one `ellipse_axis` `"major"` for each, and the two on opposite ends) |
| `internal_alignment_ellipse_minor_diameter` | `e_id`, `p1_id`, `p2_id` | the same for the minor axis |

The diameter alignments hold the pair symmetric about the center, along the axis, a diameter apart; which point takes which end isn't fixed (GCS assigns the ends once, by which point is nearer when the constraint is added), so each point keeps the end it is nearest. An ellipse tool typically writes the ellipse, a construction focus point, four axis points held by the two diameter alignments, and `p2p_distance`s on the diameters; mirrors and arrays tie a copy's minor radius to the source's with `equal` over `{ "o_id": "e1", "prop": "radmin" }`. Other GCS ellipse types (`internal_alignment_point2ellipse`, the focus alignments, `arc_of_ellipse`, …) are unknown types.

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

| `fullyConstrained` | *ACS extension.* IDs of entities with zero degrees of freedom; a line is included when both its endpoints are, an ellipse when its `radmin` and its center and focus points are. Only populated on a converged solve; `[]` otherwise |
| `stats` | *ACS extension.* `iterations` (summed across components), `initialError`, `finalError` (max across components) |

`conflicting` and `redundant` are diagnosed per component at the solved position, so a sketch that only disagrees at its starting position reports nothing. Temporary constraints are never listed and their rows are left out of the diagnosis. A constraint between fixed geometry only is conflicting if it doesn't hold and redundant if it does.

A failed solve is a normal response, not an error.

### Rejected requests

A request ACS can't read gets `{ "version": 1, "status": "invalid", "error": "..." }` and nothing is solved. When the problem is a constraint, the response also has `constraintId`, and the error reads like `constraint k1: missing field 'value'`, `constraint k1: unknown type 'made_up'`, `constraint k1: 'ghost' is not a point` or (native) `constraint k1: unsupported combination for 'distance': got a: point, b: circle; expected (a: point, b: point) or (a: point, b: line) or (a: point, b: line, extension)`. Other rejections include a wrong `version`, `maxIterations` or `vocabulary`, a duplicate id, geometry referencing a missing point (`line l1: 'p9' is not a point`), a Parameter without a numeric `value` (`param d: missing field 'value'`), an unknown Parameter id (`constraint k1: field 'distance': unknown Parameter 'd9'`), and a bad property reference (`constraint k1: field 'a': circle 'c1' has no property 'diameter'`, `... 'ghost' is not an entity`).

### C ABI

The same contract is exported to C from the static library built with `--features c-abi` (header: `include/p3d_sketch_solver.h`). It speaks the PlaneGCS dialect by default (it answers exactly as `acsSolveSketchPlaneGcs`); a request with `"vocabulary": "native"` uses the native vocabulary instead. `P3DSketch_Solve(request, &response)` returns `0` with the response for an understood request (whatever the solve `status`) and non-zero with the `invalid` response above for a malformed one. A null or non-UTF-8 request, or an internal panic, is also answered with an `invalid` response, never a crash. `*response` is always set and must be released with `P3DSketch_Free`. No state is kept between calls.

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

// Point on geometry
ConstraintType::PointOnLine(point_id, line_p1_id, line_p2_id)  // point on segment (clamped)
ConstraintType::PointOnCircle(point_id, circle_center_id, circle_id)

// Circle / arc
ConstraintType::FixedRadius(circle_or_arc_id, radius)
ConstraintType::EqualRadius(circle1_id, circle2_id)
ConstraintType::Concentric(center1_point_id, center2_point_id)
ConstraintType::TangentLineCircle(line_p1_id, line_p2_id, circle_center_id, circle_id)
ConstraintType::Tangent(c1_center_id, c1_id, c2_center_id, c2_id)  // circle-circle

// Arcs (Arc::new(id, center_id, start_id, end_id, radius, start_angle, end_angle, fixed))
ConstraintType::ArcRules(center_id, start_id, end_id, arc_id)  // implied for an arc's own points: a no-op
ConstraintType::PointOnArc(point_id, arc_center_id, arc_id)  // on the arc's span
ConstraintType::TangentLineArc(line_p1_id, line_p2_id, arc_center_id, arc_id)
ConstraintType::TangentAtPoint(shared_point_id, other_line_end_id, arc_center_id)  // tangent where line and arc share a point

// Midpoints
ConstraintType::Midpoint(midpoint_id, endpoint_a_id, endpoint_b_id)
ConstraintType::MidpointOfLineOnLine(l1p1, l1p2, l2p1, l2p2)

// Extension variants: measure against the infinite line through a line's endpoints
ConstraintType::PointOnExtension(point_id, line_p1_id, line_p2_id)
ConstraintType::DistancePointExtension(point_id, line_p1_id, line_p2_id, distance)
ConstraintType::TangentExtensionCircle(line_p1_id, line_p2_id, circle_center_id, circle_id)
ConstraintType::MidpointOfLineOnExtension(l1p1, l1p2, l2p1, l2p2)
ConstraintType::SignedDistancePointExtension(point_id, line_p1_id, line_p2_id, distance, side)

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
