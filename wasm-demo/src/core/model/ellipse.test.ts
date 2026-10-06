import { describe, expect, it } from "vitest";

import { ellipseFrame, ellipseThrough, paramAngle, pointAt } from "./ellipse";

describe("ellipse geometry", () => {
  it("puts a point at its parametric angle and reads the angle back", () => {
    const e = ellipseFrame({ x: 1, y: 2 }, { x: 4, y: 2 }, 4);
    expect(e.a).toBeCloseTo(5);
    const p = pointAt(e, 0.7);
    expect(p.x).toBeCloseTo(1 + 5 * Math.cos(0.7));
    expect(p.y).toBeCloseTo(2 + 4 * Math.sin(0.7));
    expect(paramAngle(e, p)).toBeCloseTo(0.7);
  });

  it("the drawing tool's ellipse passes through the clicked start point", () => {
    // Center, the end of a horizontal axis at 6, a point it must pass.
    const center = { x: 0, y: 0 };
    const through = { x: 3, y: 2 };
    const shape = ellipseThrough(center, { x: 6, y: 0 }, through);
    expect(shape).not.toBeNull();
    if (shape === null) return;
    const e = ellipseFrame(center, shape.focus, shape.radmin);
    expect(e.a).toBeCloseTo(6);
    const p = pointAt(e, paramAngle(e, through));
    expect(p.x).toBeCloseTo(3);
    expect(p.y).toBeCloseTo(2);
  });

  it("swaps the axes when the clicked one is the minor axis", () => {
    // Axis end at 2 along X, through (1, 5): taller than wide, so the
    // major axis runs along Y and the focus is on it.
    const center = { x: 0, y: 0 };
    const through = { x: 1, y: 5 };
    const shape = ellipseThrough(center, { x: 2, y: 0 }, through);
    expect(shape).not.toBeNull();
    if (shape === null) return;
    expect(shape.radmin).toBeCloseTo(2);
    expect(Math.abs(shape.focus.x)).toBeLessThan(1e-9);
    const e = ellipseFrame(center, shape.focus, shape.radmin);
    const p = pointAt(e, paramAngle(e, through));
    expect(p.x).toBeCloseTo(1);
    expect(p.y).toBeCloseTo(5);
  });

  it("makes no ellipse from a degenerate axis or a point on the axis", () => {
    expect(ellipseThrough({ x: 0, y: 0 }, { x: 0, y: 0 }, { x: 1, y: 1 })).toBeNull();
    expect(ellipseThrough({ x: 0, y: 0 }, { x: 5, y: 0 }, { x: 2, y: 0 })).toBeNull();
  });
});
