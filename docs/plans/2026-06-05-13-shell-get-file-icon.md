# M6 — Implement `shell.getFileIcon` natively (macOS/Windows/Linux)

**Severity:** MEDIUM (stubbed on every platform) · **Source:** Fix.md › M6 · **Crate:** `ext_shell`

## Goal
`op_shell_get_file_icon` returns real icon image bytes (e.g. PNG) for a given path at the requested size on each platform.

## Root cause (verified)
`crates/ext_shell/src/lib.rs:998–1033` — computes `_size` (then unused) and returns `ShellError::not_supported("…requires additional native bindings")` on macOS, Windows, and Linux, plus an `unreachable_code` fallback. Honest, but the op never works anywhere.

## Affected files
- `crates/ext_shell/src/lib.rs` (`op_shell_get_file_icon`, 990–1033)
- `crates/ext_shell/Cargo.toml` (per-OS deps)
- Return type: confirm the op's declared return (bytes vs base64 string) and keep it; only fill in real data.

## Implementation steps
1. **Use the computed `size`** (currently `_size`, line 999) for the requested icon dimension.
2. **macOS:** `NSWorkspace.sharedWorkspace.iconForFile:` → `NSImage`, set size, get `TIFFRepresentation`/`CGImage`, encode to PNG (`objc2-app-kit`). Return bytes.
3. **Windows:** `SHGetFileInfoW` with `SHGFI_ICON` (+`SHGFI_LARGEICON`/`SHGFI_SMALLICON` per size) → `HICON`; convert `HICON` → bitmap → PNG (`windows` crate + an image encoder). Destroy the `HICON`.
4. **Linux:** resolve the icon via the freedesktop icon theme — query the file's MIME type (`xdg-mime`/`mime_guess`) then look up the theme icon (`freedesktop-icons`/`linicon` crate) at the requested size; read the PNG/SVG. For SVG, rasterize to PNG at `size`.
5. Replace each platform's `not_supported` error with the real implementation; keep a genuine error only for truly unsupported cases (e.g. file not found → `ShellError::not_found`).
6. Respect the capability check already present (990–996) — keep it.

## Regression test (mandatory)
- Per-OS, gated by `#[cfg(target_os = …)]`: request the icon for a known-existing path (e.g. the current executable or `/` on macOS, `C:\\Windows` on Windows) and assert the returned bytes are non-empty and begin with the PNG magic (`\x89PNG`). This proves real data vs the old `not_supported` error.
- Assert a non-existent path returns a typed error, not a panic.

## Done criteria
- All three platforms return real icon bytes; no blanket `not_supported` remains; `_size` is used.
- `cargo build -p ext_shell` on the CI matrix compiles; `cargo clippy -p ext_shell -- -D warnings` clean.
- `cargo test -p ext_shell` passes incl. the per-OS icon test.

## Notes / risks
Adds image-encoding deps; reuse `ext_image_tools`' encoder if it already pulls in `image` to avoid duplication. Linux theme resolution is the fiddliest — pick a maintained crate and fall back to a generic MIME icon when a specific one is missing (a generic icon is still real data, not a stub).
