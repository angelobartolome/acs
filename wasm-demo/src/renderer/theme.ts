/** Shared glyph styling for the SVG renderer. */

export type GlyphState = "normal" | "hover" | "selected" | "highlight";

export const COLORS = {
  normal: "#94a3b8",
  hover: "#38bdf8",
  selected: "#f59e0b",
  highlight: "#a78bfa",
  fixed: "#f87171",
  draft: "#38bdf8",
  grid: "#1e293b",
  axis: "#334155",
} as const;

export function strokeFor(state: GlyphState): string {
  return COLORS[state];
}

export function glyphState(
  id: string,
  selection: readonly string[],
  hoveredId: string | null,
  highlighted: ReadonlySet<string>,
): GlyphState {
  if (selection.includes(id)) return "selected";
  if (hoveredId === id) return "hover";
  if (highlighted.has(id)) return "highlight";
  return "normal";
}
