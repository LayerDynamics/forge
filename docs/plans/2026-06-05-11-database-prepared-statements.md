# M4 — Implement (or remove) prepared-statement execution in `ext_database`

**Severity:** MEDIUM (always errors) · **Source:** Fix.md › M4 · **Crate:** `ext_database`

## Goal
`stmt.query()` / `stmt.execute()` on a prepared statement actually run, OR the prepared-statement API is removed from the SDK so callers aren't offered a path that always throws.

## Root cause (verified)
`crates/ext_database/src/lib.rs`:
- `op_database_stmt_query` (1408–1426) — always `Err(PreparedStatementError "…Use op_database_query instead.")`; params explicitly discarded via `let _ = (&state, &db_id, &stmt_id, &params);` (1418).
- `op_database_stmt_execute` (1428–1444) — always `Err(...)`.
The SDK advertises `prepare()` → `Statement.query()/execute()` (ts/init.ts:1589 notes caching "not implemented in this version").

## Affected files
- `crates/ext_database/src/lib.rs` (`op_database_stmt_*`, the prepare op, and a statement registry in `DatabaseState`)
- `crates/ext_database/ts/init.ts` + `sdk/runtime.database.ts`

## Implementation steps (full impl)
1. Read the existing `op_database_stmt_prepare` (the op that mints `stmt_id`) and `DatabaseState` to see what's stored per connection. Determine why caching was deferred (rusqlite `Statement<'conn>` borrows the `Connection`, which is hard to store across async op calls — that's the real blocker).
2. Implement statement reuse without lifetime pain: store the **SQL text + metadata** under `stmt_id` in a `HashMap` in `DatabaseState` at `prepare` time (not the borrowed `rusqlite::Statement`). On `stmt_query`/`stmt_execute`, look up the SQL by `stmt_id`, then `conn.prepare_cached(&sql)` (rusqlite's built-in statement cache gives the perf benefit without lifetime storage) and bind `params`.
3. Bind `params` (already passed in) positionally; map rows to `QueryResult` using the same row-mapping helper `op_database_query` uses. Return real results.
4. Remove the `let _ = (...)` discards and the unconditional errors.

## Regression test (mandatory)
- Open an in-memory SQLite db (`:memory:`), create a table, `prepare` an `INSERT … VALUES (?)`, call `stmt_execute` with params twice, then `prepare`+`stmt_query` a `SELECT` and assert it returns both rows. Pre-fix this errors; post-fix it passes.
- Assert a `stmt_id` that was never prepared returns a typed `Err` (not a panic).

## Done criteria
- `stmt_query`/`stmt_execute` return real data; no unconditional error remains.
- `cargo test -p ext_database` passes; `cargo clippy -p ext_database -- -D warnings` clean.

## Notes / risks
`prepare_cached` is the key to sidestepping rusqlite's `Statement` lifetime issue — verify `ext_database` opens connections in a way compatible with the cache (same `Connection` instance). If multi-threaded `Arc<Mutex<Connection>>`, the cache lives on that connection and works.
