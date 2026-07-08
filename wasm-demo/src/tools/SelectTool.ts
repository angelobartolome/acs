import type { EntityId } from "../core/model/types";
import { sketchStore, type Vec2 } from "../core/sketch/store";
import type { ITool, ToolPointerEvent } from "./types";

interface DragState {
  ids: EntityId[];
  last: Vec2;
  moved: boolean;
}

let drag: DragState | null = null;

export const selectTool: ITool = {
  id: "select",
  label: "Select",
  icon: "\u2196",
  hint: "Click to select, shift-click to multi-select, drag to move",
  cursor: "default",

  onPointerDown(e: ToolPointerEvent) {
    if (e.button !== 0) return;
    const s = sketchStore.getState();
    if (e.hitId === null) {
      s.clearSelection();
      return;
    }
    const alreadySelected = s.selection.includes(e.hitId);
    if (e.shiftKey) {
      s.selectEntity(e.hitId, true);
    } else if (!alreadySelected) {
      s.selectEntity(e.hitId, false);
    }
    const ids = sketchStore.getState().selection.includes(e.hitId)
      ? sketchStore.getState().selection
      : [e.hitId];
    drag = { ids: [...ids], last: e.world, moved: false };
  },

  onPointerMove(e: ToolPointerEvent) {
    if (drag === null) return;
    const dx = e.world.x - drag.last.x;
    const dy = e.world.y - drag.last.y;
    if (dx === 0 && dy === 0) return;
    drag.last = e.world;
    drag.moved = true;
    const s = sketchStore.getState();
    s.translateEntities(drag.ids, dx, dy);
    s.solveThrottled();
  },

  onPointerUp() {
    if (drag !== null && drag.moved) {
      sketchStore.getState().solveNow();
    }
    drag = null;
  },

  onCancel() {
    drag = null;
    sketchStore.getState().clearSelection();
  },
};
