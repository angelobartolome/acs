import type { EntityId } from "../../core/model/types";
import { strokeFor, type GlyphState } from "../theme";

interface Props {
  id: EntityId;
  /** polyline points in screen coords, e.g. "x1,y1 x2,y2 ..." */
  points: string;
  state: GlyphState;
  onHover: (id: EntityId | null) => void;
}

/**
 * Sample an arc (world coords, CCW, y-up) into screen-space polyline points.
 */
export function sampleArcScreenPoints(
  toScreen: (p: { x: number; y: number }) => { x: number; y: number },
  cx: number,
  cy: number,
  radius: number,
  startAngle: number,
  endAngle: number,
  segments = 64,
): string {
  let span = endAngle - startAngle;
  while (span <= 0) span += Math.PI * 2;
  const pts: string[] = [];
  for (let i = 0; i <= segments; i++) {
    const a = startAngle + (span * i) / segments;
    const s = toScreen({
      x: cx + radius * Math.cos(a),
      y: cy + radius * Math.sin(a),
    });
    pts.push(`${s.x},${s.y}`);
  }
  return pts.join(" ");
}

export function ArcGlyph({ id, points, state, onHover }: Props) {
  return (
    <g
      onPointerEnter={() => onHover(id)}
      onPointerLeave={() => onHover(null)}
      data-entity={id}
    >
      <polyline points={points} fill="none" stroke="transparent" strokeWidth={10} />
      <polyline
        points={points}
        fill="none"
        stroke={strokeFor(state)}
        strokeWidth={state === "normal" ? 1.5 : 2.5}
      />
    </g>
  );
}
