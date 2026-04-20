# ACS Constraint Solver — Constraint Specification

This document specifies every constraint type in ACS: the geometric meaning, the
parametric entities involved, the residual equations fed to the Dog-Leg solver, and
the analytical Jacobian used during each iteration.

## Notation

| Symbol | Meaning |
|--------|---------|
| `p = (x, y)` | A free point tracked by the parameter manager |
| `c` | A circle tracked as `(center_point_id, radius)` |
| `dᵢ = pᵢ₂ − pᵢ₁` | Direction vector of line *i* (not unit-length) |
| `R` | Residual vector (must be **0** at solution) |
| `J` | Jacobian `∂R/∂params` (analytical) |
| `θ` | Angle in radians |
| `d` | Scalar distance |

Residuals are written so that **R = 0** encodes the constraint.  All Jacobian
entries not listed are **0**.

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
| `PointOnLine(p, a, b)` | 3 points | squared distance from `p` to line `ab` = 0 |
| `EqualRadius(c1, c2)` | 2 circles | `r1 − r2 = 0` |

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

**Geometric meaning:** the line is tangent to the circle, i.e. the perpendicular
distance from the center to the line equals `r`.

Let `dx = bx − ax`, `dy = by − ay` (line direction),
`L² = dx² + dy²` (squared line length).

The signed area form gives:

```
area = dy·(cx − ax) − dx·(cy − ay)
```

**Residual (1 equation):**

```
R = area² − r²·L² = 0
```

**Jacobian** (let `A = area`):

| Parameter | ∂R/∂param |
|-----------|-----------|
| `ax` | `2A·(−dy) − r²·(−2dx)` = `−2A·dy + 2r²·dx` |
| `ay` | `2A·dx − r²·(−2dy)` = `2A·dx + 2r²·dy` |
| `bx` | `2A·dy − r²·2dx` = `2A·dy − 2r²·dx` |
| `by` | `2A·(−dx) − r²·2dy` = `−2A·dx − 2r²·dy` |
| `cx` | `2A·dy` |
| `cy` | `−2A·dx` |
| `r`  | `−2r·L²` |

Derivation notes:
- `∂area/∂ax = −dy`, `∂area/∂ay = dx`, `∂area/∂bx = dy`, `∂area/∂by = −dx`,
  `∂area/∂cx = dy`, `∂area/∂cy = −dx`
- `∂L²/∂ax = −2dx`, `∂L²/∂ay = −2dy`, `∂L²/∂bx = 2dx`, `∂L²/∂by = 2dy`

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

**Geometric meaning:** the perpendicular distance from `p` to the infinite line
through `pa`–`pb` equals `d`.

Let `dx = bx − ax`, `dy = by − ay`, `L² = dx² + dy²`,
`area = dy·(px − ax) − dx·(py − ay)`.

**Residual (1 equation):**

```
R = area² − d²·L² = 0
```

**Jacobian** (let `A = area`):

| Parameter | ∂R/∂param |
|-----------|-----------|
| `px` | `2A·dy` |
| `py` | `−2A·dx` |
| `ax` | `2A·(−dy) − d²·(−2dx)` = `−2A·dy + 2d²·dx` |
| `ay` | `2A·dx − d²·(−2dy)` = `2A·dx + 2d²·dy` |
| `bx` | `2A·dy − d²·2dx` = `2A·dy − 2d²·dx` |
| `by` | `−2A·dx − d²·2dy` = `−2A·dx − 2d²·dy` |

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

### 12  Symmetric

**Entities:** two points `p` and `q` to mirror; two points `(la, lb)` defining
the axis of symmetry.

**Geometric meaning:** `p` and `q` are mirror images of each other across
line `la→lb`.

This is encoded as two equations:

1. The midpoint `M = ((px+qx)/2, (py+qy)/2)` lies on the axis line.
2. The vector `p→q` is perpendicular to the axis.

Let `dx = bx − ax`, `dy = by − ay` (axis direction),
`mx = (px+qx)/2 − ax`, `my = (py+qy)/2 − ay` (midpoint shifted to axis origin).

**Residual (2 equations):**

```
R₀ = my·dx − mx·dy  = 0   (collinearity of midpoint with axis)
R₁ = (qx−px)·dx + (qy−py)·dy = 0   (perpendicularity of pq to axis)
```

**Jacobian for R₀** (let `epx = (px+qx)/2 − ax`, `epy = (py+qy)/2 − ay`):

| Parameter | ∂R₀/∂param |
|-----------|------------|
| `px` | `−½·dy` |
| `py` | `½·dx` |
| `qx` | `−½·dy` |
| `qy` | `½·dx` |
| `ax` | `epy − dy` (= `−epx_shift·0 + ...`) — full form: `dy − epy` ... see derivation |
| `ay` | `dx − epx`... |
| `bx` | `epy` |
| `by` | `−epx` |

Full derivation of `R₀ = epy·dx − epx·dy`:

```
∂R₀/∂ax = (∂epy/∂ax)·dx + epy·(∂dx/∂ax) − (∂epx/∂ax)·dy − epx·(∂dy/∂ax)
         = 0·dx + epy·(−1) − (−1)·dy − epx·0
         = −epy + dy

∂R₀/∂ay = (−1)·dx + 0 − 0 − epx·(−1) ... wait
         = (∂epy/∂ay)·dx − (∂epx/∂ay)·dy
         = (−1)·dx − 0·dy = −dx + epx... let me redo
```

Correct derivation (product rule, `epx = (px+qx)/2 − ax`, `epy = (py+qy)/2 − ay`,
`dx = bx − ax`, `dy = by − ay`):

```
∂epx/∂ax = −1,  ∂epy/∂ax = 0,  ∂dx/∂ax = −1,  ∂dy/∂ax = 0
∂R₀/∂ax  = epy·(−1) − (−1)·dy = −epy + dy

∂epx/∂ay = 0,   ∂epy/∂ay = −1, ∂dx/∂ay = 0,   ∂dy/∂ay = −1
∂R₀/∂ay  = (−1)·dx − epx·(−1) = −dx + epx

∂epx/∂bx = 0,   ∂epy/∂bx = 0,  ∂dx/∂bx = 1,   ∂dy/∂bx = 0
∂R₀/∂bx  = epy·1 − 0 = epy

∂epx/∂by = 0,   ∂epy/∂by = 0,  ∂dx/∂by = 0,   ∂dy/∂by = 1
∂R₀/∂by  = 0 − epx·1 = −epx

∂R₀/∂px  = (½)·dx − 0 = ½·dx... wait
```

Hmm — `R₀ = epy·dx − epx·dy`. Let me re-examine:

```
∂R₀/∂px = (∂epy/∂px)·dx − (∂epx/∂px)·dy = 0·dx − ½·dy = −½·dy
∂R₀/∂py = (∂epy/∂py)·dx = ½·dx
∂R₀/∂qx = −½·dy
∂R₀/∂qy =  ½·dx
```

So the full Jacobian table for R₀:

| Parameter | ∂R₀/∂param |
|-----------|------------|
| `px` | `−½·dy` |
| `py` | `½·dx` |
| `qx` | `−½·dy` |
| `qy` | `½·dx` |
| `ax` | `dy − epy` |
| `ay` | `epx − dx` |
| `bx` | `epy` |
| `by` | `−epx` |

**Jacobian for R₁** (`R₁ = (qx−px)·dx + (qy−py)·dy`):

Let `vx = qx − px`, `vy = qy − py`.

| Parameter | ∂R₁/∂param |
|-----------|------------|
| `px` | `−dx` |
| `py` | `−dy` |
| `qx` | `dx` |
| `qy` | `dy` |
| `ax` | `−vx` |
| `ay` | `−vy` |
| `bx` | `vx` |
| `by` | `vy` |

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
    Symmetric(String, String, String, String),         // p_id, q_id, axis_pa_id, axis_pb_id
    Perpendicular(String, String, String, String),     // l1p1, l1p2, l2p1, l2p2
}
```

---

## Solver Notes

All residuals are designed so the solver's tolerance check `‖R‖ < 1e-6` is
meaningful in physical units (coordinates / radii in whatever unit the caller uses).

The squared-distance form used by `DistancePointPoint`, `EqualLength`, and
`PointOnCircle` keeps residuals and Jacobians smooth everywhere (no
discontinuities at zero), but note the residual magnitude is in units².  For
well-conditioned problems this is fine; if large coordinate values are involved,
consider normalising geometry before solving.

The `Angle` residual mixes units (`dot · sin θ` has units of length²), so for
problems with very different length scales, a scaling pre-pass may help
convergence.
