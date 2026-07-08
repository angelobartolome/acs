import type { LineEntity, SketchEntity } from "../core/model/types";
import { useSketchStore } from "../hooks/useSketchStore";
import { TOOLS, getTool } from "../tools";

/** Left vertical toolbar: geometry tools + entity actions. */
export function Toolbar() {
  const activeTool = useSketchStore((s) => s.activeTool);
  const setTool = useSketchStore((s) => s.setTool);
  const selection = useSketchStore((s) => s.selection);
  const selectedEntities = useSketchStore((s) => s.selectedEntities);
  const setEntityFixed = useSketchStore((s) => s.setEntityFixed);
  const deleteSelection = useSketchStore((s) => s.deleteSelection);
  const solveNow = useSketchStore((s) => s.solveNow);
  const zoomToFit = useSketchStore((s) => s.zoomToFit);

  const fixables = selectedEntities().filter(
    (e): e is Exclude<SketchEntity, LineEntity> => e.kind !== "line",
  );
  const allFixed = fixables.length > 0 && fixables.every((e) => e.fixed);

  const toggleFixed = () => {
    for (const e of fixables) setEntityFixed(e.id, !allFixed);
    solveNow();
  };

  const btn =
    "flex h-10 w-10 items-center justify-center rounded text-lg transition-colors";

  return (
    <div className="flex flex-col gap-1 border-r border-slate-700 bg-slate-900 p-2">
      {TOOLS.map((t) => (
        <button
          key={t.id}
          type="button"
          title={`${t.label} — ${t.hint}`}
          className={`${btn} ${
            activeTool === t.id
              ? "bg-sky-600 text-white"
              : "bg-slate-800 text-slate-300 hover:bg-slate-700"
          }`}
          onClick={() => {
            getTool(activeTool).onCancel();
            setTool(t.id);
          }}
        >
          {t.icon}
        </button>
      ))}
      <div className="my-1 border-t border-slate-700" />
      <button
        type="button"
        title={allFixed ? "Unfix selection" : "Fix selection"}
        className={`${btn} ${
          allFixed ? "bg-rose-700 text-white" : "bg-slate-800 text-slate-300 hover:bg-slate-700"
        } disabled:opacity-30`}
        disabled={fixables.length === 0}
        onClick={toggleFixed}
      >
        {"\u{1F4CC}"}
      </button>
      <button
        type="button"
        title="Delete selection (Del)"
        className={`${btn} bg-slate-800 text-slate-300 hover:bg-rose-800 disabled:opacity-30`}
        disabled={selection.length === 0}
        onClick={deleteSelection}
      >
        {"\u2715"}
      </button>
      <button
        type="button"
        title="Zoom to fit"
        className={`${btn} bg-slate-800 text-slate-300 hover:bg-slate-700`}
        onClick={zoomToFit}
      >
        {"\u26F6"}
      </button>
    </div>
  );
}
