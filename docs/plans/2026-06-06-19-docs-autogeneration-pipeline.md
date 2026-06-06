# Plan: Self-maintaining documentation pipeline for the Forge site

**Date:** 2026-06-06
**Status:** Proposed
**Author:** planning session (Claude)
**Related:** `/Users/ryanoboyle/forge/Site.md` (the drift audit that motivated this), `2026-06-05-15-forge-smelt-implement.md`, `2026-06-05-16-ext-console-implement.md`

---

## 1. Problem

The docs site (`site/src/content/docs/`) is **hand-maintained** and drifts from the code on every feature landing. The `Site.md` audit found, in one snapshot:

- 2 shipped crates with **no page** (`ext_console`/`runtime:console`, `forge-smelt`)
- A CLI command (`forge smelt`) missing from the reference
- **Method-level drift**: `runtime:dock` missing `info`/`nextMenuEvent`/`onMenuItemClick`; `runtime:shell` documenting a renamed `setCwd` that is now `chdir`, and missing `kill`
- Stale hardcoded **counts** ("27+ extensions", "28 extensions", "30+ crates" vs real 37/37/43/36)
- A stale **roadmap** and three conflicting **version** strings

Worse, the *existing* generator is itself drift-prone: `forge docs` (`crates/forge_cli/src/docs.rs:94-126`) is driven by a **hardcoded `EXTENSIONS` array** that is missing the same crates the site is missing (`console`, `dock`, `image_tools`, `encoding`, `svelte`, `web_inspector`, `codesign`), and **nothing wires its output into the site**. Two parallel, divergent doc systems.

## 2. Goal (from the planning Q&A)

Take doc maintenance **off the devs** and automate it: the site's reference content is **generated from a single source of truth on each update**, and drift is **impossible to merge** because it is enforced at three layers (CI, pre-commit, Rust build/test). Coverage spans API reference, CLI reference, counts/index, crate-page existence, and the conceptual material (scope, usage, installation, how-tos, examples).

## 3. Governing principle (non-negotiable — prevents fabrication)

> **A generator may transform, assemble, and validate content. It may never invent prose.**

Code signatures yield *reference* (method lists, types, op tables, counts, command flags). They cannot yield *conceptual narrative* (why a module exists, install steps, guides, example walkthroughs). Therefore every doc has **exactly one authored source**, and the pipeline only moves/checks it:

| Content kind | Single source of truth | Generated into site by |
|---|---|---|
| API method/type/op reference | `sdk/runtime.*.ts` (weld-generated) + rustdoc on ops in `crates/ext_*/src/lib.rs` | forge-etch extraction |
| API/module prose (overview, capabilities, `@example`) | rustdoc `//!` + TS JSDoc `@example` in the owning crate | forge-etch (already parses `@example`/`@param`/`@returns` — `crates/forge-etch/src/js_doc.rs`) |
| Crate page | crate-level `//!` doc + `Cargo.toml` metadata | crate-doc generator |
| Counts & module index | filesystem scan of `crates/ext_*` + registry specifiers | index generator |
| CLI reference | a new **declarative command model** in `forge_cli` | CLI-doc generator |
| Example pages | a co-located `examples/<app>/doc.md` (+ auto file/manifest tables) | example generator |
| Standalone guides / getting-started / architecture / roadmap | **authored Markdown sources** kept in-repo (e.g. `docs/site-src/` or retained in place and marked authored) | copied/validated, not synthesized |

**Migrate, never regenerate-over.** Existing rich prose (the `runtime:dock` examples, `guides/code-signing.md`, example overviews) is *moved* into its source location as part of migration. The first `--all` generation run must not delete authored content that has no source yet.

## 4. Key architectural decisions (resolved)

### 4.1 Committed-generated, not gitignored build-output
The AskUserQuestion preview text said "not committed," but that was illustrative, not a decision. **I am overriding it: generated pages are committed to the repo.** Rationale:
- Enables the canonical, simplest enforcement: `regenerate && git diff --exit-code` fails CI when the committed output is stale. Works identically in CI, pre-commit, and `cargo test`.
- Enables **incremental migration**: generate the pages whose source already exists, keep hand-authored pages for the rest, convert page-by-page. A gitignored/build-output model forces a big-bang migration of *all* prose before the old files can be deleted.
- Keeps the site buildable/browsable from a clean checkout with no Rust toolchain (contributors editing only prose-in-Markdown sources still see rendered output in PRs).

### 4.2 Phased, value-first — not all-or-nothing
Phase 1 (the drift gate) is the audit logic from `Site.md` turned into an executable check. It catches **everything found today** and ships before any generator exists. Generation is layered on after.

### 4.3 CLI reference via a full clap-derive migration (late, highest-risk phase) — DECIDED
`forge_cli` currently hand-parses `env::args()` (no clap — `crates/forge_cli/src/main.rs:1262,1289,1328`). **Decision: migrate to `clap` derive** so the `#[derive(Parser/Subcommand)]` structs become the single source of truth that both arg-parsing *and* `forge.md` generation read (clap exposes its metadata for doc-gen, and `clap_mangen`/help output can seed the reference).

This is the only phase that **changes runtime behavior** — it rewrites all argument parsing in `main.rs` — so it is sequenced **last** and carries the heaviest test burden. Because it is a behavior change, it must be **byte-for-byte CLI-compatible** with today's hand-parser: same subcommand names, same flags/aliases (`--identity`/`-i`, `--out`, `--embed`, `--all-extensions`, `--extension`/`-e`, `--output`/`-o`, `--format`/`-f`), same positional semantics, same error/usage text where scripts may depend on it. Mitigation: capture a **golden snapshot of current CLI behavior** (every subcommand's parse result + help/usage text) *before* the migration, and assert the clap version reproduces it exactly.

---

## 5. Phased implementation

### Phase 1 — `docs-check` drift gate (no generation; ships first)
A Rust binary/test (`crates/forge-docs-check/` or a `tests/docs_sync.rs` in a small new crate) that mechanizes the `Site.md` audit:

1. **Crate-page existence** — every `crates/ext_*` and `forge-*` workspace member has `site/src/content/docs/crates/<page>.md`.
2. **API method drift (bidirectional)** — for each `sdk/runtime.*.ts`, every public export (incl. `export async function*`) is documented, and every `### method(` heading still exists as an export. (Fixes the false-positive on async generators that the audit hit, and catches renames like `setCwd`→`chdir`.)
3. **Counts** — derive `#ext_* = 37`, `#crates = 43`, `#sdk modules = 36`, `#registered specifiers = 37` and assert the numbers embedded in `architecture.md`/`internals.md`/`roadmap.md` match (parse them from the prose by labeled marker comments, see Phase 3 markers).
4. **CLI command presence** — the subcommand list (`dev/build/bundle/smelt/sign/icon/docs`) each appears as a heading in `crates/forge.md`.
5. **`forge docs` self-consistency** — assert `docs.rs` `EXTENSIONS` covers every `crates/ext_*` (until Phase 2 makes it dynamic).

Output: a precise punch-list and non-zero exit on drift.

**Wired into all three enforcement layers in this phase** (the check exists before generation, so enforcement delivers value immediately):
- **CI**: a job in `.github/workflows/` runs `cargo run -p forge-docs-check` on PRs (aligns with main being CI-gated under `enforce_admins` — see memory `forge-main-branch-protection`).
- **Pre-commit**: a `scripts/docs-check.sh` + a documented git hook (and/or a `make docs-check` target).
- **Rust build/test**: the check is also exposed as `#[test] fn docs_in_sync()` so `cargo test --workspace` fails on drift.

### Phase 2 — Dynamic `forge docs` + API/crate generation into the site
1. **Make `forge docs` dynamic** (`crates/forge_cli/src/docs.rs`): replace the hardcoded `EXTENSIONS` array with discovery — scan `crates/ext_*` and parse each `build.rs` for `ExtensionBuilder::new("<runtime_x>", "<specifier>")` to recover `(name, specifier)` pairs. Result: `forge docs` can never miss a new extension again. Phase 1's check #5 then becomes redundant and is removed.
2. **Starlight-target output**: ensure forge-etch's Astro emitter produces the exact frontmatter the site uses (`title` / `description` / `slug: api/runtime-<name>`) — verify against `crates/forge-etch/src/astro` + `crates/forge-etch/src/mod.rs` slug helpers; extend if needed.
3. **API page generator**: `forge docs --all-extensions --format astro -o site/src/content/docs/api` produces `runtime-*.md` from `sdk/runtime.*.ts` + rustdoc/JSDoc. Run for the subset whose source prose has been migrated (Phase 3 gates which pages are "generated" vs "authored-legacy").
4. **Crate page generator**: new emitter for `crates/*.md` from crate-level `//!` docs + `Cargo.toml` (name, version → fixes the version-string drift at the source).
5. **Index/counts generator**: emits the count values and module-list tables as an includable fragment or via marker-delimited regions.

The `git diff --exit-code` gate from Phase 1 now also guards generated output (regenerate in CI; fail if committed copy differs).

### Phase 3 — Prose migration into single sources (incremental, page-by-page)
This is the bulk of the human effort, done per module so the site is never broken:
1. For each `runtime:*` page, move authored prose (overview, capabilities notes, **complete examples**) into the owning crate as rustdoc `//!` and TS JSDoc `@example` blocks, then flip that page to "generated" and delete the hand-authored copy. **Preserve content verbatim** — this is a move, not a rewrite.
2. Introduce **marker-delimited regions** for counts/tables in `architecture.md`/`internals.md` (`<!-- forge:count:ext_crates -->…<!-- /forge:count -->`) so the generator can update just the numbers inside otherwise-authored prose.
3. Reconcile `roadmap.md`: add `runtime:console`, remove/merge the stale "In Progress" phase sections, and pull "Complete" status from the generated module index.
4. Resolve the version source of truth → `0.1.0-alpha.1` from `Cargo.toml`, surfaced into generated crate pages.

### Phase 4 — Example pages from co-located sources
1. Add `examples/<app>/doc.md` (authored overview/walkthrough) to each of the 10 example apps.
2. Example generator assembles: authored `doc.md` + auto-extracted tables (file tree, `manifest.app.toml` window/permission summary, `runtime:*` imports detected in `src/`) → `site/src/content/docs/examples/<app>.md`.
3. Drift check gains: every `examples/<app>` dir has a `doc.md` and a generated page; every generated example page's app dir still exists.

### Phase 5 — CLI reference via clap-derive migration (highest risk, last)
1. **Snapshot current behavior first**: write characterization tests that record, for every subcommand, the parsed config and the `--help`/usage text the hand-parser produces today. This is the safety net the migration is checked against.
2. **Migrate `forge_cli` to `clap` derive**: add `clap` (+ `clap_mangen` for man-page/markdown help) to `crates/forge_cli/Cargo.toml`; define `#[derive(Parser)]` `Cli` and `#[derive(Subcommand)]` `Commands` covering `dev/build/bundle/smelt/sign/icon/docs` with every existing flag and alias. Replace the `env::args()` dispatch in `main.rs` (`:1262,1289,1328`) with clap parsing. Keep behavior byte-compatible (§4.3).
3. **CLI-doc generator** emits the `crates/forge.md` Commands section from the clap command model (via clap's introspection / `clap_mangen` markdown), so a new subcommand or flag updates the reference automatically.
4. **Tests**: the §5.1 snapshots must still pass post-migration; a `docs-check` rule asserts every clap subcommand appears in `forge.md` and vice-versa.

---

## 6. New/changed components (file-level)

| Path | Change |
|---|---|
| `crates/forge-docs-check/` (new) | Drift-gate binary + `docs_sync` test (Phase 1). Houses the bidirectional SDK diff, crate/example existence, count derivation, CLI presence. |
| `.github/workflows/docs.yml` (new) | CI job: run docs-check + (Phase 2+) `regenerate && git diff --exit-code`. |
| `scripts/docs-check.sh` + `scripts/install-hooks.sh` (new) | Pre-commit hook + local `make docs-check`. |
| `Makefile` / `justfile` (new or edit) | `docs-check`, `docs-gen` targets. |
| `crates/forge_cli/src/docs.rs` | Replace hardcoded `EXTENSIONS` with discovery (Phase 2). |
| `crates/forge-etch/src/astro*` | Ensure Starlight-exact frontmatter/slug; add crate-page + index emitters (Phase 2). |
| `crates/ext_*/src/lib.rs`, `crates/ext_*/ts/init.ts` | Receive migrated prose as `//!`/JSDoc `@example` (Phase 3, incremental). |
| `site/src/content/docs/api/*`, `crates/*` | Become generated+committed (incrementally). |
| `examples/*/doc.md` (new ×10) | Authored example narratives (Phase 4). |
| `crates/forge_cli/src/main.rs` + `Cargo.toml` | clap-derive migration: `#[derive(Parser/Subcommand)]` model replaces hand-parser; add `clap` + `clap_mangen` (Phase 5). |
| `site/src/content/docs/{architecture,internals,roadmap}.md` | Marker regions for generated counts/tables (Phase 3). |

## 7. Risks & mitigations

- **Fabrication risk** → governing principle §3: generators assemble/validate only; prose has an authored source. Reviewer checklist item: "no generated narrative."
- **Destroying good prose on first `--all` run** → migrate-then-flip per page; Phase 1 ships before any generation; generation only targets pages whose source exists.
- **CLI parser regression (elevated — clap migration chosen)** → isolated to Phase 5, sequenced last; characterization snapshots of current parse results + help text captured *before* migrating; clap version must reproduce them byte-for-byte (subcommands, flags, aliases, positionals, usage text). This is the only behavior-changing phase and gets the heaviest test burden.
- **forge-etch output ≠ Starlight expectations** → Phase 2 step 2 verifies/extends frontmatter before any page is flipped; diff generated vs current `runtime-fs.md` as the golden reference.
- **Generated-vs-committed churn in PRs** → restrict generation to deterministic output (stable ordering, no timestamps) so `git diff` is clean; document `make docs-gen` in CONTRIBUTING.
- **forge-etch can't yet emit crate/index pages** → those emitters are net-new work in Phase 2, not assumed to exist.

## 8. Testing strategy

- Phase 1: unit tests for each drift rule using fixtures (a fake crate with/without a page; a renamed export); the `docs_sync` integration test runs against the real tree.
- Phase 2: golden-file test — generated `runtime-fs.md` matches a checked-in expected output; `forge docs` discovery test asserts it finds all 37 `ext_*`.
- Phase 4: example-generator test against `examples/react-app`.
- Phase 5: CLI table↔dispatch parity test; snapshot of generated `forge.md`.
- End-to-end: `cargo test --workspace` + `cd site && npm run build` both green in CI.

## 9. Definition of done

1. Adding a new `ext_*` crate with rustdoc/JSDoc and rebuilding produces its API + crate pages with **zero manual site edits**, and CI fails if the dev forgot to regenerate.
2. Renaming/removing an op fails `docs-check` until docs regenerate.
3. Counts, version, CLI commands, crate-page existence, and example-page existence are all enforced.
4. All existing authored prose is preserved (migrated, not lost).
5. The `Site.md` punch-list is fully closed and **cannot recur** without a red build.

## 10. Resolved decisions (planning Q&A, 2026-06-06)

- **Standalone guides** (`getting-started`, `architecture`, `internals`, `roadmap`, `guides/*`): **kept authored in-place** in `site/src/content/docs/`, with marker-delimited regions (`<!-- forge:count:* -->…<!-- /forge:count -->`) so the generator only rewrites derived bits (counts, tables) inside otherwise hand-written prose. (Phase 3.)
- **CLI reference model**: **full clap-derive migration** of `forge_cli` (see §4.3, Phase 5). Chosen over a declarative table for the cleaner long-term, idiomatic result; the larger arg-parsing diff is contained by the §5.1 characterization snapshots.
- **Tooling home**: a **standalone workspace crate** `crates/forge-docs-check` (testable via `cargo test -p forge-docs-check`, runnable as a binary), not an `xtask`.

No open decisions remain blocking. Implementation can begin at Phase 1.
