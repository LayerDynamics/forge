# M1 — Detect packaged mode for `runtime:app` `isPackaged`

**Severity:** MEDIUM (hardcoded `false`) · **Source:** Fix.md › M1 · **Crates:** `forge-runtime`, `ext_app`

## Goal
`isPackaged()` returns `true` when the app runs from a bundled artifact (`.app`/`.dmg`/`.msix`/AppImage) and `false` under `forge dev`.

## Root cause (verified)
`crates/forge-runtime/src/main.rs:695` — `is_packaged: false, // TODO: detect packaged mode` is set unconditionally when building `ext_app::AppInfo`.

## Affected files
- `crates/forge-runtime/src/main.rs` (AppInfo construction, ~690–700)
- Possibly `crates/ext_app/src/lib.rs` (if a helper belongs next to `AppInfo`)
- Read `crates/forge_cli/src/bundler/` (macos.rs/windows.rs/linux.rs) to learn the exact bundle layout each target produces, so detection matches reality.

## Implementation steps
1. Choose a robust signal. Best available: whether web assets were **embedded** at build time (the project already uses `FORGE_EMBED_DIR` to embed assets for release builds — CLAUDE.md "Asset Embedding"). Embedded assets ⇒ packaged. Expose a compile-time/runtime flag from the embed mechanism (e.g. a `const ASSETS_EMBEDDED: bool` in the generated `assets.rs`, or check whether the embedded asset map is non-empty).
2. Cross-check with executable location as a secondary signal: on macOS `current_exe()` path contains `.app/Contents/MacOS/`; on Windows installed under Program Files / MSIX package path; on Linux running from an AppImage (`APPIMAGE` env var is set). Combine: `is_packaged = assets_embedded || exe_in_bundle()`.
3. Set `is_packaged` from that computation instead of the literal `false`.

## Regression test (mandatory)
- Factor the detection into a pure function `detect_packaged(exe_path: &Path, assets_embedded: bool, env: &EnvLike) -> bool` and unit-test it: bundle-shaped macOS path → true; AppImage env set → true; plain `target/debug/forge-runtime` path with no embedded assets → false.

## Done criteria
- `is_packaged` is computed, not literal; `grep -n "is_packaged: false" crates/forge-runtime/src/main.rs` returns nothing.
- `cargo test -p forge-runtime` (or the crate owning the helper) passes; `cargo clippy` clean.

## Notes / risks
The embed flag is the most reliable signal since it's set exactly for release builds; exe-path heuristics are the backup. Keep both so dev-from-cargo and bundled-run are both correct.
