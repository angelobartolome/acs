import { useSketchStore } from "../hooks/useSketchStore";

function StatusPill({ ok }: { ok: boolean }) {
  return (
    <span
      className={`rounded px-2 py-0.5 text-[10px] font-semibold ${
        ok ? "bg-emerald-900 text-emerald-300" : "bg-rose-900 text-rose-300"
      }`}
    >
      {ok ? "converged" : "failed"}
    </span>
  );
}

function fmtErr(v: number): string {
  if (v === 0) return "0";
  return v.toExponential(2);
}

/** Solver status, timings, iteration stats and solve history. */
export function DiagnosticsPanel() {
  const lastOutcome = useSketchStore((s) => s.lastOutcome);
  const history = useSketchStore((s) => s.solveHistory);

  if (lastOutcome === null) {
    return (
      <div className="px-3 py-2 text-xs text-slate-500">No solve yet</div>
    );
  }

  return (
    <div className="flex flex-col gap-2 px-3 py-2 text-xs">
      <div className="flex items-center gap-2">
        <StatusPill ok={lastOutcome.ok} />
        <span className="text-slate-400">
          {lastOutcome.durationMs.toFixed(2)} ms
        </span>
      </div>
      {lastOutcome.error !== null && (
        <div className="rounded bg-rose-950 px-2 py-1 text-rose-300">
          {lastOutcome.error}
        </div>
      )}
      {lastOutcome.stats !== null && (
        <div className="grid grid-cols-2 gap-x-2 gap-y-1 text-slate-300">
          <span className="text-slate-500">iterations</span>
          <span>{lastOutcome.stats.iterations}</span>
          <span className="text-slate-500">initial error</span>
          <span>{fmtErr(lastOutcome.stats.initialError)}</span>
          <span className="text-slate-500">final error</span>
          <span>{fmtErr(lastOutcome.stats.finalError)}</span>
        </div>
      )}
      {lastOutcome.skippedConstraintIds.length > 0 && (
        <div>
          <div className="text-slate-500">skipped constraints</div>
          {lastOutcome.skippedConstraintIds.map((s) => (
            <div key={s} className="text-amber-300">
              {s}
            </div>
          ))}
        </div>
      )}
      {lastOutcome.conflictingConstraintIds.length > 0 && (
        <div>
          <div className="text-slate-500">conflicting constraints</div>
          {lastOutcome.conflictingConstraintIds.map((s) => (
            <div key={s} className="text-rose-300">
              {s}
            </div>
          ))}
        </div>
      )}
      <div>
        <div className="mb-1 text-slate-500">
          history (last {history.length})
        </div>
        <div className="max-h-32 overflow-y-auto rounded border border-slate-800">
          {history.map((h) => (
            <div
              key={h.timestamp + h.durationMs}
              className="flex items-center gap-2 border-b border-slate-800 px-2 py-0.5 last:border-b-0"
            >
              <span className={h.ok ? "text-emerald-400" : "text-rose-400"}>
                {h.ok ? "\u2713" : "\u2717"}
              </span>
              <span className="text-slate-400">
                {h.durationMs.toFixed(2)} ms
              </span>
              {h.stats !== null && (
                <span className="text-slate-500">
                  {h.stats.iterations} it · {fmtErr(h.stats.finalError)}
                </span>
              )}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
