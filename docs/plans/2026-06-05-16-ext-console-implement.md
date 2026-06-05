# L2 — Implement the `ext_console` crate (`runtime:console` capture)

**Severity:** LOW (empty scaffold) · **Source:** Fix.md › L2 · **Crate:** `ext_console` (new)

## Goal
Turn the empty `ext_console` scaffold into a working `runtime:console` extension that captures `console.*` output from both the Deno side and the WebView renderer, or delete it if redundant with existing logging.

## Root cause (verified)
`crates/ext_console/` contains only **0-byte** files: `Cargo.toml`, `build.rs`, `README.md`, `src/{lib.rs,console.rs,console_listener.rs,console_log.rs,web_console.rs}`, `ts/`. Not in `Cargo.toml` `members`; nothing references it.

## Intended purpose (from filenames)
`console.rs` / `console_listener.rs` / `console_log.rs` / `web_console.rs` indicate an extension that **captures console logs** — both Deno `console.*` and **web/renderer** console output (`web_console.rs`) — exposing them to app code (e.g. for an in-app log viewer or the web inspector). Overlaps with `ext_log` and `ext_web_inspector`; confirm scope vs those first.

## Phase 0 — Decide (required before any code)
1. Read `crates/ext_log/` and `crates/ext_web_inspector/` to see what console/log capture already exists.
2. Decide: (a) build `ext_console` as the unified console-capture extension feeding the web inspector's console panel; (b) delete if `ext_log` already covers it (deletion needs explicit user approval); (c) repurpose.
3. Record the decision in `src/lib.rs` module doc and `README.md`.

## Implementation steps (if building it)
1. Scaffold per the "Adding a New Extension" checklist in `CLAUDE.md` (steps 1–8): `Cargo.toml`, `build.rs` with `ExtensionBuilder::new("runtime_console", "runtime:console")`, `ts/init.ts`, register in `ext_registry.rs` at the correct tier, add to workspace `members`.
2. `console_listener.rs`: subscribe to Deno `console.*` (override/patch the console in `ts/init.ts`, or hook deno_core's logging) and forward structured `{level, args, timestamp}` records.
3. `web_console.rs`: receive renderer console messages over the existing IPC bridge (`window.host` / `__host_dispatch`) — renderer patches `console.*` in preload and forwards to Deno.
4. `console_log.rs`: ring-buffer of recent records in `ConsoleState`; ops `op_console_tail(n)`, `op_console_subscribe`, `op_console_clear`.
5. `lib.rs`: assemble ops + state; `#[weld_op]` annotations; generate the SDK.

## Regression test (mandatory)
- Push synthetic log records into `ConsoleState`'s buffer and assert `op_console_tail(n)` returns the last `n` in order and `op_console_clear` empties it.
- Assert the renderer-message ingestion path parses a sample IPC payload into a record.

## Done criteria
- Crate compiles, in `members`, registered in `ext_registry.rs`, SDK generated at `sdk/runtime.console.ts`, and reachable from a sample app.
- `cargo build -p ext_console` + `cargo clippy -p ext_console -- -D warnings` clean. No 0-byte files remain.
- **OR** recorded decision + user-approved deletion if redundant.

## Notes / risks
Redundancy with `ext_log`/`ext_web_inspector` is real — Phase 0 mandatory. If kept, the web-console capture should feed the inspector console panel (coordinate with plan H1/H2). Deletion requires explicit per-action user approval.
