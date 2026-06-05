# M5 — Implement `dock.setMenu` via NSMenu (macOS)

**Severity:** MEDIUM (discards menu, returns false) · **Source:** Fix.md › M5 · **Crate:** `ext_dock`

## Goal
`op_dock_set_menu` builds a real `NSMenu` from the provided `Vec<MenuItem>` and sets it as the application's dock tile menu on macOS.

## Root cause (verified)
`crates/ext_dock/src/lib.rs:497–516` — on macOS the op only `warn!("dock.set_menu is not yet implemented")` and returns `false`; the `_menu` parameter is discarded (`// TODO: Implement dock menu using NSMenu`, 505).

## Affected files
- `crates/ext_dock/src/lib.rs` (`op_dock_set_menu` 497–516; reuse existing macOS Obj-C bridging already used by `op_dock_set_icon` / badge ops in this file)
- `crates/ext_dock/Cargo.toml` (confirm the Obj-C crate already in use — the dock badge/icon ops imply `objc2`/`cocoa` is present)

## Implementation steps
1. Read how the existing macOS dock ops in this file obtain the `NSApplication` and bridge to Obj-C (badge/icon ops at lines ~340–495) — reuse the same crate and pattern (no new dependency).
2. macOS exposes the dock menu via `NSApplicationDelegate applicationDockMenu:`. Because Forge owns the app delegate (tao/wry), implement by storing the built `NSMenu` and returning it from the delegate's `applicationDockMenu:`, OR use the documented approach of setting an associated dock menu. Determine which is reachable given tao's delegate ownership; if the delegate isn't overridable, set the menu on the dock tile via `NSApp.dockTile` mechanisms.
3. Build the `NSMenu`: iterate `_menu: Vec<MenuItem>`, create `NSMenuItem`s with title/action/enabled/separator per the `MenuItem` fields (read the `MenuItem` struct in this crate for exact fields). Wire item actions back to an IPC event so the Deno side receives clicks (mirror how tray/menu clicks are dispatched elsewhere — read `ext_window`'s menu handling for the established click→IPC pattern).
4. Return `true` on success; keep the non-macOS branch returning `false` with the existing warning (legit — dock menus are macOS-only).

## Regression test (mandatory)
- The Obj-C call can't run in headless CI, so unit-test the pure `Vec<MenuItem>` → menu-model conversion (titles, separators, nesting) and assert it produces the expected intermediate structure. Gate the actual `NSMenu` set behind `#[cfg(target_os = "macos")]` + `#[ignore]` with manual-run instructions.
- Assert the non-macOS path returns `false` (unchanged, documented).

## Done criteria
- `_menu` is consumed and converted; no `// TODO` / "not yet implemented" warning remains on the macOS path.
- `cargo build -p ext_dock` (macOS) compiles; `cargo clippy -p ext_dock -- -D warnings` clean.
- Manual: set a dock menu from a sample app, right-click dock icon → items appear and clicks reach Deno.

## Notes / risks
The delegate-ownership question (step 2) is the real unknown — tao owns the `NSApplicationDelegate`. Investigate whether tao/wry exposes a dock-menu hook before hand-rolling delegate swizzling; prefer the supported hook.
