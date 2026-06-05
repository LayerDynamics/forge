# M2 — Honor `source_map` and `minify` in `op_weld_transpile`

**Severity:** MEDIUM (options accepted, silently ignored) · **Source:** Fix.md › M2 · **Crates:** `ext_weld`, `forge-weld`

## Goal
When a caller passes `source_map: true`, return a real source map; when `minify: true`, return minified output. Don't accept options that do nothing.

## Root cause (verified)
- `crates/ext_weld/src/lib.rs:280–296` — `TranspileOptions { source_map, minify }` are read into `opts` but the call `transpile_ts(&source, specifier)` ignores them, and the result hardcodes `source_map: None` (294, `// TODO: implement source map support`).
- `crates/forge-weld/src/build/transpile.rs:45` — `transpile_ts(ts_code: &str, specifier: &str) -> Result<String, TranspileError>` has no options param and returns only the code string.

## Affected files
- `crates/forge-weld/src/build/transpile.rs` (extend the transpile API)
- `crates/ext_weld/src/lib.rs` (`op_weld_transpile`, 280–296)
- `crates/ext_weld/ts/init.ts` + `sdk/runtime.weld.ts` (doc update once real)

## Implementation steps
1. In `forge-weld`, add an options-aware entry point, e.g. `transpile_ts_with(ts_code, specifier, &TranspileSettings { source_map: bool, minify: bool }) -> Result<TranspileOutput, TranspileError>` where `TranspileOutput { code: String, source_map: Option<String> }`. Keep the existing `transpile_ts` as a thin wrapper (no source map, no minify) for build-script callers.
2. Implement via `deno_ast`: when transpiling, set `EmitOptions { source_map: SourceMapOption::Separate, .. }` to get the map, and run minification when requested (deno_ast/`swc` emit supports minify; if not exposed, integrate the swc minifier already in the deno_ast dependency tree). Return the produced `.map` string.
3. In `op_weld_transpile`, pass `opts.source_map`/`opts.minify` into `transpile_ts_with` and populate `TranspileResult.source_map` from the real output (remove the hardcoded `None`).

## Regression test (mandatory)
- `transpile_ts_with` with `source_map: true` on a small TS snippet → assert `output.source_map.is_some()` and the JSON map has `"mappings"`.
- With `minify: true` → assert output length is shorter / whitespace collapsed vs unminified.
- `op_weld_transpile` round-trip: request a source map and assert the returned `TranspileResult.source_map` is `Some`.

## Done criteria
- No hardcoded `source_map: None` remains in the op; options flow through.
- `cargo test -p forge-weld -p ext_weld` passes; `cargo clippy` clean.

## Notes / risks
Confirm the exact `deno_ast` version's `EmitOptions`/`SourceMapOption` API in `Cargo.lock` before coding — the enum names have shifted across versions. Don't assume; read the dep's API.
