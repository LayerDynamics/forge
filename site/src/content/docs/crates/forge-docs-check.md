---
title: "forge-docs-check"
description: "Forge Docs Check - fails the build when the documentation site drifts from the code"
slug: crates/forge-docs-check
---

# forge-docs-check

Fails the build when the documentation site (`site/src/content/docs/`) drifts
from the code. It mechanizes the audit that produced `Site.md`: missing crate
pages, API method drift (in both directions), stale counts, missing CLI
commands, and the self-consistency of `forge docs`' own extension list.

The same logic runs three ways:
- as the `forge-docs-check` binary (prints a punch-list, exits non-zero on drift),
- as the `docs_in_sync` integration test (`cargo test` fails on drift),
- invoked from CI and the pre-commit hook.

Every expectation is derived from the filesystem via [`discovery::Workspace`],
so the checker never carries a hand-maintained list that could itself rot.
