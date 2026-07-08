import { useState } from "react";

import { sketchStore } from "../core/sketch/store";
import {
  primitivesToSketch,
  sketchToPrimitives,
} from "../core/solver/SolverService";
import { useSketchStore } from "../hooks/useSketchStore";

type Tab = "request" | "response" | "edit";

function pretty(json: string): string {
  try {
    return JSON.stringify(JSON.parse(json), null, 2);
  } catch {
    return json;
  }
}

function copyText(text: string): void {
  void navigator.clipboard.writeText(text);
}

/** Raw request/response viewer + paste/edit/apply for sketch JSON. */
export function JsonPanel() {
  const lastOutcome = useSketchStore((s) => s.lastOutcome);
  const loadSketch = useSketchStore((s) => s.loadSketch);

  const [tab, setTab] = useState<Tab>("request");
  const [editText, setEditText] = useState("");
  const [message, setMessage] = useState<string | null>(null);

  const currentSketchJson = () => {
    const { entities, constraints } = sketchStore.getState();
    return JSON.stringify(
      { primitives: sketchToPrimitives({ entities, constraints }) },
      null,
      2,
    );
  };

  const applyEdit = () => {
    try {
      const root: unknown = JSON.parse(editText);
      const prims = Array.isArray(root)
        ? root
        : typeof root === "object" &&
            root !== null &&
            Array.isArray((root as Record<string, unknown>).primitives)
          ? ((root as Record<string, unknown>).primitives as unknown[])
          : null;
      if (prims === null) {
        setMessage("Expected {\"primitives\": [...]} or a bare array");
        return;
      }
      const { sketch, warnings } = primitivesToSketch(prims);
      loadSketch(sketch);
      setMessage(
        warnings.length > 0
          ? `Applied with warnings: ${warnings.join("; ")}`
          : `Applied: ${sketch.entities.length} entities, ${sketch.constraints.length} constraints`,
      );
    } catch (err) {
      setMessage(`Invalid JSON: ${String(err)}`);
    }
  };

  const tabBtn = (t: Tab, label: string) => (
    <button
      type="button"
      className={`rounded-t px-2 py-1 text-xs ${
        tab === t
          ? "bg-slate-800 text-slate-100"
          : "bg-slate-900 text-slate-500 hover:text-slate-300"
      }`}
      onClick={() => {
        setTab(t);
        setMessage(null);
        if (t === "edit" && editText === "") setEditText(currentSketchJson());
      }}
    >
      {label}
    </button>
  );

  const viewer = (content: string) => (
    <div className="relative">
      <button
        type="button"
        className="absolute top-1 right-1 rounded bg-slate-700 px-2 py-0.5 text-[10px] text-slate-200 hover:bg-slate-600"
        onClick={() => copyText(pretty(content))}
      >
        copy
      </button>
      <pre className="max-h-64 overflow-auto rounded bg-slate-950 p-2 text-[10px] leading-tight text-slate-300">
        {content === "" ? "(empty)" : pretty(content)}
      </pre>
    </div>
  );

  return (
    <div className="flex flex-col gap-1 px-3 py-2">
      <div className="flex gap-1">
        {tabBtn("request", "request")}
        {tabBtn("response", "response")}
        {tabBtn("edit", "edit sketch")}
      </div>
      {tab === "request" && viewer(lastOutcome?.requestJson ?? "")}
      {tab === "response" && viewer(lastOutcome?.responseJson ?? "")}
      {tab === "edit" && (
        <div className="flex flex-col gap-1">
          <textarea
            className="h-48 w-full rounded border border-slate-700 bg-slate-950 p-2 font-mono text-[10px] leading-tight text-slate-300 focus:border-sky-500 focus:outline-none"
            value={editText}
            spellCheck={false}
            onChange={(e) => setEditText(e.target.value)}
            onKeyDown={(e) => e.stopPropagation()}
          />
          <div className="flex gap-2">
            <button
              type="button"
              className="rounded bg-slate-700 px-2 py-1 text-xs text-slate-200 hover:bg-slate-600"
              onClick={() => setEditText(currentSketchJson())}
            >
              Load current
            </button>
            <button
              type="button"
              className="rounded bg-sky-600 px-2 py-1 text-xs text-white hover:bg-sky-500"
              onClick={applyEdit}
            >
              Apply
            </button>
          </div>
        </div>
      )}
      {message !== null && (
        <div className="text-[10px] text-slate-400">{message}</div>
      )}
    </div>
  );
}
