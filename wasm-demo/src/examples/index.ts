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
  {
    id: "rectangle",
    label: "Dimensioned rectangle",
    description:
      "A rectangle with fixed 60\u00D730 dimensions using horizontal/vertical + distance",
    sketch: {
      entities: [
        pt("p1", 0, 0, true),
        pt("p2", 58, 4, false),
        pt("p3", 62, 33, false),
        pt("p4", -3, 29, false),
        ln("l1", "p1", "p2"),
        ln("l2", "p2", "p3"),
        ln("l3", "p3", "p4"),
        ln("l4", "p4", "p1"),
      ],
      constraints: [
        k("k1", "horizontal_l", ["l1"]),
        k("k2", "horizontal_l", ["l3"]),
        k("k3", "vertical_l", ["l2"]),
        k("k4", "vertical_l", ["l4"]),
        k("k5", "p2p_distance", ["p1", "p2"], { distance: 60 }),
        k("k6", "p2p_distance", ["p2", "p3"], { distance: 30 }),
      ],
    },
  },
  {
    id: "parallelogram",
    label: "Parallelogram",
    description: "Opposite sides constrained parallel and equal in length",
    sketch: {
      entities: [
        pt("p1", 0, 0, true),
        pt("p2", 50, 2, false),
        pt("p3", 68, 34, false),
        pt("p4", 16, 32, false),
        ln("l1", "p1", "p2"),
        ln("l2", "p2", "p3"),
        ln("l3", "p3", "p4"),
        ln("l4", "p4", "p1"),
      ],
      constraints: [
        k("k1", "parallel", ["l1", "l3"]),
        k("k2", "parallel", ["l2", "l4"]),
        k("k3", "equal_length", ["l1", "l3"]),
        k("k4", "equal_length", ["l2", "l4"]),
        k("k5", "horizontal_l", ["l1"]),
        k("k6", "p2p_distance", ["p1", "p2"], { distance: 50 }),
        k("k7", "p2p_distance", ["p2", "p3"], { distance: 36 }),
        k("k8", "l2l_angle_ll", ["l1", "l4"], { angle: 60 * DEG }),
      ],
    },
  },
  {
    id: "right-triangle",
    label: "Right triangle",
    description: "A triangle with a perpendicular corner and fixed legs",
    sketch: {
      entities: [
        pt("p1", 0, 0, true),
        pt("p2", 48, -3, false),
        pt("p3", 4, 36, false),
        ln("l1", "p1", "p2"),
        ln("l2", "p1", "p3"),
        ln("l3", "p2", "p3"),
      ],
      constraints: [
        k("k1", "horizontal_l", ["l1"]),
        k("k2", "perpendicular_ll", ["l1", "l2"]),
        k("k3", "p2p_distance", ["p1", "p2"], { distance: 45 }),
        k("k4", "p2p_distance", ["p1", "p3"], { distance: 30 }),
      ],
    },
  },
  {
    id: "equilateral",
    label: "Equilateral triangle",
    description: "Three equal-length sides with a fixed base length",
    sketch: {
      entities: [
        pt("p1", 0, 0, true),
        pt("p2", 40, 2, false),
        pt("p3", 20, 34, false),
        ln("l1", "p1", "p2"),
        ln("l2", "p1", "p3"),
        ln("l3", "p2", "p3"),
      ],
      constraints: [
        k("k1", "horizontal_l", ["l1"]),
        k("k2", "l2l_angle_ll", ["l1", "l2"], { angle: 60 * DEG }),
        k("k3", "equal_length", ["l1", "l2"]),
        k("k4", "equal_length", ["l1", "l3"]),
        k("k5", "p2p_distance", ["p1", "p2"], { distance: 40 }),
      ],
    },
  },
  {
    id: "rhombus",
    label: "Rhombus",
    description: "Four equal sides with a fixed corner angle",
    sketch: {
      entities: [
        pt("p1", 0, 0, true),
        pt("p2", 40, 3, false),
        pt("p3", 64, 34, false),
        pt("p4", 22, 31, false),
        ln("l1", "p1", "p2"),
        ln("l2", "p2", "p3"),
        ln("l3", "p3", "p4"),
        ln("l4", "p4", "p1"),
      ],
      constraints: [
        k("k1", "horizontal_l", ["l1"]),
        k("k2", "parallel", ["l1", "l3"]),
        k("k3", "parallel", ["l2", "l4"]),
        k("k4", "equal_length", ["l1", "l2"]),
        k("k5", "p2p_distance", ["p1", "p2"], { distance: 40 }),
        k("k6", "l2l_angle_ll", ["l1", "l4"], { angle: 70 * DEG }),
      ],
    },
  },
  {
    id: "isosceles",
    label: "Isosceles triangle",
    description: "Two equal sides with the apex centered over the base",
    sketch: {
      entities: [
        pt("p1", 0, 0, true),
        pt("p2", 50, 2, false),
        pt("p3", 22, 40, false),
        ln("l1", "p1", "p2"),
        ln("l2", "p1", "p3"),
        ln("l3", "p2", "p3"),
      ],
      constraints: [
        k("k1", "horizontal_l", ["l1"]),
        k("k2", "equal_length", ["l2", "l3"]),
        k("k3", "p2p_distance", ["p1", "p2"], { distance: 50 }),
        k("k4", "p2p_distance", ["p1", "p3"], { distance: 40 }),
      ],
    },
  },
  {
    id: "point-on-circle",
    label: "Point on circle",
    description: "A point pinned onto the perimeter of a fixed circle",
    sketch: {
      entities: [
        pt("p1", 0, 0, true),
        circ("c1", "p1", 30),
        pt("p2", 40, 20, false),
      ],
      constraints: [
        k("k1", "circle_radius", ["c1"], { radius: 30 }),
        k("k2", "point_on_circle", ["p2", "c1"]),
        k("k3", "coordinate_x", ["p2"], { x: 18 }),
      ],
    },
  },
  {
    id: "point-on-line",
    label: "Point on line",
    description: "A midpoint pinned onto a diagonal line",
    sketch: {
      entities: [
        pt("p1", 0, 0, true),
        pt("p2", 60, 40, true),
        ln("l1", "p1", "p2"),
        pt("p3", 20, 40, false),
      ],
      constraints: [
        k("k1", "point_on_line_pl", ["p3", "l1"]),
        k("k2", "coordinate_x", ["p3"], { x: 30 }),
      ],
    },
  },
  {
    id: "two-tangent",
    label: "Line tangent to two circles",
    description: "A common tangent line touching two fixed circles",
    sketch: {
      entities: [
        pt("p1", -40, 0, true),
        circ("c1", "p1", 20),
        pt("p2", 45, 0, true),
        circ("c2", "p2", 12),
        pt("p3", -60, 30, false),
        pt("p4", 60, 22, false),
        ln("l1", "p3", "p4"),
      ],
      constraints: [
        k("k1", "circle_radius", ["c1"], { radius: 20 }),
        k("k2", "circle_radius", ["c2"], { radius: 12 }),
        k("k3", "tangent_lc", ["l1", "c1"]),
        k("k4", "tangent_lc", ["l1", "c2"]),
        k("k5", "horizontal_l", ["l1"]),
      ],
    },
  },
  {
    id: "perpendicular-cross",
    label: "Perpendicular cross",
    description: "Two lines forced perpendicular with fixed lengths",
    sketch: {
      entities: [
        pt("p1", 0, 0, true),
        pt("p2", 50, 6, false),
        pt("p3", 10, -20, false),
        pt("p4", 4, 40, false),
        ln("l1", "p1", "p2"),
        ln("l2", "p3", "p4"),
      ],
      constraints: [
        k("k1", "horizontal_l", ["l1"]),
        k("k2", "perpendicular_ll", ["l1", "l2"]),
        k("k3", "p2p_distance", ["p1", "p2"], { distance: 50 }),
        k("k4", "p2p_distance", ["p3", "p4"], { distance: 60 }),
      ],
    },
  },
  {
    id: "angled-lines",
    label: "60\u00B0 angle",
    description: "Two lines from a shared origin held at a 60\u00B0 angle",
    sketch: {
      entities: [
        pt("p1", 0, 0, true),
        pt("p2", 50, 0, false),
        pt("p3", 30, 40, false),
        ln("l1", "p1", "p2"),
        ln("l2", "p1", "p3"),
      ],
      constraints: [
        k("k1", "horizontal_l", ["l1"]),
        k("k2", "l2l_angle_ll", ["l1", "l2"], { angle: 60 * DEG }),
        k("k3", "p2p_distance", ["p1", "p2"], { distance: 50 }),
        k("k4", "p2p_distance", ["p1", "p3"], { distance: 50 }),
      ],
    },
  },
  {
    id: "regular-pentagon",
    label: "Regular pentagon",
    description: "Five equal sides with equal interior angles",
    sketch: {
      entities: [
        pt("p1", 0, 0, true),
        pt("p2", 40, 0, false),
        pt("p3", 52.36, 38.04, false),
        pt("p4", 20, 61.55, false),
        pt("p5", -12.36, 38.04, false),
        ln("l1", "p1", "p2"),
        ln("l2", "p2", "p3"),
        ln("l3", "p3", "p4"),
        ln("l4", "p4", "p5"),
        ln("l5", "p5", "p1"),
      ],
      constraints: [
        k("k1", "horizontal_l", ["l1"]),
        k("k2", "l2l_angle_ll", ["l1", "l2"], { angle: 72 * DEG }),
        k("k3", "l2l_angle_ll", ["l2", "l3"], { angle: 72 * DEG }),
        k("k4", "l2l_angle_ll", ["l3", "l4"], { angle: 72 * DEG }),
        k("k5", "l2l_angle_ll", ["l4", "l5"], { angle: 72 * DEG }),
        k("k6", "p2p_distance", ["p1", "p2"], { distance: 40 }),
        k("k7", "p2p_distance", ["p2", "p3"], { distance: 40 }),
        k("k8", "p2p_distance", ["p5", "p1"], { distance: 40 }),
      ],
    },
  },
  {
    id: "concentric-three",
    label: "Three concentric circles",
    description: "Three circles sharing one center, two with equal radius",
    sketch: {
      entities: [
        pt("p1", 0, 0, true),
        pt("p2", 20, 8, false),
        pt("p3", -14, 12, false),
        circ("c1", "p1", 30),
        circ("c2", "p2", 20),
        circ("c3", "p3", 20),
      ],
      constraints: [
        k("k1", "p2p_coincident", ["p1", "p2"]),
        k("k2", "p2p_coincident", ["p1", "p3"]),
        k("k3", "circle_radius", ["c1"], { radius: 30 }),
        k("k4", "circle_radius", ["c2"], { radius: 18 }),
        k("k5", "equal_radius_cc", ["c2", "c3"]),
      ],
    },
  },
  {
    id: "symmetric-triangle",
    label: "Symmetric points",
    description: "A pair of points mirrored across a vertical axis at fixed X",
    sketch: {
      entities: [
        pt("p1", 0, -40, true),
        pt("p2", 0, 40, true),
        ln("l1", "p1", "p2"),
        pt("p3", -30, 15, false),
        pt("p4", 34, 20, false),
        ln("l2", "p3", "p4"),
      ],
      constraints: [
        k("k1", "p2p_symmetric_ppl", ["p3", "p4", "l1"]),
        k("k2", "horizontal_l", ["l2"]),
        k("k3", "coordinate_x", ["p3"], { x: -35 }),
      ],
    },
  },
  {
    id: "slot",
    label: "Slot shape",
    description: "Two parallel lines capped by equal-radius arcs",
    sketch: {
      entities: [
        pt("c1", -25, 0, true),
        pt("c2", 25, 0, true),
        arc("a1", "c1", 15, 90 * DEG, 270 * DEG),
        arc("a2", "c2", 15, -90 * DEG, 90 * DEG),
        pt("t1", -25, 15, false),
        pt("t2", 25, 15, false),
        pt("b1", -25, -15, false),
        pt("b2", 25, -15, false),
        ln("lt", "t1", "t2"),
        ln("lb", "b1", "b2"),
      ],
      constraints: [
        k("k1", "arc_radius", ["a1"], { radius: 15 }),
        k("k2", "equal_radius_aa", ["a1", "a2"]),
        k("k3", "horizontal_l", ["lt"]),
        k("k4", "horizontal_l", ["lb"]),
      ],
    },
  },
  {
    id: "trapezoid",
    label: "Trapezoid",
    description: "Two parallel horizontal sides of different fixed lengths",
    sketch: {
      entities: [
        pt("p1", 0, 0, true),
        pt("p2", 60, 3, false),
        pt("p3", 46, 32, false),
        pt("p4", 14, 30, false),
        ln("l1", "p1", "p2"),
        ln("l2", "p2", "p3"),
        ln("l3", "p3", "p4"),
        ln("l4", "p4", "p1"),
      ],
      constraints: [
        k("k1", "horizontal_l", ["l1"]),
        k("k2", "horizontal_l", ["l3"]),
        k("k3", "p2p_distance", ["p1", "p2"], { distance: 60 }),
        k("k4", "p2p_distance", ["p3", "p4"], { distance: 32 }),
        k("k5", "equal_length", ["l2", "l4"]),
        k("k6", "coordinate_y", ["p4"], { y: 30 }),
        k("k7", "coordinate_x", ["p4"], { x: 14 }),
      ],
    },
  },
];
