---
title: "forge-smelt"
description: "Forge Smelt - ahead-of-time TypeScript to JavaScript compiler for Forge apps"
slug: docs/crates/forge-smelt
---

# forge-smelt — ahead-of-time TypeScript → JavaScript compiler for Forge apps

## Why this crate exists (Phase-0 decision)

A Forge app's logic lives in `src/main.ts` (plus its local `.ts` module
graph) and is executed by the embedded Deno runtime. Today that TypeScript is
shipped and run as **loose source**:

- `forge bundle` ([`forge_cli`]) embeds only the `web/` frontend assets into
  the `forge-runtime` binary (`build_embedded_binary`) and copies the app's
  `src/` directory into the package as raw `.ts` files.
- `forge-runtime` then loads `src/main.ts` from disk and transpiles it (and
  every imported module) **on every launch** (`main.rs`, `ForgeModuleLoader`).

Nothing in the workspace compiles the app's TypeScript ahead of time:
`ext_bundler` only handles icons/manifests, and `forge-weld`'s transpile is a
single-module helper. `forge-smelt` fills that gap — it "smelts" the raw TS
ore into a finished JavaScript ingot: parse the entry's module graph,
transpile each module to JS, and rewrite relative import specifiers so the
emitted `.js` tree is self-consistent and loadable with no further transpile.

## Scope

**Depth 1 (this crate): transpile-in-place.** Produce a compiled `.js` tree
mirroring `src/`, with relative `./x.ts` import specifiers rewritten to
`./x.js` and `runtime:*` / bare / URL specifiers left untouched (the runtime
and import maps resolve those). `forge bundle` ships this compiled tree and
`forge-runtime` prefers a compiled `src/main.js` when present — so bundled
apps stop shipping loose `.ts` and stop re-transpiling at launch, while dev
mode keeps loading `.ts` (HMR intact).

**Depth 2 (deferred): embed-in-binary.** Linking the compiled JS (and a V8
snapshot) directly into a single self-contained executable is a larger change
that requires a `forge-runtime` module-loader rewrite. It is intentionally
out of scope here and noted as a follow-up; the [`binary`] module produces the
materialized artifact that a future Depth-2 step would embed.

## Pipeline

```text
app/src/main.ts ──▶ parse ──▶ transpile (+ specifier rewrite) ──▶ binary
  (module graph)   (graph)     (TS→JS via forge-weld)            (write .js tree)
```

## Usage

```no_run
use forge_smelt::smelt;
let out = smelt("examples/react-app", "examples/react-app/dist/src")?;
println!("compiled entry: {}", out.entry.display());
# Ok::<(), forge_smelt::SmeltError>(())
```
