import { describe, expect, it } from "vitest";

import type { Sketch } from "../model/types";
import {
  AcsSolverService,
  applySolvedPrimitives,
  buildSolveRequest,
  parseSolveResponse,
  primitivesToSketch,
  sketchToPrimitives,
} from "./SolverService";

const SKETCH: Sketch = {
  entities: [
    { kind: "point", id: "p1", x: 0, y: 0, fixed: true },
    { kind: "point", id: "p2", x: 3, y: 4, fixed: false },
    { kind: "line", id: "l1", p1: "p1", p2: "p2" },
    { kind: "circle", id: "c1", center: "p1", radius: 10, fixed: false },
    {
      kind: "arc",
      id: "a1",
      center: "p2",
      radius: 5,
      startAngle: 0,
      endAngle: Math.PI / 2,
      fixed: false,
    },
  ],
  constraints: [
    { id: "k1", type: "p2p_distance", entities: ["p1", "p2"], params: { distance: 5 } },
  ],
};

describe("payload building", () => {
  it("serializes entities with exact API field names", () => {
    const prims = sketchToPrimitives(SKETCH);
    expect(prims).toEqual([
      { id: "p1", type: "point", x: 0, y: 0, fixed: true },
      { id: "p2", type: "point", x: 3, y: 4, fixed: false },
      { id: "l1", type: "line", p1_id: "p1", p2_id: "p2" },
      { id: "c1", type: "circle", c_id: "p1", radius: 10, fixed: false },
      {
        id: "a1",
        type: "arc",
        c_id: "p2",
        radius: 5,
        start_angle: 0,
        end_angle: Math.PI / 2,
        fixed: false,
      },
      { id: "k1", type: "p2p_distance", p1_id: "p1", p2_id: "p2", distance: 5 },
    ]);
  });

  it("wraps primitives and forwards max_iterations", () => {
    const parsed = JSON.parse(buildSolveRequest(SKETCH, 42)) as {
      primitives: unknown[];
      max_iterations: number;
    };
    expect(Array.isArray(parsed.primitives)).toBe(true);
    expect(parsed.max_iterations).toBe(42);
    const noMax = JSON.parse(buildSolveRequest(SKETCH)) as Record<string, unknown>;
    expect("max_iterations" in noMax).toBe(false);
  });
});

describe("response parsing", () => {
  it("applies solved values and keeps fixed flags on success", () => {
    const response = JSON.stringify({
      ok: true,
      status: "converged",
      solveStatus: 1,
      primitives: [
        { id: "p2", type: "point", x: 3.5, y: 3.5714 },
        { id: "c1", type: "circle", c_id: "p1", radius: 11 },
        { id: "a1", type: "arc", c_id: "p2", radius: 6, start_angle: 0.1, end_angle: 1.2 },
      ],
      skipped_constraint_ids: ["k7:unknown_type"],
      conflicting_constraint_ids: [],
      fully_constrained_ids: ["p1", "p2"],
      error: null,
      stats: { iterations: 3, initial_error: 1.5, final_error: 1e-12 },
    });
    const outcome = parseSolveResponse(response, SKETCH.entities, "{}", 1.25);
    expect(outcome.ok).toBe(true);
    expect(outcome.status).toBe("converged");
    expect(outcome.fullyConstrainedIds).toEqual(["p1", "p2"]);
    expect(outcome.stats).toEqual({
      iterations: 3,
      initialError: 1.5,
      finalError: 1e-12,
    });
    expect(outcome.skippedConstraintIds).toEqual(["k7:unknown_type"]);

    const p2 = outcome.entities.find((e) => e.id === "p2");
    expect(p2).toMatchObject({ kind: "point", x: 3.5, y: 3.5714, fixed: false });
    const p1 = outcome.entities.find((e) => e.id === "p1");
    expect(p1).toMatchObject({ fixed: true, x: 0, y: 0 });
    const c1 = outcome.entities.find((e) => e.id === "c1");
    expect(c1).toMatchObject({ radius: 11 });
    const a1 = outcome.entities.find((e) => e.id === "a1");
    expect(a1).toMatchObject({ radius: 6, startAngle: 0.1, endAngle: 1.2 });
  });

  it("never applies primitives from failed solves", () => {
    const response = JSON.stringify({
      ok: false,
      status: "failed",
      solveStatus: 2,
      primitives: [{ id: "p2", type: "point", x: 999, y: 999 }],
      skipped_constraint_ids: [],
      conflicting_constraint_ids: [],
      fully_constrained_ids: ["p1", "p2"],
      error: "Solver did not converge",
      stats: { iterations: 100, initial_error: 5, final_error: 2 },
    });
    const outcome = parseSolveResponse(response, SKETCH.entities, "{}", 0.5);
    expect(outcome.ok).toBe(false);
    expect(outcome.error).toBe("Solver did not converge");
    // DOF is only meaningful on a converged solve; must be cleared on failure.
    expect(outcome.fullyConstrainedIds).toEqual([]);
    const p2 = outcome.entities.find((e) => e.id === "p2");
    expect(p2).toMatchObject({ x: 3, y: 4 });
  });

  it("survives malformed responses", () => {
    const outcome = parseSolveResponse("not json", SKETCH.entities, "{}", 0.1);
    expect(outcome.ok).toBe(false);
    expect(outcome.error).toMatch(/Invalid solver response/);
    expect(outcome.entities.length).toBe(SKETCH.entities.length);
  });
});

describe("applySolvedPrimitives", () => {
  it("ignores primitives with unknown ids", () => {
    const out = applySolvedPrimitives(SKETCH.entities, [
      { id: "zz", type: "point", x: 1, y: 1 },
    ]);
    expect(out).toEqual(SKETCH.entities);
  });
});

describe("primitivesToSketch (import)", () => {
  it("round-trips a full sketch", () => {
    const prims = sketchToPrimitives(SKETCH);
    const { sketch, warnings } = primitivesToSketch(prims);
    expect(warnings).toEqual([]);
    expect(sketch.entities).toEqual(SKETCH.entities);
    expect(sketch.constraints).toEqual(SKETCH.constraints);
  });

  it("collects warnings for malformed primitives", () => {
    const { sketch, warnings } = primitivesToSketch([
      { type: "point" }, // no id
      { id: "l1", type: "line", p1_id: "a" }, // missing p2_id
      { id: "x", type: "wat" }, // unknown type
      42, // not an object
    ]);
    expect(sketch.entities).toEqual([]);
    expect(warnings.length).toBe(4);
  });
});

describe("AcsSolverService", () => {
  it("passes the request to the injected solve fn and parses the result", () => {
    let seen = "";
    const svc = new AcsSolverService((input) => {
      seen = input;
      return JSON.stringify({
        ok: true,
        status: "converged",
        solveStatus: 1,
        primitives: [{ id: "p2", type: "point", x: 5, y: 0 }],
        skipped_constraint_ids: [],
        conflicting_constraint_ids: [],
        error: null,
        stats: { iterations: 2, initial_error: 1, final_error: 0 },
      });
    });
    const outcome = svc.solve(SKETCH, 10);
    expect(JSON.parse(seen)).toMatchObject({ max_iterations: 10 });
    expect(outcome.ok).toBe(true);
    expect(outcome.durationMs).toBeGreaterThanOrEqual(0);
    expect(outcome.entities.find((e) => e.id === "p2")).toMatchObject({
      x: 5,
      y: 0,
    });
  });

  it("reports a failed outcome when the solve fn throws", () => {
    const svc = new AcsSolverService(() => {
      throw new Error("wasm not ready");
    });
    const outcome = svc.solve(SKETCH);
    expect(outcome.ok).toBe(false);
    expect(outcome.error).toMatch(/wasm not ready/);
  });
});
