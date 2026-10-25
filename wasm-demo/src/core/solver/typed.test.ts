// The typed API (`acsSolve`, `acsCurve`): the JSON API's answers as objects, typed by the
// package's declarations (tsify). `tsc` (npm run build) checks the @ts-expect-error lines.
import { createRequire } from "node:module";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";

import { describe, expect, it } from "vitest";

import type { Primitive, SolveRequest } from "acs";

const require = createRequire(import.meta.url);
const pkgDir = dirname(require.resolve("acs/package.json"));
const acs = (await import("acs")) as typeof import("acs");
acs.initSync({ module: readFileSync(join(pkgDir, "acs_bg.wasm")) });

const json = (request: object) => JSON.parse(acs.acsSolveSketch(JSON.stringify(request)));

// Every form of the types with two (line or points, lines or arc, one point or a pair),
// Parameters, property references and pass-through fields.
const sketch: Primitive[] = [
  { type: "point", id: "o", x: 0, y: 0, fixed: true },
  { type: "point", id: "a", x: 5, y: 0.2, group: "g" },
  { type: "point", id: "b", x: 3, y: 4 },
  { type: "point", id: "c", x: 3.2, y: 7, isConstruction: true },
  { type: "line", id: "l1", p1_id: "o", p2_id: "a" },
  { type: "line", id: "l2", p1_id: "b", p2_id: "c" },
  { type: "line", id: "l3", p1_id: "o", p2_id: "b" },
  { type: "param", id: "w", value: 3, name: "Width" },
  { type: "point", id: "ac", x: 10, y: 0, fixed: true },
  { type: "point", id: "as", x: 12, y: 0 },
  { type: "point", id: "ae", x: 10, y: 2 },
  { type: "arc", id: "arc", c_id: "ac", start_id: "as", end_id: "ae", radius: 2, start_angle: 0, end_angle: 1.6 },
  { type: "point", id: "ec", x: 0, y: 10, fixed: true },
  { type: "point", id: "ef", x: 3, y: 10, fixed: true },
  { type: "ellipse", id: "el", c_id: "ec", focus1_id: "ef", radmin: 2, fixed: true },
  { type: "point", id: "e1", x: 4, y: 10.5 },
  { type: "point", id: "e2", x: 0.5, y: 12 },
  { type: "point", id: "e3", x: -0.5, y: 8 },
  { type: "point", id: "cc", x: 20, y: 0, fixed: true },
  { type: "circle", id: "circle", c_id: "cc", radius: 3 },
  { type: "horizontal", id: "h_pts", a: "o", b: "a" },
  { type: "vertical", id: "v_line", line: "l2" },
  { type: "horizontal_distance", id: "hd_line", line: "l1", value: 5 },
  { type: "horizontal_distance", id: "hd_pts", a: "o", b: "b", value: "w" },
  { type: "angle", id: "ang_lines", a: "l1", b: "l3", value: 0.9, reverseA: false },
  { type: "angle", id: "ang_arc", arc: "arc", value: 1.2 },
  { type: "ellipse_axis", id: "ax_pt", ellipse: "el", point: "e1", which: "major" },
  { type: "ellipse_axis", id: "ax_pair", ellipse: "el", a: "e2", b: "e3", which: "minor" },
  { type: "equal", id: "r", a: { entity: "circle", property: "radius" }, b: 4 },
];

describe("acsSolve", () => {
  it("answers as acsSolveSketch, with plain objects", () => {
    const request: SolveRequest = { version: 1, primitives: sketch };
    const typed = acs.acsSolve(request);
    expect(typed).toStrictEqual(json(request));
    expect(typed.status).toBe("converged");
    // Flattened forms come back as objects, not Maps.
    for (const p of "primitives" in typed ? typed.primitives : []) expect(p.constructor).toBe(Object);
  });

  it("narrows on status", () => {
    const result = acs.acsSolve({ version: 1, primitives: [...sketch, { type: "horizontal", id: "bad", line: "ghost" }] });
    if (result.status !== "invalid") throw new Error("expected a rejection");
    expect(result.constraintId).toBe("bad");
    expect(result.error).toMatch(/ghost/);
  });

  it("answers a request that doesn't match the types with a rejection", () => {
    const result = acs.acsSolve({ version: 1 } as unknown as SolveRequest);
    expect(result.status).toBe("invalid");
  });
});

describe("acsCurve", () => {
  it("answers as acsSplineCurve", () => {
    const request = { version: 1, handles: [[0, 0], [1, 2], [3, 2.5], [4, -1]] as [number, number][], interpolated: true };
    expect(acs.acsCurve(request)).toStrictEqual(JSON.parse(acs.acsSplineCurve(JSON.stringify(request))));
    const bad = { ...request, handles: request.handles.slice(0, 3), interpolated: false };
    expect(acs.acsCurve(bad)).toStrictEqual(JSON.parse(acs.acsSplineCurve(JSON.stringify(bad))));
  });
});

// Type-level: the declarations reject what the solver would.
// @ts-expect-error: horizontal takes a line or both points
export const half: Primitive = { type: "horizontal", id: "h", a: "o" };
// @ts-expect-error: which is "major" | "minor"
export const axis: Primitive = { type: "ellipse_axis", id: "x", ellipse: "el", point: "p", which: "long" };
// @ts-expect-error: a line's endpoints are p1_id and p2_id
export const typo: Primitive = { type: "line", id: "l", p1_id: "a", p2: "b" };
