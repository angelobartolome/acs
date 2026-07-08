/**
 * Framework-agnostic sketch model. No React imports allowed in core/.
 *
 * The model mirrors the `acsSolveSketch` JSON API: points carry coordinates,
 * lines reference two point ids, circles/arcs reference a center point id.
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
  radius: number;
  /** radians */
  startAngle: number;
  /** radians */
  endAngle: number;
  fixed: boolean;
}

export type SketchEntity = PointEntity | LineEntity | CircleEntity | ArcEntity;
export type EntityKind = SketchEntity["kind"];

/** An applied constraint. `entities` is ordered per the registry selection spec. */
export interface ConstraintInstance {
  id: string;
  /** JSON constraint type, e.g. "p2p_distance" */
  type: string;
  entities: EntityId[];
  /** scalar params keyed by JSON field name (distance/angle/radius/x/y) */
  params: Record<string, number>;
}

export interface Sketch {
  entities: SketchEntity[];
  constraints: ConstraintInstance[];
}

/** JSON value used for primitives exchanged with the solver. */
export type JsonPrimitive = Record<string, string | number | boolean>;

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
