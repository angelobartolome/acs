import { useSketchStore } from "../hooks/useSketchStore";
import { EXAMPLES } from "./index";

/** Preset sketch gallery — selecting an example replaces the current sketch. */
export function Gallery() {
  const loadSketch = useSketchStore((s) => s.loadSketch);

  return (
    <label className="flex items-center gap-1.5 text-xs text-slate-400">
      <span className="font-semibold tracking-wide uppercase">Examples</span>
      <select
        className="rounded border border-slate-700 bg-slate-800 px-2 py-1 text-xs text-slate-200 hover:bg-slate-700 focus:border-sky-600 focus:outline-none"
        defaultValue=""
        onChange={(e) => {
          const ex = EXAMPLES.find((x) => x.id === e.target.value);
          if (ex !== undefined) loadSketch(structuredClone(ex.sketch));
          e.target.selectedIndex = 0;
        }}
      >
        <option value="" disabled>
          Load an example…
        </option>
        {EXAMPLES.map((ex) => (
          <option key={ex.id} value={ex.id} title={ex.description}>
            {ex.label}
          </option>
        ))}
      </select>
    </label>
  );
}
