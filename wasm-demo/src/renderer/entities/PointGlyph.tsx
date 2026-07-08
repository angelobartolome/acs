import type { EntityId } from "../../core/model/types";
import { strokeFor, type GlyphState } from "../theme";
import { COLORS } from "../theme";

interface Props {
  id: EntityId;
  sx: number;
  sy: number;
  fixed: boolean;
  state: GlyphState;
  onHover: (id: EntityId | null) => void;
}

export function PointGlyph({ id, sx, sy, fixed, state, onHover }: Props) {
  const color = fixed && state === "normal" ? COLORS.fixed : strokeFor(state);
  const r = state === "selected" || state === "hover" ? 5 : 4;
  return (
    <g
      onPointerEnter={() => onHover(id)}
      onPointerLeave={() => onHover(null)}
      data-entity={id}
    >
      {/* hit area */}
      <circle cx={sx} cy={sy} r={10} fill="transparent" stroke="none" />
      <circle cx={sx} cy={sy} r={r} fill={color} stroke="#0f172a" strokeWidth={1} />
      {fixed && (
        <text
          x={sx + 6}
          y={sy - 6}
          fontSize={10}
          fill={COLORS.fixed}
          style={{ userSelect: "none", pointerEvents: "none" }}
        >
          {"\u{1F4CC}"}
        </text>
      )}
    </g>
  );
}
