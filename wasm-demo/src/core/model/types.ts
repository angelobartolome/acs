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

export type SketchEntity = PointEntity | LineEntity | CircleEntity | ArcEntity;
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
}

export interface Sketch {
  entities: SketchEntity[];
  constraints: ConstraintInstance[];
}

/** JSON value used for primitives exchanged with the solver. */
export type JsonPrimitive = Record<string, string | number | boolean | string[]>;

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
