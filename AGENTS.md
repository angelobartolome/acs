# AGENTS.md

ACS is a 2-D geometric constraint solver: a Rust crate (`src/`) with WASM bindings, plus a React demo app (`wasm-demo/`).

## Commands

- Build: `cargo build`
- Lint (CI-enforced): `cargo clippy --all-targets --all-features`
- Test all: `cargo test`
- Single test file: `cargo test --test horizontal_test` (one integration test file per constraint in `tests/`)
- Pre-solver benchmark (a test, not `cargo bench`): `cargo test presolver_bench -- --nocapture`
- WASM build: `wasm-pack build --target web --out-dir pkg`
- Demo: `npm run dev` / `npm run build` / `npm run test` (vitest) in `wasm-demo/`

CI (`.github/workflows/rust.yml`) runs clippy -> build -> test -> wasm-pack build. Keep clippy clean.

## Architecture

- `src/constraints/` — one file per constraint implementing the `Constraint` trait (`num_residuals`, `residual`, analytical `jacobian`) from `constraints/base.rs`.
- `src/solver.rs` — `ConstraintSolver`, the public Rust API (string-ID based).
- `src/dogleg_solver.rs` — Dog-Leg trust-region numeric core.
- `src/component_graph.rs` — pre-solver: splits sketches into connected components and skips already-satisfied ones.
- `src/sketch_solve.rs` — JSON sketch API; maps JSON `type` strings (e.g. `"p2p_distance"`, `"horizontal_l"`) to `ConstraintType`.
- `src/bindings/` — wasm-bindgen wrappers.
- `pkg/` — **generated** by wasm-pack and gitignored; `wasm-demo` depends on it via `"acs": "file:../pkg"`. Run `wasm-pack build --target web --out-dir pkg` before `npm install`/`npm run dev` in the demo, and re-run it after any Rust change for the demo to pick it up.

## Adding a constraint

Touch all of these (grep an existing constraint like `Concentric` as a template):
1. `src/constraints/<name>.rs` + re-export in `src/constraints/mod.rs`
2. `ConstraintType` enum + `create_constraint` match in `src/constraints/base.rs`
3. JSON type-string mapping in `src/sketch_solve.rs` (if exposed to WASM/JSON API)
4. `tests/<name>_test.rs`
5. Docs: constraint tables in `README.md`, `USAGE.md`; residual/Jacobian math in `SPEC.md`

## Conventions

- Rust edition 2024. Jacobians are analytical (derived in `SPEC.md`), not numeric — new constraints must include exact partial derivatives. Verify them with a finite-difference test (pattern: `tests/midpoint_line_on_line_test.rs`); a wrong product-rule term has shipped before.
- Geometry entities are referenced by string IDs; constraints on lines/circles usually take the underlying point IDs, not the line ID (see `ConstraintType` variant comments).
- The JSON API contract (input/output shapes, constraint type strings) is documented in `USAGE.md`; keep it in sync with `sketch_solve.rs`. Docs have drifted from code before — trust the code.

## Solver / API facts (verified in code)

- Convergence and pre-solver skip both use L-inf residual < `1e-10` (`tolf` in `dogleg_solver.rs`, `TOLF` in `solver.rs`). Per-component stats: iterations summed, errors maxed.
- `PointOnLine` constrains to the **segment** (closest-point `t` clamped to `[0,1]`); `DistancePointLine` uses the infinite line.
- JSON API: unknown constraint `type`s and constraints with missing fields are silently skipped into `skipped_constraint_ids` (`"<id>:<type>"` / `"<id>:add_error:<msg>"`), not errors. `conflicting_constraint_ids` is always `[]`. `point_on_circle`/`tangent_lc` auto-resolve `center_id` from the circle's `c_id`. Ellipse/hyperbola/parabola primitives are accepted but pass through unsolved.
- In integration tests, `build_param_manager` is `pub(crate)` — construct a `ParameterManager` directly and call `register_entity(id, EntityType::Point, &Point::new(...))`.
