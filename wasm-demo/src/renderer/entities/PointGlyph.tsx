import type { EntityId } from "../../core/model/types";
import { strokeForEntity, type GlyphState } from "../theme";
import { COLORS } from "../theme";

interface Props {
  id: EntityId;
  sx: number;
  sy: number;
  fixed: boolean;
  constrained: boolean;
  state: GlyphState;
  onHover: (id: EntityId | null) => void;
}

export function PointGlyph({ id, sx, sy, fixed, constrained, state, onHover }: Props) {
  // Fully-constrained green overrides the fixed red tint (product decision).
  const color =
    state === "normal" && !constrained && fixed
      ? COLORS.fixed
      : strokeForEntity(state, constrained);
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
