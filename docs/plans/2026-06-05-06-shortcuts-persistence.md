# H5 — Implement real shortcuts persistence (save/load)

**Severity:** HIGH (claims success, persists nothing) · **Source:** Fix.md › H5 · **Crate:** `ext_shortcuts`
**Highest-value fix in the set — the storage backend already exists.**

## Goal
`op_shortcuts_save` writes shortcuts to durable storage; `op_shortcuts_load` reads them back. Restarting the app preserves registered shortcuts.

## Root cause (verified)
`crates/ext_shortcuts/src/lib.rs`:
- `op_shortcuts_save` (740–751) — serializes to `_json` then **discards it**; only `trace!("Would save {} shortcuts to storage")` and `Ok(())`.
- `op_shortcuts_load` (758–773) — returns `Ok(vec![])` after `trace!("Would load shortcuts from storage")`.
Comments say "In production, this would call ext_storage::op_storage_set/get."

## Key facts (verified)
- `ext_storage` exposes the real ops: `op_storage_get` (lib.rs:515), `op_storage_set` (542), plus `_many` variants. Backed by SQLite at `data_dir/.forge/<app_id>/storage.db` via `StorageConnection { db_path, connection: Arc<Mutex<Connection>> }` (ext_storage/src/lib.rs:399) held in the **shared `OpState`**.
- Both extensions get the **same `app_id`** at init: `ext_registry.rs:579` (`init_storage_state(state, app_id, None)`) and `ext_registry.rs:701` (`init_shortcuts_state(state, app_id)`).
- `ShortcutsState` already holds `storage_key = "forge-shortcuts-<app_id>"` (lib.rs:235,260) and `app_id` (231).
- `save` already builds `PersistedShortcuts { shortcuts }` and serializes it (740–742); only the storage write is missing.

## Chosen approach
Reuse the shared `StorageConnection` from `OpState` (same DB the `runtime:storage` ops use), keyed by `storage_key`. This keeps shortcut data in the app's normal storage DB and avoids a second persistence mechanism.
**Ordering requirement:** storage state must be initialized before shortcuts persistence runs. Verify tier ordering in `ext_registry.rs`; if shortcuts can run before storage, fall back to lazily opening the same `storage.db` path using `ext_storage`'s path logic (`dirs::data_dir()/.forge/<app_id>/storage.db`).

## Affected files
- `crates/ext_shortcuts/src/lib.rs` (`op_shortcuts_save` 740–751, `op_shortcuts_load` 758–773)
- `crates/ext_shortcuts/Cargo.toml` (may need `ext_storage` as a dependency to reuse `StorageConnection`/types, or `rusqlite`+`serde_json` already present)
- `crates/forge-runtime/src/ext_registry.rs` (confirm storage initializes before shortcuts; read-only unless ordering must change)

## Implementation steps
1. **Save:** borrow `StorageConnection` from the shared `OpState`; write `(storage_key, serialized_json)` into the storage table using the same SQL `op_storage_set` uses (read ext_storage:542–566 for the exact upsert). Return real `Ok(())` only after a successful write; propagate `ShortcutsError::persistence_error` on failure.
2. **Load:** borrow `StorageConnection`, `SELECT` the value for `storage_key`; if present, `serde_json::from_str::<PersistedShortcuts>` and return `.shortcuts`; if absent, return `Ok(vec![])` (legitimately empty — first run). On a deserialize error return `Err`, do not swallow.
3. Remove both `trace!("Would …")` lines and the discarded `_json`.
4. If reusing `StorageConnection` causes a dependency cycle (`ext_storage` ↔ `ext_shortcuts`), instead replicate the minimal connect-and-upsert against the same `storage.db` path (self-contained, no cross-crate dep). Decide based on the actual dep graph.

## Regression test (mandatory)
- Init a temp `StorageConnection` (temp dir db) in `OpState`, register two shortcuts, call `op_shortcuts_save`, then a fresh `op_shortcuts_load` and assert it returns those two `ShortcutConfig`s (round-trip). Pre-fix this fails (load returns empty); post-fix it passes.
- Assert `load` on an empty store returns `Ok(vec![])`, and that a corrupt stored value returns `Err`.

## Done criteria
- Round-trip test passes; no `"Would save/load"` strings remain (`grep -n "Would " crates/ext_shortcuts/src/lib.rs`).
- `cargo test -p ext_shortcuts` passes; `cargo clippy -p ext_shortcuts -- -D warnings` clean.

## Notes / risks
Watch for a dep cycle (step 4). Confirm the storage table schema (key/value column names) by reading `ext_storage`'s schema/init before writing SQL — do not assume column names.
