/**
 * Integration smoke test: run every gallery example through the real wasm
 * solver (loaded synchronously from the `acs` package) via the same
 * request-building pipeline the app uses.
 */

import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { readFileSync } from "node:fs";

import { describe, expect, it } from "vitest";

import type { SketchEntity } from "../core/model/types";
import { AcsSolverService } from "../core/solver/SolverService";
import { EXAMPLES } from "./index";

const require = createRequire(import.meta.url);
const pkgDir = dirname(require.resolve("acs/package.json"));
const acs = (await import("acs")) as typeof import("acs");
acs.initSync({ module: readFileSync(join(pkgDir, "acs_bg.wasm")) });

const service = new AcsSolverService(acs.acsSolveSketch);

describe("gallery examples solve end-to-end", () => {
  for (const ex of EXAMPLES) {
    it(`${ex.label} converges`, () => {
      const outcome = service.solve(structuredClone(ex.sketch));
      expect(outcome.error).toBeNull();
      expect(outcome.ok).toBe(true);
      expect(outcome.status).toBe("converged");
      expect(outcome.dof).not.toBeNull();
      expect(outcome.stats).not.toBeNull();
      if (outcome.stats !== null) {
        expect(outcome.stats.finalError).toBeLessThan(1e-6);
      }
    });
  }
});

// ---------------------------------------------------------------------------
// Lines are segments: check each segment example lands where it should.
// ---------------------------------------------------------------------------

type XY = { x: number; y: number };

function solveExample(id: string): Map<string, SketchEntity> {
  const ex = EXAMPLES.find((e) => e.id === id);
  if (ex === undefined) throw new Error(`no example ${id}`);
  const outcome = service.solve(structuredClone(ex.sketch));
  expect(outcome.ok).toBe(true);
  return new Map(outcome.entities.map((e) => [e.id, e]));
}

function xy(ents: Map<string, SketchEntity>, id: string): XY {
  const e = ents.get(id);
  if (e === undefined || e.kind !== "point") throw new Error(`${id} not a point`);
  return { x: e.x, y: e.y };
}

/** Distance from `p` to segment `ab` and the clamped closest-point parameter. */
function toSegment(p: XY, a: XY, b: XY): { dist: number; t: number; tRaw: number } {
  const dx = b.x - a.x;
  const dy = b.y - a.y;
  const tRaw = ((p.x - a.x) * dx + (p.y - a.y) * dy) / (dx * dx + dy * dy);
  const t = Math.min(1, Math.max(0, tRaw));
  const dist = Math.hypot(p.x - (a.x + t * dx), p.y - (a.y + t * dy));
  return { dist, t, tRaw };
}

describe("lines are segments", () => {
  it("point on segment lands between the endpoints", () => {
    const e = solveExample("segment-point");
    const { dist, tRaw } = toSegment(xy(e, "p3"), xy(e, "p1"), xy(e, "p2"));
    expect(dist).toBeLessThan(1e-6);
    expect(tRaw).toBeGreaterThanOrEqual(-1e-9);
    expect(tRaw).toBeLessThanOrEqual(1 + 1e-9);
  });

  it("distance past the segment end is measured to the endpoint", () => {
    const e = solveExample("segment-distance");
    const p3 = xy(e, "p3");
    const { dist, tRaw } = toSegment(p3, xy(e, "p1"), xy(e, "p2"));
    expect(dist).toBeCloseTo(25, 6);
    // Still past the end, so it sits on a circle around the endpoint p2,
    // not on a line parallel to the segment.
    expect(tRaw).toBeGreaterThan(1);
    expect(Math.hypot(p3.x, p3.y)).toBeCloseTo(25, 6);
  });

  it("tangency point lies within the segment", () => {
    const e = solveExample("segment-tangent");
    const c = xy(e, "p3");
    expect(Math.abs(c.y)).toBeCloseTo(15, 6);
    expect(c.x).toBeGreaterThanOrEqual(-50 - 1e-9);
    expect(c.x).toBeLessThanOrEqual(10 + 1e-9);
  });

  it("midpoint lands on the segment", () => {
    const e = solveExample("segment-midpoint");
    const a = xy(e, "p3");
    const b = xy(e, "p4");
    const m = { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 };
    const { dist, tRaw } = toSegment(m, xy(e, "p1"), xy(e, "p2"));
    expect(dist).toBeLessThan(1e-6);
    expect(tRaw).toBeGreaterThanOrEqual(-1e-9);
    expect(tRaw).toBeLessThanOrEqual(1 + 1e-9);
  });
});

describe("unsolvable sketches report failure", () => {
  it("a horizontal line cannot touch two different-radius circles centered on y = 0", () => {
    const ex = EXAMPLES.find((e) => e.id === "two-tangent");
    if (ex === undefined) throw new Error("no two-tangent example");
    const sketch = structuredClone(ex.sketch);
    sketch.entities = sketch.entities.map((e) =>
      e.id === "p2" && e.kind === "point" ? { ...e, y: 0 } : e,
    );
    sketch.constraints.push({
      id: "kh",
      def: "horizontal_line",
      entities: ["l1"],
      params: {},
    });
    const outcome = service.solve(sketch);
    expect(outcome.ok).toBe(false);
    expect(outcome.fullyConstrainedIds).toEqual([]);
  });
});

describe("line tangent to two circles follows the circles", () => {
  for (const [label, p1, p2] of [
    ["c2 moved up", [-40, 0], [45, 13]],
    ["c1 moved down", [-40, -3], [45, 8]],
    ["c2 moved far up", [-40, 0], [40, 60]],
  ] as const) {
    it(`converges with a clean diagnosis when ${label}`, () => {
      const ex = EXAMPLES.find((e) => e.id === "two-tangent");
      if (ex === undefined) throw new Error("no two-tangent example");
      const sketch = structuredClone(ex.sketch);
      sketch.entities = sketch.entities.map((e) =>
        e.kind === "point" && (e.id === "p1" || e.id === "p2")
          ? { ...e, x: (e.id === "p1" ? p1 : p2)[0], y: (e.id === "p1" ? p1 : p2)[1] }
          : e,
      );
      const outcome = service.solve(sketch);
      expect(outcome.status).toBe("converged");
      expect(outcome.conflictingIds).toEqual([]);
      expect(outcome.redundantIds).toEqual([]);
    });
  }
});

describe("arcs keep their endpoint points", () => {
  it("slot lines stay on the arcs' endpoints, tangent at radius 15", () => {
    const e = solveExample("slot");
    for (const [arcId, centerId] of [
      ["a1", "c1"],
      ["a2", "c2"],
    ]) {
      const a = e.get(arcId);
      if (a === undefined || a.kind !== "arc") throw new Error(`${arcId} not an arc`);
      expect(a.radius).toBeCloseTo(15, 6);
      const c = xy(e, centerId);
      for (const [pointId, angle] of [
        [a.start, a.startAngle],
        [a.end, a.endAngle],
      ] as const) {
        const p = xy(e, pointId);
        expect(p.x).toBeCloseTo(c.x + a.radius * Math.cos(angle), 6);
        expect(p.y).toBeCloseTo(c.y + a.radius * Math.sin(angle), 6);
      }
    }
    expect(Math.abs(xy(e, "t1").y)).toBeCloseTo(15, 6);
    expect(xy(e, "t1").y).toBeCloseTo(xy(e, "t2").y, 6);
  });
});

describe("rejected requests", () => {
  it("names the constraint that references a missing entity", () => {
    const ex = EXAMPLES.find((e) => e.id === "square");
    if (ex === undefined) throw new Error("no square example");
    const sketch = structuredClone(ex.sketch);
    sketch.constraints.push({
      id: "kBad",
      def: "distance_points",
      entities: ["p1", "ghost"],
      params: { value: 1 },
    });
    const outcome = service.solve(sketch);
    expect(outcome.status).toBe("invalid");
    expect(outcome.rejectedConstraintId).toBe("kBad");
    expect(outcome.error).toContain("b: 'ghost' (not an entity)");
    expect(outcome.entities).toEqual(sketch.entities);
  });
});

describe("curved slot", () => {
  it("joins outer and inner arcs with round caps of the slot's half-width", () => {
    const ex = EXAMPLES.find((e) => e.id === "curved-slot");
    if (ex === undefined) throw new Error("no curved-slot example");
    const outcome = service.solve(structuredClone(ex.sketch));
    expect(outcome.status).toBe("converged");
    expect(outcome.conflictingIds).toEqual([]);
    expect(outcome.redundantIds).toEqual([]);
    // Only the two ends' sweep angles are left free.
    expect(outcome.dof).toBe(2);
    const e = new Map(outcome.entities.map((x) => [x.id, x]));
    const dist = (p: string, q: string) => {
      const [u, v] = [xy(e, p), xy(e, q)];
      return Math.hypot(u.x - v.x, u.y - v.y);
    };
    for (const [end, outer, inner] of [
      ["s", "oa", "ia"],
      ["e", "ob", "ib"],
    ]) {
      expect(dist(end, outer)).toBeCloseTo(8, 6);
      expect(dist(end, inner)).toBeCloseTo(8, 6);
      expect(dist("c", outer)).toBeCloseTo(48, 6);
      expect(dist("c", inner)).toBeCloseTo(32, 6);
    }
  });
});

describe("polygon built like a polygon tool", () => {
  const ex = EXAMPLES.find((e) => e.id === "polygon-rotations");
  if (ex === undefined) throw new Error("no polygon-rotations example");

  it("solves, with the rotations the others imply reported Redundant", () => {
    const outcome = service.solve(structuredClone(ex.sketch));
    expect(outcome.status).toBe("converged");
    expect(outcome.conflictingIds).toEqual([]);
    // 8 rotations pin 3 vertices (6 equations): the other 4 repeat them.
    expect(outcome.redundantIds).toHaveLength(4);
    // The center (2) and the polygon's turn about it (1).
    expect(outcome.dof).toBe(3);
  });

  // The rotations' center is free: turning the polygon moves it (through the tangency), and
  // the rotations move it with them (Guides are held only during drags).
  it("adding horizontal to the bottom side converges", () => {
    const sketch = structuredClone(ex.sketch);
    // l3 runs between the two lowest vertices (from 0.3 + 3·72° to 0.3 + 4·72°).
    sketch.constraints.push({ id: "kh", def: "horizontal_line", entities: ["l3"], params: {} });
    const outcome = service.solve(sketch);
    expect(outcome.status).toBe("converged");
  });
});
