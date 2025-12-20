---
title: "ext_trace"
description: Lightweight application tracing extension providing the runtime:trace module.
slug: crates/ext-trace
---

The `ext_trace` crate provides lightweight application-level tracing for Forge applications through the `runtime:trace` module.

## Overview

ext_trace handles:

- **Span tracing** - Manual start/end lifecycle with unique ID tracking
- **Duration measurement** - High-resolution timing via Rust `Instant`
- **Instant events** - Point-in-time events with zero duration
- **Batch export** - Periodic flushing of completed spans

**Key Characteristics:**
- Manual lifecycle (no automatic scoping)
- Flat structure (no parent-child relationships)
- In-memory buffering until flushed
- No permissions required

## Module: `runtime:trace`

```typescript
import {
  info,
  start,
  end,
  instant,
  flush
} from "runtime:trace";
```

## Quick Start

```typescript
import { start, end, flush } from "runtime:trace";

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

// Periodic export
setInterval(() => {
  const spans = flush();
  console.log(`Exported ${spans.length} spans`);
}, 60000);
```

## Key Types

### SpanRecord

Represents a completed trace span:

```typescript
interface SpanRecord {
  id: bigint;            // Unique span ID
  name: string;          // Span name
  started_at: bigint;    // Milliseconds since UNIX epoch
  duration_ms: number;   // Elapsed duration (0 for instant events)
  attributes?: unknown;  // Optional JSON-serializable attributes
  result?: unknown;      // Optional result data
}
```

### ExtensionInfo

Extension metadata:

```typescript
interface ExtensionInfo {
  name: string;          // "ext_trace"
  version: string;       // e.g., "0.1.0"
  status: string;        // "ready"
}
```

### TraceError

Error thrown when ending an invalid span:

```typescript
enum TraceError {
  SpanNotFound = "Span not found"
}
```

## API Reference

### `info(): ExtensionInfo`

Get extension information.

**Synchronous**

```typescript
import { info } from "runtime:trace";

const extensionInfo = info();
console.log(`${extensionInfo.name} v${extensionInfo.version}`);
// => "ext_trace v0.1.0"
```

**Returns:** Extension metadata

### `start(name: string, attributes?: unknown): bigint`

Start a new trace span and return its unique ID.

**Synchronous**

**Parameters:**
- `name` - Span name (e.g., "fetchUser", "processImage")
- `attributes` - Optional JSON-serializable attributes

**Returns:** Unique span ID (u64 as bigint) to pass to `end()`

**Important:** You must call `end()` for every `start()` to avoid memory leaks from accumulating active spans.

```typescript
import { start, end } from "runtime:trace";

function processData(data: any[]) {
  const spanId = start("processData", { count: data.length });

  try {
    // ... processing ...
  } finally {
    end(spanId); // Always call end() in finally block
  }
}
```

### `end(id: bigint, result?: unknown): SpanRecord`

End a trace span and return the completed record.

**Synchronous**

**Parameters:**
- `id` - Span ID returned by `start()`
- `result` - Optional JSON-serializable result data

**Returns:** Completed `SpanRecord` with duration and metadata

**Throws:** `TraceError.SpanNotFound` if ID is invalid or span was already ended

```typescript
import { start, end } from "runtime:trace";

const spanId = start("operation");
// ... do work ...
const record = end(spanId, { itemsProcessed: 100 });
console.log(`Duration: ${record.duration_ms}ms`);
```

### `instant(name: string, attributes?: unknown): SpanRecord`

Record a point-in-time event with zero duration.

**Synchronous**

**Parameters:**
- `name` - Event name (e.g., "user_click", "cache_miss", "checkpoint")
- `attributes` - Optional JSON-serializable event metadata

**Returns:** `SpanRecord` with `duration_ms: 0`

```typescript
import { instant } from "runtime:trace";

// User interaction tracking
document.getElementById("submit")?.addEventListener("click", () => {
  instant("button_click", { buttonId: "submit" });
});

// State change events
instant("status_change", { from: "loading", to: "ready" });

// Checkpoint markers
instant("workflow_start");
await step1();
instant("step1_complete");
```

### `flush(): SpanRecord[]`

Retrieve all finished spans and clear the buffer.

**Synchronous**

**Returns:** Array of all finished span records since last flush (may be empty)

This is a "drain" operation - subsequent `flush()` calls will only return spans finished after the previous flush.

```typescript
import { flush } from "runtime:trace";

// Periodic export
setInterval(async () => {
  const spans = flush();
  if (spans.length > 0) {
    await fetch("/api/traces", {
      method: "POST",
      body: JSON.stringify(spans)
    });
  }
}, 60000);
```

## Usage Patterns

### Pattern 1: Basic Span Tracking

```typescript
import { start, end } from "runtime:trace";

async function loadConfig() {
  const spanId = start("loadConfig", { env: "production" });

  try {
    const config = await fetch("/api/config").then(r => r.json());
    return end(spanId, { configKeys: Object.keys(config).length });
  } catch (error) {
    return end(spanId, { error: error.message });
  }
}
```

### Pattern 2: Instant Events for Checkpoints

```typescript
import { instant } from "runtime:trace";

async function complexWorkflow() {
  instant("workflow_start");

  await step1();
  instant("step1_complete");

  await step2();
  instant("step2_complete");

  await step3();
  instant("workflow_complete");
}
```

### Pattern 3: Periodic Export to Backend

```typescript
import { flush } from "runtime:trace";

async function exportToTracing() {
  const spans = flush();

  if (spans.length > 0) {
    try {
      await fetch("https://tracing.example.com/api/traces", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          service: "my-app",
          environment: "production",
          spans
        })
      });
      console.log(`Exported ${spans.length} trace spans`);
    } catch (error) {
      console.error("Failed to export traces:", error);
    }
  }
}

// Export every minute
setInterval(exportToTracing, 60000);
```

### Pattern 4: File Export

```typescript
import { flush } from "runtime:trace";
import { writeTextFile } from "runtime:fs";

async function saveTracesToFile() {
  const spans = flush();

  if (spans.length > 0) {
    const timestamp = new Date().toISOString();
    const filename = `traces-${timestamp}.json`;
    await writeTextFile(filename, JSON.stringify(spans, null, 2));
    console.log(`Saved ${spans.length} spans to ${filename}`);
  }
}
```

### Pattern 5: Console Debugging

```typescript
import { start, end, flush } from "runtime:trace";

// Add tracing to functions
function processItems(items: any[]) {
  const spanId = start("processItems", { count: items.length });

  for (const item of items) {
    // process item
  }

  const record = end(spanId, { processed: items.length });
  console.log(`Processed ${items.length} items in ${record.duration_ms}ms`);
}

// Periodic trace dump
function dumpTraces() {
  const spans = flush();
  console.log(`\n=== Trace Report (${spans.length} spans) ===`);

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

setInterval(dumpTraces, 10000);
```

## Error Handling

### SpanNotFound Error

Thrown when `end()` is called with an invalid or already-ended span ID:

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

Span IDs are monotonic u64 counters:
- IDs start at 1 (never 0)
- Wraps to 1 on overflow (not 0)
- Not globally unique (reset on app restart)

### Duration Measurement

Uses Rust `Instant::elapsed()` for high-precision timing:
- `Instant::now()` captured on `start()`
- Monotonic (unaffected by system clock changes)
- Typically nanosecond resolution

### Wall-Clock Timestamps

`started_at` uses `SystemTime` for export compatibility:
- Milliseconds since January 1, 1970 00:00:00 UTC
- Compatible with JavaScript `Date`, databases, etc.
- May jump backward if system clock is adjusted

### Memory Management

**Active Spans:**
- Stored until `end()` called
- Memory grows unbounded if `end()` never called
- Use try/finally to ensure `end()` is always called

**Finished Spans:**
- Stored until `flush()` called
- Memory grows indefinitely if never flushed
- Recommend periodic `flush()` every 60 seconds

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

**❌ Bad:**
```typescript
// Spans accumulate forever in memory
```

**✅ Good:**
```typescript
setInterval(() => {
  flush();
}, 60000);
```

### 3. Reusing span IDs

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

## File Structure

```text
crates/ext_trace/
├── src/
│   └── lib.rs        # Extension implementation
├── ts/
│   └── init.ts       # TypeScript module shim
├── build.rs          # forge-weld build configuration
└── Cargo.toml
```

## Rust Implementation

Operations are annotated with forge-weld macros for automatic TypeScript binding generation:

```rust
// src/lib.rs
use deno_core::{op2, Extension, OpState};
use forge_weld_macro::{weld_op, weld_struct};
use serde::{Deserialize, Serialize};

#[weld_struct]
#[derive(Debug, Serialize)]
pub struct SpanRecord {
    pub id: u64,
    pub name: String,
    pub started_at: u128,
    pub duration_ms: f64,
    pub attributes: Option<Value>,
    pub result: Option<Value>,
}

#[weld_op]
#[op2]
#[bigint]
fn op_trace_start(
    state: &mut OpState,
    #[string] name: String,
    #[serde] attributes: Option<Value>,
) -> u64 {
    // implementation
}

#[weld_op]
#[op2]
#[serde]
fn op_trace_end(
    state: &mut OpState,
    #[bigint] id: u64,
    #[serde] result: Option<Value>,
) -> Result<SpanRecord, TraceError> {
    // implementation
}
```

## Build Configuration

```rust
// build.rs
use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_trace", "runtime:trace")
        .ts_path("ts/init.ts")
        .ops(&[
            "op_trace_info",
            "op_trace_start",
            "op_trace_end",
            "op_trace_instant",
            "op_trace_flush",
        ])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build runtime_trace extension");
}
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `deno_core` | Op definitions, Extension, OpState |
| `serde` | Serialization derive macros |
| `serde_json` | JSON Value for attributes/results |
| `thiserror` | Error type derivation |
| `deno_error` | JsError derive for TraceError |
| `forge-weld` | Build-time code generation |
| `forge-weld-macro` | `#[weld_op]`, `#[weld_struct]` macros |
| `linkme` | Compile-time symbol collection |

## Related

- [ext_log](/docs/crates/ext-log) - Structured logging
- [ext_monitor](/docs/crates/ext-monitor) - System and runtime monitoring
- [ext_devtools](/docs/crates/ext-devtools) - Developer tools integration
- [Architecture](/docs/architecture) - Full system architecture
