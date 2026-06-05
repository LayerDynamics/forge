# H2 — Make `op_web_inspector_get_metrics` aggregate real data

**Severity:** HIGH (hardcoded availability/zeros) · **Source:** Fix.md › H2 · **Crate:** `ext_web_inspector`

## Goal
Return real aggregated metrics. `system_available`/`runtime_available`/`trace_available` must reflect whether those subsystems are actually present; counts must be measured, not literal `0`.

## Root cause (verified)
`crates/ext_web_inspector/src/lib.rs:673–688` — returns `AggregatedMetrics { system_available: true, runtime_available: true, trace_available: true, active_span_count: 0, finished_span_count: 0, signal_subscriptions: 0, ipc_channel_count: 0, window_count: <real> }`. Comment 674–675: "would aggregate … For now, return a placeholder." Only `window_count` is real.

## Key asset (verified)
The bridge layer is already real: `crates/ext_web_inspector/src/bridge/mod.rs` reads live data from `MonitorState` (`get_runtime` 204–214, `get_disks` 217–235, `get_network` 238–255) and there are `MonitorBridge`/`DebuggerBridge` marked "REAL IMPLEMENTATION" (146, 594). Wire `get_metrics` through these bridges and through `try_borrow::<…State>()` presence checks.

## Affected files
- `crates/ext_web_inspector/src/lib.rs` (`op_web_inspector_get_metrics`, ~669–688)
- `crates/ext_web_inspector/src/bridge/mod.rs` (read; possibly add small accessors for span/signal/ipc counts)

## Implementation steps
1. Derive each `*_available` from real presence: `system_available = state.try_borrow::<MonitorState>().is_some()` (system metrics come from `MonitorState.system`); `trace_available = state.try_borrow::<TraceState>().is_some()`; `runtime_available` likewise from whatever state backs runtime metrics.
2. `active_span_count` / `finished_span_count`: read from `TraceState` via the trace bridge (the CDP router already reads active spans — `cdp/router.rs:355–365` collects them; reuse that source). If finished-span tracking doesn't exist, add a counter to `TraceState` rather than returning `0`.
3. `signal_subscriptions`: read from the signals bridge (`SignalsBridge` exists — `cdp/router.rs:381`). Use the real subscription count.
4. `ipc_channel_count`: read from the IPC/window state if exposed; if genuinely unavailable, omit the field or return it via an `Option`/explicit "unavailable" rather than a fake `0`.
5. Keep `window_count` (already real).

## Regression test (mandatory)
- With a populated `MonitorState`/`TraceState` in `OpState`, assert `get_metrics` reports `system_available == true` and span/signal counts equal to the seeded values.
- With those states **absent** from `OpState`, assert the corresponding `*_available` is `false` (proves the flags are no longer hardcoded `true`).

## Done criteria
- No hardcoded `true`/`0` literals remain in the returned struct except where provably correct.
- `cargo test -p ext_web_inspector` passes incl. new tests.
- `cargo clippy -p ext_web_inspector -- -D warnings` clean.

## Notes / risks
Depends on `TraceState` exposing span counts; if it doesn't, this plan includes adding that accessor (coordinate with H4 which also wants `op_trace_clear` in `ext_trace`).
