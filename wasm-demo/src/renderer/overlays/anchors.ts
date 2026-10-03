import type {
  ConstraintInstance,
  EntityId,
  SketchEntity,
} from "../../core/model/types";
import { isArc, isCircle, isLine, isPoint } from "../../core/model/types";
import { getConstraintDef } from "../../core/constraints/registry";
import type { Vec2 } from "../../core/sketch/store";

export type Resolver = (id: EntityId) => SketchEntity | undefined;

function entityAnchor(id: EntityId, resolve: Resolver): Vec2 | null {
  const e = resolve(id);
  if (e === undefined) return null;
  if (isPoint(e)) return { x: e.x, y: e.y };
  if (isLine(e)) {
    const p1 = resolve(e.p1);
    const p2 = resolve(e.p2);
    if (p1 !== undefined && isPoint(p1) && p2 !== undefined && isPoint(p2)) {
      return { x: (p1.x + p2.x) / 2, y: (p1.y + p2.y) / 2 };
    }
    return null;
  }
  if (isCircle(e) || isArc(e)) {
    const c = resolve(e.center);
    if (c !== undefined && isPoint(c)) return { x: c.x, y: c.y };
  }
  return null;
}

/** World-space anchor for a constraint: mean of its entity anchors. */
export function constraintAnchor(
  c: ConstraintInstance,
  resolve: Resolver,
): Vec2 | null {
  const anchors = c.entities
    .map((id) => entityAnchor(id, resolve))
    .filter((a): a is Vec2 => a !== null);
  if (anchors.length === 0) return null;
  return {
    x: anchors.reduce((s, a) => s + a.x, 0) / anchors.length,
    y: anchors.reduce((s, a) => s + a.y, 0) / anchors.length,
  };
}


export function badgeText(defKey: string): string {
  return getConstraintDef(defKey)?.badge ?? defKey.slice(0, 2).toUpperCase();
}

