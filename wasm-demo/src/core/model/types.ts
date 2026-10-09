/**
 * Framework-agnostic sketch model. No React imports allowed in core/.
 *
 * The model mirrors the `acsSolveSketch` JSON API: points carry coordinates,
 * lines reference two point ids, circles reference a center point id, and
 * arcs reference center, start and end point ids; the
 * solver always keeps an arc's endpoints on it.
 */

export type EntityId = string;

export interface PointEntity {
  kind: "point";
  id: EntityId;
  x: number;
  y: number;
  fixed: boolean;
}

export interface LineEntity {
  kind: "line";
  id: EntityId;
  /** id of the start point */
  p1: EntityId;
  /** id of the end point */
  p2: EntityId;
}

export interface CircleEntity {
  kind: "circle";
  id: EntityId;
  /** id of the center point */
  center: EntityId;
  radius: number;
  fixed: boolean;
}

export interface ArcEntity {
  kind: "arc";
  id: EntityId;
  /** id of the center point */
  center: EntityId;
  /** id of the start point (at `startAngle`) */
  start: EntityId;
  /** id of the end point (at `endAngle`) */
  end: EntityId;
  radius: number;
  /** radians */
  startAngle: number;
  /** radians; the arc sweeps counter-clockwise from startAngle */
  endAngle: number;
  fixed: boolean;
}

/**
 * An arc of an ellipse (`elliptical_arc`): the ellipse's center, a focus
 * (which sets the major axis direction) and minor radius `radmin`, plus
 * start and end points at *parametric* angles, measured from the major axis
 * towards the minor one: the point at `t` is `c + a·cos t·u + radmin·sin t·n`
 * (see `ellipse.ts`). It sweeps counter-clockwise like an arc; the solver
 * keeps its endpoints on it.
 */
export interface EllipticalArcEntity {
  kind: "elliptical_arc";
  id: EntityId;
  /** id of the center point */
  center: EntityId;
  /** id of the focus point (center → focus is the major axis) */
  focus: EntityId;
  /** id of the start point (at `startAngle`) */
  start: EntityId;
  /** id of the end point (at `endAngle`) */
  end: EntityId;
  radmin: number;
  /** radians, parametric */
  startAngle: number;
  /** radians, parametric; sweeps counter-clockwise from startAngle */
  endAngle: number;
  fixed: boolean;
}

/** A spline's solved B-spline, as the solver's response reports it (`curve`). */
export interface SplineCurve {
  degree: number;
  knots: number[];
  controlPoints: [number, number][];
}

/**
 * A spline: a cubic B-spline whose handles are points, either fit points the
 * curve passes through (`interpolated`) or its control points. It has no
 * values of its own; the solver computes the curve from the handles and
 * reports it back (`curve`, absent until the first solve), which is what the
 * demo draws.
 */
export interface SplineEntity {
  kind: "spline";
  id: EntityId;
  /** handle point ids, in order along the curve */
  points: EntityId[];
  interpolated: boolean;
  curve?: SplineCurve;
}

export type SketchEntity =
  | PointEntity
  | LineEntity
  | CircleEntity
  | ArcEntity
  | EllipticalArcEntity
  | SplineEntity;
export type EntityKind = SketchEntity["kind"];

/** An applied constraint. `entities` is ordered per the registry selection spec. */
export interface ConstraintInstance {
  id: string;
  /**
   * key of its registry entry (a variant of a native type), e.g.
   * "distance_point_line" for a `distance` from a point to a line
   */
  def: string;
  entities: EntityId[];
  /** scalar params keyed by JSON field name (value/side/angle/distance/count) */
  params: Record<string, number>;
  /**
   * where its contacts sit along a spline (`on`/`tangent` with a spline), as
   * the last solve reported them (`curve_params`); sent back so the next
   * solve keeps each contact where it was instead of re-seeding it
   */
  curveParams?: number[];
}

export interface Sketch {
  entities: SketchEntity[];
  constraints: ConstraintInstance[];
}

/** JSON value used for primitives exchanged with the solver. */
export type JsonPrimitive = Record<string, string | number | boolean | string[] | number[]>;

export function isPoint(e: SketchEntity): e is PointEntity {
  return e.kind === "point";
}
export function isLine(e: SketchEntity): e is LineEntity {
  return e.kind === "line";
}
export function isCircle(e: SketchEntity): e is CircleEntity {
  return e.kind === "circle";
}
export function isArc(e: SketchEntity): e is ArcEntity {
  return e.kind === "arc";
}
export function isEllipticalArc(e: SketchEntity): e is EllipticalArcEntity {
  return e.kind === "elliptical_arc";
}
export function isSpline(e: SketchEntity): e is SplineEntity {
  return e.kind === "spline";
}
