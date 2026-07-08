/** Preset sketches for the gallery. All coordinates in world units. */

import type {
  ConstraintInstance,
  EntityId,
  Sketch,
  SketchEntity,
} from "../core/model/types";

function pt(id: string, x: number, y: number, fixed = false): SketchEntity {
  return { kind: "point", id, x, y, fixed };
}
function ln(id: string, p1: EntityId, p2: EntityId): SketchEntity {
  return { kind: "line", id, p1, p2 };
}
function circ(
  id: string,
  center: EntityId,
  radius: number,
  fixed = false,
): SketchEntity {
  return { kind: "circle", id, center, radius, fixed };
}
function arc(
  id: string,
  center: EntityId,
  radius: number,
  startAngle: number,
  endAngle: number,
): SketchEntity {
  return { kind: "arc", id, center, radius, startAngle, endAngle, fixed: false };
}
function k(
  id: string,
  type: string,
  entities: EntityId[],
  params: Record<string, number> = {},
): ConstraintInstance {
  return { id, type, entities, params };
}

export interface Example {
  id: string;
  label: string;
  description: string;
  sketch: Sketch;
}

const DEG = Math.PI / 180;

export const EXAMPLES: readonly Example[] = [
  {
    id: "square",
    label: "Constrained square",
    description:
      "Four lines constrained into a 40-unit square anchored at the origin",
    sketch: {
      entities: [
        pt("p1", 0, 0, true),
        pt("p2", 42, 3, false),
        pt("p3", 44, 38, false),
        pt("p4", -2, 41, false),
        ln("l1", "p1", "p2"),
        ln("l2", "p2", "p3"),
        ln("l3", "p3", "p4"),
        ln("l4", "p4", "p1"),
      ],
      constraints: [
        k("k1", "horizontal_l", ["l1"]),
        k("k2", "vertical_l", ["l2"]),
        k("k3", "horizontal_l", ["l3"]),
        k("k4", "vertical_l", ["l4"]),
        k("k5", "equal_length", ["l1", "l2"]),
        k("k6", "p2p_distance", ["p1", "p2"], { distance: 40 }),
      ],
    },
  },
  {
    id: "tangent",
    label: "Tangent line + circle",
    description: "A horizontal line kept tangent to a fixed-radius circle",
    sketch: {
      entities: [
        pt("p1", 0, 0, true),
        circ("c1", "p1", 20),
        pt("p2", -40, 32, false),
        pt("p3", 40, 28, false),
        ln("l1", "p2", "p3"),
      ],
      constraints: [
        k("k1", "circle_radius", ["c1"], { radius: 20 }),
        k("k2", "horizontal_l", ["l1"]),
        k("k3", "tangent_lc", ["l1", "c1"]),
      ],
    },
  },
  {
    id: "concentric",
    label: "Concentric + equal radius",
    description:
      "Two circles made concentric (coincident centers) with equal radii",
    sketch: {
      entities: [
        pt("p1", -30, 0, true),
        pt("p2", 30, 12, false),
        circ("c1", "p1", 25),
        circ("c2", "p2", 14),
      ],
      constraints: [
        k("k1", "p2p_coincident", ["p1", "p2"]),
        k("k2", "equal_radius_cc", ["c1", "c2"]),
        k("k3", "circle_radius", ["c1"], { radius: 25 }),
      ],
    },
  },
  {
    id: "symmetry",
    label: "Symmetry about line",
    description: "Two points kept symmetric about a fixed vertical axis",
    sketch: {
      entities: [
        pt("p1", 0, -40, true),
        pt("p2", 0, 40, true),
        ln("l1", "p1", "p2"),
        pt("p3", -25, 10, false),
        pt("p4", 32, 18, false),
        ln("l2", "p3", "p4"),
      ],
      constraints: [
        k("k1", "p2p_symmetric_ppl", ["p3", "p4", "l1"]),
        k("k2", "p2p_distance", ["p3", "p4"], { distance: 50 }),
      ],
    },
  },
  {
    id: "dims",
    label: "Midpoint + angle dims",
    description:
      "A 45\u00B0 angle dimension between two lines plus a midpoint-on-line constraint",
    sketch: {
      entities: [
        pt("p1", 0, 0, true),
        pt("p2", 40, 0, false),
        pt("p3", 30, 28, false),
        ln("l1", "p1", "p2"),
        ln("l2", "p1", "p3"),
        pt("p4", -20, 22, false),
        pt("p5", 24, 26, false),
        ln("l3", "p4", "p5"),
      ],
      constraints: [
        k("k1", "horizontal_l", ["l1"]),
        k("k2", "p2p_distance", ["p1", "p2"], { distance: 40 }),
        k("k3", "l2l_angle_ll", ["l1", "l2"], { angle: 45 * DEG }),
        k("k4", "p2p_distance", ["p1", "p3"], { distance: 40 }),
        k("k5", "midpoint_on_line_ll", ["l3", "l2"]),
      ],
    },
  },
  {
    id: "arc",
    label: "Arc with radius",
    description: "A quarter arc held at radius 30 via arc_radius",
    sketch: {
      entities: [
        pt("p1", 0, 0, true),
        arc("a1", "p1", 22, 0, 90 * DEG),
        pt("p2", 55, 0, true),
        arc("a2", "p2", 12, 90 * DEG, 270 * DEG),
      ],
      constraints: [
        k("k1", "arc_radius", ["a1"], { radius: 30 }),
        k("k2", "equal_radius_aa", ["a1", "a2"]),
      ],
    },
  },
];
