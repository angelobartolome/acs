/** Shared glyph styling for the SVG renderer. */

export type GlyphState = "normal" | "hover" | "selected" | "highlight";

export const COLORS = {
  normal: "#94a3b8",
  hover: "#38bdf8",
  selected: "#f59e0b",
  highlight: "#a78bfa",
  fixed: "#f87171",
  constrained: "#10b981",
  draft: "#38bdf8",
  grid: "#1e293b",
  axis: "#334155",
} as const;

export function strokeFor(state: GlyphState): string {
  return COLORS[state];
}

/**
 * Stroke color for an entity, accounting for its fully-constrained state.
 *
 * Interaction feedback (selected/hover/highlight) always takes priority; the
 * green "fully constrained" (DOF = 0) tint only shows in the resting `normal`
 * state. Per product decision this green overrides the fixed red tint.
 */
export function strokeForEntity(state: GlyphState, constrained: boolean): string {
  if (state === "normal" && constrained) return COLORS.constrained;
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
