/**
 * Sketch store — zustand, framework-agnostic (no React imports).
 *
 * Owns entities, constraints, selection, viewport and the solve loop.
 * Solved positions are fed back into the store so the solver's pre-solver
 * can skip already-satisfied components on subsequent (drag) solves.
 */

import { createStore } from "zustand/vanilla";

import type {
  ConstraintInstance,
  EntityId,
  Sketch,
  SketchEntity,
} from "../model/types";
import { isArc, isCircle, isLine, isPoint } from "../model/types";
import type { ISolverService, SolveOutcome } from "../solver/SolverService";

// ---------------------------------------------------------------------------
// viewport
// ---------------------------------------------------------------------------

export interface Viewport {
  /** screen x of world origin */
  offsetX: number;
  /** screen y of world origin */
  offsetY: number;
  /** pixels per world unit */
  scale: number;
}

export interface Vec2 {
  x: number;
  y: number;
}

export function worldToScreen(vp: Viewport, p: Vec2): Vec2 {
  return { x: p.x * vp.scale + vp.offsetX, y: -p.y * vp.scale + vp.offsetY };
}

export function screenToWorld(vp: Viewport, p: Vec2): Vec2 {
  return { x: (p.x - vp.offsetX) / vp.scale, y: -(p.y - vp.offsetY) / vp.scale };
}

// ---------------------------------------------------------------------------
// tool + draft state
// ---------------------------------------------------------------------------

export type ToolId = "select" | "point" | "line" | "circle" | "arc" | "pan";

export interface DraftClick {
  x: number;
  y: number;
  /** id of an existing point snapped onto, if any */
  snap: EntityId | null;
}

export interface ToolDraft {
  clicks: DraftClick[];
  cursor: Vec2 | null;
}

// ---------------------------------------------------------------------------
// solver service injection (set once after wasm init)
// ---------------------------------------------------------------------------

let solverService: ISolverService | null = null;

export function setSolverService(svc: ISolverService): void {
  solverService = svc;
}

const SOLVE_THROTTLE_MS = 30;
const HISTORY_LIMIT = 50;

let throttleTimer: ReturnType<typeof setTimeout> | null = null;
let lastSolveAt = 0;

// ---------------------------------------------------------------------------
// store
// ---------------------------------------------------------------------------

export interface SketchState {
  entities: SketchEntity[];
  constraints: ConstraintInstance[];
  selection: EntityId[];
  hoveredId: EntityId | null;
  /** constraint hovered in the list — its entities get highlighted */
  highlightConstraintId: string | null;
  activeTool: ToolId;
  draft: ToolDraft | null;
  viewport: Viewport;
  canvasSize: { width: number; height: number };
  lastOutcome: SolveOutcome | null;
  solveHistory: SolveOutcome[];

  // --- queries ---
  getEntity: (id: EntityId) => SketchEntity | undefined;
  selectedEntities: () => SketchEntity[];

  // --- ui ---
  setTool: (tool: ToolId) => void;
  setViewport: (vp: Viewport) => void;
  setCanvasSize: (width: number, height: number) => void;
  setHovered: (id: EntityId | null) => void;
  setHighlightConstraint: (id: string | null) => void;
  setDraft: (draft: ToolDraft | null) => void;
  selectEntity: (id: EntityId, additive: boolean) => void;
  clearSelection: () => void;
  zoomToFit: () => void;

  // --- entity CRUD ---
  addPoint: (x: number, y: number, fixed?: boolean) => EntityId;
  addLine: (p1: EntityId, p2: EntityId) => EntityId;
  addCircle: (center: EntityId, radius: number) => EntityId;
  addArc: (
    center: EntityId,
    radius: number,
    startAngle: number,
    endAngle: number,
  ) => EntityId;
  setPointPosition: (id: EntityId, x: number, y: number) => void;
  translateEntities: (ids: EntityId[], dx: number, dy: number) => void;
  setEntityFixed: (id: EntityId, fixed: boolean) => void;
  updateEntityValues: (
    id: EntityId,
    patch: Partial<{
      x: number;
      y: number;
      radius: number;
      startAngle: number;
      endAngle: number;
    }>,
  ) => void;
  deleteEntities: (ids: EntityId[]) => void;
  deleteSelection: () => void;

  // --- constraints ---
  addConstraint: (
    type: string,
    entities: EntityId[],
    params: Record<string, number>,
  ) => string;
  removeConstraint: (id: string) => void;
  updateConstraintParams: (id: string, params: Record<string, number>) => void;

  // --- sketch level ---
  loadSketch: (sketch: Sketch) => void;
  clearSketch: () => void;

  // --- solving ---
  solveNow: () => void;
  solveThrottled: () => void;
}

function nextId(
  prefix: string,
  entities: readonly SketchEntity[],
  constraints: readonly ConstraintInstance[],
): string {
  const re = new RegExp(`^${prefix}(\\d+)$`);
  let max = 0;
  const consider = (id: string): void => {
    const m = re.exec(id);
    if (m !== null) max = Math.max(max, Number(m[1]));
  };
  for (const e of entities) consider(e.id);
  for (const c of constraints) consider(c.id);
  return `${prefix}${max + 1}`;
}

export const sketchStore = createStore<SketchState>()((set, get) => ({
  entities: [],
  constraints: [],
  selection: [],
  hoveredId: null,
  highlightConstraintId: null,
  activeTool: "select",
  draft: null,
  viewport: { offsetX: 0, offsetY: 0, scale: 4 },
  canvasSize: { width: 0, height: 0 },
  lastOutcome: null,
  solveHistory: [],

  getEntity: (id) => get().entities.find((e) => e.id === id),
  selectedEntities: () => {
    const { entities, selection } = get();
    return selection
      .map((id) => entities.find((e) => e.id === id))
      .filter((e): e is SketchEntity => e !== undefined);
  },

  setTool: (tool) =>
    set({ activeTool: tool, draft: null, hoveredId: null }),
  setViewport: (vp) => set({ viewport: vp }),
  setCanvasSize: (width, height) =>
    set((s) => {
      // first measurement: center the origin
      if (s.canvasSize.width === 0 && s.canvasSize.height === 0) {
        return {
          canvasSize: { width, height },
          viewport: {
            offsetX: width / 2,
            offsetY: height / 2,
            scale: s.viewport.scale,
          },
        };
      }
      return { canvasSize: { width, height } };
    }),
  setHovered: (id) => set({ hoveredId: id }),
  setHighlightConstraint: (id) => set({ highlightConstraintId: id }),
  setDraft: (draft) => set({ draft }),

  selectEntity: (id, additive) =>
    set((s) => {
      if (additive) {
        return s.selection.includes(id)
          ? { selection: s.selection.filter((x) => x !== id) }
          : { selection: [...s.selection, id] };
      }
      return { selection: [id] };
    }),
  clearSelection: () => set({ selection: [] }),

  zoomToFit: () => {
    const { entities, canvasSize } = get();
    const pts = entities.filter(isPoint);
    if (pts.length === 0 || canvasSize.width === 0) return;
    let minX = Infinity;
    let minY = Infinity;
    let maxX = -Infinity;
    let maxY = -Infinity;
    const include = (x: number, y: number): void => {
      minX = Math.min(minX, x);
      minY = Math.min(minY, y);
      maxX = Math.max(maxX, x);
      maxY = Math.max(maxY, y);
    };
    for (const p of pts) include(p.x, p.y);
    for (const e of entities) {
      if (isCircle(e) || isArc(e)) {
        const c = get().getEntity(e.center);
        if (c !== undefined && isPoint(c)) {
          include(c.x - e.radius, c.y - e.radius);
          include(c.x + e.radius, c.y + e.radius);
        }
      }
    }
    const spanX = Math.max(maxX - minX, 1);
    const spanY = Math.max(maxY - minY, 1);
    const margin = 80;
    const scale = Math.min(
      (canvasSize.width - margin * 2) / spanX,
      (canvasSize.height - margin * 2) / spanY,
    );
    const clamped = Math.min(Math.max(scale, 0.01), 200);
    const cx = (minX + maxX) / 2;
    const cy = (minY + maxY) / 2;
    set({
      viewport: {
        scale: clamped,
        offsetX: canvasSize.width / 2 - cx * clamped,
        offsetY: canvasSize.height / 2 + cy * clamped,
      },
    });
  },

  addPoint: (x, y, fixed = false) => {
    const id = nextId("p", get().entities, get().constraints);
    set((s) => ({
      entities: [...s.entities, { kind: "point", id, x, y, fixed }],
    }));
    return id;
  },
  addLine: (p1, p2) => {
    const id = nextId("l", get().entities, get().constraints);
    set((s) => ({
      entities: [...s.entities, { kind: "line", id, p1, p2 }],
    }));
    return id;
  },
  addCircle: (center, radius) => {
    const id = nextId("c", get().entities, get().constraints);
    set((s) => ({
      entities: [
        ...s.entities,
        { kind: "circle", id, center, radius, fixed: false },
      ],
    }));
    return id;
  },
  addArc: (center, radius, startAngle, endAngle) => {
    const id = nextId("a", get().entities, get().constraints);
    set((s) => ({
      entities: [
        ...s.entities,
        { kind: "arc", id, center, radius, startAngle, endAngle, fixed: false },
      ],
    }));
    return id;
  },

  setPointPosition: (id, x, y) =>
    set((s) => ({
      entities: s.entities.map((e) =>
        e.id === id && isPoint(e) ? { ...e, x, y } : e,
      ),
    })),

  translateEntities: (ids, dx, dy) => {
    const { entities } = get();
    // resolve to the set of points that must move
    const pointIds = new Set<EntityId>();
    for (const id of ids) {
      const e = entities.find((x) => x.id === id);
      if (e === undefined) continue;
      if (isPoint(e)) pointIds.add(e.id);
      else if (isLine(e)) {
        pointIds.add(e.p1);
        pointIds.add(e.p2);
      } else pointIds.add(e.center);
    }
    set((s) => ({
      entities: s.entities.map((e) =>
        isPoint(e) && pointIds.has(e.id)
          ? { ...e, x: e.x + dx, y: e.y + dy }
          : e,
      ),
    }));
  },

  setEntityFixed: (id, fixed) =>
    set((s) => ({
      entities: s.entities.map((e) =>
        e.id === id && !isLine(e) ? { ...e, fixed } : e,
      ),
    })),

  updateEntityValues: (id, patch) =>
    set((s) => ({
      entities: s.entities.map((e) => {
        if (e.id !== id) return e;
        if (isPoint(e)) {
          return { ...e, x: patch.x ?? e.x, y: patch.y ?? e.y };
        }
        if (isCircle(e)) {
          return { ...e, radius: patch.radius ?? e.radius };
        }
        if (isArc(e)) {
          return {
            ...e,
            radius: patch.radius ?? e.radius,
            startAngle: patch.startAngle ?? e.startAngle,
            endAngle: patch.endAngle ?? e.endAngle,
          };
        }
        return e;
      }),
    })),

  deleteEntities: (ids) => {
    const doomed = new Set<EntityId>(ids);
    const { entities } = get();
    // cascade: entities depending on a doomed point die too
    let grew = true;
    while (grew) {
      grew = false;
      for (const e of entities) {
        if (doomed.has(e.id)) continue;
        const deps = isLine(e)
          ? [e.p1, e.p2]
          : isCircle(e) || isArc(e)
            ? [e.center]
            : [];
        if (deps.some((d) => doomed.has(d))) {
          doomed.add(e.id);
          grew = true;
        }
      }
    }
    set((s) => ({
      entities: s.entities.filter((e) => !doomed.has(e.id)),
      constraints: s.constraints.filter(
        (c) => !c.entities.some((eid) => doomed.has(eid)),
      ),
      selection: s.selection.filter((id) => !doomed.has(id)),
      hoveredId:
        s.hoveredId !== null && doomed.has(s.hoveredId) ? null : s.hoveredId,
    }));
    get().solveNow();
  },
  deleteSelection: () => {
    const sel = get().selection;
    if (sel.length > 0) get().deleteEntities(sel);
  },

  addConstraint: (type, entityIds, params) => {
    const id = nextId("k", get().entities, get().constraints);
    set((s) => ({
      constraints: [
        ...s.constraints,
        { id, type, entities: entityIds, params },
      ],
    }));
    get().solveNow();
    return id;
  },
  removeConstraint: (id) => {
    set((s) => ({
      constraints: s.constraints.filter((c) => c.id !== id),
      highlightConstraintId:
        s.highlightConstraintId === id ? null : s.highlightConstraintId,
    }));
    get().solveNow();
  },
  updateConstraintParams: (id, params) => {
    set((s) => ({
      constraints: s.constraints.map((c) =>
        c.id === id ? { ...c, params: { ...c.params, ...params } } : c,
      ),
    }));
    get().solveNow();
  },

  loadSketch: (sketch) => {
    set({
      entities: sketch.entities,
      constraints: sketch.constraints,
      selection: [],
      hoveredId: null,
      highlightConstraintId: null,
      draft: null,
      lastOutcome: null,
    });
    get().solveNow();
    get().zoomToFit();
  },
  clearSketch: () =>
    set({
      entities: [],
      constraints: [],
      selection: [],
      hoveredId: null,
      highlightConstraintId: null,
      draft: null,
      lastOutcome: null,
    }),

  solveNow: () => {
    if (solverService === null) return;
    const { entities, constraints } = get();
    if (entities.length === 0) {
      set({ lastOutcome: null });
      return;
    }
    const outcome = solverService.solve({ entities, constraints });
    set((s) => ({
      lastOutcome: outcome,
      solveHistory: [outcome, ...s.solveHistory].slice(0, HISTORY_LIMIT),
      // feed solved positions back only when converged — never silently
      // apply non-converged results
      entities: outcome.ok ? outcome.entities : s.entities,
    }));
  },

  solveThrottled: () => {
    const now = performance.now();
    if (now - lastSolveAt >= SOLVE_THROTTLE_MS) {
      lastSolveAt = now;
      get().solveNow();
      return;
    }
    if (throttleTimer === null) {
      throttleTimer = setTimeout(() => {
        throttleTimer = null;
        lastSolveAt = performance.now();
        get().solveNow();
      }, SOLVE_THROTTLE_MS);
    }
  },
}));
