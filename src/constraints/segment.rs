//! Shared geometry for constraints that measure a point against a Line.
//!
//! A Line is a bounded segment between its endpoints `a` and `b`. Every helper
//! returns a value and its partial derivatives with respect to
//! `(px, py, ax, ay, bx, by)`, in that order.

#![allow(non_snake_case)]

/// Value and partials w.r.t. `(px, py, ax, ay, bx, by)`.
pub(crate) type Grad6 = (f64, [f64; 6]);

const DEGENERATE_L2: f64 = 1e-24;

/// Signed perpendicular distance from `p` to the infinite line through `a`
/// and `b`: C / L with C = dx·(py − ay) − dy·(px − ax). For a degenerate
/// segment (L ≈ 0) the unnormalized C is returned.
pub(crate) fn line_signed_distance(px: f64, py: f64, ax: f64, ay: f64, bx: f64, by: f64) -> Grad6 {
    let dx = bx - ax;
    let dy = by - ay;
    let l2 = dx * dx + dy * dy;
    let C = dx * (py - ay) - dy * (px - ax);
    let dC = [-dy, dx, by - py, px - bx, py - ay, ax - px];

    if l2 < DEGENERATE_L2 {
        return (C, dC);
    }

    // ∂(C / L) = ∂C / L − C·∂L / L²
    let L = l2.sqrt();
    let dL = [0.0, 0.0, -dx / L, -dy / L, dx / L, dy / L];
    let mut g = [0.0; 6];
    for k in 0..6 {
        g[k] = dC[k] / L - C * dL[k] / l2;
    }
    (C / L, g)
}

/// Distance from `p` to the closest point of segment `ab`, signed by which
/// side of the line `p` is on. Within the segment's span it equals
/// [`line_signed_distance`]. Past an endpoint it is the distance to that
/// endpoint. The two agree at the boundary.
pub(crate) fn segment_signed_distance(px: f64, py: f64, ax: f64, ay: f64, bx: f64, by: f64) -> Grad6 {
    let dx = bx - ax;
    let dy = by - ay;
    let l2 = dx * dx + dy * dy;
    let t = if l2 < DEGENERATE_L2 {
        0.0 // Degenerate segment: treat as the point `a`.
    } else {
        ((px - ax) * dx + (py - ay) * dy) / l2
    };

    if l2 >= DEGENERATE_L2 && (0.0..=1.0).contains(&t) {
        return line_signed_distance(px, py, ax, ay, bx, by);
    }

    let C = dx * (py - ay) - dy * (px - ax);
    let sign = if C >= 0.0 { 1.0 } else { -1.0 };

    // Past an endpoint: R = sign · |p − e|, with e = a (t < 0) or b (t > 1).
    let (ex, ey, e_is_a) = if t < 0.0 || l2 < DEGENERATE_L2 {
        (ax, ay, true)
    } else {
        (bx, by, false)
    };
    let (vx, vy) = (px - ex, py - ey);
    let dist = (vx * vx + vy * vy).sqrt();
    if dist < 1e-300 {
        return (0.0, [0.0; 6]);
    }
    let (gx, gy) = (sign * vx / dist, sign * vy / dist);
    let g = if e_is_a {
        [gx, gy, -gx, -gy, 0.0, 0.0]
    } else {
        [gx, gy, 0.0, 0.0, -gx, -gy]
    };
    (sign * dist, g)
}

/// Unsigned distance from `p` to segment `ab`: |segment_signed_distance|.
pub(crate) fn segment_distance(px: f64, py: f64, ax: f64, ay: f64, bx: f64, by: f64) -> Grad6 {
    let (r, g) = segment_signed_distance(px, py, ax, ay, bx, by);
    let s = if r >= 0.0 { 1.0 } else { -1.0 };
    (s * r, g.map(|v| s * v))
}

/// How far the foot of the perpendicular from `p` falls outside segment `ab`,
/// measured along the line: negative before `a`, positive past `b`, and 0
/// while the foot lies on the segment.
pub(crate) fn foot_overshoot(px: f64, py: f64, ax: f64, ay: f64, bx: f64, by: f64) -> Grad6 {
    let dx = bx - ax;
    let dy = by - ay;
    let l2 = dx * dx + dy * dy;
    if l2 < DEGENERATE_L2 {
        return (0.0, [0.0; 6]);
    }
    let L = l2.sqrt();

    // s = N / L is the signed projection length of p − a onto the line.
    let N = (px - ax) * dx + (py - ay) * dy;
    let dN = [dx, dy, -dx - (px - ax), -dy - (py - ay), px - ax, py - ay];
    let dL = [0.0, 0.0, -dx / L, -dy / L, dx / L, dy / L];
    let s = N / L;
    let mut ds = [0.0; 6];
    for k in 0..6 {
        ds[k] = dN[k] / L - N * dL[k] / l2;
    }

    if s < 0.0 {
        (s, ds)
    } else if s > L {
        let mut g = [0.0; 6];
        for k in 0..6 {
            g[k] = ds[k] - dL[k];
        }
        (s - L, g)
    } else {
        (0.0, [0.0; 6])
    }
}
