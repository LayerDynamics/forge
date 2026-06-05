# H3 — Make `op_web_inspector_inject_panel` actually inject

**Severity:** HIGH (flips a flag, injects nothing) · **Source:** Fix.md › H3 · **Crate:** `ext_web_inspector`

## Goal
`inject_panel` must perform the real panel injection via the platform adapter and set `session.panel_injected = true` **only** on confirmed success.

## Root cause (verified)
`crates/ext_web_inspector/src/lib.rs:640–650` — sets `session.panel_injected = true` and returns `Ok(true)` with comment "Platform-specific injection would happen here // For now, just mark as injected" (645–646). `op_web_inspector_is_panel_injected` (656–663) then reports `true` for a panel that was never injected.

## Dependency
Requires the platform adapter's `inject_panel()` to be real → **depends on plan H1** (`2026-06-05-02-web-inspector-platform-native.md`). Sequence H1 before H3.

## Affected files
- `crates/ext_web_inspector/src/lib.rs` (`op_web_inspector_inject_panel`, ~625–651)
- `crates/ext_web_inspector/src/platform/mod.rs` (`inject_panel` per adapter — delivered by H1)

## Implementation steps
1. Resolve the platform adapter for the session's window and call its real `inject_panel(window_id, &assets)` (the panel HTML/JS/CSS assets are already validated in the adapter, platform/mod.rs:215–222).
2. Set `session.panel_injected = true` only if injection returns `Ok`. On `Err`, propagate the `WebInspectorError` and leave the flag `false`.
3. Keep the early-return for the already-injected case (640–643).

## Regression test (mandatory)
- Inject into a fresh session, then assert `op_web_inspector_is_panel_injected` returns `true` **only after** a successful adapter call. Use a test adapter whose `inject_panel` is forced to `Err` and assert the flag stays `false` and the op returns the error — proving the flag is no longer set unconditionally.

## Done criteria
- `panel_injected` is never set on an injection error path.
- `cargo test -p ext_web_inspector` passes; `cargo clippy -p ext_web_inspector -- -D warnings` clean.

## Notes / risks
Introduce a small `PlatformAdapter` trait seam (if not already present) so tests can substitute a fake adapter — this also helps H1's testability.
