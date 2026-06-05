# L1 — Implement the `forge-smelt` crate (TS → standalone binary compiler)

**Severity:** LOW (empty scaffold) · **Source:** Fix.md › L1 · **Crate:** `forge-smelt` (new)

## Goal
Turn the empty `forge-smelt` scaffold into a working crate, and wire it into the workspace — or, if the capability is redundant, fold it into an existing crate. Decide the crate's purpose deliberately before coding.

## Root cause (verified)
`crates/forge-smelt/` contains only **0-byte** files: `Cargo.toml`, `src/lib.rs`, `mod.rs`, `compile.rs`, `binary.rs`, `parse.rs`, `transpile.rs`. The crate is **not** in `Cargo.toml` `members` and nothing references `forge_smelt`/`forge-smelt`.

## Intended purpose (from filenames + project context)
The module names (`parse` → `transpile` → `compile` → `binary`) and the name "smelt" (refining raw material into a finished ingot) indicate a **TypeScript → self-contained executable** compiler: parse/transpile app TS, then emit a single distributable binary. This overlaps conceptually with `forge bundle` (CLI bundler) and `ext_bundler`; **first confirm it isn't redundant** with those before building.

## Phase 0 — Decide (required before any code)
1. Read `crates/forge_cli/src/bundler/` and `crates/ext_bundler/` to see exactly what bundling/compilation already exists.
2. Decide one of: (a) `forge-smelt` is the dedicated TS→binary compile pipeline that `forge bundle` will call; (b) the capability already lives in the bundler and `forge-smelt` should be **deleted** (see plan L2's deletion-approval note — removing working-tree dirs needs explicit user approval); (c) repurpose to a distinct need.
3. Record the decision at the top of the implemented `lib.rs` as a module doc.

## Implementation steps (if building it — direction a)
1. `Cargo.toml`: name `forge-smelt`, add to workspace `members`; deps `deno_ast` (transpile), the embedding/asset mechanism, and whatever `forge-runtime` uses to embed assets.
2. `parse.rs`: load + parse an app's entry TS (reuse `forge-weld`'s `transpile` rather than duplicating).
3. `transpile.rs`: TS→JS via the shared transpile path.
4. `compile.rs`: orchestrate — collect modules, resolve `runtime:*` imports, embed assets.
5. `binary.rs`: produce the standalone artifact (link against `forge-runtime` with embedded snapshot/assets via `FORGE_EMBED_DIR`).
6. `lib.rs`/`mod.rs`: public API `smelt(app_dir, out_path) -> Result<PathBuf, SmeltError>`; wire it into `forge_cli` (`forge bundle`/a new `forge smelt`).

## Regression test (mandatory)
- A unit test that runs `parse`+`transpile` on a tiny in-repo TS fixture and asserts valid JS output.
- An integration test (may be `#[ignore]` for cost) that smelts a minimal example app and asserts the output binary exists and is executable.

## Done criteria
- Crate is in `members`, compiles, and is referenced by at least one caller (no orphan).
- `cargo build -p forge-smelt` + `cargo clippy -p forge-smelt -- -D warnings` clean.
- No 0-byte source files remain.
- **OR** a recorded decision + user-approved deletion if judged redundant.

## Notes / risks
High redundancy risk with the existing bundler — Phase 0 is mandatory; do not build a parallel pipeline without confirming it's needed. Deleting the scaffold instead is a valid outcome but requires explicit per-action user approval (working-tree deletion rule).
