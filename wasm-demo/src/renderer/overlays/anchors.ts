import type {
  ConstraintInstance,
  EntityId,
  SketchEntity,
} from "../../core/model/types";
import { isArc, isCircle, isLine, isPoint } from "../../core/model/types";
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

/** Short badge text per constraint type (falls back to the first letters). */
const BADGES: Record<string, string> = {
  horizontal_pp: "H",
  vertical_pp: "V",
  horizontal_l: "H",
  vertical_l: "V",
  parallel: "\u2225",
  perpendicular_ll: "\u22A5",
  perpendicular_pppp: "\u22A5",
  p2p_coincident: "\u2261",
  point_on_line_pl: "\u22C5L",
  point_on_line_ppp: "\u22C5L",
  point_on_circle: "\u22C5O",
  p2p_distance: "\u2194",
  p2l_distance: "\u2194",
  l2l_angle_pppp: "\u2220",
  l2l_angle_ll: "\u2220",
  equal_length: "=",
  equal_radius_cc: "R=",
  equal_radius_aa: "R=",
  circle_radius: "R",
  arc_radius: "R",
  tangent_lc: "tan",
  midpoint_on_line_ll: "M",
  midpoint_on_line_pppp: "M",
  p2p_symmetric_ppp: "S",
  p2p_symmetric_ppl: "S",
  coordinate_x: "X",
  coordinate_y: "Y",
};

export function badgeText(type: string): string {
  return BADGES[type] ?? type.slice(0, 2).toUpperCase();
}

/** Extract constraint ids from skip entries shaped like "id:reason". */
export function skippedIdSet(skipped: readonly string[]): Set<string> {
  return new Set(skipped.map((s) => s.split(":")[0]));
}
