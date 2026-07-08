import { useEffect, useRef, type ReactNode } from "react";

import { sketchStore } from "./core/sketch/store";
import {
  primitivesToSketch,
  sketchToPrimitives,
} from "./core/solver/SolverService";
import { Gallery } from "./examples/Gallery";
import { useSketchStore } from "./hooks/useSketchStore";
import { ConstraintBar } from "./panels/ConstraintBar";
import { ConstraintList } from "./panels/ConstraintList";
import { DiagnosticsPanel } from "./panels/DiagnosticsPanel";
import { InspectorPanel } from "./panels/InspectorPanel";
import { JsonPanel } from "./panels/JsonPanel";
import { Toolbar } from "./panels/Toolbar";
import { SketchCanvas } from "./renderer/SketchCanvas";
import { getTool } from "./tools";

function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <details open className="border-b border-slate-700">
      <summary className="cursor-pointer bg-slate-900 px-3 py-1.5 text-xs font-semibold tracking-wide text-slate-400 uppercase select-none">
        {title}
      </summary>
      {children}
    </details>
  );
}

function StatusBadge() {
  const lastOutcome = useSketchStore((s) => s.lastOutcome);
  if (lastOutcome === null) return null;
  return (
    <span
      className={`rounded px-2 py-0.5 text-xs font-semibold ${
        lastOutcome.ok
          ? "bg-emerald-900 text-emerald-300"
          : "bg-rose-900 text-rose-300"
      }`}
      title={lastOutcome.error ?? undefined}
    >
      {lastOutcome.ok ? "converged" : "not converged"}
    </span>
  );
}

export default function App() {
  const fileInput = useRef<HTMLInputElement | null>(null);

  // global keyboard shortcuts
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const target = e.target;
      if (
        target instanceof HTMLElement &&
        (target.tagName === "INPUT" || target.tagName === "TEXTAREA")
      ) {
        return;
      }
      const s = sketchStore.getState();
      if (e.key === "Escape") {
        getTool(s.activeTool).onCancel();
        s.setDraft(null);
        s.clearSelection();
      } else if (e.key === "Delete" || e.key === "Backspace") {
        s.deleteSelection();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  const exportSketch = () => {
    const { entities, constraints } = sketchStore.getState();
    const json = JSON.stringify(
      { primitives: sketchToPrimitives({ entities, constraints }) },
      null,
      2,
    );
    const blob = new Blob([json], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "sketch.json";
    a.click();
    URL.revokeObjectURL(url);
  };

  const importSketch = (file: File) => {
    void file.text().then((text) => {
      try {
        const root: unknown = JSON.parse(text);
        const prims = Array.isArray(root)
          ? root
          : typeof root === "object" &&
              root !== null &&
              Array.isArray((root as Record<string, unknown>).primitives)
            ? ((root as Record<string, unknown>).primitives as unknown[])
            : null;
        if (prims === null) {
          window.alert("Expected {\"primitives\": [...]} or a bare array");
          return;
        }
        const { sketch, warnings } = primitivesToSketch(prims);
        sketchStore.getState().loadSketch(sketch);
        if (warnings.length > 0) {
          window.alert(`Imported with warnings:\n${warnings.join("\n")}`);
        }
      } catch (err) {
        window.alert(`Invalid JSON: ${String(err)}`);
      }
    });
  };

  return (
    <div className="flex h-full flex-col">
      {/* top bar */}
      <header className="flex items-center gap-3 border-b border-slate-700 bg-slate-900 px-3 py-2">
        <h1 className="text-sm font-bold text-slate-100">ACS Sketch Demo</h1>
        <StatusBadge />
        <div className="mx-2 h-5 border-l border-slate-700" />
        <Gallery />
        <div className="ml-auto flex items-center gap-1">
          <button
            type="button"
            className="rounded bg-slate-800 px-2 py-1 text-xs text-slate-300 hover:bg-slate-700"
            onClick={() => fileInput.current?.click()}
          >
            Import
          </button>
          <button
            type="button"
            className="rounded bg-slate-800 px-2 py-1 text-xs text-slate-300 hover:bg-slate-700"
            onClick={exportSketch}
          >
            Export
          </button>
          <button
            type="button"
            className="rounded bg-slate-800 px-2 py-1 text-xs text-slate-300 hover:bg-rose-800"
            onClick={() => sketchStore.getState().clearSketch()}
          >
            Clear
          </button>
          <input
            ref={fileInput}
            type="file"
            accept=".json,application/json"
            className="hidden"
            onChange={(e) => {
              const f = e.target.files?.[0];
              if (f !== undefined) importSketch(f);
              e.target.value = "";
            }}
          />
        </div>
      </header>

      <ConstraintBar />

      <div className="flex min-h-0 flex-1">
        <Toolbar />
        <main className="min-w-0 flex-1">
          <SketchCanvas />
        </main>
        <aside className="w-80 shrink-0 overflow-y-auto border-l border-slate-700 bg-slate-900/60">
          <Section title="Inspector">
            <InspectorPanel />
          </Section>
          <Section title="Constraints">
            <ConstraintList />
          </Section>
          <Section title="Diagnostics">
            <DiagnosticsPanel />
          </Section>
          <Section title="JSON">
            <JsonPanel />
          </Section>
        </aside>
      </div>
    </div>
  );
}
