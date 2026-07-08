import type { ToolId } from "../core/sketch/store";
import { selectTool } from "./SelectTool";
import {
  addArcTool,
  addCircleTool,
  addLineTool,
  addPointTool,
  panTool,
} from "./geometryTools";
import type { ITool } from "./types";

export const TOOLS: readonly ITool[] = [
  selectTool,
  addPointTool,
  addLineTool,
  addCircleTool,
  addArcTool,
  panTool,
];

const BY_ID = new Map<ToolId, ITool>(TOOLS.map((t) => [t.id, t]));

export function getTool(id: ToolId): ITool {
  const tool = BY_ID.get(id);
  if (tool === undefined) throw new Error(`Unknown tool: ${id}`);
  return tool;
}

export type { ITool, ToolPointerEvent } from "./types";
