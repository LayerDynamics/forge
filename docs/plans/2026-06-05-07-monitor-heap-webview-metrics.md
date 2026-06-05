# H6 — Real metrics (or honest errors) for `ext_monitor` heap & webview ops

**Severity:** HIGH (zeroed structs presented as metrics) · **Source:** Fix.md › H6 · **Crate:** `ext_monitor`

## Goal
`op_monitor_heap` returns real V8 heap stats; `op_monitor_webview` returns real WebView metrics. Where a real value is genuinely unobtainable in the current architecture, return the **existing** typed error (`9806 WebViewMetricsUnavailable`) instead of a zeroed `::default()`.

## Root cause (verified)
`crates/ext_monitor/src/lib.rs`:
- `op_monitor_heap` (957–965) — `Ok(HeapStats::default())` (all zeros), comment "placeholder that returns default values."
- `op_monitor_webview` (971–979) — `Ok(WebViewStats::default())`, comment "placeholder that returns empty stats."
- Runtime metrics also hardcode `pending_ops_count: 0` / `module_count: 0` (950–951) — see also L3.
- Unused error code `9806 WebViewMetricsUnavailable` is already documented (lib.rs:114) for exactly this case.

## Affected files
- `crates/ext_monitor/src/lib.rs` (`op_monitor_heap` 957–965, `op_monitor_webview` 971–979)
- `crates/ext_monitor/ts/init.ts` + `sdk/runtime.monitor.ts` (doc/return-type updates if signatures change)

## Implementation steps
1. **Heap:** obtain V8 heap statistics. The op runs inside the Deno `JsRuntime`; get the isolate's `HeapStatistics` via `v8::Isolate::get_heap_statistics`. The op needs isolate access — if the current `#[op2]` signature lacks it, change it to take `&mut OpState`/the isolate handle (deno_core exposes the isolate to ops). Populate `HeapStats` (used/total/limit/etc.) from the real `v8::HeapStatistics`.
   - If isolate access proves unavailable from this op context, return `MonitorError` (a heap-specific "unavailable" code) rather than zeros.
2. **WebView:** real WebView metrics require `ext_window`/`WindowManager` coordination (window count, per-window memory). Two acceptable outcomes:
   - (Preferred) Borrow `WindowManager`/window state from the shared `OpState` and populate real window counts/visibility.
   - (Fallback, honest) Return `Err(MonitorError::webview_metrics_unavailable())` using code **9806** until the window bridge exists. This removes the lie.
3. Fix `pending_ops_count`/`module_count` (950–951) — see plan L3; if isolate access is added for heap (step 1), populate these from the same isolate/runtime context here.

## Regression test (mandatory)
- `op_monitor_heap`: assert the returned `HeapStats` has a non-zero `heap_size_limit` (V8 always reports a limit) — proves it's reading the real isolate, not `::default()`. If the chosen outcome is an error, assert it returns the typed error, not `Ok(default)`.
- `op_monitor_webview`: assert it returns either populated real data OR `Err` with code `9806` — explicitly assert it is **not** `Ok(WebViewStats::default())`.

## Done criteria
- No `Ok(::default())` placeholder remains for these ops.
- `cargo test -p ext_monitor` passes; `cargo clippy -p ext_monitor -- -D warnings` clean.
- `sdk/runtime.monitor.ts` doc no longer says "Currently returns placeholder values" for whichever path is now real.

## Notes / risks
V8 isolate access from an op is the crux; confirm deno_core's supported pattern before changing the op signature. The WebView path may legitimately end as the 9806 error until the window bridge lands — that is acceptable and honest, and is the explicitly-allowed fallback here.
