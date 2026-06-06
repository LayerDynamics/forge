# Site.md — Documentation Site Update Audit

**Subject:** What must be updated in the Forge documentation site (`/Users/ryanoboyle/forge/site`) to bring it in sync with the current codebase.

**Method:** The generated TypeScript SDK (`sdk/runtime.*.ts`), the extension registry (`crates/forge-runtime/src/ext_registry.rs`), the CLI command surface (`crates/forge_cli/src/main.rs`), and the crate list on disk were treated as ground truth and diffed against the Markdown under `site/src/content/docs/`. Every claim below carries a `file:line` citation.

> **Investigation only — no fixes applied.** This document enumerates the work; it does not perform it.

---

## 1. Executive Summary

The site is broadly accurate but trails three recent landings: **forge-smelt** (the AOT TS→JS compiler + `forge smelt` command, commit `3ac529b`/`3afab62`), **ext_console** (`runtime:console`, the most recent extension), and **dock menu click events** (M5, commits `ebefe41`/`3afab62`). Concretely that means **2 missing doc pages**, **1 missing CLI command**, and **method-level drift on 2 existing API pages**. On top of that, several **hardcoded counts** ("27+ extensions", "28 extensions", "30+ crates") and the **roadmap's "In Progress" sections** are stale, and the site carries **three conflicting version strings**.

### Ground-truth counts (authoritative, as of this audit)

| Metric | Value | Source |
|--------|-------|--------|
| `ext_*` crates | **37** | `ls crates/ext_*` |
| Total workspace crates | **43** | `ls crates/*/` |
| Generated runtime SDK modules | **36** | `sdk/runtime.*.ts` |
| Module specifiers registered | **37** (34 `runtime:*` + 3 `forge:*`) | `ext_registry.rs` |
| Authoritative version | **`0.1.0-alpha.1`** | `crates/forge_cli/Cargo.toml`, `crates/forge-runtime/Cargo.toml` |
| CLI subcommands | `dev`, `build`, `bundle`, `smelt`, `sign`, `icon`, `docs` | `crates/forge_cli/src/main.rs:108` |

---

## 2. NEW — Missing doc pages (highest priority)

These are features that exist in code with **zero coverage** on the site. Because `api/` and `crates/` are `autogenerate` sidebar groups (`site/astro.config.mjs:53-61`), creating the file is sufficient — no nav edit required.

### 2.1 `runtime:console` extension — completely undocumented
- **Code:** `crates/ext_console/` (37th ext crate); registered as `runtime:console` in `crates/forge-runtime/src/ext_registry.rs`; SDK at `sdk/runtime.console.ts`.
- **Missing crate doc:** `site/src/content/docs/crates/ext-console.md` — **does not exist**.
- **Missing API doc:** `site/src/content/docs/api/runtime-console.md` — **does not exist**.
- **Action:** Create both pages. Pull the public surface from `sdk/runtime.console.ts` and the op list / capability behavior from `crates/ext_console/src/lib.rs` and `crates/ext_console/build.rs`.

### 2.2 `forge-smelt` crate — completely undocumented
- **Code:** `crates/forge-smelt/` (workspace member); `forge_smelt::smelt(app_dir, out_dir)` and `forge_smelt::build::embed`. Wired into `forge build` (`crates/forge_cli/src/main.rs:847-863`, "Smelting TypeScript -> JavaScript") and exposed as `forge smelt` (`main.rs:114`, `:977`).
- **Missing crate doc:** `site/src/content/docs/crates/forge-smelt.md` — **does not exist**.
- **Action:** Create the crate page. Cover: the module-graph discovery from `src/main.ts`, `.ts`→`.js` specifier rewriting (leaving `runtime:*`/external untouched), how `forge-runtime` prefers a compiled `src/main.js` over `src/main.ts` in production while dev still loads `.ts` for HMR, and the standalone-binary embedding groundwork.

---

## 3. CLI command surface drift

### 3.1 `forge smelt` missing from the CLI reference
- **File:** `site/src/content/docs/crates/forge.md` — the `## Commands` section (`:35`–`:107`) documents `dev`, `build`, `bundle`, `sign`, `icon` but **omits `smelt`**.
- **Truth:** `crates/forge_cli/src/main.rs:114` → `smelt <app-dir> [--out <dir>]`; full handler at `main.rs:977` (`forge smelt <app-dir> [--out <dir>] [--embed]`).
- **Action:** Add a `### \`forge smelt\`` subsection documenting `--out` and `--embed`.

### 3.2 `forge build` now AOT-compiles — description is stale
- **File:** `site/src/content/docs/crates/forge.md:51-65` (`### forge build`) and the build-pipeline prose.
- **Truth:** `forge build` now smelts the app's TypeScript to JavaScript before bundling so the shipped bundle contains compiled JS (`crates/forge_cli/src/main.rs:847-863`).
- **Action:** Note that production bundles ship compiled `.js` (no launch-time transpile) and cross-link the new `forge-smelt` page.

### 3.3 Getting Started production section (optional but recommended)
- **File:** `site/src/content/docs/getting-started.md:368-380` ("Building for Production") lists `forge build`/`bundle`/`sign` only.
- **Action:** Mention that `forge build` compiles TS→JS, and optionally document `forge smelt` as a standalone step.

---

## 4. Method-level API drift (existing pages, missing functions)

Diffing each `sdk/runtime.<mod>.ts` export against its `api/runtime-<mod>.md` page surfaced two drifted pages. (Generic hook/handler plumbing — `onBefore`/`onAfter`/`onError`/`registerHandler`/etc. — was excluded from the diff.)

### 4.1 `runtime:dock` — missing `info`, `nextMenuEvent`, `onMenuItemClick` (M5)
- **File:** `site/src/content/docs/api/runtime-dock.md`.
- **Truth:** `sdk/runtime.dock.ts` exports `info`, `nextMenuEvent`, `onMenuItemClick`; backed by ops `op_dock_info`, `op_dock_next_menu_event`, `op_dock_set_menu` (`crates/ext_dock/build.rs`). These are the **M5 "dock setMenu + click events"** landing (`ebefe41`, `3afab62`).
- **Gap:** `setMenu` itself **is** documented (`runtime-dock.md:258`), but the **click-event delivery** (`onMenuItemClick` / `nextMenuEvent`) and `info()` are **absent** (`grep` for both returns 0 hits).
- **Action:** Add API entries + an example showing how to receive dock-menu item clicks; also update `### onBefore(...)` "Available operation names" list (`runtime-dock.md:355`) if `info`/menu-event ops should appear there. Re-check the crate page `site/src/content/docs/crates/ext-dock.md` for the same gap.

### 4.2 `runtime:shell` — stale **rename** (`setCwd`→`chdir`) + missing `kill`
- **File:** `site/src/content/docs/api/runtime-shell.md`.
- **Truth:** `sdk/runtime.shell.ts` exports `chdir` (`:689`) and `kill` (`:628`). The doc has a `### setCwd(path)` section (`runtime-shell.md:526`), but **`setCwd` is not a public export** — it survives only as an internal op-name in the type union (`runtime.shell.ts:909,924`). The public "change working directory" function is now **`chdir(path)`**.
- **Actions:**
  - **Rename** the `### setCwd(path)` section (`runtime-shell.md:526`) to `### chdir(path)` to match the export.
  - **Add** a `### kill(handle, signal?)` section — entirely undocumented.
  - Cross-check op count against `crates/ext_shell/build.rs`; `roadmap.md:114` still lists `runtime:shell` as "7 operations" — verify after these edits.

### 4.3 Reverse-drift result (phantom methods) — clean except §4.2
A reverse diff (documented `### method(` headings → must exist as an SDK export) was run across all API pages. Apparent hits for `windowEvents`/`channelEvents`/`windowEventsFor` (`runtime-ipc.md`, `runtime-window.md`) are **false positives**: those are exported as `export async function*` async generators (`runtime.ipc.ts:114,137,162`; `runtime.window.ts:528`) and are correctly documented. The **only genuine phantom** is shell `setCwd()` (covered in §4.2). No other API page documents a method that no longer exists.

> **Caveat on the §4 forward diff:** the undocumented-export check tests *presence of the function name as a word* in the page, not signature correctness, and can give a false "OK" for common-word names (`get`, `set`, `info`, `show`, `clear`). So the "25 clean pages" claim is name-level, not signature-level — a signature-accuracy pass is tracked in §9.1.

---

## 5. Stale hardcoded counts

| File:line | Current text | Should be |
|-----------|--------------|-----------|
| `site/src/content/docs/architecture.md:522` | "30+ crates" | **43** crates |
| `site/src/content/docs/architecture.md:626` | "27+ extension crates" | **37** ext crates |
| `site/src/content/docs/architecture.md:642` | "27+ runtime modules" | **36** SDK modules (34 `runtime:*` + 3 `forge:*` registered) |
| `site/src/content/docs/internals.md:199` | "Defines all 28 extensions" | registry registers **37** module specifiers (~39 descriptors) |
| `site/src/content/docs/internals.md:712` | "Build Deno JsRuntime with 28 extensions" | same correction |

- **Also verify:** `internals.md:199` cites `create_all_descriptors()` at "lines 202-428" — confirm that line range still matches `crates/forge-runtime/src/ext_registry.rs` after recent additions.

---

## 6. Stale roadmap

**File:** `site/src/content/docs/roadmap.md`.

1. **`runtime:console` omitted** from the "New Extensions (Steel Donut Release)" enumeration (`:43-64`). The header claims "40+ implemented extension modules" (`:11`) but the table lists 36 and skips `console`. **Action:** add a `runtime:console` row.
2. **Internally contradictory "In Progress" sections.** `:162` "Phase 1: High Priority Modules (In Progress)" and the Phase 2/3 sections (`:162`–`:305`) still describe `runtime:screen`, `runtime:globalShortcut`, `runtime:autoUpdater`, `runtime:database`, `runtime:protocol`, `runtime:theme` as not-yet-done — yet `ext_display`, `ext_shortcuts`, `ext_updater`, `ext_database`, `ext_protocol` are implemented, registered, and already have API + crate docs on the site. **Action:** reconcile or delete these stale phase sections so the roadmap doesn't contradict its own "Complete" table above.

---

## 7. Version inconsistency (site-wide)

Three different versions appear; the authoritative crate version is **`0.1.0-alpha.1`**.

| File:line | Claims |
|-----------|--------|
| `crates/forge_cli/Cargo.toml`, `crates/forge-runtime/Cargo.toml` | `0.1.0-alpha.1` ← authoritative |
| `site/src/content/docs/getting-started.md:63` | `0.1.0` |
| `site/src/content/docs/examples/example-deno-app.md:40` | `0.1.0` |
| `site/src/content/docs/roadmap.md:11` | `v1.0.0p-steel-donut 🍩` |
| `site/package.json` (version field) | `1.0.0` |

- **Action:** Pick one source of truth (recommend the Cargo `0.1.0-alpha.1`) and align user-facing docs. This is partly a product/release decision, not purely mechanical — flag for the maintainer.

---

## 8. Lower-priority / decide-intent items

### 8.1 SDK modules with no API page (likely intentional, confirm)
These have generated SDK modules but no `api/runtime-*.md` page. Most are build-time/tooling or internal and **already have crate pages** — confirm whether any should be promoted to app-facing API docs:
- `forge:*` build tooling: **bundler, codesign, etcher, weld** (crate docs exist: `ext-bundler.md`, `ext-codesign.md`, `ext-etcher.md`, `ext-weld.md`).
- `runtime:*` internal/diagnostic: **debugger, monitor, svelte, trace, web_inspector** (crate docs exist for all).
- **Note:** `runtime:console` is the one item in this list that is genuinely app-facing and new — already covered in §2.1.

### 8.2 `runtime:encoding` SDK/anomaly
- `runtime:encoding` is registered in `ext_registry.rs` and has a crate doc (`crates/ext-encoding.md`), but there is **no `sdk/runtime.encoding.ts`** generated and no API page.
- **Action:** Verify whether `ext_encoding` is meant to expose a TS SDK (i.e., does its `build.rs` call `.generate_sdk_module(...)`?). If yes, this is a codegen gap upstream of the docs; if no, the crate doc should state it's not directly importable from app code.

### 8.3 Homepage / components — verified OK (no action)
- `site/src/components/QuickStart.astro:16,21` uses `forge dev .` and `forge build . && forge bundle .` — still valid.
- `site/src/components/Features.astro` claims are generic (no stale counts/commands).

### 8.4 Examples — verified in sync (no action)
- 10 example directories on disk (`examples/*`) ↔ 10 example doc pages + `index.md`. Complete.

### 8.5 Crate-doc coverage — verified (only the two §2 gaps)
- 41 crate doc files vs 43 crates; `forge_cli` is documented as `crates/forge.md`. The only missing crate pages are **`ext-console.md`** and **`forge-smelt.md`** (§2).

---

## 9. Open questions (cannot be resolved from code alone)

1. **Signature-level accuracy:** The §4 diff confirms function *names* are present on the 25 "clean" pages but does not verify parameter lists / return types / option shapes match current `sdk/runtime.*.ts`. A signature-level pass per page is recommended but was out of scope for this name-presence audit.
2. **Intended API surface for internal modules (§8.1):** whether `debugger`/`monitor`/`trace`/`web_inspector`/`svelte` and the `forge:*` tooling should get app-facing API pages is a product decision.
3. **Canonical version (§7):** which version string is "correct" for the docs is a release decision.
4. **`runtime:encoding` SDK gap (§8.2):** whether the missing `sdk/runtime.encoding.ts` is intentional or a regression needs a maintainer's confirmation.

---

## 10. Prioritized action checklist

**Must do (new features shipped, zero coverage):**
- [ ] Create `site/src/content/docs/crates/ext-console.md`
- [ ] Create `site/src/content/docs/api/runtime-console.md`
- [ ] Create `site/src/content/docs/crates/forge-smelt.md`
- [ ] Add `forge smelt` to `crates/forge.md` Commands (`:35-107`); note `forge build` now smelts
- [ ] Add `info` / `nextMenuEvent` / `onMenuItemClick` to `api/runtime-dock.md` (M5 click events)
- [ ] Add `chdir` / `kill` to `api/runtime-shell.md`

**Should do (stale/contradictory):**
- [ ] Fix counts: `architecture.md:522,626,642`; `internals.md:199,712`
- [ ] Add `runtime:console` row + reconcile "In Progress" sections in `roadmap.md`
- [ ] Resolve version inconsistency (`getting-started.md:63`, `example-deno-app.md:40`, `roadmap.md:11`, `site/package.json`)

**Investigate (intent/codegen):**
- [ ] Decide API-page status for `forge:*` + internal `runtime:*` modules (§8.1)
- [ ] Confirm `runtime:encoding` SDK gap (§8.2)
- [ ] Signature-level re-verification of the 25 "clean" API pages (§9.1)
