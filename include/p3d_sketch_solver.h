/*
 * Sketch solver: solves one 2D sketch.
 *
 * Solver-neutral: any solver library can implement these two functions. The request and
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

/** The constraint vocabulary a request's constraints are written in. */
typedef enum {
    /** FreeCAD GCS's type names (`p2p_distance`, `horizontal_l`, ...). The default. */
    P3D_SKETCH_VOCABULARY_PLANEGCS = 0,
    /** The solver's own types (`distance`, `horizontal`, `on`, ...). */
    P3D_SKETCH_VOCABULARY_NATIVE = 1,
} P3DSketchVocabulary;

/**
 * Solves the sketch in `requestJson`, whose constraints are written in `vocabulary`. Returns 0
 * when the request was understood (whatever the solve status) and non-zero for a malformed
 * request, an unknown `vocabulary` included. `*responseJson` is always set and must be
 * released with P3DSketch_Free.
 */
int P3DSketch_Solve(const char *requestJson, P3DSketchVocabulary vocabulary, char **responseJson);

/** Releases a string returned by P3DSketch_Solve. */
void P3DSketch_Free(char *p);

#ifdef __cplusplus
}
#endif

#endif /* P3D_SKETCH_SOLVER_H */
