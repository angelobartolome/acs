import { useMemo } from "react";

import { getConstraintDef } from "../core/constraints/registry";
import {
  type ConstraintFlag,
  constraintFlags,
} from "../core/solver/SolverService";
import { useSketchStore } from "../hooks/useSketchStore";
import { NumberField } from "./NumberField";

const RAD = Math.PI / 180;

/** All constraints with hover-highlight, param editing and delete. */
export function ConstraintList() {
  const constraints = useSketchStore((s) => s.constraints);
  const lastOutcome = useSketchStore((s) => s.lastOutcome);
  const setHighlight = useSketchStore((s) => s.setHighlightConstraint);
  const removeConstraint = useSketchStore((s) => s.removeConstraint);
  const updateConstraintParams = useSketchStore((s) => s.updateConstraintParams);
  const highlightId = useSketchStore((s) => s.highlightConstraintId);

  const flags = useMemo(
    () =>
      lastOutcome !== null
        ? constraintFlags(lastOutcome)
        : new Map<string, ConstraintFlag>(),
    [lastOutcome],
  );

  if (constraints.length === 0) {
    return (
      <div className="px-3 py-2 text-xs text-slate-500">No constraints yet</div>
    );
  }

  return (
    <ul className="flex flex-col">
      {constraints.map((c) => {
        const def = getConstraintDef(c.def);
        return (
          <li
            key={c.id}
            className={`flex flex-col gap-1 border-b border-slate-800 px-3 py-2 text-xs ${
              highlightId === c.id ? "bg-slate-800" : ""
            }`}
            onPointerEnter={() => setHighlight(c.id)}
            onPointerLeave={() => setHighlight(null)}
          >
            <div className="flex items-center gap-2">
              <span className="font-medium text-slate-200">
                {def?.label ?? c.def}
              </span>
              <span className="text-slate-500">{c.id}</span>
              {flags.has(c.id) && (
                <span
                  className={`rounded px-1 text-[10px] ${
                    flags.get(c.id) === "redundant"
                      ? "bg-amber-900 text-amber-300"
                      : "bg-rose-900 text-rose-300"
                  }`}
                >
                  {flags.get(c.id)}
                </span>
              )}
              <button
                type="button"
                title="Delete constraint"
                className="ml-auto rounded px-1 text-slate-500 hover:bg-rose-900 hover:text-rose-200"
                onClick={() => removeConstraint(c.id)}
              >
                {"\u2715"}
              </button>
            </div>
            <div className="text-slate-500">{c.entities.join(", ")}</div>
            {(def?.scalarParams ?? []).map((p) => {
              const isAngle = p.unit === "angle";
              const value = c.params[p.key] ?? 0;
              return (
                <NumberField
                  key={p.key}
                  label={isAngle ? `${p.label} \u00B0` : p.label}
                  value={isAngle ? value / RAD : value}
                  onCommit={(n) =>
                    updateConstraintParams(c.id, {
                      [p.key]: isAngle ? n * RAD : n,
                    })
                  }
                />
              );
            })}
          </li>
        );
      })}
    </ul>
  );
}
