import { describe, expect, it } from "vitest";

import type {
  ConstraintInstance,
  LineEntity,
  PointEntity,
  SketchEntity,
} from "../model/types";
import {
  CONSTRAINT_DEFS,
  applicableConstraints,
  constraintToPrimitive,
  defaultParams,
  expandedKinds,
  getConstraintDef,
  matchSelection,
  primitiveToConstraint,
} from "./registry";

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
  it("covers all 27 JSON constraint types", () => {
    expect(CONSTRAINT_DEFS.length).toBe(27);
    const types = new Set(CONSTRAINT_DEFS.map((d) => d.type));
    expect(types.size).toBe(27);
    for (const t of [
      "horizontal_pp",
      "vertical_pp",
      "horizontal_l",
      "vertical_l",
      "parallel",
      "perpendicular_ll",
      "perpendicular_pppp",
      "p2p_coincident",
      "point_on_line_pl",
      "point_on_line_ppp",
      "point_on_circle",
      "p2p_distance",
      "p2l_distance",
      "l2l_angle_pppp",
      "l2l_angle_ll",
      "equal_length",
      "equal_radius_cc",
      "equal_radius_aa",
      "circle_radius",
      "arc_radius",
      "tangent_lc",
      "midpoint_on_line_ll",
      "midpoint_on_line_pppp",
      "p2p_symmetric_ppp",
      "p2p_symmetric_ppl",
      "coordinate_x",
      "coordinate_y",
    ]) {
      expect(types.has(t), `missing ${t}`).toBe(true);
    }
  });

  it("every def has matching entityFields and selection sizes", () => {
    for (const def of CONSTRAINT_DEFS) {
      expect(
        expandedKinds(def).length,
        `${def.type} slot count`,
      ).toBe(def.entityFields.length);
    }
  });

  it("matches selection regardless of pick order", () => {
    const def = getConstraintDef("point_on_line_pl");
    expect(def).toBeDefined();
    if (def === undefined) return;
    // point first
    expect(matchSelection(def, [P1, L1])).toEqual(["p1", "l1"]);
    // line first — still slots point into p_id
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
    const types = applicableConstraints([P1, P2]).map((d) => d.type);
    expect(types).toContain("horizontal_pp");
    expect(types).toContain("vertical_pp");
    expect(types).toContain("p2p_coincident");
    expect(types).toContain("p2p_distance");
    expect(types).not.toContain("parallel");
  });

  it("derives default scalar params from geometry", () => {
    const def = getConstraintDef("p2p_distance");
    expect(def).toBeDefined();
    if (def === undefined) return;
    const params = defaultParams(def, [P1, P2], resolve);
    expect(params.distance).toBeCloseTo(5); // 3-4-5 triangle
  });

  it("serializes constraints with exact JSON field names", () => {
    const c: ConstraintInstance = {
      id: "k1",
      type: "p2p_distance",
      entities: ["p1", "p2"],
      params: { distance: 5 },
    };
    expect(constraintToPrimitive(c)).toEqual({
      id: "k1",
      type: "p2p_distance",
      p1_id: "p1",
      p2_id: "p2",
      distance: 5,
    });

    const sym: ConstraintInstance = {
      id: "k2",
      type: "p2p_symmetric_ppl",
      entities: ["p1", "p2", "l1"],
      params: {},
    };
    expect(constraintToPrimitive(sym)).toEqual({
      id: "k2",
      type: "p2p_symmetric_ppl",
      p1_id: "p1",
      p2_id: "p2",
      l_id: "l1",
    });
  });

  it("throws on unknown types or wrong entity counts", () => {
    expect(() =>
      constraintToPrimitive({ id: "k", type: "nope", entities: [], params: {} }),
    ).toThrow();
    expect(() =>
      constraintToPrimitive({
        id: "k",
        type: "parallel",
        entities: ["l1"],
        params: {},
      }),
    ).toThrow();
  });

  it("round-trips primitives back into constraint instances", () => {
    const prim = {
      id: "k9",
      type: "l2l_angle_ll",
      l1_id: "l1",
      l2_id: "l2",
      angle: 0.5,
    };
    const c = primitiveToConstraint(prim, "fallback");
    expect(c).toEqual({
      id: "k9",
      type: "l2l_angle_ll",
      entities: ["l1", "l2"],
      params: { angle: 0.5 },
    });
    expect(primitiveToConstraint({ type: "bogus" }, "f")).toBeNull();
  });
});
