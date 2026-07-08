import type { EntityId } from "../../core/model/types";
import { strokeForEntity, type GlyphState } from "../theme";

interface Props {
  id: EntityId;
  cx: number;
  cy: number;
  /** radius in screen pixels */
  r: number;
  constrained: boolean;
  state: GlyphState;
  onHover: (id: EntityId | null) => void;
}

export function CircleGlyph({ id, cx, cy, r, constrained, state, onHover }: Props) {
  return (
    <g
      onPointerEnter={() => onHover(id)}
      onPointerLeave={() => onHover(null)}
      data-entity={id}
    >
      {/* hit area: ring only, so the interior stays clickable for other entities */}
      <circle cx={cx} cy={cy} r={r} fill="none" stroke="transparent" strokeWidth={10} />
      <circle
        cx={cx}
        cy={cy}
        r={r}
        fill="none"
        stroke={strokeForEntity(state, constrained)}
        strokeWidth={state === "normal" ? 1.5 : 2.5}
      />
    </g>
  );
}
