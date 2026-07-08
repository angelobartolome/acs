/**
 * Geometry creation tools (point / line / circle / arc) + pan.
 *
 * Multi-click tools stage their clicks in the store `draft` so the renderer
 * can show previews; clicking an existing point snaps to it instead of
 * creating a duplicate.
 */

import type { EntityId } from "../core/model/types";
import { isPoint } from "../core/model/types";
import { sketchStore, type DraftClick, type Vec2 } from "../core/sketch/store";
import type { ITool, ToolPointerEvent } from "./types";

function snapPointId(hitId: EntityId | null): EntityId | null {
  if (hitId === null) return null;
  const e = sketchStore.getState().getEntity(hitId);
  return e !== undefined && isPoint(e) ? e.id : null;
}

function clickAt(e: ToolPointerEvent): DraftClick {
  const snap = snapPointId(e.hitId);
  if (snap !== null) {
    const p = sketchStore.getState().getEntity(snap);
    if (p !== undefined && isPoint(p)) {
      return { x: p.x, y: p.y, snap };
    }
  }
  return { x: e.world.x, y: e.world.y, snap: null };
}

function resolveOrCreatePoint(c: DraftClick): EntityId {
  if (c.snap !== null) return c.snap;
  return sketchStore.getState().addPoint(c.x, c.y);
}

function pushDraftClick(c: DraftClick): DraftClick[] {
  const s = sketchStore.getState();
  const clicks = [...(s.draft?.clicks ?? []), c];
  s.setDraft({ clicks, cursor: { x: c.x, y: c.y } });
  return clicks;
}

function updateCursor(world: Vec2): void {
  const s = sketchStore.getState();
  if (s.draft !== null) {
    s.setDraft({ ...s.draft, cursor: world });
  }
}

function clearDraft(): void {
  sketchStore.getState().setDraft(null);
}

export const addPointTool: ITool = {
  id: "point",
  label: "Point",
  icon: "\u2022",
  hint: "Click to place a point",
  cursor: "crosshair",
  onPointerDown(e) {
    if (e.button !== 0) return;
    sketchStore.getState().addPoint(e.world.x, e.world.y);
  },
  onPointerMove() {},
  onPointerUp() {},
  onCancel: clearDraft,
};

export const addLineTool: ITool = {
  id: "line",
  label: "Line",
  icon: "\u2571",
  hint: "Click start point, then end point (snaps to existing points)",
  cursor: "crosshair",
  onPointerDown(e) {
    if (e.button !== 0) return;
    const clicks = pushDraftClick(clickAt(e));
    if (clicks.length === 2) {
      const s = sketchStore.getState();
      const p1 = resolveOrCreatePoint(clicks[0]);
      const p2 = resolveOrCreatePoint(clicks[1]);
      if (p1 !== p2) s.addLine(p1, p2);
      clearDraft();
    }
  },
  onPointerMove(e) {
    updateCursor(e.world);
  },
  onPointerUp() {},
  onCancel: clearDraft,
};

export const addCircleTool: ITool = {
  id: "circle",
  label: "Circle",
  icon: "\u25CB",
  hint: "Click center, then a point on the circle",
  cursor: "crosshair",
  onPointerDown(e) {
    if (e.button !== 0) return;
    const clicks = pushDraftClick(clickAt(e));
    if (clicks.length === 2) {
      const s = sketchStore.getState();
      const [c0, c1] = clicks;
      const radius = Math.hypot(c1.x - c0.x, c1.y - c0.y);
      if (radius > 1e-9) {
        const center = resolveOrCreatePoint(c0);
        s.addCircle(center, radius);
      }
      clearDraft();
    }
  },
  onPointerMove(e) {
    updateCursor(e.world);
  },
  onPointerUp() {},
  onCancel: clearDraft,
};

export const addArcTool: ITool = {
  id: "arc",
  label: "Arc",
  icon: "\u25E0",
  hint: "Click center, then arc start, then arc end",
  cursor: "crosshair",
  onPointerDown(e) {
    if (e.button !== 0) return;
    const clicks = pushDraftClick(clickAt(e));
    if (clicks.length === 3) {
      const s = sketchStore.getState();
      const [c0, c1, c2] = clicks;
      const radius = Math.hypot(c1.x - c0.x, c1.y - c0.y);
      const startAngle = Math.atan2(c1.y - c0.y, c1.x - c0.x);
      const endAngle = Math.atan2(c2.y - c0.y, c2.x - c0.x);
      if (radius > 1e-9) {
        const center = resolveOrCreatePoint(c0);
        s.addArc(center, radius, startAngle, endAngle);
      }
      clearDraft();
    }
  },
  onPointerMove(e) {
    updateCursor(e.world);
  },
  onPointerUp() {},
  onCancel: clearDraft,
};

let panLast: Vec2 | null = null;

export const panTool: ITool = {
  id: "pan",
  label: "Pan",
  icon: "\u270B",
  hint: "Drag to pan the view",
  cursor: "grab",
  onPointerDown(e) {
    panLast = e.screen;
  },
  onPointerMove(e) {
    if (panLast === null) return;
    const s = sketchStore.getState();
    const vp = s.viewport;
    s.setViewport({
      ...vp,
      offsetX: vp.offsetX + (e.screen.x - panLast.x),
      offsetY: vp.offsetY + (e.screen.y - panLast.y),
    });
    panLast = e.screen;
  },
  onPointerUp() {
    panLast = null;
  },
  onCancel() {
    panLast = null;
  },
};
