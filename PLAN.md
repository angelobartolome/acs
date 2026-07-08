# Plan: ACS Demo Rebuild — Developer + User Sketch Tool

**Approved decisions:** SVG renderer · `acsSolveSketch` JSON API · diagnostics + JSON inspector + import/export/gallery · minor Rust changes OK · new deps `zustand` + `vitest` approved.

## Phase 0 — Save plan + minor Rust enhancement

1. Write this plan to `PLAN.md` at repo root.
2. Extend `acsSolveSketch` response in `src/sketch_solve.rs` with additive `stats: { iterations, initial_error, final_error }` (already present in `SolverResult`, currently discarded at src/sketch_solve.rs:450). Add test assertion; run `cargo test` + `cargo clippy -- -D warnings`.
3. `wasm-pack build --target web --out-dir pkg` (pkg/ is currently missing; required by the demo).

## Phase 1 — New frontend architecture (`wasm-demo/src/`)

Remove `three`, `@react-three/*`, `@use-gesture/react`; add `zustand`, dev-dep `vitest`. Keep React 19 + Vite + Tailwind 4.

```
src/
├── core/                      # framework-agnostic, no React imports
│   ├── model/types.ts         # SketchEntity (point|line|circle|arc), ConstraintInstance, Sketch
│   ├── constraints/registry.ts# ConstraintRegistry — declarative metadata per constraint type (OCP)
│   ├── solver/SolverService.ts# ISolverService + AcsSolverService adapter over acsSolveSketch
│   └── sketch/store.ts        # zustand store: entities, constraints, selection, solve loop
├── tools/                     # ITool strategy: Select, AddPoint/Line/Circle/Arc, Pan
├── renderer/
│   ├── SketchCanvas.tsx       # SVG viewBox zoom/pan, world↔screen transform
│   ├── entities/              # Point/Line/Circle/Arc glyphs (hover/select/drag)
│   └── overlays/              # constraint badges, dimension labels
├── panels/
│   ├── Toolbar.tsx            # geometry tools, delete, fix/unfix
│   ├── ConstraintBar.tsx      # registry-driven; auto-enabled by selection signature
│   ├── ConstraintList.tsx     # hover-highlight, delete, skipped/conflict badges
│   ├── InspectorPanel.tsx     # entity props, fixed toggle, editable dimension values
│   ├── DiagnosticsPanel.tsx   # status, ms, iterations/errors, skipped ids, solve history
│   └── JsonPanel.tsx          # request/response viewer, copy, edit-raw-JSON + apply
├── examples/                  # preset sketches + gallery UI
└── App.tsx                    # thin layout shell
```

Key design points:

- **ConstraintRegistry** as extensibility core: each of the 26 JSON constraint types is a data entry `{ type, label, selection: [{kind, count}], scalarParams?, toPrimitive() }`. UI buttons, validation, serialization all derive from it — adding constraint #27 = one entry.
- **SolverService** feeds solved positions back into the store so the pre-solver actually skips satisfied components during drags (per USAGE.md).
- Non-converged solves surfaced in UI, never silently applied; `fixed` flags preserved end-to-end (pin glyph).

## Phase 2 — User-tool feature completeness

- Runtime create/delete for points, lines, circles, arcs; fix/unfix.
- All 26 constraints; scalar params (distance/angle/radius/coords) via numeric popover; editable afterward → re-solve.
- Click / shift-click selection, Escape clears, Delete removes; drag → throttled solve; wheel zoom + pan.

## Phase 3 — Developer-tool features

- Diagnostics panel (uses new `stats` field).
- JSON inspector with copy + paste/edit/apply raw sketch.
- Import/export sketch JSON (same format as API).
- Gallery: constrained square, tangent line+circle, concentric/equal-radius, symmetry about line, midpoint/angle dims, arc with `arc_radius`.

## Phase 4 — Verification

- Rust: `cargo test`, `cargo clippy -- -D warnings`.
- Frontend: `npm install`, `npm run build` (strict tsc, zero `any`), vitest units for ConstraintRegistry + SolverService payload building, manual smoke via `npm run dev`.

**Out of scope:** undo/redo, pre-solver component visualization, extending the dev wasm class (`src/bindings/solver.rs` left untouched, demo stops importing it).
