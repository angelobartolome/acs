/**
 * Ellipse geometry, as the solver defines it: a center `c`, a focus `f` and
 * the minor radius `b`. The major radius is `a = √(b² + |f − c|²)`, the
 * major direction `u` points from `c` to `f` (+X when they coincide) and the
 * minor direction is `n = rot90(u)`. A parametric angle `t` names the point
 * `c + a·cos t·u + b·sin t·n`.
 */

export interface XY {
  x: number;
  y: number;
}

export interface EllipseFrame {
  center: XY;
  /** major radius */
  a: number;
  /** minor radius */
  b: number;
  /** unit major direction */
  u: XY;
  /** unit minor direction, rot90(u) */
  n: XY;
}

export function ellipseFrame(center: XY, focus: XY, radmin: number): EllipseFrame {
  const dx = focus.x - center.x;
  const dy = focus.y - center.y;
  const k = Math.hypot(dx, dy);
  const u = k < 1e-12 ? { x: 1, y: 0 } : { x: dx / k, y: dy / k };
  return {
    center,
    a: Math.sqrt(radmin * radmin + k * k),
    b: radmin,
    u,
    n: { x: -u.y, y: u.x },
  };
}

/** The point at parametric angle `t`. */
export function pointAt(e: EllipseFrame, t: number): XY {
  const ca = e.a * Math.cos(t);
  const sb = e.b * Math.sin(t);
  return {
    x: e.center.x + ca * e.u.x + sb * e.n.x,
    y: e.center.y + ca * e.u.y + sb * e.n.y,
  };
}

/** The parametric angle of `p`'s direction from the center. */
export function paramAngle(e: EllipseFrame, p: XY): number {
  const dx = p.x - e.center.x;
  const dy = p.y - e.center.y;
  const along = dx * e.u.x + dy * e.u.y;
  const across = dx * e.n.x + dy * e.n.y;
  return Math.atan2(e.a * across, e.b * along);
}

/** `count + 1` points along the arc from `start` to `end` (counter-clockwise). */
export function sampleArc(e: EllipseFrame, start: number, end: number, count = 64): XY[] {
  let span = end - start;
  while (span <= 0) span += Math.PI * 2;
  return Array.from({ length: count + 1 }, (_, i) => pointAt(e, start + (span * i) / count));
}

/**
 * The ellipse a drawing tool means by three clicks: its `center`, the end of
 * one axis `axisEnd`, and a point `through` it passes. Returns the focus and
 * minor radius the solver takes (the axes swapped when the clicked axis
 * turns out to be the minor one), or null when the clicks don't make one.
 */
export function ellipseThrough(
  center: XY,
  axisEnd: XY,
  through: XY,
): { focus: XY; radmin: number } | null {
  const r = Math.hypot(axisEnd.x - center.x, axisEnd.y - center.y);
  if (r < 1e-9) return null;
  const u = { x: (axisEnd.x - center.x) / r, y: (axisEnd.y - center.y) / r };
  const n = { x: -u.y, y: u.x };
  const dx = through.x - center.x;
  const dy = through.y - center.y;
  // `through` in the clicked axis' frame; on the ellipse when
  // (x/r)² + (y/s)² = 1.
  const x = Math.min(Math.abs(dx * u.x + dy * u.y) / r, 0.999);
  const y = Math.abs(dx * n.x + dy * n.y);
  let s = y / Math.sqrt(1 - x * x);
  if (s < 1e-9) return null;
  // A circle has no major axis; keep the clicked one, barely.
  if (Math.abs(s - r) < 1e-6 * r) s = r * 0.999;
  // Major radius a along `major`, minor b.
  const [a, b, major] = r >= s ? [r, s, u] : [s, r, n];
  const k = Math.sqrt(a * a - b * b);
  return { focus: { x: center.x + k * major.x, y: center.y + k * major.y }, radmin: b };
}
