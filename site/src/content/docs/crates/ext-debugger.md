---
title: "ext_debugger"
description: Debugging protocol extension providing the runtime:debugger module.
slug: crates/ext-debugger
---

The `ext_debugger` crate provides debugging protocol support for Forge applications through the `runtime:debugger` module.

## Overview

ext_debugger handles:

- **Breakpoints** - Set and manage breakpoints
- **Stepping** - Step through code execution
- **Inspection** - Inspect variables and scope
- **Debug protocol** - Chrome DevTools Protocol support
- **Remote debugging** - Connect external debuggers

## Module: `runtime:debugger`

```typescript
import {
  connect,
  disconnect,
  setBreakpoint,
  removeBreakpoint,
  pause,
  resume,
  stepOver,
  stepInto,
  stepOut,
  evaluate
} from "runtime:debugger";
```

## Key Types

### Error Types

```rust
enum DebuggerErrorCode {
    Generic = 9600,
    ConnectFailed = 9601,
    DisconnectFailed = 9602,
    BreakpointFailed = 9603,
    NotConnected = 9604,
    InvalidState = 9605,
    EvaluationFailed = 9606,
}

struct DebuggerError {
    code: DebuggerErrorCode,
    message: String,
}
```

### Debugger Types

```rust
struct DebuggerSession {
    id: u32,
    connected: bool,
}

struct Breakpoint {
    id: u32,
    file: String,
    line: u32,
    condition: Option<String>,
}

struct StackFrame {
    id: u32,
    name: String,
    file: String,
    line: u32,
    column: u32,
}

struct Variable {
    name: String,
    value: String,
    type_name: String,
}

struct DebuggerState {
    session: Option<DebuggerSession>,
    breakpoints: HashMap<u32, Breakpoint>,
    paused: bool,
    call_stack: Vec<StackFrame>,
}
```

## Operations

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_debugger_connect` | `connect(opts?)` | Start debug session |
| `op_debugger_disconnect` | `disconnect()` | End debug session |
| `op_debugger_set_breakpoint` | `setBreakpoint(file, line, opts?)` | Set breakpoint |
| `op_debugger_remove_breakpoint` | `removeBreakpoint(id)` | Remove breakpoint |
| `op_debugger_pause` | `pause()` | Pause execution |
| `op_debugger_resume` | `resume()` | Resume execution |
| `op_debugger_step_over` | `stepOver()` | Step over |
| `op_debugger_step_into` | `stepInto()` | Step into |
| `op_debugger_step_out` | `stepOut()` | Step out |
| `op_debugger_evaluate` | `evaluate(expr)` | Evaluate expression |
| `op_debugger_get_stack` | `getCallStack()` | Get call stack |
| `op_debugger_get_variables` | `getVariables(frameId?)` | Get variables |

## Usage Examples

### Connecting Debugger

```typescript
import { connect, disconnect } from "runtime:debugger";

const session = await connect({
  port: 9222  // Chrome DevTools Protocol port
});

// Debug session active...

await disconnect();
```

### Setting Breakpoints

```typescript
import { setBreakpoint, removeBreakpoint } from "runtime:debugger";

// Simple breakpoint
const bp1 = await setBreakpoint("src/main.ts", 42);

// Conditional breakpoint
const bp2 = await setBreakpoint("src/utils.ts", 100, {
  condition: "x > 10"
});

// Remove when done
await removeBreakpoint(bp1.id);
```

### Stepping Through Code

```typescript
import { pause, resume, stepOver, stepInto, stepOut } from "runtime:debugger";

// Pause execution
await pause();

// Step operations
await stepOver();  // Execute current line, step to next
await stepInto();  // Step into function call
await stepOut();   // Step out of current function

// Continue execution
await resume();
```

### Inspecting State

```typescript
import { getCallStack, getVariables, evaluate } from "runtime:debugger";

// When paused at breakpoint
const stack = await getCallStack();
for (const frame of stack) {
  console.log(`${frame.name} at ${frame.file}:${frame.line}`);
}

// Get local variables
const vars = await getVariables(stack[0].id);
for (const v of vars) {
  console.log(`${v.name}: ${v.value} (${v.type_name})`);
}

// Evaluate expression
const result = await evaluate("user.name + ' - ' + user.email");
console.log("Result:", result);
```

## File Structure

```text
crates/ext_debugger/
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
pub struct DebuggerSession {
    pub id: u32,
    pub connected: bool,
}

#[weld_struct]
#[derive(Debug, Serialize)]
pub struct Breakpoint {
    pub id: u32,
    pub file: String,
    pub line: u32,
    pub condition: Option<String>,
}

#[weld_struct]
#[derive(Debug, Serialize)]
pub struct StackFrame {
    pub id: u32,
    pub name: String,
    pub file: String,
    pub line: u32,
    pub column: u32,
}

#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_debugger_connect(
    state: Rc<RefCell<OpState>>,
    #[serde] opts: Option<ConnectOptions>,
) -> Result<DebuggerSession, DebuggerError> {
    // implementation
}

#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_debugger_set_breakpoint(
    state: Rc<RefCell<OpState>>,
    #[string] file: String,
    #[smi] line: u32,
    #[serde] opts: Option<BreakpointOptions>,
) -> Result<Breakpoint, DebuggerError> {
    // implementation
}
```

## Build Configuration

```rust
// build.rs
use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_debugger", "runtime:debugger")
        .ts_path("ts/init.ts")
        .ops(&["op_debugger_connect", "op_debugger_disconnect", "op_debugger_set_breakpoint", /* ... */])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build runtime_debugger extension");
}
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `deno_core` | Op definitions |
| `tokio` | Async runtime |
| `tokio-tungstenite` | WebSocket for CDP |
| `serde` | Serialization |
| `serde_json` | JSON protocol messages |
| `forge-weld` | Build-time code generation |
| `forge-weld-macro` | `#[weld_op]`, `#[weld_struct]` macros |
| `linkme` | Compile-time symbol collection |

## Related

- [ext_devtools](/docs/crates/ext-devtools) - DevTools integration
- [ext_trace](/docs/crates/ext-trace) - Application tracing
- [Architecture](/docs/architecture) - Full system architecture
