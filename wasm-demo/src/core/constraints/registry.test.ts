import { createRequire } from "node:module";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";

import { describe, expect, it } from "vitest";

import type {
  ConstraintInstance,
  LineEntity,
  PointEntity,
  SketchEntity,
} from "../model/types";
import {
  type ConstraintDef,
  CONSTRAINT_DEFS,
  applicableConstraints,
  constraintToPrimitive,
  defaultParams,
  expandedKinds,
  fieldPath,
  getConstraintDef,
  matchSelection,
  primitiveToConstraint,
} from "./registry";

const require = createRequire(import.meta.url);
const pkgDir = dirname(require.resolve("acs/package.json"));
const acs = (await import("acs")) as typeof import("acs");
acs.initSync({ module: readFileSync(join(pkgDir, "acs_bg.wasm")) });

function pt(id: string, x = 0, y = 0): PointEntity {
  return { kind: "point", id, x, y, fixed: false };
}
function ln(id: string, p1: string, p2: string): LineEntity {
  return { kind: "line", id, p1, p2 };
}

const P1 = pt("p1", 0, 0);
const P2 = pt("p2", 3, 4);
const L1 = ln("l1", "p1", "p2");
const ENTITIES: SketchEntity[] = [P1, P2, L1];
const resolve = (id: string) => ENTITIES.find((e) => e.id === id);

describe("ConstraintRegistry", () => {
  it("has one entry per row of the solver's native catalog, field for field", () => {
    type Row = {
      type: string;
      fields: { name: string; index?: number; kind: string }[];
      extension?: boolean;
      internal?: boolean;
      deprecated?: boolean;
    };
    // Deprecated forms are accepted for one more release, never offered.
    const catalog = (JSON.parse(acs.acsConstraintCatalog()) as Row[]).filter(
      (r) => r.deprecated !== true,
    );
    const asRow = (def: ConstraintDef): Row => {
      const kinds = expandedKinds(def);
      const row: Row = {
        type: def.type,
        fields: [
          ...def.entityFields.map((field, i) => {
            const [name, index] = fieldPath(field);
            return index === undefined
              ? { name, kind: kinds[i] }
              : { name, index, kind: kinds[i] };
          }),
          ...(def.scalarParams ?? []).map((p) => ({ name: p.key, kind: "scalar" })),
          ...(def.valueFields ?? []).map((name) => ({ name, kind: "value" })),
          ...(def.axisFields ?? []).map((name) => ({ name, kind: "axis" })),
        ],
      };
      if (def.extension !== undefined) row.extension = def.extension;
      if (def.internal !== undefined) row.internal = def.internal;
      return row;
    };
    const key = (r: Row) =>
      JSON.stringify([
        r.type,
        r.fields.map((f) => `${f.name}${f.index === undefined ? "" : `[${f.index}]`}:${f.kind}`),
        r.extension ?? null,
        r.internal ?? null,
      ]);
    expect(CONSTRAINT_DEFS.map(asRow).map(key).sort()).toEqual(
      catalog.map(key).sort(),
    );
  });

  it("every def has a unique key and matching entityFields and selection sizes", () => {
    expect(new Set(CONSTRAINT_DEFS.map((d) => d.key)).size).toBe(
      CONSTRAINT_DEFS.length,
    );
    for (const def of CONSTRAINT_DEFS) {
      expect(
        expandedKinds(def).length,
        `${def.key} slot count`,
      ).toBe(def.entityFields.length);
    }
  });

  it("matches selection regardless of pick order", () => {
    const def = getConstraintDef("on_line");
    expect(def).toBeDefined();
    if (def === undefined) return;
    // point first
    expect(matchSelection(def, [P1, L1])).toEqual(["p1", "l1"]);
    // line first — still slots the point into `point`
    expect(matchSelection(def, [L1, P1])).toEqual(["p1", "l1"]);
  });

  it("rejects wrong selection signatures", () => {
    const def = getConstraintDef("parallel");
    expect(def).toBeDefined();
    if (def === undefined) return;
    expect(matchSelection(def, [P1, P2])).toBeNull();
    expect(matchSelection(def, [L1])).toBeNull();
    expect(matchSelection(def, [L1, L1, P1])).toBeNull();
  });

  it("finds applicable constraints for two points", () => {
    const keys = applicableConstraints([P1, P2]).map((d) => d.key);
    expect(keys).toContain("horizontal_points");
    expect(keys).toContain("vertical_points");
    expect(keys).toContain("coincident");
    expect(keys).toContain("distance_points");
    expect(keys).toContain("direction_points");
    expect(keys).not.toContain("parallel");
  });

  it("offers both the segment and the Extension variant for a point and a line", () => {
    const keys = applicableConstraints([P1, L1]).map((d) => d.key);
    expect(keys).toEqual(
      expect.arrayContaining([
        "distance_point_line",
        "distance_point_extension",
        "on_line",
        "on_extension",
        "midpoint",
        "offset",
      ]),
    );
  });

  it("never offers or imports ellipse constraints (the demo has no ellipses)", () => {
    for (const sel of [[P1], [P1, L1], [L1], [P1, P2]]) {
      const keys = applicableConstraints(sel).map((d) => d.key);
      expect(keys).not.toContain("on_ellipse");
      expect(keys).not.toContain("tangent_line_ellipse");
      expect(keys).not.toContain("ellipse_axis");
      expect(keys).not.toContain("ellipse_diameter");
    }
    expect(
      primitiveToConstraint(
        { type: "ellipse_axis", ellipse: "e1", point: "p1", which: "major" },
        "f",
        resolve,
      ),
    ).toBeNull();
  });

  it("derives default scalar params from geometry", () => {
    const def = getConstraintDef("distance_points");
    expect(def).toBeDefined();
    if (def === undefined) return;
    const params = defaultParams(def, [P1, P2], resolve);
    expect(params.value).toBeCloseTo(5); // 3-4-5 triangle
  });

  it("serializes constraints with the native type and field names", () => {
    const c: ConstraintInstance = {
      id: "k1",
      def: "distance_points",
      entities: ["p1", "p2"],
      params: { value: 5 },
    };
    expect(constraintToPrimitive(c)).toEqual({
      id: "k1",
      type: "distance",
      a: "p1",
      b: "p2",
      value: 5,
    });

    const mirror: ConstraintInstance = {
      id: "k2",
      def: "mirror",
      entities: ["p1", "p2", "l1"],
      params: {},
    };
    expect(constraintToPrimitive(mirror)).toEqual({
      id: "k2",
      type: "mirror",
      source: "p1",
      image: "p2",
      axis: "l1",
    });

    const ext: ConstraintInstance = {
      id: "k3",
      def: "on_extension",
      entities: ["p1", "l1"],
      params: {},
    };
    expect(constraintToPrimitive(ext)).toEqual({
      id: "k3",
      type: "on",
      extension: true,
      point: "p1",
      curve: "l1",
    });

    const midpoint: ConstraintInstance = {
      id: "k4",
      def: "midpoint",
      entities: ["p1", "l1"],
      params: {},
    };
    expect(constraintToPrimitive(midpoint)).toEqual({
      id: "k4",
      type: "midpoint",
      entities: ["p1", "l1"],
    });
  });

  it("throws on unknown defs or wrong entity counts", () => {
    expect(() =>
      constraintToPrimitive({ id: "k", def: "nope", entities: [], params: {} }),
    ).toThrow();
    expect(() =>
      constraintToPrimitive({
        id: "k",
        def: "parallel",
        entities: ["l1"],
        params: {},
      }),
    ).toThrow();
  });

  it("round-trips primitives back into constraint instances", () => {
    const L2 = ln("l2", "p2", "p1");
    const byId = new Map([...ENTITIES, L2].map((e) => [e.id, e]));
    const lookup = (id: string) => byId.get(id);
    const prim = { id: "k9", type: "angle", a: "l1", b: "l2", value: 0.5 };
    expect(primitiveToConstraint(prim, "fallback", lookup)).toEqual({
      id: "k9",
      def: "angle",
      entities: ["l1", "l2"],
      params: { value: 0.5 },
    });
    expect(primitiveToConstraint({ type: "bogus" }, "f", lookup)).toBeNull();
  });

  it("infers the variant on import from the referenced entities' kinds", () => {
    const kinds = (prim: Record<string, unknown>) =>
      primitiveToConstraint(prim, "f", resolve);
    expect(kinds({ type: "distance", a: "p1", b: "p2", value: 1 })?.def).toBe(
      "distance_points",
    );
    expect(kinds({ type: "distance", a: "p1", b: "l1", value: 1 })?.def).toBe(
      "distance_point_line",
    );
    expect(
      kinds({ type: "distance", a: "p1", b: "l1", value: 1, extension: true }),
    ).toMatchObject({ def: "distance_point_extension", entities: ["p1", "l1"] });
    // a and b may come in either order
    expect(kinds({ type: "distance", a: "l1", b: "p1", value: 1 })).toMatchObject({
      def: "distance_point_line",
      entities: ["p1", "l1"],
    });
    expect(kinds({ type: "horizontal", line: "l1" })?.def).toBe("horizontal_line");
    expect(kinds({ type: "horizontal", a: "p1", b: "p2" })?.def).toBe(
      "horizontal_points",
    );
    expect(kinds({ type: "midpoint", entities: ["p1", "l1"] })).toMatchObject({
      def: "midpoint",
      entities: ["p1", "l1"],
    });
    expect(kinds({ type: "midpoint", entities: ["l1", "p1"] })).toBeNull();
    expect(kinds({ type: "midpoint", entities: ["p1", "l1", "p2"] })).toBeNull();
    // `internal: true` selects the inside variant
    const C1 = { kind: "circle" as const, id: "c1", center: "p2", radius: 1, fixed: false };
    const withCircle = (id: string) => (id === "c1" ? C1 : resolve(id));
    expect(
      primitiveToConstraint({ type: "distance", a: "c1", b: "p1", value: 1 }, "f", withCircle),
    ).toMatchObject({ def: "distance_point_circle", entities: ["p1", "c1"] });
    expect(
      primitiveToConstraint(
        { type: "distance", a: "p1", b: "c1", value: 1, internal: true },
        "f",
        withCircle,
      ),
    ).toMatchObject({ def: "distance_point_circle_internal" });
    expect(
      constraintToPrimitive({
        id: "k",
        def: "distance_point_circle_internal",
        entities: ["p1", "c1"],
        params: { value: 1 },
      }),
    ).toEqual({ id: "k", type: "distance", internal: true, a: "p1", b: "c1", value: 1 });
    // no variant takes a point and a missing entity
    expect(kinds({ type: "distance", a: "p1", b: "ghost", value: 1 })).toBeNull();
  });
});
