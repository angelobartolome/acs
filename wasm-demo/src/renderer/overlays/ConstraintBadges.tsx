import { useMemo } from "react";

import { worldToScreen } from "../../core/sketch/store";
import { useSketchStore } from "../../hooks/useSketchStore";
import { badgeText, constraintAnchor, skippedIdSet } from "./anchors";
import { COLORS } from "../theme";

/**
 * Small per-constraint badges near the involved entities.
 * Hovering a badge highlights the constrained entities.
 */
export function ConstraintBadges() {
  const constraints = useSketchStore((s) => s.constraints);
  const entities = useSketchStore((s) => s.entities);
  const viewport = useSketchStore((s) => s.viewport);
  const highlightId = useSketchStore((s) => s.highlightConstraintId);
  const setHighlight = useSketchStore((s) => s.setHighlightConstraint);
  const lastOutcome = useSketchStore((s) => s.lastOutcome);

  const troubled = useMemo(() => {
    if (lastOutcome === null) return new Set<string>();
    return new Set([
      ...skippedIdSet(lastOutcome.skippedConstraintIds),
      ...lastOutcome.conflictingConstraintIds,
    ]);
  }, [lastOutcome]);

  const resolve = useMemo(() => {
    const byId = new Map(entities.map((e) => [e.id, e]));
    return (id: string) => byId.get(id);
  }, [entities]);

  // group badges by shared (rounded) anchor so they stack horizontally
  const placed = useMemo(() => {
    const seen = new Map<string, number>();
    const out: {
      id: string;
      text: string;
      x: number;
      y: number;
      bad: boolean;
    }[] = [];
    for (const c of constraints) {
      const anchor = constraintAnchor(c, resolve);
      if (anchor === null) continue;
      const s = worldToScreen(viewport, anchor);
      const key = `${Math.round(s.x / 24)}:${Math.round(s.y / 24)}`;
      const idx = seen.get(key) ?? 0;
      seen.set(key, idx + 1);
      out.push({
        id: c.id,
        text: badgeText(c.type),
        x: s.x + 10 + idx * 26,
        y: s.y - 14,
        bad: troubled.has(c.id),
      });
    }
    return out;
  }, [constraints, resolve, viewport, troubled]);

  return (
    <g>
      {placed.map((b) => (
        <g
          key={b.id}
          onPointerEnter={() => setHighlight(b.id)}
          onPointerLeave={() => setHighlight(null)}
          style={{ cursor: "default" }}
        >
          <rect
            x={b.x - 2}
            y={b.y - 10}
            width={Math.max(18, b.text.length * 8 + 6)}
            height={14}
            rx={3}
            fill={highlightId === b.id ? "#312e81" : "#1e293b"}
            stroke={b.bad ? COLORS.fixed : highlightId === b.id ? COLORS.highlight : "#475569"}
            strokeWidth={1}
          />
          <text
            x={b.x + 1}
            y={b.y + 1}
            fontSize={10}
            fill={b.bad ? COLORS.fixed : "#cbd5e1"}
            style={{ userSelect: "none" }}
          >
            {b.text}
          </text>
        </g>
      ))}
    </g>
  );
}
