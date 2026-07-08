import type { EntityId } from "../core/model/types";
import type { ToolId, Vec2 } from "../core/sketch/store";

/** Normalized pointer event passed to tools by the canvas. */
export interface ToolPointerEvent {
  world: Vec2;
  screen: Vec2;
  /** topmost entity under the cursor, if any */
  hitId: EntityId | null;
  shiftKey: boolean;
  /** 0 = left, 1 = middle, 2 = right */
  button: number;
}

/** Strategy interface — one implementation per toolbar tool. */
export interface ITool {
  id: ToolId;
  label: string;
  /** short glyph shown on the toolbar button */
  icon: string;
  hint: string;
  cursor: string;
  onPointerDown(e: ToolPointerEvent): void;
  onPointerMove(e: ToolPointerEvent): void;
  onPointerUp(e: ToolPointerEvent): void;
  /** Escape pressed or tool switched away */
  onCancel(): void;
}
