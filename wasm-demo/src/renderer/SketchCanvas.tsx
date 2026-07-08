import { useEffect, useMemo, useRef, type PointerEvent, type WheelEvent } from "react";

import { isArc, isCircle, isLine, isPoint } from "../core/model/types";
import type { PointEntity } from "../core/model/types";
import {
  screenToWorld,
  sketchStore,
  worldToScreen,
  type Vec2,
  type Viewport,
} from "../core/sketch/store";
import { useSketchStore } from "../hooks/useSketchStore";
import { getTool } from "../tools";
import type { ToolPointerEvent } from "../tools";
import { ArcGlyph, sampleArcScreenPoints } from "./entities/ArcGlyph";
import { CircleGlyph } from "./entities/CircleGlyph";
import { LineGlyph } from "./entities/LineGlyph";
import { PointGlyph } from "./entities/PointGlyph";
import { ConstraintBadges } from "./overlays/ConstraintBadges";
import { DimensionLabels } from "./overlays/DimensionLabels";
import { COLORS, glyphState } from "./theme";

const MIN_SCALE = 0.02;
const MAX_SCALE = 400;

function gridSpacing(scale: number): number {
  // world units per grid line so lines are 50..500 px apart
  return 10 ** Math.ceil(Math.log10(50 / scale));
}

function Grid({ viewport, width, height }: { viewport: Viewport; width: number; height: number }) {
  const spacing = gridSpacing(viewport.scale);
  const topLeft = screenToWorld(viewport, { x: 0, y: 0 });
  const bottomRight = screenToWorld(viewport, { x: width, y: height });
  const lines: React.ReactElement[] = [];
  const x0 = Math.floor(topLeft.x / spacing) * spacing;
  for (let x = x0; x <= bottomRight.x; x += spacing) {
    const s = worldToScreen(viewport, { x, y: 0 });
    lines.push(
      <line
        key={`v${x}`}
        x1={s.x}
        y1={0}
        x2={s.x}
        y2={height}
        stroke={Math.abs(x) < spacing / 2 ? COLORS.axis : COLORS.grid}
        strokeWidth={Math.abs(x) < spacing / 2 ? 1.5 : 1}
      />,
    );
  }
  const y0 = Math.floor(bottomRight.y / spacing) * spacing;
  for (let y = y0; y <= topLeft.y; y += spacing) {
    const s = worldToScreen(viewport, { x: 0, y });
    lines.push(
      <line
        key={`h${y}`}
        x1={0}
        y1={s.y}
        x2={width}
        y2={s.y}
        stroke={Math.abs(y) < spacing / 2 ? COLORS.axis : COLORS.grid}
        strokeWidth={Math.abs(y) < spacing / 2 ? 1.5 : 1}
      />,
    );
  }
  return <g>{lines}</g>;
}

function DraftPreview() {
  const draft = useSketchStore((s) => s.draft);
  const activeTool = useSketchStore((s) => s.activeTool);
  const viewport = useSketchStore((s) => s.viewport);
  if (draft === null || draft.clicks.length === 0) return null;

  const toScreen = (p: Vec2) => worldToScreen(viewport, p);
  const clicks = draft.clicks.map((c) => toScreen(c));
  const cursor = draft.cursor !== null ? toScreen(draft.cursor) : null;
  const first = draft.clicks[0];

  const shapes: React.ReactElement[] = clicks.map((c, i) => (
    <circle key={`d${i}`} cx={c.x} cy={c.y} r={3} fill={COLORS.draft} />
  ));

  if (cursor !== null && draft.cursor !== null) {
    if (activeTool === "line") {
      shapes.push(
        <line
          key="preview"
          x1={clicks[0].x}
          y1={clicks[0].y}
          x2={cursor.x}
          y2={cursor.y}
          stroke={COLORS.draft}
          strokeWidth={1.5}
          strokeDasharray="4 3"
        />,
      );
    } else if (activeTool === "circle") {
      const r = Math.hypot(draft.cursor.x - first.x, draft.cursor.y - first.y);
      shapes.push(
        <circle
          key="preview"
          cx={clicks[0].x}
          cy={clicks[0].y}
          r={r * viewport.scale}
          fill="none"
          stroke={COLORS.draft}
          strokeWidth={1.5}
          strokeDasharray="4 3"
        />,
      );
    } else if (activeTool === "arc") {
      if (draft.clicks.length === 1) {
        shapes.push(
          <line
            key="preview"
            x1={clicks[0].x}
            y1={clicks[0].y}
            x2={cursor.x}
            y2={cursor.y}
            stroke={COLORS.draft}
            strokeWidth={1.5}
            strokeDasharray="4 3"
          />,
        );
      } else if (draft.clicks.length === 2) {
        const start = draft.clicks[1];
        const radius = Math.hypot(start.x - first.x, start.y - first.y);
        const startAngle = Math.atan2(start.y - first.y, start.x - first.x);
        const endAngle = Math.atan2(
          draft.cursor.y - first.y,
          draft.cursor.x - first.x,
        );
        shapes.push(
          <polyline
            key="preview"
            points={sampleArcScreenPoints(
              toScreen,
              first.x,
              first.y,
              radius,
              startAngle,
              endAngle,
            )}
            fill="none"
            stroke={COLORS.draft}
            strokeWidth={1.5}
            strokeDasharray="4 3"
          />,
        );
      }
    }
  }
  return <g style={{ pointerEvents: "none" }}>{shapes}</g>;
}

export function SketchCanvas() {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const middlePan = useRef<Vec2 | null>(null);

  const entities = useSketchStore((s) => s.entities);
  const constraints = useSketchStore((s) => s.constraints);
  const viewport = useSketchStore((s) => s.viewport);
  const selection = useSketchStore((s) => s.selection);
  const hoveredId = useSketchStore((s) => s.hoveredId);
  const highlightConstraintId = useSketchStore((s) => s.highlightConstraintId);
  const activeToolId = useSketchStore((s) => s.activeTool);
  const canvasSize = useSketchStore((s) => s.canvasSize);
  const setHovered = useSketchStore((s) => s.setHovered);

  // measure canvas
  useEffect(() => {
    const el = containerRef.current;
    if (el === null) return;
    const measure = () =>
      sketchStore
        .getState()
        .setCanvasSize(el.clientWidth, el.clientHeight);
    measure();
    const ro = new ResizeObserver(measure);
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  const tool = getTool(activeToolId);

  const pointsById = useMemo(() => {
    const m = new Map<string, PointEntity>();
    for (const e of entities) if (isPoint(e)) m.set(e.id, e);
    return m;
  }, [entities]);

  const highlighted = useMemo(() => {
    if (highlightConstraintId === null) return new Set<string>();
    const c = constraints.find((k) => k.id === highlightConstraintId);
    return new Set(c?.entities ?? []);
  }, [highlightConstraintId, constraints]);

  const toToolEvent = (e: PointerEvent<SVGSVGElement>): ToolPointerEvent => {
    const rect = e.currentTarget.getBoundingClientRect();
    const screen = { x: e.clientX - rect.left, y: e.clientY - rect.top };
    return {
      screen,
      world: screenToWorld(sketchStore.getState().viewport, screen),
      hitId: sketchStore.getState().hoveredId,
      shiftKey: e.shiftKey,
      button: e.button,
    };
  };

  const onPointerDown = (e: PointerEvent<SVGSVGElement>) => {
    e.currentTarget.setPointerCapture(e.pointerId);
    const te = toToolEvent(e);
    if (e.button === 1) {
      middlePan.current = te.screen;
      e.preventDefault();
      return;
    }
    tool.onPointerDown(te);
  };

  const onPointerMove = (e: PointerEvent<SVGSVGElement>) => {
    const te = toToolEvent(e);
    if (middlePan.current !== null) {
      const s = sketchStore.getState();
      s.setViewport({
        ...s.viewport,
        offsetX: s.viewport.offsetX + (te.screen.x - middlePan.current.x),
        offsetY: s.viewport.offsetY + (te.screen.y - middlePan.current.y),
      });
      middlePan.current = te.screen;
      return;
    }
    tool.onPointerMove(te);
  };

  const onPointerUp = (e: PointerEvent<SVGSVGElement>) => {
    if (middlePan.current !== null) {
      middlePan.current = null;
      return;
    }
    tool.onPointerUp(toToolEvent(e));
  };

  const onWheel = (e: WheelEvent<SVGSVGElement>) => {
    const rect = e.currentTarget.getBoundingClientRect();
    const screen = { x: e.clientX - rect.left, y: e.clientY - rect.top };
    const s = sketchStore.getState();
    const vp = s.viewport;
    const world = screenToWorld(vp, screen);
    const factor = Math.exp(-e.deltaY * 0.0015);
    const scale = Math.min(Math.max(vp.scale * factor, MIN_SCALE), MAX_SCALE);
    s.setViewport({
      scale,
      offsetX: screen.x - world.x * scale,
      offsetY: screen.y + world.y * scale,
    });
  };

  const toScreen = (p: Vec2) => worldToScreen(viewport, p);

  const lineGlyphs: React.ReactElement[] = [];
  const circleGlyphs: React.ReactElement[] = [];
  const arcGlyphs: React.ReactElement[] = [];
  const pointGlyphs: React.ReactElement[] = [];

  for (const e of entities) {
    const state = glyphState(e.id, selection, hoveredId, highlighted);
    if (isLine(e)) {
      const p1 = pointsById.get(e.p1);
      const p2 = pointsById.get(e.p2);
      if (p1 === undefined || p2 === undefined) continue;
      const s1 = toScreen(p1);
      const s2 = toScreen(p2);
      lineGlyphs.push(
        <LineGlyph
          key={e.id}
          id={e.id}
          x1={s1.x}
          y1={s1.y}
          x2={s2.x}
          y2={s2.y}
          state={state}
          onHover={setHovered}
        />,
      );
    } else if (isCircle(e)) {
      const c = pointsById.get(e.center);
      if (c === undefined) continue;
      const sc = toScreen(c);
      circleGlyphs.push(
        <CircleGlyph
          key={e.id}
          id={e.id}
          cx={sc.x}
          cy={sc.y}
          r={e.radius * viewport.scale}
          state={state}
          onHover={setHovered}
        />,
      );
    } else if (isArc(e)) {
      const c = pointsById.get(e.center);
      if (c === undefined) continue;
      arcGlyphs.push(
        <ArcGlyph
          key={e.id}
          id={e.id}
          points={sampleArcScreenPoints(
            toScreen,
            c.x,
            c.y,
            e.radius,
            e.startAngle,
            e.endAngle,
          )}
          state={state}
          onHover={setHovered}
        />,
      );
    } else {
      const s = toScreen(e);
      pointGlyphs.push(
        <PointGlyph
          key={e.id}
          id={e.id}
          sx={s.x}
          sy={s.y}
          fixed={e.fixed}
          state={state}
          onHover={setHovered}
        />,
      );
    }
  }

  return (
    <div ref={containerRef} className="relative h-full w-full overflow-hidden">
      <svg
        className="absolute inset-0 h-full w-full"
        style={{ cursor: tool.cursor, touchAction: "none" }}
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerUp}
        onWheel={onWheel}
        onContextMenu={(e) => e.preventDefault()}
      >
        <rect width="100%" height="100%" fill="#0f172a" />
        {canvasSize.width > 0 && (
          <Grid
            viewport={viewport}
            width={canvasSize.width}
            height={canvasSize.height}
          />
        )}
        {lineGlyphs}
        {circleGlyphs}
        {arcGlyphs}
        {pointGlyphs}
        <DraftPreview />
        <ConstraintBadges />
        <DimensionLabels />
      </svg>
      <div className="pointer-events-none absolute bottom-2 left-2 text-xs text-slate-500">
        {tool.hint} · wheel to zoom · middle-drag to pan
      </div>
    </div>
  );
}
