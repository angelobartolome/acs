/**
 * Integration smoke test: run every gallery example through the real wasm
 * solver (loaded synchronously from the `acs` package) via the same
 * request-building pipeline the app uses.
 */

import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { readFileSync } from "node:fs";

import { describe, expect, it } from "vitest";

import { AcsSolverService } from "../core/solver/SolverService";
import { EXAMPLES } from "./index";

const require = createRequire(import.meta.url);
const pkgDir = dirname(require.resolve("acs/package.json"));
const acs = (await import("acs")) as typeof import("acs");
acs.initSync({ module: readFileSync(join(pkgDir, "acs_bg.wasm")) });

const service = new AcsSolverService(acs.acsSolveSketch);

describe("gallery examples solve end-to-end", () => {
  for (const ex of EXAMPLES) {
    it(`${ex.label} converges`, () => {
      const outcome = service.solve(structuredClone(ex.sketch));
      expect(outcome.error).toBeNull();
      expect(outcome.ok).toBe(true);
      expect(outcome.skippedConstraintIds).toEqual([]);
      expect(outcome.stats).not.toBeNull();
      if (outcome.stats !== null) {
        expect(outcome.stats.finalError).toBeLessThan(1e-6);
      }
    });
  }
});
