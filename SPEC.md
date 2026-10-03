# ACS Constraint Solver — Constraint Specification

This document specifies every constraint type in ACS: the geometric meaning, the
parametric entities involved, the residual equations fed to the Dog-Leg solver, and
the analytical Jacobian used during each iteration.

Constraints are named here by their internal `ConstraintType`. The JSON names
in parentheses are the native type that reaches each one, then the PlaneGCS
dialect type (`GCS ...`); `USAGE.md` lists both vocabularies in full.

## Notation

| Symbol | Meaning |
|--------|---------|
| `p = (x, y)` | A free point tracked by the parameter manager |
| `c` | A circle tracked as `(center_point_id, radius)` |
| `E` | An ellipse tracked as `(center_point_id, focus1_point_id, radmin)` (section 21) |
| `dᵢ = pᵢ₂ − pᵢ₁` | Direction vector of line *i* (not unit-length) |
| `R` | Residual vector (must be **0** at solution) |
| `J` | Jacobian `∂R/∂params` (analytical) |
| `θ` | Angle in radians |
| `d` | Scalar distance |

Residuals are written so that **R = 0** encodes the constraint.  All Jacobian
entries not listed are **0**.

**Guides.** Some constraints copy across an input: a mirror's axis (17), a
rotation's center (18), a translation's direction (19). Their partials,
Guides included, are exact like every other, so an ordinary solve moves a
Guide as the constraints need. Only a solve with soft goals (a drag) holds
the Guides: dragging a copy never moves what it is copied across (see Solver
Notes).

---

## Already-Implemented Constraints

| Type | Entities | Residuals |
|------|----------|-----------|
| `Vertical(p1, p2)` | 2 points | `x1 − x2 = 0` |
| `Horizontal(p1, p2)` | 2 points | `y1 − y2 = 0` |
| `Parallel(p1,p2,p3,p4)` | 4 points (2 lines) | `dx1·dy2 − dy1·dx2 = 0` |
| `Coincident(p1, p2)` | 2 points | `x1−x2 = 0`, `y1−y2 = 0` |
| `EqualX(p, val)` | 1 point | `x − val = 0` |
| `EqualY(p, val)` | 1 point | `y − val = 0` |
| `PointOnLine(p, a, b)` | 3 points | signed distance from `p` to the **segment** `ab` = 0 (see Segment geometry below) |
| `EqualRadius(c1, c2)` | 2 circles | `r1 − r2 = 0` |

### Segment geometry (shared)

A **Line** is the bounded segment between its endpoints `a` and `b`. Every
constraint that measures against a line uses these helpers
(`src/constraints/segment.rs`). Each is written for a point `p`.

Let `dx = bx − ax`, `dy = by − ay`, `L = √(dx² + dy²)`,
`C = dx·(py − ay) − dy·(px − ax)` (signed area),
`t = ((px − ax)·dx + (py − ay)·dy) / L²` (closest-point parameter),
`σ = +1` if `C ≥ 0` else `−1`.

**Line signed distance** `S∞ = C / L`: the distance to the Extension (the
infinite line through `a` and `b`). Used by TangentLineCircle together with
`O` below, and on its own only by the Extension constraints (section 13). For
a degenerate segment (`L ≈ 0`) the unnormalized `C` is returned. `∂S∞ = ∂C / L − C·∂L / L²`, with

| Parameter | ∂C/∂param | ∂L/∂param |
|-----------|-----------|-----------|
| `px` | `−dy` | `0` |
| `py` | `dx` | `0` |
| `ax` | `by − py` | `−dx / L` |
| `ay` | `px − bx` | `−dy / L` |
| `bx` | `py − ay` | `dx / L` |
| `by` | `ax − px` | `dy / L` |

**Segment signed distance** `S`: the distance from `p` to the closest point of
the segment, signed by side.

```
0 ≤ t ≤ 1:  S = S∞
t < 0:      S = σ·|p − a|
t > 1:      S = σ·|p − b|
```

The branches agree at `t = 0` and `t = 1`. Past endpoint `e`, with
`D = |p − e|`: `∂S/∂p = σ·(p − e)/D`, `∂S/∂e = −σ·(p − e)/D`, and 0 for the
other endpoint. A degenerate segment (`L ≈ 0`) is treated as the point `a`.

**Segment distance** `D = |S|` (unsigned), `∂D = σ_S·∂S` with `σ_S = sign(S)`.

**Foot overshoot** `O`: how far the foot of the perpendicular from `p` falls
outside the segment, measured along the line. With `s = N / L`,
`N = (px − ax)·dx + (py − ay)·dy`:

```
s < 0:      O = s
s > L:      O = s − L
otherwise:  O = 0
```

`∂s = ∂N / L − N·∂L / L²` with `∂N/∂(px, py, ax, ay, bx, by) =
(dx, dy, −dx − (px − ax), −dy − (py − ay), px − ax, py − ay)`; for `s > L`,
`∂O = ∂s − ∂L`.

None of these are squared. A squared residual has a zero Jacobian row at the
solution, which stalls convergence and hides the removed degree of freedom.

### PointOnLine

**Geometric meaning:** `p` lies on the segment `ab`.

**Residual (1 equation):** `R = S(p; a, b) = 0`. Jacobian: `∂S` above.

---

## New Constraints

### 1  Perpendicular

**Entities:** four points `(p1, p2, p3, p4)` defining two lines `L1 = p1→p2` and
`L2 = p3→p4`.

**Geometric meaning:** the lines meet at a 90° angle.

**Residual (1 equation):**

```
R = dx1·dx2 + dy1·dy2 = 0
```

where `dx1 = x2−x1`, `dy1 = y2−y1`, `dx2 = x4−x3`, `dy2 = y4−y3`.

**Jacobian:**

| Parameter | ∂R/∂param |
|-----------|-----------|
| `x1` | `−dx2` |
| `y1` | `−dy2` |
| `x2` | `dx2` |
| `y2` | `dy2` |
| `x3` | `−dx1` |
| `y3` | `−dy1` |
| `x4` | `dx1` |
| `y4` | `dy1` |

---

### 2  FixedRadius

**Entities:** one circle or arc `c` with radius `r`; a scalar target `r₀`.

**Geometric meaning:** forces the circle/arc to have exactly the given radius.

**Residual (1 equation):**

```
R = r − r₀ = 0
```

**Jacobian:**

| Parameter | ∂R/∂param |
|-----------|-----------|
| `r` | `1` |

---

### 3  PointOnCircle

**Entities:** a free point `p`, the circle's center point `c_center`, and the
circle entity `c` (for its radius `r`).

**Geometric meaning:** `p` lies exactly on the circumference of circle `c`.

**Residual (1 equation):**

```
R = (px − cx)² + (py − cy)² − r² = 0
```

**Jacobian:**

| Parameter | ∂R/∂param |
|-----------|-----------|
| `px` | `2(px − cx)` |
| `py` | `2(py − cy)` |
| `cx` | `−2(px − cx)` |
| `cy` | `−2(py − cy)` |
| `r`  | `−2r` |

---

### 4  Tangent (circle–circle, external)

**Entities:** two circles `c1` and `c2`, each with a center point and radius.
The ConstraintType stores `(c1_center_id, c1_id, c2_center_id, c2_id)`.

**Geometric meaning:** the circles touch at exactly one external point,
i.e. the distance between centers equals the sum of their radii.

**Residual (1 equation):**

```
R = (cx2 − cx1)² + (cy2 − cy1)² − (r1 + r2)² = 0
```

Let `Δx = cx2 − cx1`, `Δy = cy2 − cy1`.

**Jacobian:**

| Parameter | ∂R/∂param |
|-----------|-----------|
| `cx1` | `−2Δx` |
| `cy1` | `−2Δy` |
| `cx2` | `2Δx` |
| `cy2` | `2Δy` |
| `r1`  | `−2(r1 + r2)` |
| `r2`  | `−2(r1 + r2)` |

---

### 5  TangentLineCircle

**Entities:** two points `(pa, pb)` defining a line, the circle's center point
`c_center`, and the circle entity `c`.

**Geometric meaning:** the line is tangent to the circle, and the tangency
point lies on the segment. The center is `r` from the line, and the foot of
the perpendicular from the center falls between the endpoints.

**Residuals (2 equations)**, with the helpers evaluated at `p = c_center`:

```
R₀ = |S∞(c; a, b)| − r = 0
R₁ = O(c; a, b)        = 0
```

**Jacobian:** `∂R₀ = sign(S∞)·∂S∞`, `∂R₀/∂r = −1`; `∂R₁ = ∂O`. While the
tangency point lies on the segment, `R₁` and its row are 0, so it removes no
degree of freedom.

---

### 6  Concentric

**Entities:** the center points of two circles: `center1_id`, `center2_id`.

**Geometric meaning:** two circles share the same center.

**Residual (2 equations) — identical to Coincident:**

```
R₀ = cx1 − cx2 = 0
R₁ = cy1 − cy2 = 0
```

**Jacobian:** same as `CoincidentConstraint`.

---

### 7  DistancePointPoint

**Entities:** two points `p1`, `p2`; a scalar target distance `d`.

**Geometric meaning:** the Euclidean distance between the two points equals `d`.

**Residual (1 equation) — squared form to keep it smooth:**

```
R = (x2 − x1)² + (y2 − y1)² − d² = 0
```

Let `Δx = x2 − x1`, `Δy = y2 − y1`.

**Jacobian:**

| Parameter | ∂R/∂param |
|-----------|-----------|
| `x1` | `−2Δx` |
| `y1` | `−2Δy` |
| `x2` | `2Δx` |
| `y2` | `2Δy` |

---

### 8  DistancePointLine

**Entities:** a point `p`; two points `(pa, pb)` defining the line; a scalar
target distance `d`.

**Geometric meaning:** the distance from `p` to the closest point of the
segment `ab` equals `d`. Past an endpoint that is the distance to the endpoint.

**Residual (1 equation):**

```
d > 0:  R = D(p; a, b) − d = 0
d = 0:  R = S(p; a, b)     = 0   (same as PointOnLine; |·| has no usable gradient at 0)
```

**Jacobian:** `∂D` (or `∂S` when `d = 0`) from the shared section.

---

### 9  Angle

**Entities:** four points `(p1, p2, p3, p4)` defining two lines `L1 = p1→p2`,
`L2 = p3→p4`; a target angle `θ` in radians.

**Geometric meaning:** the (directed) angle from `L1` to `L2` equals `θ`.

Let `dx1 = x2−x1`, `dy1 = y2−y1`, `dx2 = x4−x3`, `dy2 = y4−y3`.

```
dot   = dx1·dx2 + dy1·dy2
cross = dx1·dy2 − dy1·dx2
```

**Residual (1 equation):**

```
R = dot·sin(θ) − cross·cos(θ) = 0
```

This encodes `tan(θ) = cross/dot`, i.e. `atan2(cross, dot) = θ`.

**Jacobian:**

| Parameter | ∂R/∂param |
|-----------|-----------|
| `x1` | `−dx2·sin θ + dy2·cos θ` |
| `y1` | `−dy2·sin θ − dx2·cos θ` |
| `x2` | `dx2·sin θ − dy2·cos θ` |
| `y2` | `dy2·sin θ + dx2·cos θ` |
| `x3` | `−dx1·sin θ − dy1·cos θ` |
| `y3` | `−dy1·sin θ + dx1·cos θ` |
| `x4` | `dx1·sin θ + dy1·cos θ` |
| `y4` | `dy1·sin θ − dx1·cos θ` |

Derivation: `∂dot/∂x1 = −dx2`, `∂cross/∂x1 = −dy2`, etc.
`∂R/∂p = (∂dot/∂p)·sin θ − (∂cross/∂p)·cos θ`.

---

### 10  Midpoint

**Entities:** three points — `m` (the midpoint), `a` and `b` (the endpoints).

**Geometric meaning:** `m` is the midpoint of segment `ab`.

**Residual (2 equations):**

```
R₀ = mx − (ax + bx)/2 = 0
R₁ = my − (ay + by)/2 = 0
```

**Jacobian:**

| Parameter | ∂R₀/∂param | ∂R₁/∂param |
|-----------|------------|------------|
| `mx` | `1` | `0` |
| `my` | `0` | `1` |
| `ax` | `−½` | `0` |
| `ay` | `0` | `−½` |
| `bx` | `−½` | `0` |
| `by` | `0` | `−½` |

---

### 11  EqualLength

**Entities:** four points `(p1, p2, p3, p4)` defining two line segments.

**Geometric meaning:** segment `p1p2` has the same length as segment `p3p4`.

Let `dx1 = x2−x1`, `dy1 = y2−y1`, `dx2 = x4−x3`, `dy2 = y4−y3`.

**Residual (1 equation) — squared form:**

```
R = (dx1² + dy1²) − (dx2² + dy2²) = 0
```

**Jacobian:**

| Parameter | ∂R/∂param |
|-----------|-----------|
| `x1` | `−2dx1` |
| `y1` | `−2dy1` |
| `x2` | `2dx1` |
| `y2` | `2dy1` |
| `x3` | `2dx2` |
| `y3` | `2dy2` |
| `x4` | `−2dx2` |
| `y4` | `−2dy2` |

---

### 12  MidpointOfLineOnLine

**Entities:** four points: `(l1a, l1b)` defining line L1 and `(l2a, l2b)`
defining line L2.

**Geometric meaning:** the midpoint of L1 lies on the segment L2.

**Residual (1 equation):** `R = S(m; l2a, l2b) = 0` with `m = (l1a + l1b)/2`.

**Jacobian:** `∂S` at `m`; `l1a` and `l1b` each get half of the `(px, py)`
partials, and `l2a`, `l2b` take theirs unchanged.

---

### 13  Extension variants

The **Extension** of a Line is the infinite line through its endpoints. These
constraints measure against it instead of the segment, so sketches authored
under infinite-line semantics (PlaneGCS) keep their meaning. Each takes the
same entities as its segment counterpart and replaces the segment helpers
with `S∞`; none has a "foot on the segment" condition.

| Constraint (JSON type) | Segment counterpart | Residual |
|------------------------|---------------------|----------|
| `PointOnExtension(p, a, b)` (`on` + `extension`; GCS `point_on_extension_pl`) | PointOnLine | `R = S∞(p; a, b)` |
| `DistancePointExtension(p, a, b, d)` (`distance` + `extension`; GCS `p2l_extension_distance`) | DistancePointLine | `d > 0: R = \|S∞(p; a, b)\| − d`; `d = 0: R = S∞(p; a, b)` |
| `TangentExtensionCircle(a, b, c_center, c)` (`tangent` + `extension`; GCS `tangent_extension_lc`) | TangentLineCircle | `R = \|S∞(c_center; a, b)\| − r` (R₀ of TangentLineCircle, no R₁) |
| `MidpointOfLineOnExtension(l1a, l1b, l2a, l2b)` (`midpoint_on` + `extension`; GCS `midpoint_on_extension_ll`) | MidpointOfLineOnLine | `R = S∞(m; l2a, l2b)`, `m = (l1a + l1b)/2` |

**Jacobians:** `∂S∞` from the shared section; for the `|S∞|` forms,
`sign(S∞)·∂S∞` (and `∂R/∂r = −1` for the tangent). For the midpoint, `l1a`
and `l1b` each get half of the `(px, py)` partials. As with DistancePointLine,
`d = 0` uses the signed form so the row doesn't vanish at the solution.

### 14  SignedDistancePointExtension

`SignedDistancePointExtension(p, a, b, d, side)` (`offset`; GCS
`p2l_signed_distance`), used
by Linked Offsets. Always measured against the Extension, since an offset
line's endpoints routinely stick out past its source. `σ = −1` when `side < 0`,
else `+1` (`+1` is left of `a → b`).

```
R = σ·S∞(p; a, b) − d
∂R = σ·∂S∞
```

Unlike DistancePointExtension there is no `|·|`, so the point is held on its
side (or pulled across to it) and the row never vanishes.

### 15  Difference and Equal (operands)

These relate scalar **operands**, each a constant (a number or a Parameter's
value, resolved when the constraint is built) or a solver variable named by a
property reference (native `{entity, property}`, GCS `{o_id, prop}`): a
Point's `x`/`y` or a Circle's or Arc's `radius`.
Only variable operands are columns.

| Constraint (JSON type) | Residual | Partials (variable operands) |
|------------------------|----------|------------------------------|
| `Difference(q1, q2, δ)` (`difference`; GCS `difference`) | `R = q2 − q1 − δ` | `∂q1 = −1`, `∂q2 = +1`, `∂δ = −1` |
| `Equal(q1, q2)` (`equal` over values; GCS `equal`) | `R = q1 − q2` | `∂q1 = +1`, `∂q2 = −1` |

The same variable may appear twice (its partials add). With no variable
operand the row is constant: it holds or it is Conflicting.

### 16  PointPointAngle

`PointPointAngle(p1, p2, θ)` (`direction`; GCS `p2p_angle`): the direction of `p1 → p2` is `θ`
radians counter-clockwise from +X. With `(dx, dy) = p2 − p1` rotated by `−θ`:

```
u = dx·cosθ + dy·sinθ
v = −dx·sinθ + dy·cosθ
R = atan2(v, u)                      // signed angle from θ to p1 → p2, in (−π, π]
∂R/∂dx = −dy / (dx² + dy²)
∂R/∂dy =  dx / (dx² + dy²)
∂p1 = −∂d, ∂p2 = +∂d
```

PlaneGCS's form. Unlike `dot·sinθ − cross·cosθ` (Angle) it has no root at
`θ + π`. When `p1 = p2` the residual is 0 and the row is zero.

### 17  MirrorPointExtension

`MirrorPointExtension(A, B, a, b)` (`mirror`; GCS `mirror_point_ppl`,
`p2p_symmetric_ppl`): `B` is `A` reflected
across the Extension of the axis Line `a → b` (a mirror reflects across the
whole line, as in PlaneGCS). With `D = b − a`, `L = |D|`, unit normal
`n̂ = (−Dy, Dx) / L` and `s = S∞(A; a, b)` (the shared signed distance):

```
A' = A − 2·s·n̂
R_i = B_i − A'_i = B_i − A_i + 2·s·n̂_i              (i = x, y)

Driven: A, B.  Guides: a, b.
∂R_i/∂A_k = −δ_ik + 2·n̂_i·∂s/∂A_k = −δ_ik + 2·n̂_i·n̂_k
∂R_i/∂B_k = δ_ik
∂R_i/∂a_k = 2·(n̂_i·∂s/∂a_k − s·∂n̂_i/∂D_k)
∂R_i/∂b_k = 2·(n̂_i·∂s/∂b_k + s·∂n̂_i/∂D_k)
∂n̂_i/∂D_k = Σ_m Rot_im·(δ_mk − u_m·u_k) / L,   u = D / L,  Rot = [[0, −1], [1, 0]]
```

(`∂s/∂A` is `∂S∞` w.r.t. the point, which is `n̂`; `∂s/∂a`, `∂s/∂b` are its
partials w.r.t. the line's points.) A degenerate axis (`l² ≈ 0`) uses
`R = B − A`, with zero axis partials.

### 18  CircularInstance

`CircularInstance(p0, pk, c, θ)` (`rotation`; GCS `circular_instance`): `pk` is `p0` rotated
by `θ` (counter-clockwise) about `c`. With `v = p0 − c`:

```
R₀ = pkx − (cx + cosθ·vx − sinθ·vy)
R₁ = pky − (cy + sinθ·vx + cosθ·vy)

Driven: p0, pk.  Guide: c.
        p0x    p0y    pkx  pky  cx          cy
∂R₀ = [ −cosθ, sinθ,  1,   0,   cosθ − 1,   −sinθ    ]
∂R₁ = [ −sinθ, −cosθ, 0,   1,   sinθ,       cosθ − 1 ]
```

### 19  LinearInstance

`LinearInstance(p0, pk, d1, d2, s₀, n)` (`translation`; GCS `linear_instance`): `pk` is `p0`
moved `s = s₀·n` along the unit direction `u = D / L`, `D = d2 − d1`,
`L = |D|`:

```
R_i = pk_i − p0_i − s·u_i                         (i = x, y)

Driven: p0, pk.  Guides: d1, d2.
∂R_i/∂p0_i = −1,  ∂R_i/∂pk_i = +1
∂R_i/∂d2_k = −s·(δ_ik − u_i·u_k) / L,  ∂R_i/∂d1_k = −∂R_i/∂d2_k
```

A degenerate direction (`L ≈ 0`) uses `R = pk − p0`, with zero direction
partials.

---

### 20  Arcs

An **Arc** references a center Point `c`, a start Point `s` and an end Point
`e`, and owns three solver variables: its radius `r`, start angle `α` and end
angle `β` (`Var::Radius`, `Var::StartAngle`, `Var::EndAngle`). It sweeps
counter-clockwise from `α` to `β`; the sweep is `(β − α) mod 2π` in `(0, 2π]`
(equal angles mean a full turn).

**Span overshoot** `A(φ; α, β)` (`src/constraints/arc_span.rs`): how far a
direction angle `φ` around the center falls outside the span. With
`u = (φ − α) mod 2π` and sweep `w`: `A = 0` while `u ≤ w`; otherwise the
signed angle to the nearer end, `u − w` (past `β`, `∂A/∂(φ, α, β) = (1, 0, −1)`)
or `u − 2π` (before `α`, `∂A/∂(φ, α, β) = (1, −1, 0)`). Inside the span its
row is 0, so like the segment's `O` it removes no degree of freedom there.

| Constraint (JSON type) | Residuals |
|------------------------|-----------|
| `ArcRules(c, s, e, arc)` (`arc_rules`; GCS `arc_rules`) | `R₀ = sx − cx − r·cos α`, `R₁ = sy − cy − r·sin α`, `R₂ = ex − cx − r·cos β`, `R₃ = ey − cy − r·sin β` |
| `PointOnArc(p, c, arc)` (`on`; GCS `point_on_arc`) | `R₀ = ρ − r`, `R₁ = r·A(φ; α, β)`, with `v = p − c`, `ρ = \|v\|`, `φ = atan2(vy, vx)` |
| `TangentLineArc(a, b, c, arc)` (`tangent`; GCS `tangent_la`) | `R₀ = \|S∞(c; a, b)\| − r`, `R₁ = O(c; a, b)`, `R₂ = r·A(φ; α, β)`, `φ` the direction from `c` to the tangency point |

**ArcRules Jacobian:** `∂R₀/∂(cx, sx, r, α) = (−1, 1, −cos α, r·sin α)`,
`∂R₁/∂(cy, sy, r, α) = (−1, 1, −sin α, −r·cos α)`, and the same for `R₂`, `R₃`
with `e` and `β`. Four equations over the nine variables leave a free arc with
its endpoints five degrees of freedom (center, radius, two angles). These rows
are implicit: `SketchSystem` adds them for every arc (they join its Component,
hold exactly under soft goals and count in rank and diagnosis, but are never
reported), and an explicit `ArcRules` over an arc's own points is a no-op.

**PointOnArc Jacobian:** `∂R₀/∂p = v/ρ = −∂R₀/∂c`, `∂R₀/∂r = −1`.
`∂φ/∂(px, py) = (−vy, vx)/ρ² = −∂φ/∂(cx, cy)`; `∂R₁ = r·∂A/∂φ·∂φ` for the
points, `∂R₁/∂r = A`, `∂R₁/∂(α, β) = r·∂A/∂(α, β)`. At `ρ = 0` only
`∂R₀/∂r` is set. The circle residual is unsquared (unlike PointOnCircle) and
matches PlaneGCS's `point_on_arc` (a point-to-center distance of `r`); `R₁` is
ACS's addition, since an Arc is its span, as a Line is its segment.

**TangentLineArc Jacobian:** `R₀`, `R₁` as TangentLineCircle. The tangency
point is the foot of the perpendicular from `c`, in direction
`sign(S∞)·(dy, −dx)` from `c`, so `φ = atan2(−σ·dx, σ·dy)` with `σ = sign(S∞)`
(`+1` at 0). It depends only on the line's direction:
`∂φ/∂(ax, ay, bx, by) = (dy, −dx, −dy, dx)/L²`, and `c` doesn't turn it.
`∂R₂ = r·∂A/∂φ·∂φ` for `a`, `b`; `∂R₂/∂r = A`; `∂R₂/∂(α, β) = r·∂A/∂(α, β)`.
PlaneGCS's `tangent_la` is only `R₀` against the infinite line; ACS keeps the
tangency on the segment (as `tangent_lc`) and on the arc's span.

**Tangency at a shared point** (`TangentAtPoint(p, o, c)`). When a line
endpoint *is* one of the arc's endpoints (same id; a slot's lines), both
vocabularies build this instead of `TangentLineArc`. The distance form is
degenerate there: through a point already on the arc, `|S∞| ≤ r` with equality
only at tangency, so `R₀` changes quadratically as the line turns and its row
vanishes at the solution, which makes diagnosis report spurious Redundant
constraints and `dof` too high. With `u = o − p` (the line) and `v = p − c`
(the radius at `p`):

```
R = u·v / (|u|·|v|) = 0      (u·v when |u| or |v| ≈ 0)
```

`∂(u·v)/∂(p, o, c) = (u − v, v, −u)`; `∂|u|/∂p = −u/|u|`, `∂|u|/∂o = u/|u|`,
`∂|v|/∂p = v/|v|`, `∂|v|/∂c = −v/|v|`; `∂R = ∂(u·v)/(|u||v|) − R·(∂|u|/|u| +
∂|v|/|v|)`. `p` lies on the line (an endpoint) and on the arc (its implicit
rules), so no segment or span residual is needed.

### 21  Ellipses

An **Ellipse** (GCS's model) references a center Point `c` and a
focus Point `f` and owns one solver variable, its minor radius `b`
(`Var::MinorRadius`, JSON `radmin`). Derived: `e = f − c`, focal distance
`k = |e|`, major radius `A = √(b² + k²)`, unit major direction `u = e/k`,
minor direction `n = rot90(u) = (−u_y, u_x)`, second focus `f₂ = 2c − f`.
A free ellipse has 5 degrees of freedom.

| Constraint (JSON type) | Residuals |
|------------------------|-----------|
| `PointOnEllipse(p, c, f, E)` (`on`; GCS `point_on_ellipse`) | `R₀ = \|p − f\| + \|p − f₂\| − 2A` |
| `EllipseAxisPoint(p, c, f, E, axis)` (`ellipse_axis`) | with `d = p − c`: major `R₀ = u × d`, `R₁ = \|d\| − A`; minor `R₀ = u · d`, `R₁ = \|d\| − b` |
| `EllipseDiameter(p₁, p₂, c, f, E, axis)` (GCS `internal_alignment_ellipse_major_diameter` / `_minor_diameter`) | with `m = (p₁ + p₂)/2`, `h = (p₂ − p₁)/2`: `R₀,₁ = m − c`; major `R₂ = u × h`, `R₃ = \|h\| − A`; minor `R₂ = u · h`, `R₃ = \|h\| − b` |
| `TangentLineEllipse(a, b, c, f, E)` (`tangent`; GCS `tangent_le`) | with `L = \|b − a\|`, `g = (b − a)/L`, `ν = rot90(g)`, `sᵢ = ν·(fᵢ − a)`, `σᵢ = g·(fᵢ − a)` (`f₁ = f`): `R₀ = \|c − f + s₁ν\| − A`, `R₁ = overshoot of σ_T = (s₁σ₂ + s₂σ₁)/(s₁ + s₂)` past `[0, L]` |

All are unsquared and match GCS's semantics: `PointOnEllipse` is GCS's
residual (the focal-distance sum); `R₀` of the tangent is half of GCS's
`|f₂ − f₁'| − 2A`, `f₁' = f − 2s₁ν` being `f` mirrored in the line, since
`f₂ − f₁' = 2(c − f + s₁ν)`; the alignments place points at GCS's
`EllipsePositive/NegativeMajor/Minor` points `c ± A·u`, `c ± b·n`.

ACS's choices: `EllipseAxisPoint` (native, one point) holds `p` at *either*
end of the axis (on the axis line, at the radius from `c`), whichever it
starts by. `EllipseDiameter` holds the pair symmetric about `c` along the
axis, a diameter apart, so `{p₁, p₂}` is the two ends in either order (GCS
picks each point's end once, when the constraint is added); both are four
equations, like GCS's four alignment rows. `TangentLineEllipse`'s `R₁` keeps
the tangency point on the segment, as `TangentLineCircle`'s `O` does: by the
reflection property the tangency point is where segment `f₂ f₁'` crosses the
line, at position `σ_T` along it (`σ` is unchanged by the mirror). `R₁` is 0
(zero row) while `0 ≤ σ_T ≤ L`, `σ_T` before `a`, `σ_T − L` past `b`; it is
0 when `s₁ + s₂ ≈ 0` (the line through `c`, never tangent).

**Jacobians.** The kernels compute every quantity in forward mode
(`src/constraints/ellipse.rs`, like GCS's `DeriVector2`): each value carries
its partials w.r.t. the kernel's local variables and each operation applies
the chain rule, so the partials are exact. The building blocks:

- `∂A/∂b = b/A`, `∂A/∂f = e/A = −∂A/∂c`.
- `∂u/∂f = (I − uuᵀ)/k = −∂u/∂c`; `∂n = rot90(∂u)`.
- `∂|v|/∂v = v/|v|`; `∂(u × v) = (v_y, −v_x)·∂u + (−u_y, u_x)·∂v`;
  `∂(u · v) = v·∂u + u·∂v`.
- `∂g/∂b = (I − ggᵀ)/L = −∂g/∂a`; `∂ν = rot90(∂g)`; `∂L/∂b = g = −∂L/∂a`.

For example `PointOnEllipse`: with `ŵ₁ = (p − f)/|p − f|`,
`ŵ₂ = (p − f₂)/|p − f₂|`: `∂R₀/∂p = ŵ₁ + ŵ₂`,
`∂R₀/∂f = −ŵ₁ + ŵ₂ − 2e/A`, `∂R₀/∂c = −2ŵ₂ + 2e/A`, `∂R₀/∂b = −2b/A`
(GCS's `ConstraintPointOnEllipse::grad`). Degenerate cases: a zero vector's
length has zero partials, and with `k ≈ 0` (focus on the center, a circle)
`u` is taken as `+X` with no partials.

---

## ConstraintType Enum Summary (updated)

```rust
pub enum ConstraintType {
    // Existing
    Vertical(String, String),
    Horizontal(String, String),
    Parallel(String, String, String, String),
    EqualX(String, f64),
    EqualY(String, f64),
    Coincident(String, String),
    PointOnLine(String, String, String),
    EqualRadius(String, String),

    // New / updated
    FixedRadius(String, f64),                          // circle_id, radius
    PointOnCircle(String, String, String),             // point_id, circle_center_id, circle_id
    Tangent(String, String, String, String),           // c1_center_id, c1_id, c2_center_id, c2_id
    TangentLineCircle(String, String, String, String), // line_pa, line_pb, circle_center_id, circle_id
    Concentric(String, String),                        // center1_point_id, center2_point_id
    DistancePointPoint(String, String, f64),           // p1_id, p2_id, distance
    DistancePointLine(String, String, String, f64),    // point_id, line_pa, line_pb, distance
    Angle(String, String, String, String, f64),        // l1p1, l1p2, l2p1, l2p2, angle_radians
    Midpoint(String, String, String),                  // midpoint_id, endpoint_a_id, endpoint_b_id
    EqualLength(String, String, String, String),       // l1p1, l1p2, l2p1, l2p2
    Perpendicular(String, String, String, String),     // l1p1, l1p2, l2p1, l2p2
    MidpointOfLineOnLine(String, String, String, String), // l1p1, l1p2, l2p1, l2p2

    // Extension variants
    PointOnExtension(String, String, String),             // point_id, line_pa, line_pb
    DistancePointExtension(String, String, String, f64),  // point_id, line_pa, line_pb, distance
    TangentExtensionCircle(String, String, String, String), // line_pa, line_pb, circle_center_id, circle_id
    MidpointOfLineOnExtension(String, String, String, String), // l1p1, l1p2, l2p1, l2p2

    // Linked Offsets and property relations
    SignedDistancePointExtension(String, String, String, f64, f64), // point_id, line_pa, line_pb, distance, side
    Difference(Operand, Operand, Operand),                // param1, param2, difference
    Equal(Operand, Operand),                              // param1, param2
    // Arcs
    ArcRules(String, String, String, String),       // center_id, start_id, end_id, arc_id
    PointOnArc(String, String, String),             // point_id, arc_center_id, arc_id
    TangentLineArc(String, String, String, String), // line_pa, line_pb, arc_center_id, arc_id

    // Arrays and mirror
    PointPointAngle(String, String, f64),                 // p1_id, p2_id, angle
    MirrorPointExtension(String, String, String, String), // pa_id, pb_id, axis_pa, axis_pb
    CircularInstance(String, String, String, f64),        // p0_id, pk_id, center_id, angle
    LinearInstance(String, String, String, String, f64, f64), // p0_id, pk_id, dir_p1, dir_p2, base_distance, n

    // Ellipses
    PointOnEllipse(String, String, String, String),             // point_id, center_id, focus1_id, ellipse_id
    TangentLineEllipse(String, String, String, String, String), // line_pa, line_pb, center_id, focus1_id, ellipse_id
    EllipseAxisPoint(String, String, String, String, EllipseAxis), // point_id, center_id, focus1_id, ellipse_id, Major|Minor
    EllipseDiameter(String, String, String, String, String, EllipseAxis), // p1_id, p2_id, center_id, focus1_id, ellipse_id, Major|Minor
}
```

---

## Solver Notes

All residuals are designed so the solver's tolerance check `‖R‖∞ < 1e-10` is
meaningful in physical units (coordinates / radii in whatever unit the caller
uses). The same threshold is used by the pre-solver to decide whether a
component is already satisfied and can be skipped, and the only condition under
which a solve reports `Converged`.

The squared-distance form used by `DistancePointPoint`, `EqualLength`, and
`PointOnCircle` keeps residuals and Jacobians smooth everywhere (no
discontinuities at zero), but note the residual magnitude is in units².  For
well-conditioned problems this is fine; if large coordinate values are involved,
consider normalising geometry before solving.

The `Angle` residual mixes units (`dot · sin θ` has units of length²), so for
problems with very different length scales, a scaling pre-pass may help
convergence.

**Guides in a solve.** Components are grouped by everything a constraint
reads, Guides included, and Guides are Component columns like any other
variable. An ordinary solve, the degrees-of-freedom analysis and diagnosis
use the full Jacobian, so real constraints move a free Guide as they need,
and a copy of a free Guide is not fully constrained. A solve with soft goals
(a drag) holds every Guide instead: the constraints that read it as a Guide
read it from a snapshot taken when the solve starts and contribute no Guide
partials, so a drag of a copy never moves it. If that solve converged but
other constraints moved a Guide (so the live residuals aren't solved), the
Component is solved again from a fresh snapshot, up to 50 times.
