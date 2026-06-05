# M3 — Stop silently dropping `rust_source` in `op_etcher_merge_nodes`

**Severity:** MEDIUM (param accepted, discarded) · **Source:** Fix.md › M3 · **Crate:** `ext_etcher`

## Goal
Either parse `rust_source` at runtime and merge its nodes, or remove the parameter so the API doesn't advertise a no-op. No dead `if/else` that discards input while returning success.

## Root cause (verified)
`crates/ext_etcher/src/lib.rs`:
- `op_etcher_merge_nodes` (318–349) — `rust_nodes` is computed by a dead conditional (342–347): `if rust_source.is_some() { Vec::new() } else { Vec::new() }`. `rust_source` is accepted, logged as "not implemented" (343), and dropped; the merge then returns success as if complete.
- `op_etcher_parse_rust` (303–312) — unconditional `Err("Direct Rust parsing not yet implemented…")`.

## Context (verified)
Rust metadata in forge-etch is normally collected at **build time** via the weld inventory system; `forge-etch` has a `docgen/rust.rs` path. Runtime Rust parsing from a source string is the missing capability.

## Affected files
- `crates/ext_etcher/src/lib.rs` (`op_etcher_merge_nodes` 318–349, `op_etcher_parse_rust` 303–312)
- `crates/forge-etch/src/parser.rs` / `docgen/rust.rs` (the real Rust-parsing entry points to reuse)
- `crates/ext_etcher/ts/init.ts` + `sdk/runtime.etcher.ts` (signature/doc)

## Implementation steps (pick ONE direction, stated in the plan)
**Direction A — Implement runtime Rust parsing (closes the gap):**
1. Find forge-etch's Rust source → `Vec<EtchNode>` routine (read `forge-etch/src/parser.rs` and `docgen/rust.rs`; the build-time path already parses Rust via `syn`). Expose a reusable `parse_rust_source(&str) -> Result<Vec<EtchNode>, _>`.
2. In `op_etcher_parse_rust`, call it and return real nodes instead of the error.
3. In `op_etcher_merge_nodes`, when `rust_source.is_some()`, parse it into `rust_nodes` and merge (TSDoc precedence already implemented just below at ~349).

**Direction B — Remove the dead surface (smaller, honest):**
1. Drop `rust_source` from `op_etcher_merge_nodes`' signature and its TS binding.
2. Make `op_etcher_parse_rust` return an explicit "use build-time inventory" error that is documented as the supported path (it already does — formalize it in the SDK docs and remove it from the advertised API if it can never work at runtime).

Default to **Direction A** per the "full implementation" decision; fall back to B only if runtime `syn` parsing pulls in unacceptable weight.

## Regression test (mandatory)
- Direction A: feed a small Rust source string with one `pub fn` to `op_etcher_parse_rust`; assert a node with that fn name is returned. For `merge_nodes`, pass both `ts_source` and `rust_source` and assert the merged result contains nodes from both.
- Direction B: assert the `rust_source` parameter no longer exists (compile-time) and `parse_rust` returns the documented typed error.

## Done criteria
- No dead `if x { Vec::new() } else { Vec::new() }` remains (`grep -n "Vec::new()" crates/ext_etcher/src/lib.rs` reviewed).
- `cargo test -p ext_etcher` passes; `cargo clippy -p ext_etcher -- -D warnings` clean.

## Notes / risks
Direction A adds a `syn`-based runtime parser; confirm `forge-etch` already depends on `syn` (build-time docgen does) so no new heavy dep is introduced at runtime.
