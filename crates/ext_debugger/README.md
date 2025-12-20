# ext_debugger

V8 Inspector Protocol debugger extension for Forge runtime.

## Overview

`ext_debugger` provides a complete Chrome DevTools Protocol (CDP) client implementation for debugging JavaScript/TypeScript code running in the Deno V8 runtime. It exposes the V8 Inspector Protocol via WebSocket connection, enabling programmatic control over execution, inspection of runtime state, and comprehensive debugging capabilities.

**Runtime Module:** `runtime:debugger`

## Features

### Connection Management
- WebSocket connection to V8 Inspector (local or remote)
- Automatic debugger domain and runtime enabling
- Connection status checking
- Configurable timeout and connection URL

### Breakpoint Management
- Set breakpoints by URL and line number
- Conditional breakpoints with JavaScript expressions
- Enable/disable breakpoints without removing them
- Remove individual or all breakpoints
- List breakpoints with hit counts and metadata
- Automatic V8 breakpoint location resolution

### Execution Control
- Pause execution at current statement
- Resume execution from paused state
- Step over (execute current line, pause at next)
- Step into function calls
- Step out of current function
- Continue to specific location (run-to-cursor)
- Configure exception pause behavior (none, uncaught, all)

### Stack Inspection
- Retrieve complete call stack when paused
- Access scope chain for each frame (local, closure, global)
- Inspect variables in any scope
- Navigate scope hierarchy

### Object Inspection
- Fetch properties of remote objects
- Differentiate primitives from complex objects
- Access object metadata (type, subtype, class name)
- Preview object contents without full fetch

### Expression Evaluation
- Evaluate arbitrary JavaScript expressions
- Evaluate in global context or specific call frame
- Access local variables and closures
- Produce side effects and modify state

### Script Management
- List all loaded scripts with metadata
- Retrieve source code by script ID
- Track dynamically loaded code
- Monitor script parsing events

### Event Handling
- Listen for pause events (breakpoints, steps, exceptions)
- Listen for script parsed events (module loading)
- Broadcast channels for event distribution
- Async event listeners with cleanup functions

## Usage

### Basic Debugging Session

```typescript
import * as debugger from "runtime:debugger";

// Connect to V8 Inspector
await debugger.connect();

// Set a breakpoint
const bp = await debugger.setBreakpoint("file:///src/main.ts", 42);
console.log(`Breakpoint set: ${bp.id}`);

// Listen for pause events
const cleanup = debugger.onPaused(async (event) => {
  console.log(`Paused: ${event.reason}`);

  // Print stack trace
  for (const frame of event.call_frames) {
    console.log(`  at ${frame.function_name} (${frame.url}:${frame.location.line_number})`);
  }

  // Resume execution
  await debugger.resume();
});

// Later: cleanup and disconnect
cleanup();
await debugger.disconnect();
```

### Inspecting Variables

```typescript
import { onPaused, getProperties, resume } from "runtime:debugger";

const cleanup = onPaused(async (event) => {
  const topFrame = event.call_frames[0];
  const localScope = topFrame.scope_chain.find(s => s.type === "local");

  if (localScope?.object.object_id) {
    const props = await getProperties(localScope.object.object_id);
    console.log("Local variables:");
    for (const prop of props) {
      if (prop.value) {
        console.log(`  ${prop.name} = ${prop.value.description}`);
      }
    }
  }

  await resume();
});
```

### Conditional Breakpoints

```typescript
import { setBreakpoint } from "runtime:debugger";

// Only pause when condition is true
const bp = await setBreakpoint("file:///src/auth.ts", 100, {
  condition: "user.role === 'admin'"
});

console.log(`Conditional breakpoint set: ${bp.id}`);
```

### Evaluating Expressions

```typescript
import { evaluate, onPaused } from "runtime:debugger";

const cleanup = onPaused(async (event) => {
  const topFrame = event.call_frames[0];

  // Evaluate in current frame context
  const result = await evaluate("user.email", topFrame.call_frame_id);
  console.log(`user.email = ${result.value}`);

  // Evaluate complex expression
  const users = await evaluate(`
    users.filter(u => u.role === 'admin')
         .map(u => u.name)
         .join(', ')
  `, topFrame.call_frame_id);
  console.log(`Admins: ${users.value}`);
});
```

### Exception Debugging

```typescript
import { setPauseOnExceptions, onPaused } from "runtime:debugger";

// Pause on all exceptions
await setPauseOnExceptions("all");

const cleanup = onPaused((event) => {
  if (event.reason === "exception" || event.reason === "promiseRejection") {
    console.log("Exception caught:", event.data);
    if (event.data?.exception?.description) {
      console.log("Error:", event.data.exception.description);
    }
  }
});
```

### Script Loading Monitoring

```typescript
import { onScriptParsed, setBreakpoint } from "runtime:debugger";

const cleanup = onScriptParsed(async (script) => {
  // Auto-breakpoint in new application modules
  if (script.url.startsWith("file://") && !script.url.includes("node_modules")) {
    console.log(`App script loaded: ${script.url}`);
    await setBreakpoint(script.url, 0);
  }
});
```

## Architecture

### Inspector Client

The core of the extension is the `InspectorClient` which maintains a WebSocket connection to the V8 Inspector. The inspector runs on a configurable port (default: 9229) and communicates using JSON-RPC 2.0 protocol messages.

```text
┌─────────────────┐      WebSocket      ┌─────────────────┐
│ InspectorClient │ ←──────────────────→ │  V8 Inspector   │
│  (Rust/Tokio)   │   JSON-RPC Messages  │  (Chrome CDP)   │
└─────────────────┘                      └─────────────────┘
        ↑                                         ↑
        │ State Access                            │
        ↓                                         │
┌─────────────────┐                              │
│ DebuggerState   │                              │
│ (Arc<Mutex<>>)  │                              │
│ - breakpoints   │                              │
│ - scripts       │                              │
│ - event channels│                              │
└─────────────────┘                              │
        ↑                                         │
        │ Op Calls                                │
        ↓                                         │
┌─────────────────┐                              │
│  TypeScript     │                              │
│  runtime:       │                              │
│    debugger     │                              │
└─────────────────┘                              │
        ↑                                         │
        │ Import                                  │
        ↓                                         │
┌─────────────────┐      Direct Access          │
│   Application   │ ─────────────────────────────┘
│   (main.ts)     │    V8 Runtime Introspection
└─────────────────┘
```

### State Management

The `DebuggerState` structure (wrapped in `Arc<Mutex<>>` for thread safety) maintains:
- Active WebSocket connection to V8 Inspector
- Breakpoint registry (ID to breakpoint mapping)
- Script registry (URL to script metadata mapping)
- Event broadcast channels (pause events, script events)
- Next request ID for protocol messages

### Protocol Communication

All V8 Inspector operations use JSON-RPC 2.0 format:

**Request:**
```json
{
  "id": 1,
  "method": "Debugger.setBreakpoint",
  "params": {
    "location": {
      "scriptId": "42",
      "lineNumber": 10
    }
  }
}
```

**Response:**
```json
{
  "id": 1,
  "result": {
    "breakpointId": "1:10:0:file:///src/main.ts",
    "actualLocation": {
      "scriptId": "42",
      "lineNumber": 10,
      "columnNumber": 0
    }
  }
}
```

**Event (no ID):**
```json
{
  "method": "Debugger.paused",
  "params": {
    "callFrames": [...],
    "reason": "breakpoint",
    "hitBreakpoints": ["1:10:0:file:///src/main.ts"]
  }
}
```

### Event Distribution

Pause and script events are distributed via Tokio broadcast channels:
1. V8 Inspector sends event message via WebSocket
2. `InspectorClient` receives and parses the message
3. Event is broadcast to all active receivers
4. TypeScript listeners (`onPaused`, `onScriptParsed`) receive events asynchronously

## Error Handling

All operations return appropriate errors with structured error codes (9600-9614).

### Error Codes

| Code | Error | Description |
|------|-------|-------------|
| 9600 | Generic | Generic debugger error |
| 9601 | ConnectionFailed | Failed to connect to inspector |
| 9602 | NotConnected | Not connected to inspector |
| 9603 | BreakpointFailed | Breakpoint operation failed |
| 9604 | InvalidFrameId | Invalid frame ID |
| 9605 | InvalidScopeId | Invalid scope ID |
| 9606 | EvaluationFailed | Expression evaluation failed |
| 9607 | SourceNotFound | Script/source not found |
| 9608 | StepFailed | Step operation failed |
| 9609 | PauseFailed | Pause operation failed |
| 9610 | ResumeFailed | Resume operation failed |
| 9611 | ProtocolError | Protocol error from V8 |
| 9612 | NotEnabled | Inspector not enabled |
| 9613 | Timeout | Operation timeout |
| 9614 | InvalidLocation | Invalid breakpoint location |

### Common Error Scenarios

```typescript
import { connect, setBreakpoint } from "runtime:debugger";

try {
  await connect();
} catch (error) {
  // Error 9601: Connection failed
  console.error("Failed to connect:", error);
}

try {
  await setBreakpoint("invalid.ts", 999999);
} catch (error) {
  // Error 9614: Invalid location
  console.error("Invalid breakpoint location:", error);
}
```

## Implementation Details

### V8 Inspector Connection

The connection process involves:
1. Opening WebSocket to `ws://localhost:9229/<session-id>`
2. Sending `Debugger.enable` to activate debugging domain
3. Sending `Runtime.enable` to activate runtime domain
4. Starting message receiver task for async events

### Line Number Indexing

**Important:** V8 Inspector uses 0-based line and column numbering, which differs from most text editors (1-based). The TypeScript API preserves V8's 0-based convention for consistency with Chrome DevTools.

```typescript
// Line 42 in your editor (1-based)
const bp = await setBreakpoint("file:///src/main.ts", 41); // Use 41 (0-based)
```

### Remote Object References

Complex objects (arrays, objects, functions) are not sent inline. Instead, V8 assigns each object a unique `object_id` string. To inspect object properties, use `getProperties()` with the object ID. Primitive values (number, string, boolean, null) are sent inline.

```typescript
// Primitive value (inline)
const primitiveObj: RemoteObject = {
  type: "number",
  value: 42,
  description: "42"
};

// Complex object (requires getProperties)
const complexObj: RemoteObject = {
  type: "object",
  subtype: "array",
  object_id: "obj-123",  // Use this ID to fetch properties
  description: "Array(5)"
};

if (complexObj.object_id) {
  const props = await getProperties(complexObj.object_id);
  console.log("Properties:", props);
}
```

### Breakpoint Resolution

When setting a breakpoint, V8 may adjust the location to the nearest executable statement. For example, setting a breakpoint on a comment line will resolve to the next code line. The returned `Breakpoint` struct contains the actual resolved location.

```typescript
const bp = await setBreakpoint("file:///src/main.ts", 10);
console.log(`Requested line: 10, Actual line: ${bp.location.line_number}`);
```

## Thread Safety

- `DebuggerState` is wrapped in `Arc<Mutex<>>` for safe concurrent access
- WebSocket connection is `Arc<RwLock<>>` to allow concurrent reads
- Broadcast channels enable safe multi-consumer event distribution
- All ops are async and properly synchronized

## Performance Considerations

- **WebSocket Latency**: Communication adds ~1ms latency on localhost
- **Object Inspection**: Large object inspection can be slow - fetch only needed properties
- **Event Broadcasting**: Minimal overhead using Tokio channels
- **State Locks**: Held briefly with no blocking I/O under lock

## Testing

```bash
# Run all tests
cargo test -p ext_debugger

# Run with output
cargo test -p ext_debugger -- --nocapture

# Run specific test
cargo test -p ext_debugger connection_lifecycle
```

Tests cover:
- Connection lifecycle (connect, disconnect, reconnect)
- Breakpoint operations (set, remove, enable/disable, list)
- Execution control (pause, resume, step operations)
- Expression evaluation (global and frame contexts)
- Event handling (pause and script events)
- Error conditions (not connected, invalid IDs, protocol errors)

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `deno_core` | Op definitions and runtime integration |
| `tokio` | Async runtime for WebSocket and event handling |
| `tokio-tungstenite` | WebSocket client for V8 Inspector Protocol |
| `serde` | Serialization framework |
| `serde_json` | JSON protocol message parsing |
| `thiserror` | Error type definitions |
| `tracing` | Logging and diagnostics |
| `forge-weld` | Build-time code generation |
| `forge-weld-macro` | Procedural macros for type bindings |
| `linkme` | Compile-time symbol collection |

## See Also

- [Chrome DevTools Protocol Documentation](https://chromedevtools.github.io/devtools-protocol/)
- [V8 Inspector Protocol Viewer](https://chromedevtools.github.io/devtools-protocol/v8/)
- [ext_devtools](../ext_devtools/) - DevTools frontend integration
- [ext_trace](../ext_trace/) - Application tracing and profiling
- [Forge Documentation](../../site/) - Full framework documentation

## License

Part of the Forge project. See the repository root for license information.
