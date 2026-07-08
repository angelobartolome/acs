/**
 * ConstraintRegistry — declarative metadata for every JSON constraint type
 * supported by `acsSolveSketch`.
 *
 * UI buttons, selection validation, default scalar values and serialization
 * all derive from these entries. Supporting a new constraint type is a single
 * new entry in `CONSTRAINT_DEFS`.
 */

import type {
  ConstraintInstance,
  EntityId,
  EntityKind,
  JsonPrimitive,
  SketchEntity,
} from "../model/types";
import { isArc, isCircle, isLine, isPoint } from "../model/types";

/** Looks up an entity by id (usually backed by the store). */
export type EntityResolver = (id: EntityId) => SketchEntity | undefined;

export interface SelectionSlot {
  kind: EntityKind;
  count: number;
}

export interface ScalarParamDef {
  /** JSON field name: distance | angle | radius | x | y */
  key: string;
  label: string;
  /** angle params are stored in radians but edited in degrees */
  unit: "length" | "angle" | "coordinate";
  /** derive a sensible default from the selected entities (already slot-ordered) */
  defaultFrom?: (entities: SketchEntity[], resolve: EntityResolver) => number;
}

export interface ConstraintDef {
  /** JSON `type` string */
  type: string;
  label: string;
  description: string;
  /** required selection, in slot order */
  selection: SelectionSlot[];
  /** JSON field name per selected entity, in slot order */
  entityFields: string[];
  scalarParams?: ScalarParamDef[];
}

// ---------------------------------------------------------------------------
// geometry helpers used for scalar defaults
// ---------------------------------------------------------------------------

interface XY {
  x: number;
  y: number;
}

function pointXY(e: SketchEntity | undefined): XY {
  return e !== undefined && isPoint(e) ? { x: e.x, y: e.y } : { x: 0, y: 0 };
}

function lineEndpoints(
  e: SketchEntity | undefined,
  resolve: EntityResolver,
): [XY, XY] {
  if (e !== undefined && isLine(e)) {
    return [pointXY(resolve(e.p1)), pointXY(resolve(e.p2))];
  }
  return [
    { x: 0, y: 0 },
    { x: 1, y: 0 },
  ];
}

function distance(a: XY, b: XY): number {
  return Math.hypot(b.x - a.x, b.y - a.y);
}

function pointLineDistance(p: XY, a: XY, b: XY): number {
  const len = distance(a, b);
  if (len < 1e-12) return distance(p, a);
  return (
    Math.abs((b.x - a.x) * (a.y - p.y) - (a.x - p.x) * (b.y - a.y)) / len
  );
}

function angleBetween(u: XY, v: XY): number {
  const cross = u.x * v.y - u.y * v.x;
  const dot = u.x * v.x + u.y * v.y;
  return Math.atan2(cross, dot);
}

function currentRadius(e: SketchEntity | undefined): number {
  if (e !== undefined && (isCircle(e) || isArc(e))) return e.radius;
  return 1;
}

// ---------------------------------------------------------------------------
// registry entries — one per JSON constraint type (27 total)
// ---------------------------------------------------------------------------

const P = (count: number): SelectionSlot => ({ kind: "point", count });
const L = (count: number): SelectionSlot => ({ kind: "line", count });
const C = (count: number): SelectionSlot => ({ kind: "circle", count });
const A = (count: number): SelectionSlot => ({ kind: "arc", count });

export const CONSTRAINT_DEFS: readonly ConstraintDef[] = [
  {
    type: "horizontal_pp",
    label: "Horizontal",
    description: "Two points share the same Y",
    selection: [P(2)],
    entityFields: ["p1_id", "p2_id"],
  },
  {
    type: "vertical_pp",
    label: "Vertical",
    description: "Two points share the same X",
    selection: [P(2)],
    entityFields: ["p1_id", "p2_id"],
  },
  {
    type: "horizontal_l",
    label: "Horizontal",
    description: "Line is horizontal",
    selection: [L(1)],
    entityFields: ["l_id"],
  },
  {
    type: "vertical_l",
    label: "Vertical",
    description: "Line is vertical",
    selection: [L(1)],
    entityFields: ["l_id"],
  },
  {
    type: "parallel",
    label: "Parallel",
    description: "Two lines are parallel",
    selection: [L(2)],
    entityFields: ["l1_id", "l2_id"],
  },
  {
    type: "perpendicular_ll",
    label: "Perpendicular",
    description: "Two lines are perpendicular",
    selection: [L(2)],
    entityFields: ["l1_id", "l2_id"],
  },
  {
    type: "perpendicular_pppp",
    label: "Perpendicular (4 pts)",
    description: "Segments p1p2 and p3p4 are perpendicular",
    selection: [P(4)],
    entityFields: ["l1p1_id", "l1p2_id", "l2p1_id", "l2p2_id"],
  },
  {
    type: "p2p_coincident",
    label: "Coincident",
    description: "Two points coincide",
    selection: [P(2)],
    entityFields: ["p1_id", "p2_id"],
  },
  {
    type: "point_on_line_pl",
    label: "Point on Line",
    description: "Point lies on the line",
    selection: [P(1), L(1)],
    entityFields: ["p_id", "l_id"],
  },
  {
    type: "point_on_line_ppp",
    label: "Point on Line (3 pts)",
    description: "First point lies on the segment of the other two",
    selection: [P(3)],
    entityFields: ["p_id", "lp1_id", "lp2_id"],
  },
  {
    type: "point_on_circle",
    label: "Point on Circle",
    description: "Point lies on the circle",
    selection: [P(1), C(1)],
    entityFields: ["p_id", "c_id"],
  },
  {
    type: "p2p_distance",
    label: "Distance",
    description: "Distance between two points",
    selection: [P(2)],
    entityFields: ["p1_id", "p2_id"],
    scalarParams: [
      {
        key: "distance",
        label: "Distance",
        unit: "length",
        defaultFrom: (ents) => distance(pointXY(ents[0]), pointXY(ents[1])),
      },
    ],
  },
  {
    type: "p2l_distance",
    label: "Distance",
    description: "Distance between a point and a line",
    selection: [P(1), L(1)],
    entityFields: ["p_id", "l_id"],
    scalarParams: [
      {
        key: "distance",
        label: "Distance",
        unit: "length",
        defaultFrom: (ents, resolve) => {
          const [a, b] = lineEndpoints(ents[1], resolve);
          return pointLineDistance(pointXY(ents[0]), a, b);
        },
      },
    ],
  },
  {
    type: "l2l_angle_pppp",
    label: "Angle (4 pts)",
    description: "Angle between segments p1p2 and p3p4",
    selection: [P(4)],
    entityFields: ["l1p1_id", "l1p2_id", "l2p1_id", "l2p2_id"],
    scalarParams: [
      {
        key: "angle",
        label: "Angle",
        unit: "angle",
        defaultFrom: (ents) => {
          const [a, b, c, d] = ents.map(pointXY);
          return angleBetween(
            { x: b.x - a.x, y: b.y - a.y },
            { x: d.x - c.x, y: d.y - c.y },
          );
        },
      },
    ],
  },
  {
    type: "l2l_angle_ll",
    label: "Angle",
    description: "Angle between two lines",
    selection: [L(2)],
    entityFields: ["l1_id", "l2_id"],
    scalarParams: [
      {
        key: "angle",
        label: "Angle",
        unit: "angle",
        defaultFrom: (ents, resolve) => {
          const [a, b] = lineEndpoints(ents[0], resolve);
          const [c, d] = lineEndpoints(ents[1], resolve);
          return angleBetween(
            { x: b.x - a.x, y: b.y - a.y },
            { x: d.x - c.x, y: d.y - c.y },
          );
        },
      },
    ],
  },
  {
    type: "equal_length",
    label: "Equal Length",
    description: "Two lines have equal length",
    selection: [L(2)],
    entityFields: ["l1_id", "l2_id"],
  },
  {
    type: "equal_radius_cc",
    label: "Equal Radius",
    description: "Two circles have equal radius",
    selection: [C(2)],
    entityFields: ["c1_id", "c2_id"],
  },
  {
    type: "equal_radius_aa",
    label: "Equal Radius",
    description: "Two arcs have equal radius",
    selection: [A(2)],
    entityFields: ["a1_id", "a2_id"],
  },
  {
    type: "circle_radius",
    label: "Radius",
    description: "Fix the circle radius",
    selection: [C(1)],
    entityFields: ["c_id"],
    scalarParams: [
      {
        key: "radius",
        label: "Radius",
        unit: "length",
        defaultFrom: (ents) => currentRadius(ents[0]),
      },
    ],
  },
  {
    type: "arc_radius",
    label: "Radius",
    description: "Fix the arc radius",
    selection: [A(1)],
    entityFields: ["a_id"],
    scalarParams: [
      {
        key: "radius",
        label: "Radius",
        unit: "length",
        defaultFrom: (ents) => currentRadius(ents[0]),
      },
    ],
  },
  {
    type: "tangent_lc",
    label: "Tangent",
    description: "Line is tangent to the circle",
    selection: [L(1), C(1)],
    entityFields: ["l_id", "c_id"],
  },
  {
    type: "midpoint_on_line_ll",
    label: "Midpoint on Line",
    description: "Midpoint of the first line lies on the second",
    selection: [L(2)],
    entityFields: ["l1_id", "l2_id"],
  },
  {
    type: "midpoint_on_line_pppp",
    label: "Midpoint on Line (4 pts)",
    description: "Midpoint of p1p2 lies on segment p3p4",
    selection: [P(4)],
    entityFields: ["l1p1_id", "l1p2_id", "l2p1_id", "l2p2_id"],
  },
  {
    type: "p2p_symmetric_ppp",
    label: "Symmetric (pt)",
    description: "Third point is the midpoint of the first two",
    selection: [P(3)],
    entityFields: ["p1_id", "p2_id", "p_id"],
  },
  {
    type: "p2p_symmetric_ppl",
    label: "Symmetric (line)",
    description: "Two points are symmetric about the line",
    selection: [P(2), L(1)],
    entityFields: ["p1_id", "p2_id", "l_id"],
  },
  {
    type: "coordinate_x",
    label: "Fix X",
    description: "Fix the X coordinate of a point",
    selection: [P(1)],
    entityFields: ["p_id"],
    scalarParams: [
      {
        key: "x",
        label: "X",
        unit: "coordinate",
        defaultFrom: (ents) => pointXY(ents[0]).x,
      },
    ],
  },
  {
    type: "coordinate_y",
    label: "Fix Y",
    description: "Fix the Y coordinate of a point",
    selection: [P(1)],
    entityFields: ["p_id"],
    scalarParams: [
      {
        key: "y",
        label: "Y",
        unit: "coordinate",
        defaultFrom: (ents) => pointXY(ents[0]).y,
      },
    ],
  },
];

const DEF_BY_TYPE: ReadonlyMap<string, ConstraintDef> = new Map(
  CONSTRAINT_DEFS.map((d) => [d.type, d]),
);

export function getConstraintDef(type: string): ConstraintDef | undefined {
  return DEF_BY_TYPE.get(type);
}

// ---------------------------------------------------------------------------
// selection matching
// ---------------------------------------------------------------------------

/** Expand a selection spec into a flat kind sequence, e.g. [point, point, line]. */
export function expandedKinds(def: ConstraintDef): EntityKind[] {
  return def.selection.flatMap((s) =>
    Array.from({ length: s.count }, () => s.kind),
  );
}

/**
 * Check whether the ordered user selection can satisfy `def`, and if so,
 * return entity ids arranged in slot order (kind pools consumed in selection
 * order). Returns null when the selection does not match.
 */
export function matchSelection(
  def: ConstraintDef,
  selected: readonly SketchEntity[],
): EntityId[] | null {
  const kinds = expandedKinds(def);
  if (kinds.length !== selected.length) return null;

  const pools: Record<EntityKind, EntityId[]> = {
    point: [],
    line: [],
    circle: [],
    arc: [],
  };
  for (const e of selected) pools[e.kind].push(e.id);

  const result: EntityId[] = [];
  for (const k of kinds) {
    const id = pools[k].shift();
    if (id === undefined) return null;
    result.push(id);
  }
  // all pools must be exhausted
  if (pools.point.length + pools.line.length + pools.circle.length + pools.arc.length > 0) {
    return null;
  }
  return result;
}

/** All constraint defs applicable to the current (ordered) selection. */
export function applicableConstraints(
  selected: readonly SketchEntity[],
): ConstraintDef[] {
  if (selected.length === 0) return [];
  return CONSTRAINT_DEFS.filter((d) => matchSelection(d, selected) !== null);
}

/** Compute default scalar params for a def given slot-ordered entities. */
export function defaultParams(
  def: ConstraintDef,
  slotEntities: readonly SketchEntity[],
  resolve: EntityResolver,
): Record<string, number> {
  const params: Record<string, number> = {};
  for (const p of def.scalarParams ?? []) {
    params[p.key] =
      p.defaultFrom !== undefined
        ? p.defaultFrom([...slotEntities], resolve)
        : 0;
  }
  return params;
}

// ---------------------------------------------------------------------------
// serialization
// ---------------------------------------------------------------------------

/** Serialize a constraint instance to its `acsSolveSketch` JSON primitive. */
export function constraintToPrimitive(c: ConstraintInstance): JsonPrimitive {
  const def = getConstraintDef(c.type);
  if (def === undefined) {
    throw new Error(`Unknown constraint type: ${c.type}`);
  }
  if (c.entities.length !== def.entityFields.length) {
    throw new Error(
      `Constraint ${c.id} (${c.type}) expects ${def.entityFields.length} entities, got ${c.entities.length}`,
    );
  }
  const prim: JsonPrimitive = { id: c.id, type: c.type };
  def.entityFields.forEach((field, i) => {
    prim[field] = c.entities[i];
  });
  for (const p of def.scalarParams ?? []) {
    prim[p.key] = c.params[p.key] ?? 0;
  }
  return prim;
}

/**
 * Parse a JSON constraint primitive back into a ConstraintInstance
 * (used by import / raw JSON apply). Returns null for unknown types.
 */
export function primitiveToConstraint(
  prim: Record<string, unknown>,
  fallbackId: string,
): ConstraintInstance | null {
  const type = typeof prim.type === "string" ? prim.type : "";
  const def = getConstraintDef(type);
  if (def === undefined) return null;

  const entities: EntityId[] = [];
  for (const field of def.entityFields) {
    const v = prim[field];
    if (typeof v !== "string") return null;
    entities.push(v);
  }
  const params: Record<string, number> = {};
  for (const p of def.scalarParams ?? []) {
    const v = prim[p.key];
    if (typeof v === "number") params[p.key] = v;
    else if (typeof v === "string" && v !== "" && !Number.isNaN(Number(v)))
      params[p.key] = Number(v);
    else params[p.key] = 0;
  }
  const id = typeof prim.id === "string" && prim.id !== "" ? prim.id : fallbackId;
  return { id, type, entities, params };
}
