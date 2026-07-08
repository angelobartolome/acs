import { useState } from "react";

import {
  applicableConstraints,
  defaultParams,
  matchSelection,
  type ConstraintDef,
} from "../core/constraints/registry";
import type { EntityId } from "../core/model/types";
import { useSketchStore } from "../hooks/useSketchStore";

interface PendingConstraint {
  def: ConstraintDef;
  slotEntities: EntityId[];
  params: Record<string, number>;
}

const RAD = Math.PI / 180;

/**
 * Registry-driven constraint bar: buttons enable automatically based on the
 * current selection signature. Constraints with scalar params open a small
 * value popover before being applied.
 */
export function ConstraintBar() {
  const selectedEntities = useSketchStore((s) => s.selectedEntities);
  const selection = useSketchStore((s) => s.selection);
  const getEntity = useSketchStore((s) => s.getEntity);
  const addConstraint = useSketchStore((s) => s.addConstraint);
  const clearSelection = useSketchStore((s) => s.clearSelection);

  const [pending, setPending] = useState<PendingConstraint | null>(null);

  const selected = selectedEntities();
  const defs = applicableConstraints(selected);

  const apply = (def: ConstraintDef) => {
    const slots = matchSelection(def, selected);
    if (slots === null) return;
    const slotEntities = slots
      .map((id) => getEntity(id))
      .filter((e): e is NonNullable<typeof e> => e !== undefined);
    const params = defaultParams(def, slotEntities, getEntity);
    if ((def.scalarParams ?? []).length > 0) {
      setPending({ def, slotEntities: slots, params });
      return;
    }
    addConstraint(def.type, slots, params);
    clearSelection();
  };

  const confirmPending = () => {
    if (pending === null) return;
    addConstraint(pending.def.type, pending.slotEntities, pending.params);
    setPending(null);
    clearSelection();
  };

  return (
    <div className="relative flex min-h-11 items-center gap-1 border-b border-slate-700 bg-slate-900 px-2 py-1">
      <span className="mr-2 text-xs text-slate-500">
        {selection.length === 0
          ? "Select entities to see applicable constraints"
          : `${selection.length} selected`}
      </span>
      {defs.map((def) => (
        <button
          key={def.type}
          type="button"
          title={def.description}
          className="rounded bg-slate-800 px-2 py-1 text-xs text-slate-200 hover:bg-sky-700"
          onClick={() => apply(def)}
        >
          {def.label}
        </button>
      ))}
      {pending !== null && (
        <div className="absolute top-full left-2 z-20 mt-1 flex flex-col gap-2 rounded border border-slate-600 bg-slate-800 p-3 shadow-xl">
          <div className="text-xs font-semibold text-slate-200">
            {pending.def.label}
          </div>
          {(pending.def.scalarParams ?? []).map((p) => {
            const raw = pending.params[p.key] ?? 0;
            const isAngle = p.unit === "angle";
            const shown = isAngle ? raw / RAD : raw;
            return (
              <label key={p.key} className="flex items-center gap-2 text-xs">
                <span className="w-16 text-slate-400">
                  {p.label}
                  {isAngle ? " (\u00B0)" : ""}
                </span>
                <input
                  type="number"
                  autoFocus
                  className="w-28 rounded border border-slate-600 bg-slate-900 px-2 py-1 text-slate-200 focus:border-sky-500 focus:outline-none"
                  defaultValue={Math.round(shown * 1000) / 1000}
                  onChange={(e) => {
                    const n = Number(e.target.value);
                    if (Number.isFinite(n)) {
                      setPending((prev) =>
                        prev === null
                          ? null
                          : {
                              ...prev,
                              params: {
                                ...prev.params,
                                [p.key]: isAngle ? n * RAD : n,
                              },
                            },
                      );
                    }
                  }}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") confirmPending();
                    if (e.key === "Escape") setPending(null);
                    e.stopPropagation();
                  }}
                />
              </label>
            );
          })}
          <div className="flex justify-end gap-2">
            <button
              type="button"
              className="rounded bg-slate-700 px-2 py-1 text-xs text-slate-300 hover:bg-slate-600"
              onClick={() => setPending(null)}
            >
              Cancel
            </button>
            <button
              type="button"
              className="rounded bg-sky-600 px-2 py-1 text-xs text-white hover:bg-sky-500"
              onClick={confirmPending}
            >
              Apply
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
