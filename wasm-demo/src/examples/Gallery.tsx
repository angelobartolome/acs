import { useSketchStore } from "../hooks/useSketchStore";
import { EXAMPLES } from "./index";

/** Preset sketch gallery — loading replaces the current sketch. */
export function Gallery() {
  const loadSketch = useSketchStore((s) => s.loadSketch);

  return (
    <div className="flex flex-wrap items-center gap-1">
      {EXAMPLES.map((ex) => (
        <button
          key={ex.id}
          type="button"
          title={ex.description}
          className="rounded bg-slate-800 px-2 py-1 text-xs text-slate-300 hover:bg-sky-700 hover:text-white"
          onClick={() => loadSketch(structuredClone(ex.sketch))}
        >
          {ex.label}
        </button>
      ))}
    </div>
  );
}
