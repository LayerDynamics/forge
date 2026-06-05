# L3 — Populate hardcoded-zero metric fields

**Severity:** LOW (single fake fields inside otherwise-real structs) · **Source:** Fix.md › L3 · **Crates:** `ext_web_inspector`, `ext_monitor`, `forge-etch`

## Goal
Replace the remaining individual hardcoded-`0` / empty fields with real values, or document them as deliberately-unavailable rather than silently zero.

## Root cause (verified)
- `crates/ext_web_inspector/src/bridge/mod.rs:207` — `pending_ops_count: 0 // Would need JsRuntime access` (the rest of `RuntimeMetrics` is real).
- `crates/ext_monitor/src/lib.rs:950–951` — `pending_ops_count: 0` and `module_count: 0` in the runtime-metrics path.
- `crates/forge-etch/src/parser.rs:676` — `elements: vec![] // Would need recursive extraction` (decorator argument elements not extracted).

## Affected files
- `crates/ext_monitor/src/lib.rs` (runtime metrics builder, ~948–954)
- `crates/ext_web_inspector/src/bridge/mod.rs` (`get_runtime`, 204–214)
- `crates/forge-etch/src/parser.rs` (~660–680, decorator/element extraction)

## Implementation steps
1. **`pending_ops_count` / `module_count`:** these need `JsRuntime`/isolate access, which the metric ops don't currently have. This is the same access problem as plan **H6** (heap stats). Solve it once: when H6 gains isolate access, expose op/module counts from the same context and populate both the `ext_monitor` runtime path and the `ext_web_inspector` bridge from it. If isolate access is deferred, change these fields to `Option<u32>`/document them as "unavailable" so a `0` isn't read as "zero pending ops."
2. **`forge-etch` decorator elements:** implement the recursive extraction at parser.rs:676 — walk the decorator's argument AST and collect child elements into the `elements` vec instead of leaving it empty. Read the surrounding parse function to match the existing node-building style.

## Regression test (mandatory)
- `forge-etch`: parse a TS snippet with a decorator that has nested arguments and assert `elements` is non-empty with the expected entries (pre-fix it's empty).
- `ext_monitor`: if op/module counts become real, assert they're non-zero in a context with active ops/modules; if they become `Option`, assert they're `None` when isolate access is absent (not a misleading `0`).

## Done criteria
- No `: 0,` / `vec![]` placeholder with a "would need" comment remains in these three spots (`grep -n "Would need" …`).
- `cargo test` passes for the touched crates; `cargo clippy -- -D warnings` clean.

## Notes / risks
The op/module-count fields are coupled to H6's isolate-access work — sequence L3 after H6 (or merge them). The forge-etch element extraction is independent and can land on its own. Lowest priority overall.
