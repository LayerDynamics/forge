# ext_trace

Lightweight application-level tracing extension for Forge applications, providing the `runtime:trace` module.

## Overview

`ext_trace` provides simple span-based performance tracking with manual start/end lifecycle management. Unlike full distributed tracing systems (OpenTelemetry, Jaeger), it offers minimalist in-memory span tracking designed for application-level instrumentation.

**Key Features:**
- Manual span lifecycle: `start()` -> `end()` with unique ID tracking
- High-resolution duration measurement (Rust `Instant`)
- Point-in-time events via `instant()` (zero duration)
- Batch export via `flush()` for periodic data collection
- Flat structure (no parent-child relationships)
- No permissions required

**Runtime Module:** `runtime:trace`

## Usage Examples

### Basic Span Tracking

```typescript
import { start, end } from "runtime:trace";

async function fetchUser(id: number) {
  const spanId = start("fetchUser", { userId: id });

  try {
    const response = await fetch(`/api/users/${id}`);
    const user = await response.json();
    return end(spanId, { status: response.status });
  } catch (error) {
    return end(spanId, { error: error.message });
  }
}

// Outputs SpanRecord: { id, name, started_at, duration_ms, attributes, result }
```

### Instant Events

```typescript
import { instant } from "runtime:trace";

// Record point-in-time events without span lifecycle
instant("cache_miss", { key: "user:123" });
instant("user_click", { target: "submit_button" });
instant("state_change", { from: "loading", to: "ready" });
```

### Periodic Export

```typescript
import { flush } from "runtime:trace";

// Export spans every 60 seconds
setInterval(() => {
  const spans = flush();
  if (spans.length > 0) {
    console.log(`Exporting ${spans.length} spans`);
    // Send to backend, write to file, etc.
  }
}, 60000);
```

### Console Logging

```typescript
import { start, end, flush } from "runtime:trace";

function processItems(items: any[]) {
  const spanId = start("processItems", { count: items.length });

  for (const item of items) {
    // process item
  }

  const record = end(spanId, { processed: items.length });
  console.log(`Processed ${items.length} items in ${record.duration_ms}ms`);
}

// Later: dump all traces
function dumpTraces() {
  const spans = flush();
  console.log("\n=== Trace Report ===");
  for (const span of spans) {
    console.log(`${span.name}: ${span.duration_ms.toFixed(2)}ms`);
    if (span.attributes) {
      console.log(`  Attributes:`, span.attributes);
    }
    if (span.result) {
      console.log(`  Result:`, span.result);
    }
  }
}
```

### File Export

```typescript
import { flush } from "runtime:trace";
import { writeTextFile } from "runtime:fs";

async function saveTracesToFile() {
  const spans = flush();
  const timestamp = new Date().toISOString();
  const filename = `traces-${timestamp}.json`;
  await writeTextFile(filename, JSON.stringify(spans, null, 2));
  console.log(`Saved ${spans.length} spans to ${filename}`);
}
```

### HTTP Export

```typescript
import { flush } from "runtime:trace";

async function exportToBackend() {
  const spans = flush();
  if (spans.length > 0) {
    await fetch("https://tracing.example.com/api/traces", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(spans)
    });
  }
}

// Export every minute
setInterval(exportToBackend, 60000);
```

## Architecture

```text
┌────────────────────────────────────────────────────────────┐
│ TypeScript Application (runtime:trace)                     │
│  spanId = start(name, attrs?) -> end(spanId, result?)      │
└────────────────┬───────────────────────────────────────────┘
                 │ Deno Ops (op_trace_*)
                 ↓
┌────────────────────────────────────────────────────────────┐
│ ext_trace (TraceState in OpState)                          │
│  - active: HashMap<u64, ActiveSpan>                        │
│  - finished: Vec<SpanRecord>                               │
│  - next_id: u64 (monotonic counter)                        │
└────────────────┬───────────────────────────────────────────┘
                 │ std::time::Instant/SystemTime
                 ↓
┌────────────────────────────────────────────────────────────┐
│ High-Resolution Timing                                     │
│  - Instant::now() for duration measurement                 │
│  - SystemTime for wall-clock timestamps (UNIX epoch)       │
└────────────────────────────────────────────────────────────┘
```

## API Reference

### `info(): ExtensionInfo`

Get extension metadata (name, version, status).

**Returns:** `{ name: string, version: string, status: string }`

### `start(name: string, attributes?: unknown): bigint`

Start a new trace span and return its unique ID.

**Parameters:**
- `name` - Span name (e.g., "fetchUser", "processImage")
- `attributes` - Optional JSON-serializable attributes

**Returns:** Unique span ID (u64 as bigint)

**Important:** You must call `end()` for every `start()` to avoid memory leaks. Use try/finally to ensure `end()` is always called.

### `end(id: bigint, result?: unknown): SpanRecord`

End a trace span and return the completed record.

**Parameters:**
- `id` - Span ID returned by `start()`
- `result` - Optional JSON-serializable result data

**Returns:** Completed `SpanRecord` with duration and metadata

**Throws:** `TraceError.SpanNotFound` if ID is invalid or span was already ended

### `instant(name: string, attributes?: unknown): SpanRecord`

Record a point-in-time event with zero duration.

**Parameters:**
- `name` - Event name (e.g., "user_click", "cache_miss")
- `attributes` - Optional JSON-serializable event metadata

**Returns:** `SpanRecord` with `duration_ms: 0`

### `flush(): SpanRecord[]`

Retrieve all finished spans and clear the buffer.

**Returns:** Array of all finished span records since last flush

This is a "drain" operation - subsequent `flush()` calls will only return spans finished after the previous flush.

## Data Types

### `SpanRecord`

Represents a completed trace span:

```typescript
interface SpanRecord {
  id: bigint;            // Unique span ID (matches ID from start())
  name: string;          // Span name
  started_at: bigint;    // Wall-clock timestamp (milliseconds since UNIX epoch)
  duration_ms: number;   // Elapsed duration in milliseconds (0 for instant events)
  attributes?: unknown;  // Optional arbitrary attributes (JSON-serializable)
  result?: unknown;      // Optional result data (JSON-serializable)
}
```

## Error Handling

### `TraceError.SpanNotFound`

Thrown when `end()` is called with an invalid or already-finished span ID.

```typescript
import { start, end } from "runtime:trace";

const spanId = start("operation");
end(spanId); // OK

try {
  end(spanId); // Throws: SpanNotFound
} catch (error) {
  console.error("Span already ended:", error.message);
}
```

## Implementation Details

### Span ID Generation

Span IDs are generated using a wrapping monotonic counter:
- IDs start at 1 (never 0)
- Wraps to 1 on overflow (not 0)
- Monotonically increasing within a runtime instance
- Not globally unique (reset on application restart)

### Duration Measurement

Duration is calculated using `Instant::elapsed()`:
- `Instant::now()` captured on `start()`
- `Instant::elapsed()` called on `end()` to get duration
- Monotonic (unaffected by system clock changes)
- High precision (typically nanosecond resolution)

### Wall-Clock Timestamps

`started_at` uses `SystemTime` for export compatibility:
- Milliseconds since January 1, 1970 00:00:00 UTC
- Compatible with JavaScript `Date`, database timestamps, etc.
- May jump backward if system clock is adjusted

### Memory Management

**Active Spans:**
- Stored in `HashMap<u64, ActiveSpan>` until `end()` called
- Memory grows unbounded if `end()` never called
- **Recommendation:** Use try/finally to ensure `end()` is always called

**Finished Spans:**
- Stored in `Vec<SpanRecord>` until `flush()` called
- Memory grows until explicitly flushed
- **Recommendation:** Call `flush()` periodically (e.g., every 60 seconds)

### Instant Events

`instant()` creates a `SpanRecord` with `duration_ms: 0`:
- Generates new ID but never adds to active HashMap
- Directly appends to finished Vec
- Useful for marking checkpoints, state changes, discrete events

## Platform Support

| Platform | Support | Notes |
|----------|---------|-------|
| macOS    | ✓       | Full support |
| Windows  | ✓       | Full support |
| Linux    | ✓       | Full support |
| FreeBSD  | ✓       | Full support (via std::time) |
| OpenBSD  | ✓       | Full support (via std::time) |
| NetBSD   | ✓       | Full support (via std::time) |

Uses only `std::time` primitives - no platform-specific code.

## Common Pitfalls

### 1. Forgetting to call `end()`

Active spans accumulate in memory if `end()` is never called.

**❌ Bad:**
```typescript
const spanId = start("operation");
// ... operation ...
// Forgot to call end() - memory leak!
```

**✅ Good:**
```typescript
const spanId = start("operation");
try {
  // ... operation ...
} finally {
  end(spanId);
}
```

### 2. Forgetting to call `flush()`

Finished spans accumulate indefinitely if never flushed.

**❌ Bad:**
```typescript
// Spans accumulate forever in memory
```

**✅ Good:**
```typescript
setInterval(() => {
  const spans = flush();
  console.log(`Flushed ${spans.length} spans`);
}, 60000);
```

### 3. Reusing span IDs

Once ended, a span ID cannot be reused.

**❌ Bad:**
```typescript
const spanId = start("operation");
end(spanId);
end(spanId); // Throws: SpanNotFound
```

**✅ Good:**
```typescript
const spanId1 = start("operation1");
end(spanId1);

const spanId2 = start("operation2"); // New ID
end(spanId2);
```

### 4. Assuming zero overhead

Every `start()` and `end()` allocates memory. For critical hot paths, consider sampling at the application level.

**❌ Bad:**
```typescript
for (let i = 0; i < 1000000; i++) {
  const spanId = start("hotLoop"); // Allocates 1M times!
  processItem(i);
  end(spanId);
}
```

**✅ Good:**
```typescript
// Sample every 1000th iteration
for (let i = 0; i < 1000000; i++) {
  const shouldTrace = i % 1000 === 0;
  const spanId = shouldTrace ? start("hotLoop") : null;
  processItem(i);
  if (spanId) end(spanId);
}
```

## Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| `deno_core` | workspace | Op definitions, Extension, OpState |
| `serde` | workspace | Serialization derive macros |
| `serde_json` | workspace | JSON Value for attributes/results |
| `thiserror` | workspace | Error type derivation |
| `deno_error` | workspace | JsError derive for TraceError |
| `forge-weld` | workspace | Build-time code generation |
| `forge-weld-macro` | workspace | `#[weld_op]`, `#[weld_struct]` macros |
| `linkme` | workspace | Compile-time symbol collection |

## Testing

```bash
# Run extension tests
cargo test -p ext_trace

# Run with debug logging
RUST_LOG=ext_trace=trace cargo test -p ext_trace
```

## File Structure

```
crates/ext_trace/
├── src/
│   └── lib.rs          # Extension implementation
├── ts/
│   └── init.ts         # TypeScript module shim
├── build.rs            # forge-weld build configuration
├── Cargo.toml
└── README.md
```

## Related Extensions

- [`ext_log`](../ext_log) - Structured logging
- [`ext_devtools`](../ext_devtools) - Developer tools integration
- [`ext_monitor`](../ext_monitor) - System and runtime monitoring

## License

Same as the parent Forge project.
