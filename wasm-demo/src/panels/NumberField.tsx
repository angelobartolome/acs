import { useEffect, useState } from "react";

interface Props {
  label: string;
  value: number;
  onCommit: (value: number) => void;
  step?: number;
  disabled?: boolean;
}

/** Numeric input that commits on Enter/blur (keeps typing smooth). */
export function NumberField({ label, value, onCommit, step, disabled }: Props) {
  const [text, setText] = useState<string>(String(round(value)));
  const [focused, setFocused] = useState(false);

  useEffect(() => {
    if (!focused) setText(String(round(value)));
  }, [value, focused]);

  const commit = () => {
    const n = Number(text);
    if (Number.isFinite(n) && n !== value) onCommit(n);
  };

  return (
    <label className="flex items-center gap-2 text-xs">
      <span className="w-14 shrink-0 text-slate-400">{label}</span>
      <input
        type="number"
        className="w-full rounded border border-slate-600 bg-slate-800 px-2 py-1 text-slate-200 focus:border-sky-500 focus:outline-none disabled:opacity-50"
        value={text}
        step={step ?? 0.1}
        disabled={disabled === true}
        onFocus={() => setFocused(true)}
        onBlur={() => {
          setFocused(false);
          commit();
        }}
        onChange={(e) => setText(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter") {
            commit();
            e.currentTarget.blur();
          }
          e.stopPropagation();
        }}
      />
    </label>
  );
}

function round(v: number): number {
  return Math.round(v * 10000) / 10000;
}
