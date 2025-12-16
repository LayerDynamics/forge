---
title: "ext_trace"
description: Application tracing extension providing the runtime:trace module.
slug: crates/ext-trace
---

The `ext_trace` crate provides application-level tracing and performance instrumentation for Forge applications through the `runtime:trace` module.

## Overview

ext_trace handles:

- **Span tracing** - Track operation duration
- **Event recording** - Log trace events
- **Context propagation** - Link related operations
- **Export** - Export trace data for analysis
- **Sampling** - Control trace collection rate

## Module: `runtime:trace`

```typescript
import {
  startSpan,
  finishSpan,
  recordEvent,
  exportRecords,
  setEnabled
} from "runtime:trace";
```

## Key Types

### Error Types

```rust
enum TraceErrorCode {
    Generic = 9300,
    SpanNotFound = 9301,
    ExportFailed = 9302,
    InvalidSpan = 9303,
}

struct TraceError {
    code: TraceErrorCode,
    message: String,
}
```

### Trace Types

```rust
struct SpanHandle {
    id: u64,
    name: String,
}

struct Span {
    id: u64,
    parent_id: Option<u64>,
    name: String,
    start_time: Instant,
    end_time: Option<Instant>,
    attributes: HashMap<String, Value>,
    events: Vec<TraceEvent>,
}

struct TraceEvent {
    name: String,
    timestamp: Instant,
    attributes: HashMap<String, Value>,
}

struct TraceRecord {
    span_id: u64,
    name: String,
    duration_ms: f64,
    parent_id: Option<u64>,
    attributes: HashMap<String, Value>,
    events: Vec<TraceEvent>,
}

struct TraceState {
    spans: HashMap<u64, Span>,
    completed: Vec<TraceRecord>,
    enabled: bool,
    next_id: u64,
}
```

## Operations

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_trace_start_span` | `startSpan(name, opts?)` | Start a new trace span |
| `op_trace_finish_span` | `finishSpan(handle)` | End a trace span |
| `op_trace_record_event` | `recordEvent(name, attrs?)` | Record event in current span |
| `op_trace_export` | `exportRecords()` | Export completed traces |
| `op_trace_set_enabled` | `setEnabled(enabled)` | Enable/disable tracing |
| `op_trace_clear` | `clear()` | Clear collected traces |

## Usage Examples

### Basic Span Tracing

```typescript
import { startSpan, finishSpan } from "runtime:trace";

async function processData(data: any[]) {
  const span = await startSpan("processData");

  try {
    // Do work...
    for (const item of data) {
      await processItem(item);
    }
  } finally {
    await finishSpan(span);
  }
}
```

### Nested Spans

```typescript
import { startSpan, finishSpan } from "runtime:trace";

async function handleRequest(req: Request) {
  const requestSpan = await startSpan("handleRequest", {
    attributes: { method: req.method, path: req.url }
  });

  try {
    // Parse body
    const parseSpan = await startSpan("parseBody", { parent: requestSpan });
    const body = await req.json();
    await finishSpan(parseSpan);

    // Process
    const processSpan = await startSpan("processRequest", { parent: requestSpan });
    const result = await process(body);
    await finishSpan(processSpan);

    return result;
  } finally {
    await finishSpan(requestSpan);
  }
}
```

### Recording Events

```typescript
import { startSpan, finishSpan, recordEvent } from "runtime:trace";

async function fetchWithRetry(url: string) {
  const span = await startSpan("fetchWithRetry", { attributes: { url } });

  try {
    for (let attempt = 1; attempt <= 3; attempt++) {
      try {
        await recordEvent("attempt", { attemptNumber: attempt });
        const response = await fetch(url);
        await recordEvent("success", { status: response.status });
        return response;
      } catch (e) {
        await recordEvent("error", { error: e.message, attempt });
        if (attempt === 3) throw e;
      }
    }
  } finally {
    await finishSpan(span);
  }
}
```

### Exporting Traces

```typescript
import { exportRecords, clear } from "runtime:trace";

async function flushTraces() {
  const records = await exportRecords();

  for (const record of records) {
    console.log(`${record.name}: ${record.duration_ms}ms`);
    for (const event of record.events) {
      console.log(`  - ${event.name}`);
    }
  }

  // Clear after export
  await clear();
}
```

## Trace Output Format

```json
{
  "span_id": 12345,
  "name": "handleRequest",
  "duration_ms": 45.32,
  "parent_id": null,
  "attributes": {
    "method": "POST",
    "path": "/api/users"
  },
  "events": [
    { "name": "parseBody", "timestamp": 1234567890 },
    { "name": "queryDatabase", "timestamp": 1234567895 }
  ]
}
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
pub struct SpanHandle {
    pub id: u64,
    pub name: String,
}

#[weld_struct]
#[derive(Debug, Serialize)]
pub struct TraceRecord {
    pub span_id: u64,
    pub name: String,
    pub duration_ms: f64,
    pub parent_id: Option<u64>,
}

#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_trace_start_span(
    state: Rc<RefCell<OpState>>,
    #[string] name: String,
    #[serde] opts: Option<SpanOptions>,
) -> Result<SpanHandle, TraceError> {
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
        .ops(&["op_trace_start_span", "op_trace_finish_span", "op_trace_export", /* ... */])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build runtime_trace extension");
}
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `deno_core` | Op definitions |
| `tracing` | Tracing infrastructure |
| `serde` | Serialization |
| `serde_json` | JSON export |
| `tokio` | Async runtime |
| `forge-weld` | Build-time code generation |
| `forge-weld-macro` | `#[weld_op]`, `#[weld_struct]` macros |
| `linkme` | Compile-time symbol collection |

## Related

- [ext_log](/docs/crates/ext-log) - Structured logging
- [ext_devtools](/docs/crates/ext-devtools) - Developer tools
- [Architecture](/docs/architecture) - Full system architecture
