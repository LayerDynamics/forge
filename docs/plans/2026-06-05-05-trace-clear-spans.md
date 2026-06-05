# H4 — Implement CDP `clear_spans` (real span clearing via `ext_trace`)

**Severity:** HIGH (reports success, clears nothing) · **Source:** Fix.md › H4 · **Crates:** `ext_web_inspector`, `ext_trace`

## Goal
The CDP `Tracing.clear`-style call must actually clear trace spans and return the real number cleared, or a typed error — never `Ok(0)` as a placebo.

## Root cause (verified)
`crates/ext_web_inspector/src/cdp/router.rs:368–374` — `clear_spans()` returns `Ok(0)` with comment "Clearing spans requires mutable access to TraceState // This would need an op_trace_clear or similar in ext_trace // For now, return 0 as a placeholder."

## Affected files
- `crates/ext_trace/src/lib.rs` — add `op_trace_clear` (and/or a `TraceState::clear()` method) that clears active/finished spans and returns the count removed.
- `crates/ext_web_inspector/src/cdp/router.rs` (`clear_spans`, 368–374) — call the new trace clearing path.
- `crates/ext_trace/build.rs` — register `op_trace_clear` in `.ops(&[…])` so the SDK exposes it.

## Implementation steps
1. In `ext_trace`, add `TraceState::clear(&mut self) -> u32` returning the number of spans removed; clear the active and finished span collections.
2. Expose `#[weld_op] #[op2] op_trace_clear(state: &mut OpState) -> Result<u32, TraceError>` calling `clear()`. Add it to `build.rs` ops + regenerate SDK.
3. In `cdp/router.rs::clear_spans`, obtain mutable `TraceState` from the shared `OpState` (the router has `&Rc<RefCell<OpState>>`) and call `clear()`, returning the real count. On absent `TraceState`, return a `WebInspectorError` (not `Ok(0)`).
4. Confirm `op_web_inspector_get_metrics` (H2) reads the same `TraceState` span counts so they stay consistent after a clear.

## Regression test (mandatory)
- In `ext_trace`: seed `TraceState` with N spans, call `clear()`, assert it returns N and the state is empty.
- In `ext_web_inspector`: with a seeded `TraceState`, call `clear_spans` and assert it returns N (not `0`); with no `TraceState`, assert it returns `Err`.

## Done criteria
- `clear_spans` returns measured counts; no `Ok(0)` placeholder remains.
- `cargo test -p ext_trace -p ext_web_inspector` passes; `cargo clippy` clean for both.
- `op_trace_clear` present in generated `sdk/runtime.trace.ts`.

## Notes / risks
Mutable borrow of `TraceState` from `Rc<RefCell<OpState>>` inside the CDP router — ensure no overlapping borrow is held when calling. Coordinate ordering with H2 (shared `TraceState` span-count source).
