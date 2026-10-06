/**
 * ConstraintRegistry — declarative metadata for every variant of ACS's
 * native constraint vocabulary, as `acsSolveSketch` accepts it and
 * `acsConstraintCatalog` lists it.
 *
 * A native type (`distance`, `on`, `tangent`, …) has one variant per
 * combination of entity kinds (and `extension` and `internal` flags); the solver infers the
 * variant from the entities a constraint references. The demo keeps one entry
 * per variant, under its own `key`, so each can be offered for the selection
 * it fits. UI buttons, selection validation, default scalar values and
 * serialization all derive from these entries.
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

/**
 * Kind of entity a selection slot takes: the demo's entity kinds, plus
 * `ellipse` and `elliptical_arc`, which the solver supports but the demo
 * can't draw yet (so a def with such a slot is never offered and is skipped
 * on import).
 */
export type SlotKind = EntityKind | "ellipse" | "elliptical_arc";

export interface SelectionSlot {
  kind: SlotKind;
  count: number;
}

export interface ScalarParamDef {
  /** JSON field name: value | side | angle | distance | count */
  key: string;
  label: string;
  /** angle params are stored in radians but edited in degrees; a side is +1 or -1; a count is an instance index */
  unit: "length" | "angle" | "coordinate" | "side" | "count";
  /** derive a sensible default from the selected entities (already slot-ordered) */
  defaultFrom?: (entities: SketchEntity[], resolve: EntityResolver) => number;
}

export interface ConstraintDef {
  /** unique id of this variant in the demo, stored in `ConstraintInstance.def` */
  key: string;
  /** native JSON `type` string, shared by the type's variants */
  type: string;
  /**
   * the JSON `extension` flag this variant sends: `true` for an Extension
   * variant, `false` for its segment sibling, absent when the type has none
   */
  extension?: boolean;
  /**
   * the JSON `internal` flag this variant sends: `true` for an inside
   * variant (inside tangency, a distance measured inside a circle), `false`
   * for its outside sibling, absent when the type has none
   */
  internal?: boolean;
  label: string;
  /** short text drawn on the canvas next to the constrained entities */
  badge: string;
  description: string;
  /** required selection, in slot order */
  selection: SelectionSlot[];
  /**
   * JSON field name per selected entity, in slot order; `name[i]` is element
   * `i` of the array field `name` (catalog `{ name, index }`), see
   * {@link fieldPath}
   */
  entityFields: string[];
  scalarParams?: ScalarParamDef[];
  /**
   * JSON fields that take a number, a Parameter id or an
   * `{entity, property}` property reference (catalog kind `value`). The demo's selection model
   * can't express these, so such constraints are never offered and are
   * skipped on import.
   */
  valueFields?: string[];
  /**
   * JSON fields naming one of an ellipse's axes, `"major"` or `"minor"`
   * (catalog kind `axis`). Only ellipse constraints have them.
   */
  axisFields?: string[];
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

/** Distance from `p` to the closest point of segment `ab` (lines are segments). */
function pointLineDistance(p: XY, a: XY, b: XY): number {
  const dx = b.x - a.x;
  const dy = b.y - a.y;
  const len2 = dx * dx + dy * dy;
  if (len2 < 1e-24) return distance(p, a);
  const t = Math.min(1, Math.max(0, ((p.x - a.x) * dx + (p.y - a.y) * dy) / len2));
  return distance(p, { x: a.x + t * dx, y: a.y + t * dy });
}

/** Perpendicular distance from `p` to the Extension (infinite line) through `ab`. */
function pointExtensionDistance(p: XY, a: XY, b: XY): number {
  const dx = b.x - a.x;
  const dy = b.y - a.y;
  const len = Math.hypot(dx, dy);
  if (len < 1e-12) return distance(p, a);
  return Math.abs(dx * (p.y - a.y) - dy * (p.x - a.x)) / len;
}

/** +1 when `p` is left of a → b (or on it), -1 when right. */
function sideOf(p: XY, a: XY, b: XY): number {
  return (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x) < 0 ? -1 : 1;
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

function centerXY(e: SketchEntity | undefined, resolve: EntityResolver): XY {
  if (e !== undefined && (isCircle(e) || isArc(e))) return pointXY(resolve(e.center));
  return { x: 0, y: 0 };
}

/** Counter-clockwise sweep of an arc in (0, 2π]; 0 is a full turn. */
function arcSweep(e: SketchEntity | undefined): number {
  if (e === undefined || !isArc(e)) return Math.PI / 2;
  const s = (((e.endAngle - e.startAngle) % (2 * Math.PI)) + 2 * Math.PI) % (2 * Math.PI);
  return s === 0 ? 2 * Math.PI : s;
}

// ---------------------------------------------------------------------------
// registry entries — one per row (variant) of acsConstraintCatalog
// ---------------------------------------------------------------------------

const P = (count: number): SelectionSlot => ({ kind: "point", count });
const L = (count: number): SelectionSlot => ({ kind: "line", count });
const C = (count: number): SelectionSlot => ({ kind: "circle", count });
const A = (count: number): SelectionSlot => ({ kind: "arc", count });
const E = (count: number): SelectionSlot => ({ kind: "ellipse", count });
const EA = (count: number): SelectionSlot => ({ kind: "elliptical_arc", count });

function lineAngle(ents: SketchEntity[], resolve: EntityResolver): number {
  const [a, b] = lineEndpoints(ents[0], resolve);
  const [c, d] = lineEndpoints(ents[1], resolve);
  return angleBetween(
    { x: b.x - a.x, y: b.y - a.y },
    { x: d.x - c.x, y: d.y - c.y },
  );
}

function direction(a: XY, b: XY): number {
  return Math.atan2(b.y - a.y, b.x - a.x);
}

const pointLineDistanceParam: ScalarParamDef = {
  key: "value",
  label: "Distance",
  unit: "length",
  defaultFrom: (ents, resolve) => {
    const [a, b] = lineEndpoints(ents[1], resolve);
    return pointLineDistance(pointXY(ents[0]), a, b);
  },
};

const pointExtensionDistanceParam: ScalarParamDef = {
  key: "value",
  label: "Distance",
  unit: "length",
  defaultFrom: (ents, resolve) => {
    const [a, b] = lineEndpoints(ents[1], resolve);
    return pointExtensionDistance(pointXY(ents[0]), a, b);
  },
};

const lengthParam = (
  label: string,
  measure: (ents: SketchEntity[], resolve: EntityResolver) => number,
): ScalarParamDef => ({ key: "value", label, unit: "length", defaultFrom: measure });

/** Gap from a point (slot 0) to a circle (slot 1): outside, or `inside`. */
const pointCircleGap = (inside: boolean) =>
  lengthParam("Distance", (ents, resolve) => {
    const gap = distance(pointXY(ents[0]), centerXY(ents[1], resolve)) - currentRadius(ents[1]);
    return inside ? -gap : gap;
  });

/** Gap between two circles: outside, or the smaller inside the larger. */
const circleCircleGap = (inside: boolean) =>
  lengthParam("Distance", (ents, resolve) => {
    const d = distance(centerXY(ents[0], resolve), centerXY(ents[1], resolve));
    const [r1, r2] = [currentRadius(ents[0]), currentRadius(ents[1])];
    return inside ? Math.abs(r1 - r2) - d : d - r1 - r2;
  });

const radiusParam: ScalarParamDef = {
  key: "value",
  label: "Radius",
  unit: "length",
  defaultFrom: (ents) => currentRadius(ents[0]),
};

export const CONSTRAINT_DEFS: readonly ConstraintDef[] = [
  {
    key: "coincident",
    type: "coincident",
    label: "Coincident",
    badge: "≡",
    description: "Two points coincide",
    selection: [P(2)],
    entityFields: ["a", "b"],
  },
  {
    key: "horizontal_line",
    type: "horizontal",
    label: "Horizontal",
    badge: "H",
    description: "Line is horizontal",
    selection: [L(1)],
    entityFields: ["line"],
  },
  {
    key: "horizontal_points",
    type: "horizontal",
    label: "Horizontal",
    badge: "H",
    description: "Two points share the same Y",
    selection: [P(2)],
    entityFields: ["a", "b"],
  },
  {
    key: "vertical_line",
    type: "vertical",
    label: "Vertical",
    badge: "V",
    description: "Line is vertical",
    selection: [L(1)],
    entityFields: ["line"],
  },
  {
    key: "vertical_points",
    type: "vertical",
    label: "Vertical",
    badge: "V",
    description: "Two points share the same X",
    selection: [P(2)],
    entityFields: ["a", "b"],
  },
  {
    key: "parallel",
    type: "parallel",
    label: "Parallel",
    badge: "∥",
    description: "Two lines are parallel",
    selection: [L(2)],
    entityFields: ["a", "b"],
  },
  {
    key: "perpendicular",
    type: "perpendicular",
    label: "Perpendicular",
    badge: "⊥",
    description: "Two lines are perpendicular",
    selection: [L(2)],
    entityFields: ["a", "b"],
  },
  {
    key: "collinear",
    type: "collinear",
    label: "Collinear",
    badge: "⋯",
    description: "Both lines lie on one infinite line (they need not overlap)",
    selection: [L(2)],
    entityFields: ["a", "b"],
  },
  {
    key: "normal_circle",
    type: "normal",
    label: "Normal",
    badge: "⊥O",
    description: "The line's Extension passes through the circle's center",
    selection: [L(1), C(1)],
    entityFields: ["line", "curve"],
  },
  {
    key: "normal_arc",
    type: "normal",
    label: "Normal",
    badge: "⊥A",
    description: "The line's Extension passes through the arc's center",
    selection: [L(1), A(1)],
    entityFields: ["line", "curve"],
  },
  {
    key: "angle",
    type: "angle",
    label: "Angle",
    badge: "∠",
    description: "Directed angle from the first line to the second",
    selection: [L(2)],
    entityFields: ["a", "b"],
    scalarParams: [
      { key: "value", label: "Angle", unit: "angle", defaultFrom: lineAngle },
    ],
  },
  {
    key: "angle_arc",
    type: "angle",
    label: "Arc Angle",
    badge: "∠",
    description: "Sweep of the arc, between 0° and 360°",
    selection: [A(1)],
    entityFields: ["arc"],
    scalarParams: [
      { key: "value", label: "Angle", unit: "angle", defaultFrom: (ents) => arcSweep(ents[0]) },
    ],
  },
  {
    key: "direction_line",
    type: "direction",
    label: "Direction",
    badge: "∠",
    description: "Direction of the line, measured CCW from +X",
    selection: [L(1)],
    entityFields: ["line"],
    scalarParams: [
      {
        key: "value",
        label: "Angle",
        unit: "angle",
        defaultFrom: (ents, resolve) => {
          const [a, b] = lineEndpoints(ents[0], resolve);
          return direction(a, b);
        },
      },
    ],
  },
  {
    key: "direction_points",
    type: "direction",
    label: "Direction",
    badge: "∠",
    description: "Direction from the first point to the second, measured CCW from +X",
    selection: [P(2)],
    entityFields: ["a", "b"],
    scalarParams: [
      {
        key: "value",
        label: "Angle",
        unit: "angle",
        defaultFrom: (ents) => direction(pointXY(ents[0]), pointXY(ents[1])),
      },
    ],
  },
  {
    key: "distance_points",
    type: "distance",
    label: "Distance",
    badge: "↔",
    description: "Distance between two points",
    selection: [P(2)],
    entityFields: ["a", "b"],
    scalarParams: [
      {
        key: "value",
        label: "Distance",
        unit: "length",
        defaultFrom: (ents) => distance(pointXY(ents[0]), pointXY(ents[1])),
      },
    ],
  },
  {
    key: "distance_point_line",
    type: "distance",
    extension: false,
    label: "Distance",
    badge: "↔",
    description: "Distance from a point to a line segment",
    selection: [P(1), L(1)],
    entityFields: ["a", "b"],
    scalarParams: [pointLineDistanceParam],
  },
  {
    key: "distance_point_extension",
    type: "distance",
    extension: true,
    label: "Distance to Extension",
    badge: "↔E",
    description: "Perpendicular distance from a point to the line's Extension",
    selection: [P(1), L(1)],
    entityFields: ["a", "b"],
    scalarParams: [pointExtensionDistanceParam],
  },
  {
    key: "distance_point_circle",
    type: "distance",
    internal: false,
    label: "Distance",
    badge: "↔",
    description: "Gap between a point and a circle, outside it",
    selection: [P(1), C(1)],
    entityFields: ["a", "b"],
    scalarParams: [pointCircleGap(false)],
  },
  {
    key: "distance_point_circle_internal",
    type: "distance",
    internal: true,
    label: "Distance Inside",
    badge: "↔i",
    description: "Gap between a point and a circle, inside it",
    selection: [P(1), C(1)],
    entityFields: ["a", "b"],
    scalarParams: [pointCircleGap(true)],
  },
  {
    key: "distance_line_circle",
    type: "distance",
    extension: false,
    label: "Distance",
    badge: "↔",
    description: "Gap between a line segment and a circle",
    selection: [L(1), C(1)],
    entityFields: ["a", "b"],
    scalarParams: [
      lengthParam("Distance", (ents, resolve) => {
        const [a, b] = lineEndpoints(ents[0], resolve);
        return pointLineDistance(centerXY(ents[1], resolve), a, b) - currentRadius(ents[1]);
      }),
    ],
  },
  {
    key: "distance_extension_circle",
    type: "distance",
    extension: true,
    label: "Distance to Extension",
    badge: "↔E",
    description: "Gap between the line's Extension and a circle",
    selection: [L(1), C(1)],
    entityFields: ["a", "b"],
    scalarParams: [
      lengthParam("Distance", (ents, resolve) => {
        const [a, b] = lineEndpoints(ents[0], resolve);
        return pointExtensionDistance(centerXY(ents[1], resolve), a, b) - currentRadius(ents[1]);
      }),
    ],
  },
  {
    key: "distance_circles",
    type: "distance",
    internal: false,
    label: "Distance",
    badge: "↔",
    description: "Gap between two circles, outside each other",
    selection: [C(2)],
    entityFields: ["a", "b"],
    scalarParams: [circleCircleGap(false)],
  },
  {
    key: "distance_circles_internal",
    type: "distance",
    internal: true,
    label: "Distance Inside",
    badge: "↔i",
    description: "Gap between the smaller circle and the inside of the larger",
    selection: [C(2)],
    entityFields: ["a", "b"],
    scalarParams: [circleCircleGap(true)],
  },
  {
    key: "distance_circle_arc",
    type: "distance",
    internal: false,
    label: "Distance",
    badge: "↔",
    description: "Gap between a circle and an arc, outside each other, within the arc",
    selection: [C(1), A(1)],
    entityFields: ["a", "b"],
    scalarParams: [circleCircleGap(false)],
  },
  {
    key: "distance_circle_arc_internal",
    type: "distance",
    internal: true,
    label: "Distance Inside",
    badge: "↔i",
    description: "Gap between a circle and an arc, one inside the other, within the arc",
    selection: [C(1), A(1)],
    entityFields: ["a", "b"],
    scalarParams: [circleCircleGap(true)],
  },
  {
    key: "distance_arcs",
    type: "distance",
    internal: false,
    label: "Distance",
    badge: "↔",
    description: "Gap between two arcs, outside each other, within both arcs",
    selection: [A(2)],
    entityFields: ["a", "b"],
    scalarParams: [circleCircleGap(false)],
  },
  {
    key: "distance_arcs_internal",
    type: "distance",
    internal: true,
    label: "Distance Inside",
    badge: "↔i",
    description: "Gap between two arcs, one inside the other, within both arcs",
    selection: [A(2)],
    entityFields: ["a", "b"],
    scalarParams: [circleCircleGap(true)],
  },
  {
    key: "distance_lines",
    type: "distance",
    label: "Distance",
    badge: "↔",
    description: "Second line is parallel to the first, this far from its Extension",
    selection: [L(2)],
    entityFields: ["a", "b"],
    scalarParams: [
      lengthParam("Distance", (ents, resolve) => {
        const [a, b] = lineEndpoints(ents[0], resolve);
        const [c, d] = lineEndpoints(ents[1], resolve);
        return (pointExtensionDistance(c, a, b) + pointExtensionDistance(d, a, b)) / 2;
      }),
    ],
  },
  {
    key: "offset",
    type: "offset",
    label: "Offset",
    badge: "↔±",
    description: "Distance from a point to the line's Extension, on one side of it",
    selection: [P(1), L(1)],
    entityFields: ["point", "line"],
    scalarParams: [
      pointExtensionDistanceParam,
      {
        key: "side",
        label: "Side",
        unit: "side",
        defaultFrom: (ents, resolve) => {
          const [a, b] = lineEndpoints(ents[1], resolve);
          return sideOf(pointXY(ents[0]), a, b);
        },
      },
    ],
  },
  {
    key: "on_line",
    type: "on",
    extension: false,
    label: "Point on Line",
    badge: "⋅L",
    description: "Point lies on the line segment",
    selection: [P(1), L(1)],
    entityFields: ["point", "curve"],
  },
  {
    key: "on_extension",
    type: "on",
    extension: true,
    label: "Point on Extension",
    badge: "⋅E",
    description: "Point lies on the line's Extension (the infinite line through it)",
    selection: [P(1), L(1)],
    entityFields: ["point", "curve"],
  },
  {
    key: "on_circle",
    type: "on",
    label: "Point on Circle",
    badge: "⋅O",
    description: "Point lies on the circle",
    selection: [P(1), C(1)],
    entityFields: ["point", "curve"],
  },
  {
    key: "on_arc",
    type: "on",
    label: "Point on Arc",
    badge: "⋅A",
    description: "Point lies on the arc, between its start and end",
    selection: [P(1), A(1)],
    entityFields: ["point", "curve"],
  },
  {
    key: "on_ellipse",
    type: "on",
    label: "Point on Ellipse",
    badge: "⋅E",
    description: "Point lies on the ellipse",
    selection: [P(1), E(1)],
    entityFields: ["point", "curve"],
  },
  {
    key: "on_elliptical_arc",
    type: "on",
    label: "Point on Elliptical Arc",
    badge: "⋅EA",
    description: "Point lies on the elliptical arc, between its start and end",
    selection: [P(1), EA(1)],
    entityFields: ["point", "curve"],
  },
  {
    key: "midpoint",
    type: "midpoint",
    label: "Midpoint",
    badge: "M",
    description: "Point is the midpoint of the line",
    selection: [P(1), L(1)],
    entityFields: ["entities[0]", "entities[1]"],
  },
  {
    key: "midpoint_on_line",
    type: "midpoint",
    extension: false,
    label: "Midpoint on Line",
    badge: "M",
    description: "Midpoint of the first line lies on the second segment",
    selection: [L(2)],
    entityFields: ["entities[0]", "entities[1]"],
  },
  {
    key: "midpoint_on_extension",
    type: "midpoint",
    extension: true,
    label: "Midpoint on Extension",
    badge: "ME",
    description: "Midpoint of the first line lies on the second line's Extension",
    selection: [L(2)],
    entityFields: ["entities[0]", "entities[1]"],
  },
  {
    key: "tangent_line_circle",
    type: "tangent",
    extension: false,
    label: "Tangent",
    badge: "tan",
    description: "Line is tangent to the circle, touching within the segment",
    selection: [L(1), C(1)],
    entityFields: ["a", "b"],
  },
  {
    key: "tangent_extension_circle",
    type: "tangent",
    extension: true,
    label: "Tangent to Extension",
    badge: "tanE",
    description: "Circle is tangent to the line's Extension, touching anywhere along it",
    selection: [L(1), C(1)],
    entityFields: ["a", "b"],
  },
  {
    key: "tangent_line_arc",
    type: "tangent",
    label: "Tangent",
    badge: "tan",
    description: "Line is tangent to the arc, touching within the segment and the arc",
    selection: [L(1), A(1)],
    entityFields: ["a", "b"],
  },
  {
    key: "tangent_line_ellipse",
    type: "tangent",
    label: "Tangent",
    badge: "tan",
    description: "Line is tangent to the ellipse, touching within the segment",
    selection: [L(1), E(1)],
    entityFields: ["a", "b"],
  },
  {
    key: "tangent_line_elliptical_arc",
    type: "tangent",
    label: "Tangent",
    badge: "tan",
    description:
      "Line is tangent to the elliptical arc, touching within the segment and the arc (or along its tangent at an endpoint they share)",
    selection: [L(1), EA(1)],
    entityFields: ["a", "b"],
  },
  {
    key: "tangent_arc_elliptical_arc",
    type: "tangent",
    label: "Tangent",
    badge: "tan",
    description: "Arc and elliptical arc are tangent at an endpoint they share",
    selection: [A(1), EA(1)],
    entityFields: ["a", "b"],
  },
  {
    key: "tangent_circles",
    type: "tangent",
    internal: false,
    label: "Tangent Circles",
    badge: "tan",
    description: "Two circles touch externally",
    selection: [C(2)],
    entityFields: ["a", "b"],
  },
  {
    key: "tangent_circles_internal",
    type: "tangent",
    internal: true,
    label: "Tangent Inside",
    badge: "tanI",
    description: "Two circles touch, one inside the other",
    selection: [C(2)],
    entityFields: ["a", "b"],
  },
  {
    key: "tangent_circle_arc",
    type: "tangent",
    internal: false,
    label: "Tangent",
    badge: "tan",
    description: "A circle and an arc touch externally, within the arc",
    selection: [C(1), A(1)],
    entityFields: ["a", "b"],
  },
  {
    key: "tangent_circle_arc_internal",
    type: "tangent",
    internal: true,
    label: "Tangent Inside",
    badge: "tanI",
    description: "A circle and an arc touch, one inside the other, within the arc",
    selection: [C(1), A(1)],
    entityFields: ["a", "b"],
  },
  {
    key: "tangent_arcs",
    type: "tangent",
    internal: false,
    label: "Tangent",
    badge: "tan",
    description: "Two arcs touch externally, within both arcs (or at an endpoint they share)",
    selection: [A(2)],
    entityFields: ["a", "b"],
  },
  {
    key: "tangent_arcs_internal",
    type: "tangent",
    internal: true,
    label: "Tangent Inside",
    badge: "tanI",
    description: "Two arcs touch, one inside the other, within both arcs (or at an endpoint they share)",
    selection: [A(2)],
    entityFields: ["a", "b"],
  },
  {
    key: "concentric_circles",
    type: "concentric",
    label: "Concentric",
    badge: "◎",
    description: "Two circles share a center",
    selection: [C(2)],
    entityFields: ["a", "b"],
  },
  {
    key: "concentric_circle_arc",
    type: "concentric",
    label: "Concentric",
    badge: "◎",
    description: "A circle and an arc share a center",
    selection: [C(1), A(1)],
    entityFields: ["a", "b"],
  },
  {
    key: "concentric_arcs",
    type: "concentric",
    label: "Concentric",
    badge: "◎",
    description: "Two arcs share a center",
    selection: [A(2)],
    entityFields: ["a", "b"],
  },
  {
    key: "equal_length",
    type: "equal",
    label: "Equal Length",
    badge: "=",
    description: "Two lines have equal length",
    selection: [L(2)],
    entityFields: ["a", "b"],
  },
  {
    key: "equal_radius_circles",
    type: "equal",
    label: "Equal Radius",
    badge: "R=",
    description: "Two circles have equal radius",
    selection: [C(2)],
    entityFields: ["a", "b"],
  },
  {
    key: "equal_radius_circle_arc",
    type: "equal",
    label: "Equal Radius",
    badge: "R=",
    description: "A circle and an arc have equal radius",
    selection: [C(1), A(1)],
    entityFields: ["a", "b"],
  },
  {
    key: "equal_radius_arcs",
    type: "equal",
    label: "Equal Radius",
    badge: "R=",
    description: "Two arcs have equal radius",
    selection: [A(2)],
    entityFields: ["a", "b"],
  },
  {
    key: "equal_values",
    type: "equal",
    label: "Equal",
    badge: "=",
    description: "a = b (numbers, Parameters or entity properties)",
    selection: [],
    entityFields: [],
    valueFields: ["a", "b"],
  },
  {
    key: "radius_circle",
    type: "radius",
    label: "Radius",
    badge: "R",
    description: "Fix the circle radius",
    selection: [C(1)],
    entityFields: ["curve"],
    scalarParams: [radiusParam],
  },
  {
    key: "radius_arc",
    type: "radius",
    label: "Radius",
    badge: "R",
    description: "Fix the arc radius",
    selection: [A(1)],
    entityFields: ["curve"],
    scalarParams: [radiusParam],
  },
  {
    key: "diameter_circle",
    type: "diameter",
    label: "Diameter",
    badge: "⌀",
    description: "Fix the circle diameter",
    selection: [C(1)],
    entityFields: ["curve"],
    scalarParams: [lengthParam("Diameter", (ents) => 2 * currentRadius(ents[0]))],
  },
  {
    key: "diameter_arc",
    type: "diameter",
    label: "Diameter",
    badge: "⌀",
    description: "Fix the arc diameter",
    selection: [A(1)],
    entityFields: ["curve"],
    scalarParams: [lengthParam("Diameter", (ents) => 2 * currentRadius(ents[0]))],
  },
  {
    key: "length_line",
    type: "length",
    label: "Length",
    badge: "L",
    description: "Fix the line length",
    selection: [L(1)],
    entityFields: ["curve"],
    scalarParams: [
      lengthParam("Length", (ents, resolve) => {
        const [a, b] = lineEndpoints(ents[0], resolve);
        return distance(a, b);
      }),
    ],
  },
  {
    key: "length_arc",
    type: "length",
    label: "Length",
    badge: "L",
    description: "Fix the arc length (radius × sweep)",
    selection: [A(1)],
    entityFields: ["curve"],
    scalarParams: [
      lengthParam("Length", (ents) => currentRadius(ents[0]) * arcSweep(ents[0])),
    ],
  },
  {
    key: "difference",
    type: "difference",
    label: "Difference",
    badge: "Δ",
    description: "b - a = value (numbers, Parameters or entity properties)",
    selection: [],
    entityFields: [],
    valueFields: ["a", "b", "value"],
  },
  {
    key: "x",
    type: "x",
    label: "Fix X",
    badge: "X",
    description: "Fix the X coordinate of a point",
    selection: [P(1)],
    entityFields: ["point"],
    scalarParams: [
      {
        key: "value",
        label: "X",
        unit: "coordinate",
        defaultFrom: (ents) => pointXY(ents[0]).x,
      },
    ],
  },
  {
    key: "y",
    type: "y",
    label: "Fix Y",
    badge: "Y",
    description: "Fix the Y coordinate of a point",
    selection: [P(1)],
    entityFields: ["point"],
    scalarParams: [
      {
        key: "value",
        label: "Y",
        unit: "coordinate",
        defaultFrom: (ents) => pointXY(ents[0]).y,
      },
    ],
  },
  {
    key: "mirror",
    type: "mirror",
    label: "Mirror",
    badge: "⇄",
    description: "Second point is the first mirrored across the line's Extension",
    selection: [P(2), L(1)],
    entityFields: ["source", "image", "axis"],
  },
  {
    key: "rotation",
    type: "rotation",
    label: "Rotation",
    badge: "↻",
    description: "Second point is the first rotated about the third (center) by an angle",
    selection: [P(3)],
    entityFields: ["source", "copy", "center"],
    scalarParams: [
      {
        key: "angle",
        label: "Angle",
        unit: "angle",
        defaultFrom: (ents) => {
          const [p0, pk, c] = ents.map(pointXY);
          return angleBetween(
            { x: p0.x - c.x, y: p0.y - c.y },
            { x: pk.x - c.x, y: pk.y - c.y },
          );
        },
      },
    ],
  },
  {
    key: "translation",
    type: "translation",
    label: "Translation",
    badge: "⇉",
    description:
      "Second point is the first moved distance × count along from → to",
    selection: [P(4)],
    entityFields: ["source", "copy", "from", "to"],
    scalarParams: [
      {
        key: "distance",
        label: "Spacing",
        unit: "length",
        defaultFrom: (ents) => distance(pointXY(ents[0]), pointXY(ents[1])),
      },
      { key: "count", label: "Count", unit: "count", defaultFrom: () => 1 },
    ],
  },
  {
    key: "ellipse_axis",
    type: "ellipse_axis",
    label: "Ellipse Axis",
    badge: "ax",
    description: "Point is an end of the ellipse's major or minor axis",
    selection: [E(1), P(1)],
    entityFields: ["ellipse", "point"],
    axisFields: ["which"],
  },
  {
    key: "ellipse_diameter",
    type: "ellipse_axis",
    label: "Ellipse Diameter",
    badge: "⌀",
    description: "The two points are opposite ends of the ellipse's major or minor axis",
    selection: [E(1), P(2)],
    entityFields: ["ellipse", "a", "b"],
    axisFields: ["which"],
  },
  {
    key: "elliptical_arc_axis",
    type: "ellipse_axis",
    label: "Elliptical Arc Axis",
    badge: "ax",
    description: "Point is an end of the elliptical arc's major or minor axis",
    selection: [EA(1), P(1)],
    entityFields: ["ellipse", "point"],
    axisFields: ["which"],
  },
  {
    key: "elliptical_arc_diameter",
    type: "ellipse_axis",
    label: "Elliptical Arc Diameter",
    badge: "⌀",
    description:
      "The two points are opposite ends of the elliptical arc's major or minor axis",
    selection: [EA(1), P(2)],
    entityFields: ["ellipse", "a", "b"],
    axisFields: ["which"],
  },
];

const DEF_BY_KEY: ReadonlyMap<string, ConstraintDef> = new Map(
  CONSTRAINT_DEFS.map((d) => [d.key, d]),
);

export function getConstraintDef(key: string): ConstraintDef | undefined {
  return DEF_BY_KEY.get(key);
}

// ---------------------------------------------------------------------------
// selection matching
// ---------------------------------------------------------------------------

/** Expand a selection spec into a flat kind sequence, e.g. [point, point, line]. */
export function expandedKinds(def: ConstraintDef): SlotKind[] {
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
    if (k === "ellipse" || k === "elliptical_arc") return null; // the demo has no ellipses
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

/** Splits an entity field into its JSON key and array index: `entities[1]` → `["entities", 1]`. */
export function fieldPath(field: string): [string, number | undefined] {
  const m = /^(.+)\[(\d+)\]$/.exec(field);
  return m === null ? [field, undefined] : [m[1], Number(m[2])];
}

function readField(prim: Record<string, unknown>, field: string): unknown {
  const [key, index] = fieldPath(field);
  if (index === undefined) return prim[key];
  const array = prim[key];
  return Array.isArray(array) ? array[index] : undefined;
}

function writeField(prim: JsonPrimitive, field: string, value: string): void {
  const [key, index] = fieldPath(field);
  if (index === undefined) {
    prim[key] = value;
    return;
  }
  const existing = prim[key];
  const array = Array.isArray(existing) ? existing : [];
  array[index] = value;
  prim[key] = array;
}

/** Elements each array field of `def` reads, by key. */
function arrayLengths(def: ConstraintDef): Map<string, number> {
  const lengths = new Map<string, number>();
  for (const field of def.entityFields) {
    const [key, index] = fieldPath(field);
    if (index !== undefined) lengths.set(key, (lengths.get(key) ?? 0) + 1);
  }
  return lengths;
}

/** Serialize a constraint instance to its `acsSolveSketch` JSON primitive. */
export function constraintToPrimitive(c: ConstraintInstance): JsonPrimitive {
  const def = getConstraintDef(c.def);
  if (def === undefined) {
    throw new Error(`Unknown constraint: ${c.def}`);
  }
  if (c.entities.length !== def.entityFields.length) {
    throw new Error(
      `Constraint ${c.id} (${c.def}) expects ${def.entityFields.length} entities, got ${c.entities.length}`,
    );
  }
  const prim: JsonPrimitive = { id: c.id, type: def.type };
  if (def.extension === true) prim.extension = true;
  if (def.internal === true) prim.internal = true;
  def.entityFields.forEach((field, i) => {
    writeField(prim, field, c.entities[i]);
  });
  for (const p of def.scalarParams ?? []) {
    prim[p.key] = c.params[p.key] ?? 0;
  }
  return prim;
}

/**
 * The entity ids `def` reads from `prim`, in slot order, if each names an
 * entity of the slot's kind. Like the solver, `a` and `b` may come in either
 * order.
 */
function matchPrimitive(
  def: ConstraintDef,
  prim: Record<string, unknown>,
  resolve: EntityResolver,
): EntityId[] | null {
  const kinds = expandedKinds(def);
  for (const [key, length] of arrayLengths(def)) {
    const array = prim[key];
    if (!Array.isArray(array) || array.length !== length) return null;
  }
  const read = (fields: string[]): EntityId[] | null => {
    const ids: EntityId[] = [];
    for (const [i, field] of fields.entries()) {
      const v = readField(prim, field);
      if (typeof v !== "string" || resolve(v)?.kind !== kinds[i]) return null;
      ids.push(v);
    }
    return ids;
  };
  const swapped = def.entityFields.map((f) =>
    f === "a" ? "b" : f === "b" ? "a" : f,
  );
  return read(def.entityFields) ?? read(swapped);
}

/**
 * Parse a native JSON constraint primitive back into a ConstraintInstance
 * (used by import / raw JSON apply), choosing the variant whose entity kinds
 * (looked up with `resolve`) and `extension` and `internal` flags match, as
 * the solver does.
 * Returns null for unknown types, unsupported combinations and constraints
 * over values, which the demo doesn't edit.
 */
export function primitiveToConstraint(
  prim: Record<string, unknown>,
  fallbackId: string,
  resolve: EntityResolver,
): ConstraintInstance | null {
  const type = typeof prim.type === "string" ? prim.type : "";
  const extension = prim.extension === true;
  const internal = prim.internal === true;
  for (const def of CONSTRAINT_DEFS) {
    if (def.type !== type || (def.valueFields ?? []).length > 0) continue;
    if ((def.axisFields ?? []).length > 0) continue;
    if ((def.extension === true) !== extension) continue;
    if ((def.internal === true) !== internal) continue;
    const entities = matchPrimitive(def, prim, resolve);
    if (entities === null) continue;

    const params: Record<string, number> = {};
    for (const p of def.scalarParams ?? []) {
      const v = prim[p.key];
      if (typeof v === "number") params[p.key] = v;
      else if (typeof v === "string" && v !== "" && !Number.isNaN(Number(v)))
        params[p.key] = Number(v);
      else params[p.key] = 0;
    }
    const id =
      typeof prim.id === "string" && prim.id !== "" ? prim.id : fallbackId;
    return { id, def: def.key, entities, params };
  }
  return null;
}
