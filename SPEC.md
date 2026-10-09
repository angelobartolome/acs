# ACS Constraint Solver — Constraint Specification

This document specifies every constraint type in ACS: the geometric meaning, the
parametric entities involved, the residual equations fed to the Dog-Leg solver, and
the analytical Jacobian used during each iteration.

Constraints are named here by their internal `ConstraintType`. The JSON name
in parentheses is the native type that reaches each one; `USAGE.md` lists the
vocabulary in full.

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
Guides included, are exact like every other, so every solve, drags
included, moves a free Guide as the constraints need (see Solver Notes).

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
under infinite-line semantics keep their meaning. Each takes the
same entities as its segment counterpart and replaces the segment helpers
with `S∞`; none has a "foot on the segment" condition.

| Constraint (JSON type) | Segment counterpart | Residual |
|------------------------|---------------------|----------|
| `PointOnExtension(p, a, b)` (`on` + `extension`) | PointOnLine | `R = S∞(p; a, b)` |
| `DistancePointExtension(p, a, b, d)` (`distance` + `extension`) | DistancePointLine | `d > 0: R = \|S∞(p; a, b)\| − d`; `d = 0: R = S∞(p; a, b)` |
| `TangentExtensionCircle(a, b, c_center, c)` (`tangent` + `extension`) | TangentLineCircle | `R = \|S∞(c_center; a, b)\| − r` (R₀ of TangentLineCircle, no R₁) |
| `MidpointOfLineOnExtension(l1a, l1b, l2a, l2b)` (`midpoint` `[line, line]` + `extension`) | MidpointOfLineOnLine | `R = S∞(m; l2a, l2b)`, `m = (l1a + l1b)/2` |

**Jacobians:** `∂S∞` from the shared section; for the `|S∞|` forms,
`sign(S∞)·∂S∞` (and `∂R/∂r = −1` for the tangent). For the midpoint, `l1a`
and `l1b` each get half of the `(px, py)` partials. As with DistancePointLine,
`d = 0` uses the signed form so the row doesn't vanish at the solution.

### 14  SignedDistancePointExtension

`SignedDistancePointExtension(p, a, b, d, side)` (`offset`), used
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
property reference (`{entity, property}`): a
Point's `x`/`y` or a Circle's or Arc's `radius`.
Only variable operands are columns.

| Constraint (JSON type) | Residual | Partials (variable operands) |
|------------------------|----------|------------------------------|
| `Difference(q1, q2, δ)` (`difference`) | `R = q2 − q1 − δ` | `∂q1 = −1`, `∂q2 = +1`, `∂δ = −1` |
| `Equal(q1, q2)` (`equal` over values) | `R = q1 − q2` | `∂q1 = +1`, `∂q2 = −1` |

The same variable may appear twice (its partials add). With no variable
operand the row is constant: it holds or it is Conflicting.

The horizontal and vertical distance dimensions are built as `Difference`
over coordinates (no kernel of their own; one row, so diagnosis reports one
constraint):

| JSON type | Built as | Residual | Partials |
|-----------|----------|----------|----------|
| `horizontal_distance` (`a`, `b` or `line` `p1→p2`) | `Difference(X(a), X(b), value)` | `R = b.x − a.x − value` | `∂a.x = −1`, `∂b.x = +1` |
| `vertical_distance` (`a`, `b` or `line` `p1→p2`) | `Difference(Y(a), Y(b), value)` | `R = b.y − a.y − value` | `∂a.y = −1`, `∂b.y = +1` |

Signed and linear: a negative `value` puts `b` left of (below) `a`, and the
row never vanishes.

### 16  PointPointAngle

`PointPointAngle(p1, p2, θ)` (`direction`): the direction of `p1 → p2` is `θ`
radians counter-clockwise from +X. With `(dx, dy) = p2 − p1` rotated by `−θ`:

```
u = dx·cosθ + dy·sinθ
v = −dx·sinθ + dy·cosθ
R = atan2(v, u)                      // signed angle from θ to p1 → p2, in (−π, π]
∂R/∂dx = −dy / (dx² + dy²)
∂R/∂dy =  dx / (dx² + dy²)
∂p1 = −∂d, ∂p2 = +∂d
```

Unlike `dot·sinθ − cross·cosθ` (Angle) it has no root at
`θ + π`. When `p1 = p2` the residual is 0 and the row is zero.

### 17  MirrorPointExtension

`MirrorPointExtension(A, B, a, b)` (`mirror`): `B` is `A` reflected
across the Extension of the axis Line `a → b` (a mirror reflects across the
whole line). With `D = b − a`, `L = |D|`, unit normal
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

`CircularInstance(p0, pk, c, θ)` (`rotation`): `pk` is `p0` rotated
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

`LinearInstance(p0, pk, d1, d2, s₀, n)` (`translation`): `pk` is `p0`
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
| `ArcRules(c, s, e, arc)` (implicit, every arc) | `R₀ = sx − cx − r·cos α`, `R₁ = sy − cy − r·sin α`, `R₂ = ex − cx − r·cos β`, `R₃ = ey − cy − r·sin β` |
| `PointOnArc(p, c, arc)` (`on`) | `R₀ = ρ − r`, `R₁ = r·A(φ; α, β)`, with `v = p − c`, `ρ = \|v\|`, `φ = atan2(vy, vx)` |
| `TangentLineArc(a, b, c, arc)` (`tangent`) | `R₀ = \|S∞(c; a, b)\| − r`, `R₁ = O(c; a, b)`, `R₂ = r·A(φ; α, β)`, `φ` the direction from `c` to the tangency point |
| `MidpointOfArc(p, c, arc)` (`midpoint` `[point, arc]`) | `R₀ = px − cx − r·cos m`, `R₁ = py − cy − r·sin m`, `m = α + w/2`, `w` the sweep |

**ArcRules Jacobian:** `∂R₀/∂(cx, sx, r, α) = (−1, 1, −cos α, r·sin α)`,
`∂R₁/∂(cy, sy, r, α) = (−1, 1, −sin α, −r·cos α)`, and the same for `R₂`, `R₃`
with `e` and `β`. Four equations over the nine variables leave a free arc with
its endpoints five degrees of freedom (center, radius, two angles). These rows
are implicit: `SketchSystem` adds them for every arc (they join its Component,
hold exactly under soft goals and count in rank and diagnosis, but are never
reported); no JSON type builds them.

**PointOnArc Jacobian:** `∂R₀/∂p = v/ρ = −∂R₀/∂c`, `∂R₀/∂r = −1`.
`∂φ/∂(px, py) = (−vy, vx)/ρ² = −∂φ/∂(cx, cy)`; `∂R₁ = r·∂A/∂φ·∂φ` for the
points, `∂R₁/∂r = A`, `∂R₁/∂(α, β) = r·∂A/∂(α, β)`. At `ρ = 0` only
`∂R₀/∂r` is set. The circle residual is unsquared (unlike PointOnCircle) and
is a point-to-center distance of `r`; `R₁` is
ACS's addition, since an Arc is its span, as a Line is its segment.

**MidpointOfArc Jacobian:** the middle of the span by angle, so `p` is on
the circle (2 equations: a free point at the middle of a free arc loses 2
DOF). Between wraps `w = β − α + 2πk` for a fixed integer `k`, so
`m = (α + β)/2 + πk` and `∂m/∂α = ∂m/∂β = ½`:
`∂R₀/∂(px, cx, r, α, β) = (1, −1, −cos m, ½r·sin m, ½r·sin m)`,
`∂R₁/∂(py, cy, r, α, β) = (1, −1, −sin m, −½r·cos m, −½r·cos m)`.
Where the sweep wraps (`β` passing `α`, between a full turn and none) `m`
jumps by `π` and the middle flips to the opposite side, as the arc itself
jumps there; `(α + β)/2` alone would be the wrong side whenever the span
crosses `±π` (`α = 1`, `β = −1`: the middle is at `π`, not `0`).

**TangentLineArc Jacobian:** `R₀`, `R₁` as TangentLineCircle. The tangency
point is the foot of the perpendicular from `c`, in direction
`sign(S∞)·(dy, −dx)` from `c`, so `φ = atan2(−σ·dx, σ·dy)` with `σ = sign(S∞)`
(`+1` at 0). It depends only on the line's direction:
`∂φ/∂(ax, ay, bx, by) = (dy, −dx, −dy, dx)/L²`, and `c` doesn't turn it.
`∂R₂ = r·∂A/∂φ·∂φ` for `a`, `b`; `∂R₂/∂r = A`; `∂R₂/∂(α, β) = r·∂A/∂(α, β)`.
`R₀` alone would be tangency to the infinite line; ACS keeps the tangency on
the segment (as line–circle `tangent`) and on the arc's span.

**Tangency at a shared point** (`TangentAtPoint(p, o, c)`). When a line
endpoint *is* one of the arc's endpoints (same id; a slot's lines),
`tangent` builds this instead of `TangentLineArc`. The distance form is
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

**Curve–curve tangency** (`tangent` between circles and arcs; `TangentCirclesInternal(c₁, C₁, c₂, C₂)`, `TangentCircleArc(c₁, C, c₂, arc, internal)`, `TangentArcs(c₁, arc₁, c₂, arc₂, internal)`, all `TangentCurvesConstraint` in `src/constraints/tangent_curves.rs`; external circle–circle stays §4's `Tangent`). The same kernel with a gap `g` is `distance` circle–arc and arc–arc (`DistanceCircleArc(c₁, C, c₂, arc, g, internal)`, `DistanceArcs(c₁, arc₁, c₂, arc₂, g, internal)`): the curves held `g` apart along the line of centers, outside or inside, with the nearest points on the spans; tangency is `g = 0`, and `g` doesn't enter the Jacobian. With `w = c₂ − c₁`, `d = |w|`, and for inside tangency `s = sign(r₁ − r₂)` (`+1` at equal radii):

| Residual | Value |
|----------|-------|
| `R₀` (external) | `d − (r₁ + r₂) − g` |
| `R₀` (internal) | `d − s·(r₁ − r₂) + g` |
| `Rₖ`, one per Arc side, `a` then `b` | `rₖ·A(φₖ; αₖ, βₖ)` |

The tangency point is on the line of centers: from `c₁` in direction `+w` and
from `c₂` in direction `−w` (external), or from both in direction `s·w`
(inside, from the larger circle's center through the smaller's). `φₖ` is that
direction's angle. `internal` is chosen by the caller and never inferred, so a
solve can't flip one form into the other; `s` follows the radii, which can
only swap order through concentric equal circles.

**Jacobian:** `∂R₀/∂c₂ = w/d = −∂R₀/∂c₁`; `∂R₀/∂(r₁, r₂) = (−1, −1)`
(external) or `(−s, s)` (internal). At `d ≈ 0` only the radius partials are
set. `φ` turns with `w` alone, either way along it:
`∂φ/∂c₁ = (w_y, −w_x)/d² = −∂φ/∂c₂`; `∂Rₖ = rₖ·∂A/∂φ·∂φ` for the centers,
`∂Rₖ/∂rₖ = A`, `∂Rₖ/∂(αₖ, βₖ) = rₖ·∂A/∂(α, β)`.

**Arcs tangent at a shared endpoint** (`TangentArcsAtPoint(p, c₁, c₂,
internal)`). When the two arcs share an endpoint Point (same id; a fillet
chain), the native catalog builds this instead of `TangentArcs`, for the same
reason as `TangentAtPoint`: through a point on both circles, `d` can only fall
short of `r₁ + r₂` (or exceed `|r₁ − r₂|`), so `R₀` changes quadratically as an
arc turns about the point and its row vanishes at tangency. With `v₁ = p − c₁`,
`v₂ = p − c₂` and `θ = atan2(v₁ × v₂, v₁ · v₂)`, the angle from `v₁` to `v₂`:

```
R = θ                                   (internal: centers on the same side of p)
R = atan2(−v₁ × v₂, −v₁ · v₂) = θ − π   (external: opposite sides), wrapped to (−π, π]
(v₁ × v₂ when |v₁| or |v₂| ≈ 0)
```

Each has a single zero, the side named. `∂θ/∂v₁ = (v₁y, −v₁x)/|v₁|²`,
`∂θ/∂v₂ = (−v₂y, v₂x)/|v₂|²` (both forms), and `∂v₁/∂p = ∂v₂/∂p = I`,
`∂v₁/∂c₁ = ∂v₂/∂c₂ = −I`. `p` lies on both spans (an endpoint of each), so no
span residual is needed.

### 21  Ellipses

An **Ellipse** (GCS's model) references a center Point `c` and a
focus Point `f` and owns one solver variable, its minor radius `b`
(`Var::MinorRadius`, JSON `radmin`). Derived: `e = f − c`, focal distance
`k = |e|`, major radius `A = √(b² + k²)`, unit major direction `u = e/k`,
minor direction `n = rot90(u) = (−u_y, u_x)`, second focus `f₂ = 2c − f`.
A free ellipse has 5 degrees of freedom.

| Constraint (JSON type) | Residuals |
|------------------------|-----------|
| `PointOnEllipse(p, c, f, E)` (`on`) | `R₀ = \|p − f\| + \|p − f₂\| − 2A` |
| `EllipseAxisPoint(p, c, f, E, axis)` (`ellipse_axis`) | with `d = p − c`: major `R₀ = u × d`, `R₁ = \|d\| − A`; minor `R₀ = u · d`, `R₁ = \|d\| − b` |
| `EllipseDiameter(p₁, p₂, c, f, E, axis)` (`ellipse_axis` with `a`, `b`) | with `m = (p₁ + p₂)/2`, `h = (p₂ − p₁)/2`: `R₀,₁ = m − c`; major `R₂ = u × h`, `R₃ = \|h\| − A`; minor `R₂ = u · h`, `R₃ = \|h\| − b` |
| `TangentLineEllipse(a, b, c, f, E)` (`tangent`) | with `L = \|b − a\|`, `g = (b − a)/L`, `ν = rot90(g)`, `sᵢ = ν·(fᵢ − a)`, `σᵢ = g·(fᵢ − a)` (`f₁ = f`): `R₀ = \|c − f + s₁ν\| − A`, `R₁ = overshoot of σ_T = (s₁σ₂ + s₂σ₁)/(s₁ + s₂)` past `[0, L]` |
| `TangentLineEllipseAtPoint(p, o, c, f, E)` (`tangent` whose line endpoint `p` an `on` holds on the curve) | `R = ĝ × τ̂(t(p))`, `ĝ = (o − p)/\|o − p\|`, `τ̂` the unit tangent at `p`'s parametric angle `t(p)` (§24's `τ` and `t(p)`) |

**Tangency at a held endpoint** (`TangentLineEllipseAtPoint`). When a real
`on` holds a Line endpoint `p` on the same ellipse (or elliptical arc, §24)
that the Line is `tangent` to, `constraint_catalog::tangent_at_held_endpoints`
builds this instead of `TangentLineEllipse` (or `TangentLineEllipticalArc`),
as it builds `TangentAtPoint` for a circle or arc (§20). Through a point
already on the curve, `R₀` only changes quadratically as `p` slides along
the line: `p` drifts from the tangent point (~2e-5 on a 5 × 4 ellipse) and
diagnosis reports the `tangent` Redundant with `dof` one too high. The angle
between the line and the tangent at `p` changes linearly. It reads `p`'s
angle `t(p)`, not a variable; the `on` keeps `p` on the curve (and an
elliptical arc's span), and `p` is on the segment, so both forms have the
same solutions. Its `E` is the ellipse's or the elliptical arc's id (both
own `b` as `Var::MinorRadius`). Temporary `on`/`tangent` are left alone.

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

### 22  Collinear and Normal

`Collinear(a1, a2, b1, b2)` (native `collinear {a, b}`): both endpoints of
Line B lie on Line A's Extension. Two residuals, one constraint, so
diagnosis reports it Redundant or Conflicting as a unit:

`R₀ = S∞(b1; a1, a2)`,  `R₁ = S∞(b2; a1, a2)`

**Jacobian:** `∂S∞` from the shared section, per row: the `(px, py)`
partials go to that row's endpoint of B, the `(ax, ay, bx, by)` partials to
`a1`, `a2`. The segments need not overlap. When B shares an endpoint with A
(same Point), that row is identically 0 with a zero Jacobian row; the other
row still removes the one degree of freedom, and since diagnosis drops whole
constraints the zero row is never reported Redundant on its own.

Normal (native `normal {line, curve}`, curve a circle or arc) has no kernel
of its own: it is `PointOnExtension(center, line_pa, line_pb)`, the line's
Extension passing through the curve's center. Direction only: neither the
segment nor an arc's span is checked, so the line need not reach the curve.

### 23  Dimensions and distances to circles and lines

Dimensions hold the number given; the distances are **signed gaps**, so each
residual crosses zero linearly (negative when the entities overlap) instead
of touching it from one side. `σ = −1` for `internal`, else `+1`.
`u = (p − c)/|p − c|` (zero when `|p − c| ≈ 0`).

| Constraint (JSON) | Residuals | Partials |
|-------------------|-----------|----------|
| `Diameter(c, D)` (`diameter`) | `R = 2r − D` | `∂r = 2` |
| `ArcLength(A, ℓ)` (`length`, arc) | `R = r·s − ℓ`, `s = (β − α) mod 2π` in `(0, 2π]` (0 → full turn) | `∂r = s`, `∂α = −r`, `∂β = r` |
| `ArcSweep(A, θ)` (`angle`, arc; `0 < θ < 2π`, else rejected) | `R = wrap(β − α − θ)`, `wrap(t) = ((t + π) mod 2π) − π` | `∂α = −1`, `∂β = 1` |
| `DistancePointCircle(p, c, C, d, internal)` (`distance` point–circle) | `R = σ(\|p − c\| − r) − d` | `∂p = σu`, `∂c = −σu`, `∂r = −σ` |
| `DistanceLineCircle(a, b, c, C, d)` (`distance` line–circle) | `R = D(c, ab) − r − d`, `D` the unsigned segment distance (Segment geometry) | `∂(c, a, b)` from `segment_distance`, `∂r = −1` |
| `DistanceExtensionCircle(a, b, c, C, d)` (`distance` line–circle, `extension`) | `R = \|C/L\| − r − d` (as `TangentExtensionCircle`) | `sign(C)·∂(C/L)`, `∂r = −1` |
| `DistancePointArc(p, c, A, d, internal)` (`distance` point–arc) | `R₀ = σ(\|p − c\| − r) − d`; `R₁ = r·overshoot(φ; α, β)`, `φ = atan2(p − c)` (as `PointOnArc`) | `R₀` as `DistancePointCircle`; `R₁`: `∂p = r·o′·(−v_y, v_x)/ρ²`, `∂c = −∂p`, `∂r = overshoot`, `∂α, ∂β = r·∂overshoot` |
| `DistanceLineArc(a, b, c, A, d)` (`distance` line–arc) | `R₀ = D(c, ab) − r − d`; `R₁ = r·overshoot(φ; α, β)`, `φ` the direction of `q − c`, `q` the segment's nearest point to `c` | `R₀` as `DistanceLineCircle`; `R₁`: foot (`0 ≤ t ≤ 1`) `∂(a, b) = r·o′·(dy, −dx, −dy, dx)/L²`, `∂c = 0`; past an end `q = e`: `∂e = r·o′·(−v_y, v_x)/\|v\|²`, `∂c = −∂e`, `v = e − c` |
| `DistanceExtensionArc(a, b, c, A, d)` (`distance` line–arc, `extension`) | `R₀ = \|C/L\| − r − d`; `R₁` with `q` always the foot (as `TangentLineArc`'s span row) | `R₀` as `DistanceExtensionCircle`; `R₁` the foot case above |
| `DistanceCircleCircle(c₁, C₁, c₂, C₂, d, false)` | `R = \|c₂ − c₁\| − r₁ − r₂ − d` | with `w = (c₂ − c₁)/\|c₂ − c₁\|`: `∂c₁ = −w`, `∂c₂ = w`, `∂r₁ = ∂r₂ = −1` |
| `DistanceCircleCircle(…, d, true)` (`internal`) | `R = \|r₁ − r₂\| − \|c₂ − c₁\| − d` | `∂c₁ = w`, `∂c₂ = −w`, `∂r₁ = τ`, `∂r₂ = −τ`, `τ = +1` if `r₁ ≥ r₂` else `−1` |
| `DistanceLineLine(a₁, a₂, b₁, b₂, d)` (`distance` line–line) | `R₀ = ς·C(b₁)/L − d`, `R₁ = ς·C(b₂)/L − d`, signed distances to the line through `a₁, a₂`; `ς = +1` if `C(b₁) + C(b₂) ≥ 0` else `−1` | `ς·∂(C/L)` per row (Segment geometry), `p` = `b₁` or `b₂` |

`ArcSweep` unwraps the sweep near the target `θ`: a sweep taken mod 2π jumps by
2π where the end angle crosses the start (the 0/2π seam), as a drag across it
does; `wrap` is continuous there and jumps only half a turn from `θ`.
`ArcLength`'s sweep jumps where the arc itself does, between a full turn and
none. `DistanceLineLine`'s two equations make `b` parallel to `a` (an explicit
`Parallel` on the pair is redundant). The internal circle–circle gap is
symmetric in its circles; it is kinked only at `r₁ = r₂` (where the gap is
`−|c₂ − c₁| ≤ 0`). Distances to arcs (circle–arc and arc–arc) don't clamp
the gap to the span (that residual would be kinked); like curve–curve
tangency (§ above), they keep the gap row smooth and add one span row per
arc. Point–arc and line–arc do the same: `R₀` is the point–circle or
line–circle gap, smooth, and `R₁` (`o′ = ∂overshoot/∂φ`) keeps the ray from
the center through the nearest point `q` on the span. Off the span the gap
isn't measured to the arc's endpoint (with a signed gap that residual would
jump sign there, for a point inside the circle); the solve turns the ray
back onto the span instead. `R₁` is 0 with a zero row on the span (like
`PointOnArc`'s), and continuous where `q` switches from the foot to an
endpoint (its partials jump there).

---

### 24  Elliptical arcs

An **EllipticalArc** is an Ellipse (§21: center `c`, focus `f`, minor radius
`b`, so `A`, `u`, `n` as there) plus start and end Points `s`, `e` and two
more variables, its start and end angles `α`, `β` (`Var::StartAngle`,
`Var::EndAngle`; its values are `[b, α, β]`, so `Var::MinorRadius` reads its
`b` and every §21 kernel takes it). The angles are *parametric*: the point
at `t` is `P(t) = c + A·cos t·u + b·sin t·n`, with tangent
`τ(t) = P'(t) = −A·sin t·u + b·cos t·n`. A point `p`'s parametric angle is
`t(p) = atan2(A·(d·n), b·(d·u))`, `d = p − c` (both arguments scaled by
`A·b > 0`, so `b → 0` stays finite). It sweeps counter-clockwise from `α` to
`β`; `overshoot(t, α, β)` is §20's span measure (`span_overshoot`). A free
elliptical arc has 7 degrees of freedom.

| Constraint (JSON) | Residuals |
|-------------------|-----------|
| `EllipticalArcRules(c, f, s, e, E)` (implicit, one per elliptical arc) | `R₀,₁ = s − P(α)`, `R₂,₃ = e − P(β)` |
| `PointOnEllipticalArc(p, c, f, E)` (`on`) | `R₀ = \|p − f\| + \|p − f₂\| − 2A` (as `PointOnEllipse`), `R₁ = A · overshoot(t(p), α, β)` |
| `TangentLineEllipticalArc(a, b, c, f, E)` (`tangent` line–elliptical arc) | `R₀`, `R₁` as `TangentLineEllipse`, `R₂ = A · overshoot(t(T), α, β)` with `T = a + σ_T·g` its tangency point (`R₂ = 0` when `T` is undefined, as `R₁`) |
| `TangentLineEllipticalArcAtPoint(p, o, c, f, E, end)` (`tangent`, line endpoint = arc endpoint) | `R = ĝ × τ̂(θ)`, `ĝ = (o − p)/\|o − p\|`, `θ` the angle of that end (`α` or `β`) |
| `TangentArcEllipticalArcAtPoint(p, k, c, f, E, end)` (`tangent` arc–elliptical arc, shared endpoint) | `R = r̂ · τ̂(θ)`, `r̂ = (p − k)/\|p − k\|` the Arc's radius at `p` |

The at-point forms are chosen, as `TangentAtPoint` is for an Arc, when the
Line's endpoint (or the Arc's) *is* the elliptical arc's endpoint: through a
point already on the curve the segment-and-span form only changes
quadratically. They read the end's angle variable rather than the point's
angle, which the rules tie to it. A Line endpoint that is not the arc's
endpoint but that a real `on` holds on the elliptical arc gets §21's
`TangentLineEllipseAtPoint` instead (`R = ĝ × τ̂(t(p))`, the point's own
angle `t(p)`), for the same reason. An Arc and an elliptical arc have no other
tangency: the catalog row's check rejects a pair with no shared endpoint.

**Jacobians.** Forward mode, as §21, with `∂ sin t = cos t·∂t`,
`∂ cos t = −sin t·∂t` and `∂ atan2(y, x) = (x·∂y − y·∂x)/(x² + y²)` (zero at
the origin); `overshoot` contributes `(∂/∂t, ∂/∂α, ∂/∂β)` from
`span_overshoot`, chained through `t(p)` or `t(T)`. `tests/jacobian_fd_test.rs`
checks every kernel, the rules included.

### 25  Splines

A **Spline** is a clamped [B-spline](https://en.wikipedia.org/wiki/B-spline)
`C(t)` whose handles are Points (`src/spline.rs`, `src/geometry.rs`
`Spline`). It owns no solver variables: the curve is a function of its
handles, so a free spline has `2n` degrees of freedom for `n` handles and
every point constraint (`coincident`, `fixed`, drags, …) works on them
unchanged.

- **Control points** (`interpolated: false`): degree `p = min(3, n − 1)`,
  knots as given (validated: `n + p + 1` of them, non-decreasing, clamped)
  or clamped uniform on `[0, 1]`. `∂C/∂Pᵢ = Nᵢ(t)`.
- **Fit points** (`interpolated: true`, `n ≥ 2`): a cubic through
  `Q₀ … Qₙ₋₁` at centripetal parameters `u₀ = 0`, `uᵢ = Σ_{k<i} Δₖ / ΣΔ`,
  `uₙ₋₁ = 1`, `Δₖ = (|Qₖ₊₁ − Qₖ|² + ε²)^¼` (ε = 1e-12: coincident fit points
  give a 1e-6 step rather than a repeated knot, and `Δ` stays smooth at 0),
  knots `[0,0,0,0, u₁ … uₙ₋₂, 1,1,1,1]`, and natural end conditions. The
  `n + 2` control points solve `A(u)·P = [0; Q₀; …; Qₙ₋₁; 0]`, rows
  `C″(0) = 0`, `C(uᵢ) = Qᵢ`, `C″(1) = 0`. Two fit points give a straight
  segment.

The JSON response reports each spline's `curve` (degree, knots, control
points), so either form can be rebuilt from the other: a fit-point spline's
`curve`, sent as a control-point spline, is the same curve
(`tests/spline_test.rs`).

**Jacobians: exact, through the parameterisation.** The control points of
a fit-point spline depend on its fit points both linearly (the right-hand
side) and nonlinearly (`A` depends on the knots, which are the centripetal
parameters, which depend on the chord lengths). Nothing is held fixed
within an iteration: the curve is built and evaluated in forward mode
(`Dv`, a value with a dynamic vector of partials, the `D<N>` of §21 sized at
run time), through the parameters, the knots, the basis functions and their
derivatives ([The NURBS Book](https://en.wikipedia.org/wiki/Non-uniform_rational_B-spline)
A2.3, with knots as variables), and Gaussian elimination with partial
pivoting (pivots chosen by value, piecewise constant). `tests/jacobian_fd_test.rs`
checks every kernel in both forms.

**Past the ends.** Evaluated outside `[t₀, t₁]`, the curve continues along
its end tangent, `C(t₁) + (t − t₁)·C′(t₁)` (C¹). A contact parameter a step
carries past an end then lands on a gently varying curve while its span row
(below) brings it back; continuing the end spans' cubics instead swings away
fast enough to stall Dog-Leg from a poor start.

**Curve parameters (design note).** `on` and `tangent` need to know *where*
on the curve the contact is. ACS holds that as a solver variable, a **curve
parameter** (`Var::CurveParam`, a `CurveParam` entity), owned by the constraint
that places the contact, the way an Arc holds its angles (and as GCS holds
a point's B-spline parameter). `ConstraintSolver::add_constraint` creates it
under an ID the constraint names (the catalog uses `"<constraint id>#k"`;
an ID an entity already has is rejected) and starts it at the nearest
contact: for `on`, the point's closest point on the curve (sampling plus
Newton); for `tangent`, the best of a sampling, refined by Gauss–Newton on
the constraint's own rows over its parameters alone, so a sketch that
already holds starts exactly at its contact and the pre-solver skips it.
The alternative, finding the contact inside `eval` (a projection) with the
partials by the envelope theorem, needs no new variable but re-picks the
contact at every evaluation: a branch can jump mid-solve, the tangency of
two splines (a two-dimensional critical point) has no robust choice, and
a circle could only touch from the side the nearest point is on. With the
parameter as an unknown the contact moves smoothly and every form is a set
of plain equations.

The parameter enters the analysis like any free variable, with three rules
for what it isn't (an entity's value):

- `dof` counts a real constraint's parameters as free variables, which its
  own rows take back (`on`: two equations, one parameter: one degree of
  freedom less). Parameters of temporary constraints are left out, like
  their rows.
- `fullyConstrained` never lists a parameter. A spline is listed when all
  its handles are.
- Redundant (§ `SketchSystem::diagnose`): dropping a constraint also drops
  each parameter it owns that no kept constraint reads, so the rank
  test is `rank(J_kept) + (vanished parameters) = rank(J)`. A second `on`
  holding the same point on the same spline brings two rows and a
  parameter, rank one more, and is reported Redundant. No other constraint
  owns a parameter, so nothing else changes.

**Residuals.** `q = C(t)`, `τ̂ = C′(t)/|C′(t)|`, `n̂ = rot90(τ̂)`;
`span(t)` = `t − t₀` before the domain, `t − t₁` past it, 0 inside (an Arc's
span overshoot for a parameter); `∠(u, τ̂)` the angle from `τ̂` to the unit
direction `u` modulo `π`, in `(−π/2, π/2]` (0 when parallel either way;
derivative 1 except at the jump at perpendicular); `S` the length of the
spline's control polygon (with partials), which weighs an angle against
distances.

| Constraint (JSON) | Residuals |
|-------------------|-----------|
| `PointOnSpline(p, C, t)` (`on`) | `R₀,₁ = p − q`, `R₂ = span(t)` |
| `TangentLineSpline(a, b, C, t)` (`tangent` line–spline) | with `g = (b − a)/\|b − a\|`, `σ = g·(q − a)`, `f = a + σ·g`: `R₀ = n̂·(f − q)`, `R₁ = S·∠(g, τ̂)`, `R₂` = overshoot of `σ` past `[0, \|b − a\|]` (the segment row of `TangentLineCircle`), `R₃ = span(t)` |
| `TangentCircleSpline(c, Γ, C, t)` (`tangent` circle–spline) | with `v = q − c`: `R₀ = \|v\| − r`, `R₁ = v·τ̂`, `R₂ = span(t)` |
| `TangentArcSpline(c, Γ, C, t)` (`tangent` arc–spline) | the circle's `R₀`, `R₁`, then `R₂ = r·A(atan2(v); α, β)` (§20), `R₃ = span(t)` |
| `TangentEllipseSpline(c, f, E, C, t)` (`tangent` ellipse–spline) | `R₀ = \|q − f\| + \|q − f₂\| − 2A` (§21), `R₁ = S·∠(rot90(m̂), τ̂)`, `m = (q − f)/\|q − f\| + (q − f₂)/\|q − f₂\|` (the ellipse's normal at `q`), `R₂ = span(t)` |
| `TangentSplines(C, t, D, s)` (`tangent` spline–spline) | with `w = D(s) − q`: `R₀ = n̂·w`, `R₁ = τ̂·w`, `R₂ = S·∠(τ̂_D, τ̂)`, `R₃ = span_D(s)`, `R₄ = span(t)` |
| `TangentLineSplineAtPoint(p, o, C, at)` (line ending at the spline's end, or held `on` it) | `R = ∠((o − p)/\|o − p\|, τ̂(t_at))` |
| `TangentArcSplineAtPoint(p, k, C, at)` (arc and spline sharing an end) | `R = ∠(rot90((p − k)/\|p − k\|), τ̂(t_at))` |
| `TangentSplinesAtPoint(C, at, D, at₂)` (splines sharing an end) | `R = ∠(τ̂_D(t_at₂), τ̂(t_at))` |

`at` is the start (`t₀`), the end (`t₁`) or a curve parameter another
constraint owns (`SplineAt`). Each general form has one equation more than
the parameters it owns and removes one degree of freedom; the span and
segment rows are 0 with zero rows inside, like an Arc's.

The rows are chosen to converge from poor starts (`tests/spline_test.rs`
`tangents_solve_from_poor_starts`: segments, ellipses and splines far from
the curve and at the wrong angle, in under 30 iterations). Earlier forms
took hundreds of iterations or stalled, for reasons worth keeping:

- *A gap measured where sliding the contact changes it to first order*
  (`ν·(q − a)` against the line's normal, or `C(t) − D(s)` for two
  splines): Gauss–Newton zeroes it by sliding the contact along the
  curve's tangent, which the curve doesn't follow, and the trust region
  collapses. Measured along the spline's own normal (`n̂·(f − q)`,
  `n̂·w`) or as a distance to a foot point (`|v| − r` with `v·τ̂ = 0`), it
  doesn't change as the contact slides.
- *The sine or cosine of an angle* (`g × τ̂`, `n̂·τ̂`) has a zero derivative
  at perpendicular, a false minimum. The angle itself doesn't.
- *A row scaled by the line's length* (`n̂·(b − a)`): the solve shrinks the
  line to a point on the curve instead.
- *An angle row against length rows unweighted*: Dog-Leg's steepest-descent
  step is lopsided; `S·∠` fixes it.

Through a point already on the curve, the general line form is degenerate
(as §20's distance forms are), so the catalog builds the at-point forms
when a line or arc *ends* at the spline's first or last handle or two
splines share an end handle, and `constraint_catalog::tangent_at_held_endpoints`
rewrites a line–spline `tangent` whose line endpoint a real `on` holds on
that spline to `TangentLineSplineAtPoint` at that `on`'s parameter.

Not supported: `distance` to a spline, tangency with an elliptical arc,
closed (periodic) splines, weights (NURBS). A contact that settles exactly
at the end of the curve has its span row active, so `dof` reports one
fewer there (an Arc's span row does the same at its ends).

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
    PointOnArc(String, String, String),             // point_id, arc_center_id, arc_id
    MidpointOfArc(String, String, String),          // point_id, arc_center_id, arc_id
    TangentLineArc(String, String, String, String), // line_pa, line_pb, arc_center_id, arc_id
    TangentAtPoint(String, String, String),         // shared_point_id, other_line_end_id, arc_center_id
    TangentCirclesInternal(String, String, String, String), // c1_center_id, c1_id, c2_center_id, c2_id
    TangentCircleArc(String, String, String, String, bool), // circle_center_id, circle_id, arc_center_id, arc_id, internal
    TangentArcs(String, String, String, String, bool),      // arc1_center_id, arc1_id, arc2_center_id, arc2_id, internal
    TangentArcsAtPoint(String, String, String, bool),       // shared_point_id, arc1_center_id, arc2_center_id, internal

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

    // Elliptical arcs (§24)
    PointOnEllipticalArc(String, String, String, String),                    // point_id, center_id, focus1_id, elliptical_arc_id
    TangentLineEllipticalArc(String, String, String, String, String),        // line_pa_id, line_pb_id, center_id, focus1_id, elliptical_arc_id
    TangentLineEllipticalArcAtPoint(String, String, String, String, String, ArcEnd), // point_id, other_id, center_id, focus1_id, elliptical_arc_id, Start|End
    TangentArcEllipticalArcAtPoint(String, String, String, String, String, ArcEnd),  // point_id, arc_center_id, center_id, focus1_id, elliptical_arc_id, Start|End

    // Collinear (native `normal` is PointOnExtension(center, line_pa, line_pb))
    Collinear(String, String, String, String),            // a_p1, a_p2, b_p1, b_p2
    // Dimensions and distances
    Diameter(String, f64),                                  // circle_or_arc_id, diameter
    ArcLength(String, f64),                                 // arc_id, length
    ArcSweep(String, f64),                                  // arc_id, sweep (0 < sweep < 2π)
    DistancePointCircle(String, String, String, f64, bool), // point_id, circle_center_id, circle_id, gap, internal
    DistanceLineCircle(String, String, String, String, f64), // line_pa, line_pb, circle_center_id, circle_id, gap
    DistanceExtensionCircle(String, String, String, String, f64), // line_pa, line_pb, circle_center_id, circle_id, gap
    DistanceCircleCircle(String, String, String, String, f64, bool), // c1_center_id, c1_id, c2_center_id, c2_id, gap, internal
    DistanceCircleArc(String, String, String, String, f64, bool), // circle_center_id, circle_id, arc_center_id, arc_id, gap, internal
    DistanceArcs(String, String, String, String, f64, bool), // arc1_center_id, arc1_id, arc2_center_id, arc2_id, gap, internal
    DistancePointArc(String, String, String, f64, bool), // point_id, arc_center_id, arc_id, gap, internal
    DistanceLineArc(String, String, String, String, f64), // line_pa, line_pb, arc_center_id, arc_id, gap
    DistanceExtensionArc(String, String, String, String, f64), // line_pa, line_pb, arc_center_id, arc_id, gap
    DistanceLineLine(String, String, String, String, f64),  // a_p1, a_p2, b_p1, b_p2, distance

    // Splines (§25): curve parameters are IDs the constraint owns; SplineAt is Start | End | Param(id)
    PointOnSpline(String, Spline, String),                  // point_id, spline, param_id
    TangentLineSpline(String, String, Spline, String),      // line_pa, line_pb, spline, param_id
    TangentCircleSpline(String, String, Spline, String),    // circle_center_id, circle_id, spline, param_id
    TangentArcSpline(String, String, Spline, String),       // arc_center_id, arc_id, spline, param_id
    TangentEllipseSpline(String, String, String, Spline, String), // center_id, focus1_id, ellipse_id, spline, param_id
    TangentSplines(Spline, String, Spline, String),         // spline1, param1_id, spline2, param2_id
    TangentLineSplineAtPoint(String, String, Spline, SplineAt), // point_id, other_line_end_id, spline, at
    TangentArcSplineAtPoint(String, String, Spline, SplineAt),  // point_id, arc_center_id, spline, at
    TangentSplinesAtPoint(Spline, SplineAt, Spline, SplineAt),  // spline1, at1, spline2, at2
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
and a copy of a free Guide is not fully constrained. Solves with soft goals
(drags) treat Guides the same way: a drag of a copy may move a free Guide.
(Holding Guides during drags made a copy's drag chase a Guide that other
constraints moved, and diverge.)
