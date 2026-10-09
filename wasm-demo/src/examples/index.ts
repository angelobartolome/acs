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
/** An arc from `start` to `end` (counter-clockwise); the solver keeps its endpoints on it. */
function arc(
  id: string,
  center: EntityId,
  start: EntityId,
  end: EntityId,
  radius: number,
  startAngle: number,
  endAngle: number,
): SketchEntity {
  return {
    kind: "arc",
    id,
    center,
    start,
    end,
    radius,
    startAngle,
    endAngle,
    fixed: false,
  };
}
/**
 * An elliptical arc from `start` to `end` (counter-clockwise, parametric
 * angles) on the ellipse with this center, focus and minor radius.
 */
function earc(
  id: string,
  center: EntityId,
  focus: EntityId,
  start: EntityId,
  end: EntityId,
  radmin: number,
  startAngle: number,
  endAngle: number,
  fixed = false,
): SketchEntity {
  return {
    kind: "elliptical_arc",
    id,
    center,
    focus,
    start,
    end,
    radmin,
    startAngle,
    endAngle,
    fixed,
  };
}
/** A spline through fit points (`interpolated`) or on control points. */
function spl(id: string, points: EntityId[], interpolated = true): SketchEntity {
  return { kind: "spline", id, points, interpolated };
}
/** A constraint by registry key (a native type's variant). */
function k(
  id: string,
  def: string,
  entities: EntityId[],
  params: Record<string, number> = {},
): ConstraintInstance {
  return { id, def, entities, params };
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
        k("k1", "horizontal_line", ["l1"]),
        k("k2", "vertical_line", ["l2"]),
        k("k3", "horizontal_line", ["l3"]),
        k("k4", "vertical_line", ["l4"]),
        k("k5", "equal_length", ["l1", "l2"]),
        k("k6", "distance_points", ["p1", "p2"], { value: 40 }),
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
        k("k1", "radius_circle", ["c1"], { value: 20 }),
        k("k2", "horizontal_line", ["l1"]),
        k("k3", "tangent_line_circle", ["l1", "c1"]),
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
        k("k1", "coincident", ["p1", "p2"]),
        k("k2", "equal_radius_circles", ["c1", "c2"]),
        k("k3", "radius_circle", ["c1"], { value: 25 }),
      ],
    },
  },
  {
    id: "symmetry",
    label: "Mirror about line",
    description: "Two points kept mirrored across a fixed vertical axis",
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
        k("k1", "mirror", ["p3", "p4", "l1"]),
        k("k2", "distance_points", ["p3", "p4"], { value: 50 }),
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
        k("k1", "horizontal_line", ["l1"]),
        k("k2", "distance_points", ["p1", "p2"], { value: 40 }),
        k("k3", "angle", ["l1", "l2"], { value: 45 * DEG }),
        k("k4", "distance_points", ["p1", "p3"], { value: 40 }),
        k("k5", "midpoint_on_line", ["l3", "l2"]),
      ],
    },
  },
  {
    id: "arc",
    label: "Arc with radius",
    description:
      "A quarter arc held at radius 30 via arc_radius; a line shares its end point",
    sketch: {
      entities: [
        pt("p1", 0, 0, true),
        pt("s1", 22, 0),
        pt("e1", 0, 22),
        arc("a1", "p1", "s1", "e1", 22, 0, 90 * DEG),
        pt("p2", 55, 0, true),
        pt("s2", 55, 12),
        pt("e2", 55, -12),
        arc("a2", "p2", "s2", "e2", 12, 90 * DEG, 270 * DEG),
        pt("q", -30, 22),
        ln("l1", "e1", "q"),
      ],
      constraints: [
        k("k1", "radius_arc", ["a1"], { value: 30 }),
        k("k2", "equal_radius_arcs", ["a1", "a2"]),
        k("k3", "tangent_line_arc", ["l1", "a1"]),
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
        k("k1", "horizontal_line", ["l1"]),
        k("k2", "horizontal_line", ["l3"]),
        k("k3", "vertical_line", ["l2"]),
        k("k4", "vertical_line", ["l4"]),
        k("k5", "distance_points", ["p1", "p2"], { value: 60 }),
        k("k6", "distance_points", ["p2", "p3"], { value: 30 }),
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
        k("k5", "horizontal_line", ["l1"]),
        k("k6", "distance_points", ["p1", "p2"], { value: 50 }),
        k("k7", "distance_points", ["p2", "p3"], { value: 36 }),
        k("k8", "angle", ["l1", "l4"], { value: 60 * DEG }),
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
        k("k1", "horizontal_line", ["l1"]),
        k("k2", "perpendicular", ["l1", "l2"]),
        k("k3", "distance_points", ["p1", "p2"], { value: 45 }),
        k("k4", "distance_points", ["p1", "p3"], { value: 30 }),
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
        k("k1", "horizontal_line", ["l1"]),
        k("k2", "angle", ["l1", "l2"], { value: 60 * DEG }),
        k("k3", "equal_length", ["l1", "l2"]),
        k("k4", "equal_length", ["l1", "l3"]),
        k("k5", "distance_points", ["p1", "p2"], { value: 40 }),
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
        k("k1", "horizontal_line", ["l1"]),
        k("k2", "parallel", ["l1", "l3"]),
        k("k3", "parallel", ["l2", "l4"]),
        k("k4", "equal_length", ["l1", "l2"]),
        k("k5", "distance_points", ["p1", "p2"], { value: 40 }),
        k("k6", "angle", ["l1", "l4"], { value: 70 * DEG }),
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
        k("k1", "horizontal_line", ["l1"]),
        k("k2", "equal_length", ["l2", "l3"]),
        k("k3", "distance_points", ["p1", "p2"], { value: 50 }),
        k("k4", "distance_points", ["p1", "p3"], { value: 40 }),
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
        k("k1", "radius_circle", ["c1"], { value: 30 }),
        k("k2", "on_circle", ["p2", "c1"]),
        k("k3", "x", ["p2"], { value: 18 }),
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
        k("k1", "on_line", ["p3", "l1"]),
        k("k2", "x", ["p3"], { value: 30 }),
      ],
    },
  },
  {
    id: "two-tangent",
    label: "Line tangent to two circles",
    description:
      "A line tangent to two fixed circles: move either circle and the line follows",
    sketch: {
      entities: [
        pt("p1", -40, 0, true),
        circ("c1", "p1", 20),
        pt("p2", 45, 8, true),
        circ("c2", "p2", 12),
        pt("p3", -60, 30, false),
        pt("p4", 60, 22, false),
        ln("l1", "p3", "p4"),
      ],
      constraints: [
        k("k1", "radius_circle", ["c1"], { value: 20 }),
        k("k2", "radius_circle", ["c2"], { value: 12 }),
        k("k3", "tangent_line_circle", ["l1", "c1"]),
        k("k4", "tangent_line_circle", ["l1", "c2"]),
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
        k("k1", "horizontal_line", ["l1"]),
        k("k2", "perpendicular", ["l1", "l2"]),
        k("k3", "distance_points", ["p1", "p2"], { value: 50 }),
        k("k4", "distance_points", ["p3", "p4"], { value: 60 }),
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
        k("k1", "horizontal_line", ["l1"]),
        k("k2", "angle", ["l1", "l2"], { value: 60 * DEG }),
        k("k3", "distance_points", ["p1", "p2"], { value: 50 }),
        k("k4", "distance_points", ["p1", "p3"], { value: 50 }),
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
        k("k1", "horizontal_line", ["l1"]),
        k("k2", "angle", ["l1", "l2"], { value: 72 * DEG }),
        k("k3", "angle", ["l2", "l3"], { value: 72 * DEG }),
        k("k4", "angle", ["l3", "l4"], { value: 72 * DEG }),
        k("k5", "angle", ["l4", "l5"], { value: 72 * DEG }),
        k("k6", "distance_points", ["p1", "p2"], { value: 40 }),
        k("k7", "distance_points", ["p2", "p3"], { value: 40 }),
        k("k8", "distance_points", ["p5", "p1"], { value: 40 }),
      ],
    },
  },
  {
    id: "polygon-rotations",
    label: "Polygon (pattern)",
    description:
      "A pentagon built the way a polygon tool builds it: one side tangent to a dimensioned circle, the other vertices rotated copies of its endpoints about a free center",
    sketch: (() => {
      // Five sides around center (3, -2), inscribed circle of radius 20, first vertex at
      // 0.3 rad; every vertex is shared by the two sides that meet there (the tool merges the
      // copies' endpoints). Each copy step k rotates both endpoints of the first side by
      // k·72°, so with 5 sides most rotations are implied by the others.
      const n = 5;
      const r = 20;
      const [cx, cy] = [3, -2];
      const R = r / Math.cos(Math.PI / n);
      const step = (2 * Math.PI) / n;
      const v = (k: number) => `v${k % n}`;
      const vertices = Array.from({ length: n }, (_, k) =>
        pt(v(k), cx + R * Math.cos(0.3 + k * step), cy + R * Math.sin(0.3 + k * step)),
      );
      const sides = Array.from({ length: n }, (_, k) => ln(`l${k}`, v(k), v(k + 1)));
      const rotations = Array.from({ length: n - 1 }, (_, i) => i + 1).flatMap((copy) => [
        k(`ra${copy}`, "rotation", ["v0", v(copy), "c"], { angle: copy * step }),
        k(`rb${copy}`, "rotation", ["v1", v(copy + 1), "c"], { angle: copy * step }),
      ]);
      return {
        entities: [pt("c", cx, cy), circ("incircle", "c", r), ...vertices, ...sides],
        constraints: [
          k("kr", "radius_circle", ["incircle"], { value: r }),
          k("kt", "tangent_line_circle", ["l0", "incircle"]),
          ...rotations,
        ],
      };
    })(),
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
        k("k1", "coincident", ["p1", "p2"]),
        k("k2", "coincident", ["p1", "p3"]),
        k("k3", "radius_circle", ["c1"], { value: 30 }),
        k("k4", "radius_circle", ["c2"], { value: 18 }),
        k("k5", "equal_radius_circles", ["c2", "c3"]),
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
        k("k1", "mirror", ["p3", "p4", "l1"]),
        k("k2", "horizontal_line", ["l2"]),
        k("k3", "x", ["p3"], { value: -35 }),
      ],
    },
  },
  {
    id: "slot",
    label: "Slot shape",
    description:
      "Two lines tangent to equal-radius arcs, sharing the arcs' endpoints",
    sketch: {
      entities: [
        pt("c1", -25, 0, true),
        pt("c2", 25, 0, true),
        pt("t1", -24, 13, false),
        pt("t2", 26, 16, false),
        pt("b1", -26, -14, false),
        pt("b2", 24, -16, false),
        arc("a1", "c1", "t1", "b1", 14, 90 * DEG, 270 * DEG),
        arc("a2", "c2", "b2", "t2", 16, -90 * DEG, 90 * DEG),
        ln("lt", "t1", "t2"),
        ln("lb", "b1", "b2"),
      ],
      constraints: [
        k("k1", "radius_arc", ["a1"], { value: 15 }),
        k("k2", "equal_radius_arcs", ["a1", "a2"]),
        k("k3", "tangent_line_arc", ["lt", "a1"]),
        k("k4", "tangent_line_arc", ["lt", "a2"]),
        k("k5", "tangent_line_arc", ["lb", "a1"]),
        k("k6", "tangent_line_arc", ["lb", "a2"]),
      ],
    },
  },
  {
    id: "curved-slot",
    label: "Curved slot",
    description:
      "A slot that follows an arc: outer and inner arcs around the same center, joined by round caps tangent to both. Drag an end to change its sweep.",
    sketch: (() => {
      // Center-line arc of radius 40 from 10° to 120°, slot half-width 8.
      const R = 40;
      const w = 8;
      const a = 10 * DEG;
      const b = 120 * DEG;
      const at = (r: number, t: number, dx = 0, dy = 0): [number, number] => [
        r * Math.cos(t) + dx,
        r * Math.sin(t) + dy,
      ];
      return {
        entities: [
          pt("c", 0, 0, true),
          pt("s", ...at(R, a, 1, -1)),
          pt("e", ...at(R, b, -1, 1)),
          pt("oa", ...at(R + w, a, 1.5, 0)),
          pt("ob", ...at(R + w, b, 0, 1)),
          pt("ia", ...at(R - w, a, -1, 1)),
          pt("ib", ...at(R - w, b, 1, -1)),
          arc("path", "c", "s", "e", R, a, b),
          arc("outer", "c", "oa", "ob", R + w, a, b),
          arc("inner", "c", "ia", "ib", R - w, a, b),
          // Caps run counter-clockwise around the slot's ends.
          arc("cap1", "s", "ia", "oa", w, a + Math.PI, a + 2 * Math.PI),
          arc("cap2", "e", "ob", "ib", w, b, b + Math.PI),
          // Radial construction lines: each end's points lie on them, so the
          // caps meet the outer and inner arcs tangentially.
          ln("r1", "c", "s"),
          ln("r2", "c", "e"),
        ],
        constraints: [
          k("k1", "radius_arc", ["path"], { value: R }),
          k("k2", "radius_arc", ["cap1"], { value: w }),
          k("k3", "on_extension", ["oa", "r1"]),
          k("k4", "on_extension", ["ia", "r1"]),
          k("k5", "on_extension", ["ob", "r2"]),
          k("k6", "on_extension", ["ib", "r2"]),
        ],
      };
    })(),
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
        k("k1", "horizontal_line", ["l1"]),
        k("k2", "horizontal_line", ["l3"]),
        k("k3", "distance_points", ["p1", "p2"], { value: 60 }),
        k("k4", "distance_points", ["p3", "p4"], { value: 32 }),
        k("k5", "equal_length", ["l2", "l4"]),
        k("k6", "y", ["p4"], { value: 30 }),
        k("k7", "x", ["p4"], { value: 14 }),
      ],
    },
  },
  {
    id: "segment-point",
    label: "Point on segment",
    description:
      "A point starting past the end of a line is pulled onto the segment, not its extension",
    sketch: {
      entities: [
        pt("p1", -40, -10, true),
        pt("p2", 20, 20, true),
        ln("l1", "p1", "p2"),
        pt("p3", 60, 50, false),
      ],
      constraints: [k("k1", "on_line", ["p3", "l1"])],
    },
  },
  {
    id: "segment-distance",
    label: "Distance past segment end",
    description:
      "Past a line's end, point-to-line distance is measured to the nearest endpoint",
    sketch: {
      entities: [
        pt("p1", -40, 0, true),
        pt("p2", 0, 0, true),
        ln("l1", "p1", "p2"),
        pt("p3", 40, 15, false),
      ],
      constraints: [
        k("k1", "distance_point_line", ["p3", "l1"], { value: 25 }),
      ],
    },
  },
  {
    id: "segment-tangent",
    label: "Tangent within segment",
    description:
      "A circle beyond a line's end slides back so the tangency point lies on the segment",
    sketch: {
      entities: [
        pt("p1", -50, 0, true),
        pt("p2", 10, 0, true),
        ln("l1", "p1", "p2"),
        pt("p3", 45, 30, false),
        circ("c1", "p3", 15),
      ],
      constraints: [
        k("k1", "radius_circle", ["c1"], { value: 15 }),
        k("k2", "tangent_line_circle", ["l1", "c1"]),
      ],
    },
  },
  {
    id: "segment-midpoint",
    label: "Midpoint on segment",
    description:
      "A horizontal line whose midpoint must land on a diagonal segment, not its extension",
    sketch: {
      entities: [
        pt("p1", -20, -20, true),
        pt("p2", 20, 20, true),
        ln("l1", "p1", "p2"),
        pt("p3", 40, -30, false),
        pt("p4", 80, -10, false),
        ln("l2", "p3", "p4"),
      ],
      constraints: [
        k("k1", "midpoint_on_line", ["l2", "l1"]),
        k("k2", "horizontal_line", ["l2"]),
        k("k3", "distance_points", ["p3", "p4"], { value: 40 }),
      ],
    },
  },
  {
    id: "elliptical-arc",
    label: "Elliptical arc",
    description:
      "A fixed elliptical arc, as a projected edge (center, focus at 30 so a = 50, radmin 40): a line runs tangent off one end, an arc off the other, and a point is held on its span",
    sketch: {
      entities: [
        pt("ec", 0, 0, true),
        pt("ef", 30, 0, true),
        pt("es", 50 * Math.cos(20 * DEG), 40 * Math.sin(20 * DEG), true),
        pt("ee", 50 * Math.cos(150 * DEG), 40 * Math.sin(150 * DEG), true),
        earc("ea1", "ec", "ef", "es", "ee", 40, 20 * DEG, 150 * DEG, true),
        pt("q", -70, -10),
        ln("l1", "ee", "q"),
        pt("ac", 40, 5),
        pt("ae", 60, -10),
        arc("a1", "ac", "es", "ae", 15, 60 * DEG, 330 * DEG),
        pt("m", 10, 45),
      ],
      constraints: [
        k("k1", "tangent_line_elliptical_arc", ["l1", "ea1"]),
        k("k2", "length_line", ["l1"], { value: 35 }),
        k("k3", "tangent_arc_elliptical_arc", ["a1", "ea1"]),
        k("k4", "radius_arc", ["a1"], { value: 15 }),
        k("k5", "on_elliptical_arc", ["m", "ea1"]),
      ],
    },
  },
  {
    id: "spline",
    label: "Spline",
    description:
      "A fit-point spline between two fixed ends: a line leaves its right end along its tangent, a control-point spline continues its left end smoothly, a circle of radius 12 rests on it and a point is held on it. Drag the fit points to reshape it",
    sketch: {
      entities: [
        pt("s0", -80, 0, true),
        pt("s1", -40, 35),
        pt("s2", 0, 10),
        pt("s3", 40, 40),
        pt("s4", 80, 0, true),
        spl("sp1", ["s0", "s1", "s2", "s3", "s4"]),
        pt("q", 120, -30),
        ln("l1", "s4", "q"),
        pt("c1", -105, -35),
        pt("c2", -130, 10),
        pt("c3", -160, -20, true),
        spl("sp2", ["s0", "c1", "c2", "c3"], false),
        pt("cc", 0, 45),
        circ("circ", "cc", 12),
        pt("m", 20, 15),
      ],
      constraints: [
        k("k1", "tangent_line_spline", ["l1", "sp1"]),
        k("k2", "length_line", ["l1"], { value: 45 }),
        k("k3", "tangent_splines", ["sp1", "sp2"]),
        k("k4", "tangent_circle_spline", ["circ", "sp1"]),
        k("k5", "radius_circle", ["circ"], { value: 12 }),
        k("k6", "on_spline", ["m", "sp1"]),
      ],
    },
  },
];
