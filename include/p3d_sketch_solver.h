/*
 * Sketch solver: solves one 2D sketch, and builds a spline's curve without solving.
 *
 * Solver-neutral: any solver library can implement these functions. The request and
 * response are versioned JSON in the sketch primitive format.
 *
 * Request:  { "version": 1, "primitives": [...], "maxIterations"?: n }
 * Response: { "version": 1, "status": "converged" | "failed" | "invalid",
 *             "primitives": [...], "conflicting": [...], "redundant": [...], "dof": n,
 *             "error"?: "..." }
 *
 * Every solver keeps Reference Geometry (`isReference`) fixed, radii included, resolves
 * `param` primitives by id, and reports conflicts and redundancies at the solved position.
 * No state is kept between calls.
 */
#ifndef P3D_SKETCH_SOLVER_H
#define P3D_SKETCH_SOLVER_H

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Solves the sketch in `requestJson`. Returns 0 when the request was understood (whatever
 * the solve status) and non-zero for a malformed request. `*responseJson` is always set and
 * must be released with P3DSketch_Free.
 */
int P3DSketch_Solve(const char *requestJson, char **responseJson);

/**
 * A spline's curve from its handles, without solving: the `curve` a solve reports for a
 * `spline` primitive with these handles.
 *
 * Request:  { "version": 1, "handles": [[x, y], ...], "interpolated": true | false,
 *             "knots"?: [...] }
 * Response: { "version": 1, "curve": { "degree": 3, "knots": [...],
 *                                      "control_points": [[x, y], ...] } }
 *        or { "version": 1, "error": "..." }
 *
 * Returns 0 with a curve and non-zero with an `error` (a malformed request, or handles and
 * knots a sketch's spline would be rejected for). `*responseJson` is always set and must be
 * released with P3DSketch_Free.
 */
int P3DSpline_Curve(const char *requestJson, char **responseJson);

/** Releases a string returned by P3DSketch_Solve or P3DSpline_Curve. */
void P3DSketch_Free(char *p);

#ifdef __cplusplus
}
#endif

#endif /* P3D_SKETCH_SOLVER_H */
