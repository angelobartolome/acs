import { useMemo } from "react";

import { getConstraintDef } from "../../core/constraints/registry";
import { worldToScreen } from "../../core/sketch/store";
import { useSketchStore } from "../../hooks/useSketchStore";
import { constraintAnchor } from "./anchors";
import { COLORS } from "../theme";

function formatParam(unit: string, value: number): string {
  if (unit === "angle") return `${((value * 180) / Math.PI).toFixed(1)}\u00B0`;
  return value.toFixed(2);
}

/** Dimension value labels for constraints that carry scalar params. */
export function DimensionLabels() {
  const constraints = useSketchStore((s) => s.constraints);
  const entities = useSketchStore((s) => s.entities);
  const viewport = useSketchStore((s) => s.viewport);
  const setHighlight = useSketchStore((s) => s.setHighlightConstraint);

  const resolve = useMemo(() => {
    const byId = new Map(entities.map((e) => [e.id, e]));
    return (id: string) => byId.get(id);
  }, [entities]);

  const labels = useMemo(() => {
    const out: { id: string; text: string; x: number; y: number }[] = [];
    for (const c of constraints) {
      const def = getConstraintDef(c.type);
      if (def === undefined || def.scalarParams === undefined) continue;
      const anchor = constraintAnchor(c, resolve);
      if (anchor === null) continue;
      const s = worldToScreen(viewport, anchor);
      const text = def.scalarParams
        .map((p) => formatParam(p.unit, c.params[p.key] ?? 0))
        .join(", ");
      out.push({ id: c.id, text, x: s.x + 10, y: s.y + 16 });
    }
    return out;
  }, [constraints, resolve, viewport]);

  return (
    <g>
      {labels.map((l) => (
        <text
          key={l.id}
          x={l.x}
          y={l.y}
          fontSize={11}
          fill={COLORS.selected}
          stroke="#0f172a"
          strokeWidth={3}
          paintOrder="stroke"
          style={{ userSelect: "none" }}
          onPointerEnter={() => setHighlight(l.id)}
          onPointerLeave={() => setHighlight(null)}
        >
          {l.text}
        </text>
      ))}
    </g>
  );
}
