/**
 * SolverService — adapter over the `acsSolveSketch` JSON API.
 *
 * Payload building and response parsing are pure functions (unit-testable
 * without wasm); the wasm entry point is injected into `AcsSolverService`.
 */

import type {
  ArcEntity,
  CircleEntity,
  ConstraintInstance,
  JsonPrimitive,
  LineEntity,
  PointEntity,
  Sketch,
  SketchEntity,
} from "../model/types";
import {
  constraintToPrimitive,
  primitiveToConstraint,
} from "../constraints/registry";

export interface SolveStats {
  iterations: number;
  initialError: number;
  finalError: number;
}

export interface SolveOutcome {
  ok: boolean;
  status: "converged" | "failed";
  /** entities with solved positions applied (input entities if failed) */
  entities: SketchEntity[];
  skippedConstraintIds: string[];
  conflictingConstraintIds: string[];
  /** IDs of entities that are fully constrained (degrees of freedom = 0) */
  fullyConstrainedIds: string[];
  error: string | null;
  stats: SolveStats | null;
  durationMs: number;
  /** raw JSON exchanged with the solver, for the JSON inspector */
  requestJson: string;
  responseJson: string;
  timestamp: number;
}

export interface ISolverService {
  solve(sketch: Sketch, maxIterations?: number): SolveOutcome;
}

// ---------------------------------------------------------------------------
// sketch -> primitives
// ---------------------------------------------------------------------------

export function entityToPrimitive(e: SketchEntity): JsonPrimitive {
  switch (e.kind) {
    case "point":
      return { id: e.id, type: "point", x: e.x, y: e.y, fixed: e.fixed };
    case "line":
      return { id: e.id, type: "line", p1_id: e.p1, p2_id: e.p2 };
    case "circle":
      return {
        id: e.id,
        type: "circle",
        c_id: e.center,
        radius: e.radius,
        fixed: e.fixed,
      };
    case "arc":
      return {
        id: e.id,
        type: "arc",
        c_id: e.center,
        radius: e.radius,
        start_angle: e.startAngle,
        end_angle: e.endAngle,
        fixed: e.fixed,
      };
  }
}

export function sketchToPrimitives(sketch: Sketch): JsonPrimitive[] {
  return [
    ...sketch.entities.map(entityToPrimitive),
    ...sketch.constraints.map(constraintToPrimitive),
  ];
}

export function buildSolveRequest(
  sketch: Sketch,
  maxIterations?: number,
): string {
  const body: {
    primitives: JsonPrimitive[];
    max_iterations?: number;
  } = { primitives: sketchToPrimitives(sketch) };
  if (maxIterations !== undefined) body.max_iterations = maxIterations;
  return JSON.stringify(body);
}

// ---------------------------------------------------------------------------
// primitives -> sketch (import / raw JSON apply)
// ---------------------------------------------------------------------------

export interface ImportResult {
  sketch: Sketch;
  /** primitives that could not be interpreted */
  warnings: string[];
}

function asNumber(v: unknown, fallback = 0): number {
  if (typeof v === "number" && Number.isFinite(v)) return v;
  if (typeof v === "string" && v !== "" && !Number.isNaN(Number(v)))
    return Number(v);
  return fallback;
}

function asString(v: unknown): string | null {
  return typeof v === "string" && v !== "" ? v : null;
}

export function primitivesToSketch(prims: readonly unknown[]): ImportResult {
  const entities: SketchEntity[] = [];
  const constraints: ConstraintInstance[] = [];
  const warnings: string[] = [];
  let anon = 0;

  for (const raw of prims) {
    if (typeof raw !== "object" || raw === null || Array.isArray(raw)) {
      warnings.push("Skipped non-object primitive");
      continue;
    }
    const prim = raw as Record<string, unknown>;
    const type = asString(prim.type);
    const id = asString(prim.id);
    if (type === null) {
      warnings.push("Skipped primitive without a type");
      continue;
    }

    switch (type) {
      case "point": {
        if (id === null) {
          warnings.push("Skipped point without id");
          break;
        }
        const p: PointEntity = {
          kind: "point",
          id,
          x: asNumber(prim.x),
          y: asNumber(prim.y),
          fixed: prim.fixed === true,
        };
        entities.push(p);
        break;
      }
      case "line": {
        const p1 = asString(prim.p1_id);
        const p2 = asString(prim.p2_id);
        if (id === null || p1 === null || p2 === null) {
          warnings.push(`Skipped malformed line ${id ?? "?"}`);
          break;
        }
        const l: LineEntity = { kind: "line", id, p1, p2 };
        entities.push(l);
        break;
      }
      case "circle": {
        const center = asString(prim.c_id);
        if (id === null || center === null) {
          warnings.push(`Skipped malformed circle ${id ?? "?"}`);
          break;
        }
        const c: CircleEntity = {
          kind: "circle",
          id,
          center,
          radius: asNumber(prim.radius),
          fixed: prim.fixed === true,
        };
        entities.push(c);
        break;
      }
      case "arc": {
        const center = asString(prim.c_id);
        if (id === null || center === null) {
          warnings.push(`Skipped malformed arc ${id ?? "?"}`);
          break;
        }
        const a: ArcEntity = {
          kind: "arc",
          id,
          center,
          radius: asNumber(prim.radius),
          startAngle: asNumber(prim.start_angle),
          endAngle: asNumber(prim.end_angle),
          fixed: prim.fixed === true,
        };
        entities.push(a);
        break;
      }
      default: {
        anon += 1;
        const c = primitiveToConstraint(prim, `k_import_${anon}`);
        if (c !== null) constraints.push(c);
        else warnings.push(`Skipped unknown primitive type "${type}"`);
      }
    }
  }

  return { sketch: { entities, constraints }, warnings };
}

// ---------------------------------------------------------------------------
// response parsing
// ---------------------------------------------------------------------------

function parseStats(v: unknown): SolveStats | null {
  if (typeof v !== "object" || v === null) return null;
  const o = v as Record<string, unknown>;
  return {
    iterations: asNumber(o.iterations),
    initialError: asNumber(o.initial_error),
    finalError: asNumber(o.final_error),
  };
}

function stringArray(v: unknown): string[] {
  if (!Array.isArray(v)) return [];
  return v.filter((x): x is string => typeof x === "string");
}

/**
 * Merge solved values from response primitives back into the input entities.
 * Only geometric values change; topology (ids, refs, fixed flags) is kept
 * from the input so flags survive end-to-end.
 */
export function applySolvedPrimitives(
  entities: readonly SketchEntity[],
  solvedPrims: readonly unknown[],
): SketchEntity[] {
  const byId = new Map<string, Record<string, unknown>>();
  for (const raw of solvedPrims) {
    if (typeof raw === "object" && raw !== null && !Array.isArray(raw)) {
      const o = raw as Record<string, unknown>;
      const id = asString(o.id);
      if (id !== null) byId.set(id, o);
    }
  }

  return entities.map((e) => {
    const solved = byId.get(e.id);
    if (solved === undefined) return e;
    switch (e.kind) {
      case "point":
        return {
          ...e,
          x: asNumber(solved.x, e.x),
          y: asNumber(solved.y, e.y),
        };
      case "circle":
        return { ...e, radius: asNumber(solved.radius, e.radius) };
      case "arc":
        return {
          ...e,
          radius: asNumber(solved.radius, e.radius),
          startAngle: asNumber(solved.start_angle, e.startAngle),
          endAngle: asNumber(solved.end_angle, e.endAngle),
        };
      case "line":
        return e;
    }
  });
}

export function parseSolveResponse(
  responseJson: string,
  inputEntities: readonly SketchEntity[],
  requestJson: string,
  durationMs: number,
): SolveOutcome {
  let root: unknown;
  try {
    root = JSON.parse(responseJson);
  } catch (err) {
    return {
      ok: false,
      status: "failed",
      entities: [...inputEntities],
      skippedConstraintIds: [],
      conflictingConstraintIds: [],
      fullyConstrainedIds: [],
      error: `Invalid solver response: ${String(err)}`,
      stats: null,
      durationMs,
      requestJson,
      responseJson,
      timestamp: Date.now(),
    };
  }

  const o = (
    typeof root === "object" && root !== null ? root : {}
  ) as Record<string, unknown>;

  const ok = o.ok === true;
  const solvedPrims = Array.isArray(o.primitives) ? o.primitives : [];

  return {
    ok,
    status: ok ? "converged" : "failed",
    // Never silently apply non-converged results.
    entities: ok
      ? applySolvedPrimitives(inputEntities, solvedPrims)
      : [...inputEntities],
    skippedConstraintIds: stringArray(o.skipped_constraint_ids),
    conflictingConstraintIds: stringArray(o.conflicting_constraint_ids),
    // Only meaningful on a converged solve; the solver returns [] otherwise.
    fullyConstrainedIds: ok ? stringArray(o.fully_constrained_ids) : [],
    error: typeof o.error === "string" ? o.error : null,
    stats: parseStats(o.stats),
    durationMs,
    requestJson,
    responseJson,
    timestamp: Date.now(),
  };
}

// ---------------------------------------------------------------------------
// service
// ---------------------------------------------------------------------------

export type SolveFn = (inputJson: string) => string;

export class AcsSolverService implements ISolverService {
  private readonly solveFn: SolveFn;

  constructor(solveFn: SolveFn) {
    this.solveFn = solveFn;
  }

  solve(sketch: Sketch, maxIterations?: number): SolveOutcome {
    const requestJson = buildSolveRequest(sketch, maxIterations);
    const started = performance.now();
    let responseJson: string;
    try {
      responseJson = this.solveFn(requestJson);
    } catch (err) {
      const durationMs = performance.now() - started;
      return {
        ok: false,
        status: "failed",
        entities: [...sketch.entities],
        skippedConstraintIds: [],
        conflictingConstraintIds: [],
        fullyConstrainedIds: [],
        error: `Solver threw: ${String(err)}`,
        stats: null,
        durationMs,
        requestJson,
        responseJson: "",
        timestamp: Date.now(),
      };
    }
    const durationMs = performance.now() - started;
    return parseSolveResponse(
      responseJson,
      sketch.entities,
      requestJson,
      durationMs,
    );
  }
}
