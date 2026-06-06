# ext_console

`runtime:console` — console output capture for Forge apps.

Captures `console.*` calls into a bounded, in-memory ring buffer on the host so
app code can read recent logs back: an in-app log viewer, runtime diagnostics,
or the web inspector's console panel.

## Scope vs. `ext_log`

`ext_log` is a **stateless forwarder** — `op_log_emit` pushes a level + message
into host `tracing` and keeps nothing. `ext_console` is **complementary**: it
*retains* recent records and exposes them for retrieval. They do not overlap.

## Sources

Both ingestion paths feed the same buffer and produce identically-shaped
records:

- **Deno side** — `ts/init.ts` `install()` patches `console.*` to forward each
  call to `op_console_push` (the original console still runs).
- **Renderer side** — the WebView forwards console messages over the
  `window.host` IPC bridge; `web_console::parse_renderer_message` turns each
  payload into a record tagged `renderer`.

## Ops

| Op | Purpose |
|----|---------|
| `op_console_info` | Extension metadata |
| `op_console_push(level, args, source?)` | Capture a record (timestamped on capture) |
| `op_console_tail(n)` | Most recent `n` records, oldest-first |
| `op_console_clear()` | Drop all records, returns the count removed |

## Record shape

```jsonc
{
  "level": "warn",            // log | info | warn | error | debug
  "args": ["disk low", 12],   // original console arguments as JSON
  "timestamp_ms": 1700000000000,
  "source": "deno"            // deno | renderer
}
```

The buffer retains the most recent `DEFAULT_CAPACITY` (1000) records, evicting
the oldest first.
