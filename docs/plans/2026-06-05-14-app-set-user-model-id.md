# M7 — Implement `app.setUserModelId` on Windows

**Severity:** MEDIUM (errors on its only relevant platform) · **Source:** Fix.md › M7 · **Crate:** `ext_app`

## Goal
`op_app_set_user_model_id` calls `SetCurrentProcessExplicitAppUserModelID` on Windows so taskbar grouping / toast notifications attribute to the app correctly. Non-Windows stays a legitimate no-op.

## Root cause (verified)
`crates/ext_app/src/lib.rs:901–924` — on `target_os = "windows"` returns `Err(AppError::not_supported("User model ID requires Windows-specific implementation"))`; comment names the exact API (`SetCurrentProcessExplicitAppUserModelID`, 913). Non-Windows is already a correct `Ok(())` no-op (921–922).

## Affected files
- `crates/ext_app/src/lib.rs` (`op_app_set_user_model_id`, 901–924)
- `crates/ext_app/Cargo.toml` (add `windows` crate with the `Win32_UI_Shell` feature, under `[target.'cfg(windows)'.dependencies]`)

## Implementation steps
1. Add the `windows` crate (Shell feature) as a Windows-only dependency.
2. In the `#[cfg(target_os = "windows")]` branch, convert `_app_id` (rename to `app_id`) to a wide string and call `windows::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID(PCWSTR)`. Map a failing `HRESULT` to `AppError` with the HRESULT in the message; return `Ok(())` on `S_OK`.
3. Remove the `not_supported` error. Keep the non-Windows `Ok(())`.

## Regression test (mandatory)
- Windows (`#[cfg(target_os = "windows")]`): call the op with a valid AppUserModelID string and assert it returns `Ok(())`. Optionally call `GetCurrentProcessExplicitAppUserModelID` and assert it round-trips the value (strongest proof). Pre-fix this returns `Err`; post-fix `Ok`.
- Non-Windows: assert it returns `Ok(())` (unchanged).

## Done criteria
- Windows path calls the real API; no `not_supported` error remains there.
- `cargo build -p ext_app` on Windows compiles; `cargo clippy -p ext_app -- -D warnings` clean on the matrix.

## Notes / risks
AppUserModelID should ideally be set very early in process startup to fully affect taskbar grouping; note in the SDK doc that calling it after windows are shown may have limited effect. Validate the string is non-empty before the call.
