/**
 * Drawing a spline: points along its solved B-spline (`SplineCurve`, from the
 * solver's response) by de Boor's algorithm. The demo doesn't build the
 * curve itself; until the first solve it draws the handle polygon.
 */

import type { EntityId, PointEntity, SplineCurve, SplineEntity } from "./types";

type XY = { x: number; y: number };

/** The point of `curve` at parameter `t` (clamped to its domain). */
export function splinePoint(curve: SplineCurve, t: number): XY {
  const { degree: p, knots: u, controlPoints: c } = curve;
  const n = c.length;
  const tt = Math.min(Math.max(t, u[0]), u[u.length - 1]);
  // the knot span: u[k] <= t < u[k + 1], the last one at the end
  let k = p;
  while (k < n - 1 && tt >= u[k + 1]) k += 1;
  const d = Array.from({ length: p + 1 }, (_, j) => [...c[j + k - p]]);
  for (let r = 1; r <= p; r++) {
    for (let j = p; j >= r; j--) {
      const i = j + k - p;
      const den = u[i + p - r + 1] - u[i];
      const a = den === 0 ? 0 : (tt - u[i]) / den;
      d[j] = [(1 - a) * d[j - 1][0] + a * d[j][0], (1 - a) * d[j - 1][1] + a * d[j][1]];
    }
  }
  return { x: d[p][0], y: d[p][1] };
}

/** `count + 1` points evenly spaced in parameter along `curve`. */
export function sampleSplineCurve(curve: SplineCurve, count = 96): XY[] {
  const lo = curve.knots[0];
  const hi = curve.knots[curve.knots.length - 1];
  return Array.from({ length: count + 1 }, (_, i) => splinePoint(curve, lo + ((hi - lo) * i) / count));
}

/** What to draw for spline `e`: its solved curve, else its handle polygon. */
export function splineOutline(
  e: SplineEntity,
  point: (id: EntityId) => PointEntity | undefined,
): XY[] {
  if (e.curve !== undefined) return sampleSplineCurve(e.curve);
  return e.points.map(point).filter((p): p is PointEntity => p !== undefined);
}
