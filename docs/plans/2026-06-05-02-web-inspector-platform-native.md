# H1 — Implement the `ext_web_inspector` platform adapter layer (native, all OSes)

**Severity:** HIGH (fake success on shipped SDK ops) · **Source:** Fix.md › H1 · **Crate:** `ext_web_inspector`

## Goal
Make `open_devtools`, `close_devtools`, `is_devtools_open`, `inject_panel`, `send_cdp_message`, and `on_cdp_event` perform the real native operation on each platform. Until a given method is genuinely implemented, it must return a typed `WebInspectorError`, never `Ok(())`/fake `CdpResponse::success`.

## Root cause (verified)
`crates/ext_web_inspector/src/platform/mod.rs` — all three adapters are no-ops that report success:
- **WebKit (macOS):** `inject_panel` returns `Ok(())` (244); `send_cdp_message` returns fake success for `*.enable`/`*.disable`, else error (258–266); `is_devtools_open` → `false` (273–275); `open_devtools`/`close_devtools` → `Ok(())` no-op (277–286); `on_cdp_event` empty (269–271).
- **WebView2 (Windows):** identical pattern — `send_cdp_message` 361–381, `is_devtools_open` 387–389, `open_devtools`/`close_devtools` 391–399.
- **WebKitGTK (Linux):** identical pattern — `send_cdp_message` ~473, inspector calls ~453/488/494.

The comments already name the exact native APIs to call.

## Affected files
- `crates/ext_web_inspector/src/platform/mod.rs` (all three adapter impls)
- Possibly new files: `platform/macos.rs`, `platform/windows.rs`, `platform/linux.rs` if splitting per-OS keeps `#[cfg]` clean.
- `crates/ext_web_inspector/Cargo.toml` (add native deps: `objc2`/`objc2-foundation` for macOS, `webview2-com`/`windows` for Windows, `webkit2gtk` for Linux — check which wry/tao already pulls in to avoid duplication).
- Read `crates/forge-runtime` window/webview plumbing to obtain the native WebView handle per `window_id` (the adapter needs a handle to the real `WKWebView`/`ICoreWebView2`/`WebKitWebView`).

## Implementation steps
1. **Handle acquisition (prerequisite):** The adapters currently only have `window_id: &str`. Determine how to map `window_id` → native webview handle. Inspect `ext_window`/`ext_webview` `WindowManager` for an accessor; if none exists, add one that returns the raw platform webview pointer. This is the load-bearing dependency — do it first.
2. **macOS / WebKit:**
   - `open_devtools`: enable `WKWebView` developer extras (`WKPreferences` `setValue:forKey:@"developerExtrasEnabled"`) and show the inspector via the private `_inspector` `show` selector (the same mechanism wry exposes via `WebView::open_devtools`; prefer wry's API if reachable rather than private selectors).
   - `close_devtools`: hide/close the inspector.
   - `is_devtools_open`: query inspector visibility.
   - `send_cdp_message`: WebKit speaks the Web Inspector protocol, not CDP. Map the small CDP subset actually used (Runtime/Log/Debugger enable, evaluate) to `evaluateJavaScript:` + Web Inspector where possible; for unmapped methods return `WebInspectorError::cdp_error`.
   - `inject_panel`: run the `browser.devtools.panels.create` script (already drafted in the comment, lines 226–242) via `evaluateJavaScript:`.
3. **Windows / WebView2:** WebView2 supports CDP natively. Implement `send_cdp_message` via `ICoreWebView2.CallDevToolsProtocolMethod`, `open_devtools` via `OpenDevToolsWindow()`, `on_cdp_event` via `ICoreWebView2DevToolsProtocolEventReceiver`. `is_devtools_open` has no direct API — track open state locally from `OpenDevToolsWindow` calls.
4. **Linux / WebKitGTK:** `open_devtools`/`close_devtools` via `webkit_web_inspector_show()`/`close()` on `webkit_web_view_get_inspector()`; `is_devtools_open` via inspector visibility; CDP is not native — map the used subset or return errors.
5. **No fake success:** every method that cannot yet do the real thing returns `WebInspectorError::platform_unsupported`/`cdp_error`. Remove the `Ok(())`/`CdpResponse::success(...)` stubs.

## Regression test (mandatory)
Native GUI calls can't run in CI headless, so split testable logic out:
- Unit-test the CDP method-mapping function (pure: `&str method` → mapped action enum / error) for both "mapped" and "unmapped returns error" cases — asserting unmapped methods now yield `Err`, not fake success.
- Add a test asserting the adapters' default/unimplemented methods return `Err(WebInspectorError::…)` rather than `Ok` (guards against regression to fake success).
- Gate any test needing a real webview behind `#[ignore]` with a comment on how to run it manually (`forge dev` + open DevTools).

## Done criteria
- No `Ok(())`/`CdpResponse::success` remains for an unimplemented path (`grep -n "For now" crates/ext_web_inspector/src/platform/mod.rs` returns nothing).
- `cargo build -p ext_web_inspector` on macOS, Windows, Linux (CI matrix) all compile.
- `cargo clippy -p ext_web_inspector -- -D warnings` clean.
- Manual: `forge dev examples/example-deno-app`, call `runtime:web_inspector` open DevTools → inspector actually appears (per OS).

## Notes / risks
Largest plan in the set; native FFI per OS. Strongly prefer reusing wry/tao's existing devtools hooks over private selectors. If handle acquisition (step 1) proves infeasible without large window-manager surgery, land the "honest error" conversion first (still removes the lie) and track native impl as a follow-up — but the chosen direction is full native impl, so treat that as the target.
