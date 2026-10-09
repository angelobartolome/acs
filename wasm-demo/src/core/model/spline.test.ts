import { describe, expect, it } from "vitest";

import { sampleSplineCurve, splineOutline, splinePoint } from "./spline";
import type { PointEntity, SplineCurve, SplineEntity } from "./types";

/** A cubic Bézier as a clamped B-spline: one span. */
const BEZIER: SplineCurve = {
  degree: 3,
  knots: [0, 0, 0, 0, 1, 1, 1, 1],
  controlPoints: [
    [0, 0],
    [1, 2],
    [3, 2],
    [4, 0],
  ],
};

describe("splinePoint", () => {
  it("starts and ends at the end control points", () => {
    expect(splinePoint(BEZIER, 0)).toEqual({ x: 0, y: 0 });
    expect(splinePoint(BEZIER, 1)).toEqual({ x: 4, y: 0 });
  });

  it("matches the Bézier formula inside", () => {
    // (P0 + 3·P1 + 3·P2 + P3) / 8 at t = 1/2
    const p = splinePoint(BEZIER, 0.5);
    expect(p.x).toBeCloseTo(2, 12);
    expect(p.y).toBeCloseTo(1.5, 12);
  });

  it("evaluates across knot spans and clamps outside the domain", () => {
    // Two spans: interior knot 0.5; uniform clamped, five control points.
    const curve: SplineCurve = {
      degree: 3,
      knots: [0, 0, 0, 0, 0.5, 1, 1, 1, 1],
      controlPoints: [
        [0, 0],
        [1, 1],
        [2, 0],
        [3, 1],
        [4, 0],
      ],
    };
    // Symmetric control polygon: the middle of the curve is on x = 2.
    expect(splinePoint(curve, 0.5).x).toBeCloseTo(2, 12);
    expect(splinePoint(curve, -1)).toEqual(splinePoint(curve, 0));
    expect(splinePoint(curve, 2)).toEqual(splinePoint(curve, 1));
  });

  it("samples count + 1 points from end to end", () => {
    const pts = sampleSplineCurve(BEZIER, 8);
    expect(pts).toHaveLength(9);
    expect(pts[0]).toEqual({ x: 0, y: 0 });
    expect(pts[8]).toEqual({ x: 4, y: 0 });
  });
});

describe("splineOutline", () => {
  const points: Record<string, PointEntity> = {
    a: { kind: "point", id: "a", x: 0, y: 0, fixed: false },
    b: { kind: "point", id: "b", x: 2, y: 3, fixed: false },
  };
  const spline: SplineEntity = { kind: "spline", id: "s", points: ["a", "b"], interpolated: true };

  it("is the handle polygon before the first solve", () => {
    expect(splineOutline(spline, (id) => points[id])).toEqual([points.a, points.b]);
  });

  it("is the solved curve once there is one", () => {
    expect(splineOutline({ ...spline, curve: BEZIER }, (id) => points[id])).toHaveLength(97);
  });
});
