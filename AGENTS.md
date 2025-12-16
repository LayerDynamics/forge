# Repository Guidelines

## Project Structure & Module Organization
- Rust workspace under `crates/`; main entry crates: `crates/forge_cli` (CLI package name `forge_cli`, binary `forge`) and `crates/forge-runtime` (runtime). Capability extensions live in `crates/ext_*`; codegen in `crates/forge-weld*`.
- TypeScript SDK in `sdk/`; generated bindings in `sdk/generated/` (do not hand-edit). Example apps in `examples/`.
- Docs in `docs/`; Astro site in `site/`. App UIs live in `web/` folders loaded via `app://` with permissions in `manifest.app.toml`.

## Build, Test, and Development Commands
- Build workspace: `cargo build --workspace` (add `--release` for optimized artifacts).
- Run tests: `cargo test --workspace`; scope to a crate with `cargo test -p ext_net`.
- CLI smoke: `cargo run -p forge_cli -- dev examples/example-deno-app` to validate runtime + CLI integration.
- Docs site: `cd site && npm install && npm run dev` (build with `npm run build`).

## Coding Style & Naming Conventions
- Rust: format with `cargo fmt`; lint via `cargo clippy --workspace --all-targets` and fix warnings when feasible. Prefer small modules and `snake_case` files.
- TypeScript/Deno: adhere to `deno.json` strictness; use `camelCase` for functions/vars and `PascalCase` for types. Keep SDK exports stable and avoid manual edits in `sdk/generated/`.
- Config/manifests: TOML/JSON with consistent indentation (4 spaces for TOML/Rust, 2 for JSON).

## Testing Guidelines
- Add focused unit tests alongside crates (`src/lib.rs` or `tests/`). Keep deterministic; prefer temp dirs and avoid network calls unless guarded.
- Cover runtime capability or binding changes with targeted crate tests or an `examples/` app flow. When touching app permissions, ensure `manifest.app.toml` reflects needs.

## Commit & Pull Request Guidelines
- Commits: short, imperative (e.g., `Add wasm reftype support`); reference issues/PRs with `#123` when applicable.
- PRs: include a brief summary, tests run, and any docs updates (note `docs/` or `site/` edits). Add screenshots/gifs for UI tweaks. Ensure formatted code, passing tests, and no unintended changes in `sdk/generated/`.

## Security & Configuration Tips
- Respect capability model; only expand `manifest.app.toml` permissions when required.
- Do not commit secrets. Keep scripts in `scripts/` portable and checksum-verifiable where possible.

## Known Placeholders to Implement
- `crates/forge-runtime/src/main.rs`: packaged-mode detection is marked TODO; review runtime packaging docs and runtime expectations, then implement detection logic consistent with existing capability gating and CLI flags.
- Extension integration: ensure every `ext_*` crate is wired into `forge-runtime`, all exposed methods are bound via `forge-weld` so `runtime:*` imports stay typed, and matching SDK surfaces land in `sdk/` (regenerate `sdk/generated/` as needed, not hand-edited).

## Runtime Notes
- macOS may log benign XPC warnings (e.g., `scheduleApplicationNotification` / `Connection invalid` for `com.apple.hiservices-xpcservice`) when launching examples in dev mode; they do not affect app behavior.
