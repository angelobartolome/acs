/*
 * C smoke test for the sketch solver C ABI (include/p3d_sketch_solver.h).
 *
 * Reads a request from stdin (or passes NULL with --null), solves it with
 * P3DSketch_Solve, writes the response to stdout, releases it with
 * P3DSketch_Free, and exits with P3DSketch_Solve's return value.
 * Built and driven by tests/c_abi_test.rs.
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "p3d_sketch_solver.h"

static char *read_stdin(void) {
    size_t cap = 4096, len = 0;
    char *buf = malloc(cap);
    if (!buf) return NULL;
    size_t n;
    while ((n = fread(buf + len, 1, cap - len - 1, stdin)) > 0) {
        len += n;
        if (cap - len < 2) {
            cap *= 2;
            char *grown = realloc(buf, cap);
            if (!grown) { free(buf); return NULL; }
            buf = grown;
        }
    }
    buf[len] = '\0';
    return buf;
}

int main(int argc, char **argv) {
    char *request = NULL;
    if (!(argc > 1 && strcmp(argv[1], "--null") == 0)) {
        request = read_stdin();
        if (!request) return 100;
    }

    char *response = NULL;
    int code = P3DSketch_Solve(request, &response);
    free(request);
    if (!response) return 101;

    fputs(response, stdout);
    P3DSketch_Free(response);
    return code;
}
