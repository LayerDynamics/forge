# L4 — Review/document the by-design `ext_codesign` Linux stub

**Severity:** LOW (review/doc task — likely no code change) · **Source:** Fix.md › L4 · **Crate:** `ext_codesign`

## Goal
Confirm the Linux codesign stub is intentional and honest, and ensure the SDK/docs clearly state Linux code signing is unsupported so callers aren't surprised. This is a **verification + documentation** task, not a stub-removal — included for completeness per the "cover all severities" decision.

## Root cause (verified)
`crates/ext_codesign/src/os_linux.rs` is explicitly documented as "Linux stub implementation for code signing" that "provides stub implementations that return appropriate errors." Returning typed errors is honest (Linux has no single native codesign equivalent like macOS `codesign` / Windows `signtool`). This is **not** a fake-success lie.

## Steps
1. Read `crates/ext_codesign/src/os_linux.rs` in full and confirm every public op returns a typed `Err` (no `Ok(())`/fake-success path). If any op silently returns success, that one *is* a real bug — escalate it out of this plan into a HIGH-severity fix.
2. Confirm the error messages are actionable (e.g. "code signing is not supported on Linux; use GPG-detached signatures / AppImage signing externally").
3. Update docs:
   - `sdk/runtime.codesign.ts` / `crates/ext_codesign/ts/init.ts` — mark Linux as unsupported in the op doc comments.
   - The codesign API docs under `site/`/`docs/` if present.
4. Optional enhancement (only if desired): implement a real Linux signing path that matches the platform's norms — GPG detached signature for the artifact, or `appimagetool --sign` for AppImages. If pursued, that becomes its own implementation plan with a regression test; otherwise leave as documented-unsupported.

## Regression test (mandatory only if behavior changes)
- If step 1 finds a fake-success op: add a test asserting it now returns the typed error.
- If the optional Linux signing is implemented: add a test that signs a fixture and verifies the signature.
- If purely documentation: no test required (no behavior change) — note this explicitly in the PR.

## Done criteria
- Confirmed: every Linux codesign op returns an honest typed error (or real impl).
- SDK/docs state Linux codesign support status clearly.
- No change to passing builds: `cargo clippy -p ext_codesign -- -D warnings` clean.

## Notes / risks
This is the one finding expected to need **no functional code change** — its value is verifying the "honest error vs silent lie" classification holds and that users are told. If verification surprises (a hidden fake-success), reclassify and fix.
