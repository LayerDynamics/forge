---
title: "ext_console"
description: "Console capture extension for Forge (runtime:console)"
slug: docs/crates/ext-console
---

Console capture extension for Forge (`runtime:console`).

Captures `console.*` output into a bounded, queryable ring buffer so app
code can read recent logs (e.g. for an in-app log viewer or the web
inspector's console panel). Two ingestion paths feed the same buffer:

- **Deno side:** `ts/init.ts` patches `console.*` to forward each call to
  `op_console_push` (while still calling the
  original console).
- **Renderer side:** the WebView forwards console messages over the
  `window.host` IPC bridge; [`web_console::parse_renderer_message`] turns the
  payload into a record and pushes it through the same buffer.

## Scope vs. `ext_log`

`ext_log` is a stateless forwarder to host `tracing` (fire-and-forget, no
retrieval). `ext_console` is complementary: it *retains* recent records and
exposes `op_console_tail` / `op_console_clear` for reading them back. The two
do not overlap.
