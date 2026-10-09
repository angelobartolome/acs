/**
 * SolverService — adapter over the `acsSolveSketch` JSON API, which uses
 * the `P3DSketch_Solve` contract: request `{ version: 1, primitives,
 * maxIterations? }`; response `{ version, status, primitives, conflicting,
 * redundant, dof }`, or `{ version, status: "invalid", error, constraintId? }`
 * for a rejected request; plus ACS's `stats` and `fullyConstrained`.
 *
 * Payload building and response parsing are pure functions (unit-testable
 * without wasm); the wasm entry point is injected into `AcsSolverService`.
 */

import type {
  ArcEntity,
  EllipticalArcEntity,
  CircleEntity,
  ConstraintInstance,
  JsonPrimitive,
  LineEntity,
  PointEntity,
  Sketch,
  SketchEntity,
  SplineCurve,
  SplineEntity,
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

export type SolveStatus = "converged" | "failed" | "invalid";

export interface SolveOutcome {
  ok: boolean;
  /** `invalid`: the solver couldn't read the request (see `error`) */
  status: SolveStatus;
  /** entities with solved positions applied (input entities if failed) */
  entities: SketchEntity[];
  /** constraints that can't hold together with the others */
  conflictingIds: string[];
  /** constraints already implied by the others */
  redundantIds: string[];
  /** the constraint an `invalid` response rejected, if any */
  rejectedConstraintId: string | null;
  /** degrees of freedom the constraints leave (null if not solved) */
  dof: number | null;
  /** IDs of entities that are fully constrained (degrees of freedom = 0) */
  fullyConstrainedIds: string[];
  /** each spline contact's solved curve parameters, by constraint id */
  curveParams: Map<string, number[]>;
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
        start_id: e.start,
        end_id: e.end,
        radius: e.radius,
        start_angle: e.startAngle,
        end_angle: e.endAngle,
        fixed: e.fixed,
      };
    case "elliptical_arc":
      return {
        id: e.id,
        type: "elliptical_arc",
        c_id: e.center,
        focus1_id: e.focus,
        start_id: e.start,
        end_id: e.end,
        radmin: e.radmin,
        start_angle: e.startAngle,
        end_angle: e.endAngle,
        fixed: e.fixed,
      };
    case "spline":
      return {
        id: e.id,
        type: "spline",
        points: e.points,
        interpolated: e.interpolated,
      };
  }
}

/** A response's `curve` of a spline, if it reads as one. */
function splineCurve(v: unknown): SplineCurve | undefined {
  if (typeof v !== "object" || v === null) return undefined;
  const o = v as Record<string, unknown>;
  const knots = Array.isArray(o.knots) ? o.knots.filter((k): k is number => typeof k === "number") : [];
  const cps = Array.isArray(o.control_points)
    ? o.control_points.filter(
        (p): p is [number, number] =>
          Array.isArray(p) && typeof p[0] === "number" && typeof p[1] === "number",
      )
    : [];
  if (typeof o.degree !== "number" || cps.length < 2 || knots.length !== cps.length + o.degree + 1) {
    return undefined;
  }
  return { degree: o.degree, knots, controlPoints: cps };
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
    version: 1;
    primitives: JsonPrimitive[];
    maxIterations?: number;
  } = { version: 1, primitives: sketchToPrimitives(sketch) };
  if (maxIterations !== undefined) body.maxIterations = maxIterations;
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
  const constraintPrims: Record<string, unknown>[] = [];
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
        const start = asString(prim.start_id);
        const end = asString(prim.end_id);
        if (id === null || center === null || start === null || end === null) {
          warnings.push(`Skipped malformed arc ${id ?? "?"}`);
          break;
        }
        const a: ArcEntity = {
          kind: "arc",
          id,
          center,
          start,
          end,
          radius: asNumber(prim.radius),
          startAngle: asNumber(prim.start_angle),
          endAngle: asNumber(prim.end_angle),
          fixed: prim.fixed === true,
        };
        entities.push(a);
        break;
      }
      case "elliptical_arc": {
        const center = asString(prim.c_id);
        const focus = asString(prim.focus1_id);
        const start = asString(prim.start_id);
        const end = asString(prim.end_id);
        if (
          id === null ||
          center === null ||
          focus === null ||
          start === null ||
          end === null
        ) {
          warnings.push(`Skipped malformed elliptical arc ${id ?? "?"}`);
          break;
        }
        const ea: EllipticalArcEntity = {
          kind: "elliptical_arc",
          id,
          center,
          focus,
          start,
          end,
          radmin: asNumber(prim.radmin),
          startAngle: asNumber(prim.start_angle),
          endAngle: asNumber(prim.end_angle),
          fixed: prim.fixed === true || prim.isReference === true,
        };
        entities.push(ea);
        break;
      }
      case "spline": {
        const points = stringArray(prim.points);
        if (id === null || points.length < 2 || typeof prim.interpolated !== "boolean") {
          warnings.push(`Skipped malformed spline ${id ?? "?"}`);
          break;
        }
        const s: SplineEntity = {
          kind: "spline",
          id,
          points,
          interpolated: prim.interpolated,
          curve: splineCurve((prim as Record<string, unknown>).curve),
        };
        entities.push(s);
        break;
      }
      default:
        constraintPrims.push(prim);
    }
  }

  // Constraints last: their variant is inferred from the kinds of the
  // entities they reference, wherever those appear in the array.
  const byId = new Map(entities.map((e) => [e.id, e]));
  for (const prim of constraintPrims) {
    anon += 1;
    const c = primitiveToConstraint(prim, `k_import_${anon}`, (id) =>
      byId.get(id),
    );
    if (c !== null) constraints.push(c);
    else
      warnings.push(
        `Skipped unsupported constraint "${String(prim.type)}" ${asString(prim.id) ?? ""}`.trimEnd(),
      );
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
    initialError: asNumber(o.initialError),
    finalError: asNumber(o.finalError),
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
      case "elliptical_arc":
        return {
          ...e,
          radmin: asNumber(solved.radmin, e.radmin),
          startAngle: asNumber(solved.start_angle, e.startAngle),
          endAngle: asNumber(solved.end_angle, e.endAngle),
        };
      case "spline":
        return { ...e, curve: splineCurve(solved.curve) ?? e.curve };
      case "line":
        return e;
    }
  });
}

/** The `curve_params` the response reports, by constraint id. */
function solvedCurveParams(solvedPrims: readonly unknown[]): Map<string, number[]> {
  const out = new Map<string, number[]>();
  for (const raw of solvedPrims) {
    if (typeof raw !== "object" || raw === null) continue;
    const o = raw as Record<string, unknown>;
    const id = asString(o.id);
    if (id !== null && Array.isArray(o.curve_params)) {
      out.set(id, o.curve_params.filter((v): v is number => typeof v === "number"));
    }
  }
  return out;
}

function invalidOutcome(
  inputEntities: readonly SketchEntity[],
  error: string,
  requestJson: string,
  responseJson: string,
  durationMs: number,
): SolveOutcome {
  return {
    ok: false,
    status: "invalid",
    entities: [...inputEntities],
    conflictingIds: [],
    redundantIds: [],
    rejectedConstraintId: null,
    dof: null,
    fullyConstrainedIds: [],
    curveParams: new Map(),
    error,
    stats: null,
    durationMs,
    requestJson,
    responseJson,
    timestamp: Date.now(),
  };
}

function parseStatus(v: unknown): SolveStatus {
  return v === "converged" || v === "failed" ? v : "invalid";
}

export type ConstraintFlag = "conflicting" | "redundant" | "rejected";

/** What the last solve said about individual constraints, by constraint id. */
export function constraintFlags(outcome: SolveOutcome): Map<string, ConstraintFlag> {
  const flags = new Map<string, ConstraintFlag>();
  for (const id of outcome.redundantIds) flags.set(id, "redundant");
  for (const id of outcome.conflictingIds) flags.set(id, "conflicting");
  if (outcome.rejectedConstraintId !== null)
    flags.set(outcome.rejectedConstraintId, "rejected");
  return flags;
}

/** IDs of constraints the last solve flagged as a problem: conflicting or rejected. */
export function problemConstraintIds(outcome: SolveOutcome): Set<string> {
  const ids = new Set<string>();
  for (const [id, flag] of constraintFlags(outcome))
    if (flag !== "redundant") ids.add(id);
  return ids;
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
    return invalidOutcome(
      inputEntities,
      `Invalid solver response: ${String(err)}`,
      requestJson,
      responseJson,
      durationMs,
    );
  }

  const o = (
    typeof root === "object" && root !== null ? root : {}
  ) as Record<string, unknown>;

  const status = parseStatus(o.status);
  const ok = status === "converged";
  const solvedPrims = Array.isArray(o.primitives) ? o.primitives : [];
  const error = typeof o.error === "string" ? o.error : null;

  return {
    ok,
    status,
    // Never silently apply non-converged results.
    entities: ok
      ? applySolvedPrimitives(inputEntities, solvedPrims)
      : [...inputEntities],
    conflictingIds: stringArray(o.conflicting),
    redundantIds: stringArray(o.redundant),
    rejectedConstraintId:
      status === "invalid" ? asString(o.constraintId) : null,
    dof: typeof o.dof === "number" ? o.dof : null,
    // Only meaningful on a converged solve; the solver returns [] otherwise.
    fullyConstrainedIds: ok ? stringArray(o.fullyConstrained) : [],
    curveParams: ok ? solvedCurveParams(solvedPrims) : new Map(),
    error,
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
      return invalidOutcome(
        sketch.entities,
        `Solver threw: ${String(err)}`,
        requestJson,
        "",
        performance.now() - started,
      );
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
