import { isArc, isCircle, isLine, isPoint } from "../core/model/types";
import { useSketchStore } from "../hooks/useSketchStore";
import { NumberField } from "./NumberField";

const RAD = Math.PI / 180;

/** Properties of the (single) selected entity, with editable values. */
export function InspectorPanel() {
  const selectedEntities = useSketchStore((s) => s.selectedEntities);
  const updateEntityValues = useSketchStore((s) => s.updateEntityValues);
  const setEntityFixed = useSketchStore((s) => s.setEntityFixed);
  const solveNow = useSketchStore((s) => s.solveNow);
  const getEntity = useSketchStore((s) => s.getEntity);
  // subscribe to entity data so edits re-render
  useSketchStore((s) => s.entities);
  useSketchStore((s) => s.selection);

  const selected = selectedEntities();
  if (selected.length !== 1) {
    return (
      <div className="px-3 py-2 text-xs text-slate-500">
        {selected.length === 0
          ? "Select an entity to inspect"
          : `${selected.length} entities selected`}
      </div>
    );
  }

  const e = selected[0];
  const commit = (
    patch: Partial<{
      x: number;
      y: number;
      radius: number;
      startAngle: number;
      endAngle: number;
    }>,
  ) => {
    updateEntityValues(e.id, patch);
    solveNow();
  };

  return (
    <div className="flex flex-col gap-2 px-3 py-2">
      <div className="flex items-center gap-2 text-xs">
        <span className="font-medium text-slate-200">{e.kind}</span>
        <span className="text-slate-500">{e.id}</span>
        {!isLine(e) && (
          <label className="ml-auto flex items-center gap-1 text-slate-300">
            <input
              type="checkbox"
              checked={e.fixed}
              onChange={(ev) => {
                setEntityFixed(e.id, ev.target.checked);
                solveNow();
              }}
            />
            fixed
          </label>
        )}
      </div>
      {isPoint(e) && (
        <>
          <NumberField label="x" value={e.x} onCommit={(n) => commit({ x: n })} />
          <NumberField label="y" value={e.y} onCommit={(n) => commit({ y: n })} />
        </>
      )}
      {isLine(e) && (
        <div className="text-xs text-slate-400">
          {e.p1} {"\u2192"} {e.p2}
          {(() => {
            const p1 = getEntity(e.p1);
            const p2 = getEntity(e.p2);
            if (
              p1 !== undefined &&
              isPoint(p1) &&
              p2 !== undefined &&
              isPoint(p2)
            ) {
              const len = Math.hypot(p2.x - p1.x, p2.y - p1.y);
              return <div>length: {len.toFixed(3)}</div>;
            }
            return null;
          })()}
        </div>
      )}
      {isCircle(e) && (
        <>
          <NumberField
            label="radius"
            value={e.radius}
            onCommit={(n) => commit({ radius: n })}
          />
          <div className="text-xs text-slate-500">center: {e.center}</div>
        </>
      )}
      {isArc(e) && (
        <>
          <NumberField
            label="radius"
            value={e.radius}
            onCommit={(n) => commit({ radius: n })}
          />
          <NumberField
            label={"start \u00B0"}
            value={e.startAngle / RAD}
            onCommit={(n) => commit({ startAngle: n * RAD })}
          />
          <NumberField
            label={"end \u00B0"}
            value={e.endAngle / RAD}
            onCommit={(n) => commit({ endAngle: n * RAD })}
          />
          <div className="text-xs text-slate-500">center: {e.center}</div>
        </>
      )}
    </div>
  );
}
