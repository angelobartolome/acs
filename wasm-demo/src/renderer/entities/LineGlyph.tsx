import type { EntityId } from "../../core/model/types";
import { strokeFor, type GlyphState } from "../theme";

interface Props {
  id: EntityId;
  x1: number;
  y1: number;
  x2: number;
  y2: number;
  state: GlyphState;
  onHover: (id: EntityId | null) => void;
}

export function LineGlyph({ id, x1, y1, x2, y2, state, onHover }: Props) {
  return (
    <g
      onPointerEnter={() => onHover(id)}
      onPointerLeave={() => onHover(null)}
      data-entity={id}
    >
      {/* hit area */}
      <line x1={x1} y1={y1} x2={x2} y2={y2} stroke="transparent" strokeWidth={10} />
      <line
        x1={x1}
        y1={y1}
        x2={x2}
        y2={y2}
        stroke={strokeFor(state)}
        strokeWidth={state === "normal" ? 1.5 : 2.5}
      />
    </g>
  );
}
