---
title: "ext_debugger"
description: V8 Inspector Protocol debugging extension providing the runtime:debugger module.
slug: crates/ext-debugger
---

The `ext_debugger` crate provides comprehensive debugging capabilities for Forge applications through the `runtime:debugger` module, implementing a complete Chrome DevTools Protocol (CDP) client for V8 runtime introspection.

## Overview

ext_debugger enables full programmatic control over JavaScript/TypeScript execution via the V8 Inspector Protocol. It provides WebSocket-based communication with V8's debugging infrastructure, allowing you to set breakpoints, step through code, inspect variables, evaluate expressions, and monitor script loading - all from TypeScript.

### Key Capabilities

- **Breakpoint Management**: Set, remove, enable/disable breakpoints with optional conditions
- **Execution Control**: Pause, resume, step over/into/out of functions
- **Stack Inspection**: Access call frames, scope chains, and variable values
- **Object Inspection**: Fetch properties of complex runtime objects
- **Expression Evaluation**: Execute arbitrary JavaScript in global or frame context
- **Script Management**: List loaded scripts and retrieve source code
- **Event Handling**: React to pause and script loading events
- **Exception Debugging**: Configure pause-on-exception behavior

## Module: `runtime:debugger`

```typescript
import {
  // Connection
  connect,
  disconnect,
  isConnected,

  // Breakpoints
  setBreakpoint,
  removeBreakpoint,
  removeAllBreakpoints,
  listBreakpoints,
  enableBreakpoint,
  disableBreakpoint,

  // Execution Control
  pause,
  resume,
  stepOver,
  stepInto,
  stepOut,
  continueToLocation,
  setPauseOnExceptions,

  // Inspection
  getCallFrames,
  getScopeChain,
  getProperties,
  evaluate,
  setVariableValue,

  // Scripts
  getScriptSource,
  listScripts,

  // Events
  onPaused,
  onScriptParsed
} from "runtime:debugger";
```

## Quick Start

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

// Cleanup when done
cleanup();
await debugger.disconnect();
```

## Connection Management

### connect()

Establish WebSocket connection to V8 Inspector.

```typescript
import { connect } from "runtime:debugger";

// Connect with defaults (localhost:9229)
await connect();

// Connect with custom options
await connect({
  url: "ws://localhost:9229",
  timeout: 5000
});
```

**Options:**
- `url`: Inspector WebSocket URL (default: `ws://localhost:9229`)
- `timeout`: Connection timeout in milliseconds (default: 3000)

**Throws:**
- Error [9601] if connection fails
- Error [9613] if connection times out

### disconnect()

Close the inspector connection and cleanup resources.

```typescript
import { disconnect } from "runtime:debugger";

await disconnect();
```

### isConnected()

Check if currently connected to the inspector.

```typescript
import { isConnected } from "runtime:debugger";

if (await isConnected()) {
  console.log("Debugger is connected");
}
```

## Breakpoint Management

### setBreakpoint()

Set a breakpoint at a specific file and line.

```typescript
import { setBreakpoint } from "runtime:debugger";

// Simple breakpoint
const bp1 = await setBreakpoint("file:///src/main.ts", 42);

// Conditional breakpoint
const bp2 = await setBreakpoint("file:///src/auth.ts", 100, {
  condition: "user.role === 'admin'"
});

// Column-specific breakpoint
const bp3 = await setBreakpoint("file:///src/utils.ts", 25, {
  column_number: 15,
  condition: "data.length > 1000"
});
```

**Important:** Line numbers are 0-based (line 1 in editor = lineNumber 0).

**Parameters:**
- `url`: Script URL (must match exactly, e.g., `file:///src/main.ts`)
- `lineNumber`: Line number (0-based)
- `options`: Optional breakpoint configuration
  - `condition`: JavaScript expression for conditional breakpoint
  - `column_number`: Column number for precise breakpoint placement

**Returns:** `Breakpoint` with V8-assigned ID and actual location (may differ from requested if V8 adjusts to nearest executable statement).

### Conditional Breakpoints

Conditional breakpoints only pause when the expression evaluates to truthy:

```typescript
// Only pause when debugging is enabled
await setBreakpoint("file:///src/app.ts", 50, {
  condition: "config.debug === true"
});

// Only pause for specific user
await setBreakpoint("file:///src/handlers.ts", 75, {
  condition: "request.userId === '12345'"
});

// Only pause when array is large
await setBreakpoint("file:///src/process.ts", 120, {
  condition: "items.length > 100"
});
```

### removeBreakpoint()

Remove a breakpoint by ID.

```typescript
import { setBreakpoint, removeBreakpoint } from "runtime:debugger";

const bp = await setBreakpoint("file:///src/main.ts", 42);
await removeBreakpoint(bp.id);
```

### removeAllBreakpoints()

Remove all active breakpoints.

```typescript
import { removeAllBreakpoints } from "runtime:debugger";

await removeAllBreakpoints();
console.log("All breakpoints cleared");
```

### listBreakpoints()

Get all active breakpoints with metadata.

```typescript
import { listBreakpoints } from "runtime:debugger";

const breakpoints = await listBreakpoints();
for (const bp of breakpoints) {
  console.log(`${bp.id}: ${bp.location.script_id}:${bp.location.line_number}`);
  console.log(`  Enabled: ${bp.enabled}, Hit count: ${bp.hit_count}`);
  if (bp.condition) {
    console.log(`  Condition: ${bp.condition}`);
  }
}
```

### enableBreakpoint() / disableBreakpoint()

Toggle breakpoints without removing them.

```typescript
import { setBreakpoint, disableBreakpoint, enableBreakpoint } from "runtime:debugger";

const bp = await setBreakpoint("file:///src/main.ts", 42);

// Temporarily disable
await disableBreakpoint(bp.id);
console.log("Breakpoint disabled");

// Re-enable later
await enableBreakpoint(bp.id);
console.log("Breakpoint enabled");
```

## Execution Control

### pause()

Pause execution at the current statement.

```typescript
import { pause } from "runtime:debugger";

await pause();
console.log("Execution will pause at next statement");
```

### resume()

Resume execution from paused state.

```typescript
import { resume, onPaused } from "runtime:debugger";

const cleanup = onPaused(async (event) => {
  console.log("Paused, resuming...");
  await resume();
});
```

### Step Operations

Control step-by-step execution:

```typescript
import { stepOver, stepInto, stepOut } from "runtime:debugger";

// Execute current line, pause at next line
await stepOver();

// Enter function call
await stepInto();

// Exit current function
await stepOut();
```

**Step Behavior:**
- `stepOver()`: Execute current line completely, pause at next line in same function
- `stepInto()`: If current line is a function call, enter the function; otherwise same as stepOver
- `stepOut()`: Execute until current function returns, pause in calling function

### continueToLocation()

Continue execution until reaching a specific location (run-to-cursor).

```typescript
import { continueToLocation } from "runtime:debugger";

// Continue to line 100 in current script
await continueToLocation({
  script_id: "42",
  line_number: 100
});
```

### setPauseOnExceptions()

Configure when to pause on exceptions.

```typescript
import { setPauseOnExceptions } from "runtime:debugger";

// Never pause on exceptions
await setPauseOnExceptions("none");

// Pause only on uncaught exceptions
await setPauseOnExceptions("uncaught");

// Pause on all exceptions (including caught)
await setPauseOnExceptions("all");
```

**States:**
- `"none"`: Normal execution, let error handlers work
- `"uncaught"`: Find exceptions that crash the app
- `"all"`: Debug exception handling logic, trace error propagation

## Stack Inspection

### getCallFrames()

Retrieve the complete call stack when paused.

```typescript
import { onPaused, getCallFrames } from "runtime:debugger";

const cleanup = onPaused(async (event) => {
  const frames = await getCallFrames();

  console.log("Call stack:");
  for (let i = 0; i < frames.length; i++) {
    const frame = frames[i];
    console.log(`${i}: ${frame.function_name} at ${frame.url}:${frame.location.line_number}`);
  }
});
```

### getScopeChain()

Get the scope chain for a specific call frame.

```typescript
import { onPaused, getScopeChain } from "runtime:debugger";

const cleanup = onPaused(async (event) => {
  const topFrame = event.call_frames[0];
  const scopes = await getScopeChain(topFrame.call_frame_id);

  console.log("Scope chain:");
  for (const scope of scopes) {
    console.log(`- ${scope.type}: ${scope.name || '(anonymous)'}`);
  }
});
```

**Scope Types:**
- `global`: Global scope
- `local`: Function local scope
- `closure`: Closure scope
- `catch`: Catch block scope
- `block`: Block scope
- `script`: Script scope
- `eval`: Eval scope
- `module`: Module scope
- `wasmExpressionStack`: WebAssembly expression stack

## Object Inspection

### getProperties()

Fetch properties of a remote object.

```typescript
import { onPaused, getProperties } from "runtime:debugger";

const cleanup = onPaused(async (event) => {
  const topFrame = event.call_frames[0];
  const localScope = topFrame.scope_chain.find(s => s.type === "local");

  if (localScope?.object.object_id) {
    const props = await getProperties(localScope.object.object_id);

    console.log("Local variables:");
    for (const prop of props) {
      if (prop.value) {
        console.log(`  ${prop.name}: ${prop.value.description} (${prop.value.type})`);
      }
    }
  }
});
```

**Property Types:**
- **Data properties**: Regular properties with values
- **Accessor properties**: Getter/setter properties
- **Internal properties**: V8 internal properties (prefixed with `[[]]`)

### Remote Objects

V8 uses two representations for values:

**Primitives** (sent inline):
```typescript
{
  type: "number",
  value: 42,
  description: "42"
}
```

**Complex Objects** (require getProperties):
```typescript
{
  type: "object",
  subtype: "array",
  object_id: "obj-123",  // Use this to fetch properties
  description: "Array(5)",
  preview: { /* optional preview */ }
}
```

## Expression Evaluation

### evaluate()

Execute arbitrary JavaScript expressions.

```typescript
import { evaluate, onPaused } from "runtime:debugger";

// Global evaluation
const result1 = await evaluate("1 + 2 + 3");
console.log(result1.value);  // 6

// Evaluate in call frame context (when paused)
const cleanup = onPaused(async (event) => {
  const topFrame = event.call_frames[0];

  // Access local variables
  const result2 = await evaluate("localVar * 2", topFrame.call_frame_id);
  console.log(`localVar * 2 = ${result2.value}`);

  // Complex expressions
  const result3 = await evaluate(`
    users.filter(u => u.role === 'admin')
         .map(u => u.name)
         .join(', ')
  `, topFrame.call_frame_id);
  console.log(`Admins: ${result3.value}`);
});
```

**Capabilities:**
- Access and modify global state
- Access local variables and closures (in frame context)
- Call functions and produce side effects
- Return complex objects (via remote object reference)

### setVariableValue()

Modify variable values during debugging.

```typescript
import { setVariableValue, onPaused } from "runtime:debugger";

const cleanup = onPaused(async (event) => {
  const topFrame = event.call_frames[0];

  // Modify local variable (scope 0)
  await setVariableValue(0, "counter", 100, topFrame.call_frame_id);

  // Modify closure variable (scope 1)
  await setVariableValue(1, "config", { debug: true }, topFrame.call_frame_id);
});
```

**Parameters:**
- `scopeNumber`: Index in scope chain (0 = local, higher = outer scopes)
- `variableName`: Name of variable to modify
- `newValue`: New value to assign
- `callFrameId`: Call frame ID from pause event

## Script Management

### listScripts()

List all loaded scripts.

```typescript
import { listScripts } from "runtime:debugger";

const scripts = await listScripts();

// Filter to application scripts
const appScripts = scripts.filter(s =>
  s.url.startsWith("file://") && !s.url.includes("node_modules")
);

console.log("Application scripts:");
for (const script of appScripts) {
  console.log(`  ${script.url}`);
  console.log(`    ID: ${script.script_id}, Lines: ${script.start_line}-${script.end_line}`);
}
```

### getScriptSource()

Retrieve source code by script ID.

```typescript
import { getScriptSource, listScripts } from "runtime:debugger";

const scripts = await listScripts();
const mainScript = scripts.find(s => s.url.endsWith("/main.ts"));

if (mainScript) {
  const source = await getScriptSource(mainScript.script_id);
  console.log(`Source of ${mainScript.url}:\n${source}`);
}
```

## Event Handling

### onPaused()

Listen for pause events.

```typescript
import { onPaused, setBreakpoint, resume } from "runtime:debugger";

// Set up breakpoint
await setBreakpoint("file:///src/main.ts", 42);

// Listen for pause events
const cleanup = onPaused(async (event) => {
  console.log(`Paused: ${event.reason}`);

  // Handle different pause reasons
  switch (event.reason) {
    case "breakpoint":
      console.log("Hit breakpoint");
      break;
    case "exception":
      console.log("Exception:", event.data?.exception?.description);
      break;
    case "debugCommand":
      console.log("Manual pause or step");
      break;
    default:
      console.log("Other pause:", event.reason);
  }

  // Print stack trace
  for (const frame of event.call_frames) {
    console.log(`  at ${frame.function_name} (${frame.url}:${frame.location.line_number})`);
  }

  await resume();
});

// Later: stop listening
cleanup();
```

**Pause Reasons:**
- `"breakpoint"`: Breakpoint hit
- `"exception"`: Exception thrown
- `"promiseRejection"`: Unhandled promise rejection
- `"debugCommand"`: Manual pause or step operation
- `"assert"`: Assertion failed
- `"OOM"`: Out of memory
- And others...

### onScriptParsed()

Listen for script loading events.

```typescript
import { onScriptParsed, setBreakpoint } from "runtime:debugger";

// Auto-set breakpoints in new app modules
const cleanup = onScriptParsed(async (script) => {
  if (script.url.startsWith("file://") && !script.url.includes("node_modules")) {
    console.log(`New script loaded: ${script.url}`);

    // Set breakpoint at first line
    await setBreakpoint(script.url, 0);
  }
});

// Later: stop listening
cleanup();
```

## Advanced Patterns

### Interactive Debugging REPL

```typescript
import * as debugger from "runtime:debugger";
import * as readline from "node:readline/promises";

await debugger.connect();
await debugger.setBreakpoint("file:///src/main.ts", 42);

const rl = readline.createInterface({
  input: Deno.stdin,
  output: Deno.stdout
});

const cleanup = debugger.onPaused(async (event) => {
  console.log(`\nPaused at ${event.call_frames[0].url}:${event.call_frames[0].location.line_number}`);

  while (true) {
    const command = await rl.question("debug> ");

    if (command === "continue" || command === "c") {
      await debugger.resume();
      break;
    } else if (command === "step" || command === "s") {
      await debugger.stepOver();
      break;
    } else if (command === "locals") {
      const topFrame = event.call_frames[0];
      const localScope = topFrame.scope_chain.find(s => s.type === "local");
      if (localScope?.object.object_id) {
        const props = await debugger.getProperties(localScope.object.object_id);
        for (const prop of props) {
          if (prop.value) {
            console.log(`  ${prop.name} = ${prop.value.description}`);
          }
        }
      }
    } else if (command.startsWith("eval ")) {
      const expr = command.slice(5);
      const result = await debugger.evaluate(expr, event.call_frames[0].call_frame_id);
      console.log(`  => ${result.description || result.value}`);
    }
  }
});
```

### Code Coverage Tracking

```typescript
import { onScriptParsed, listScripts } from "runtime:debugger";

const loadedScripts = new Set<string>();

const cleanup = onScriptParsed((script) => {
  if (script.url.startsWith("file://")) {
    loadedScripts.add(script.url);
  }
});

// Later: analyze coverage
setTimeout(async () => {
  const allScripts = await listScripts();
  const appScripts = allScripts.filter(s => s.url.startsWith("file://"));

  console.log(`Loaded: ${loadedScripts.size}/${appScripts.length} scripts`);

  const notLoaded = appScripts.filter(s => !loadedScripts.has(s.url));
  if (notLoaded.length > 0) {
    console.log("\nNever loaded:");
    for (const script of notLoaded) {
      console.log(`  ${script.url}`);
    }
  }
}, 10000);
```

### Watchpoint Simulation

```typescript
import { onPaused, evaluate, setBreakpoint, resume } from "runtime:debugger";

// Watch a variable by setting conditional breakpoint
await setBreakpoint("file:///src/app.ts", 50, {
  condition: "oldValue !== currentValue"
});

const cleanup = onPaused(async (event) => {
  const oldValue = await evaluate("oldValue", event.call_frames[0].call_frame_id);
  const newValue = await evaluate("currentValue", event.call_frames[0].call_frame_id);

  console.log(`Variable changed: ${oldValue.value} => ${newValue.value}`);

  await resume();
});
```

## Error Handling

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

### Error Handling Patterns

```typescript
import { connect, setBreakpoint } from "runtime:debugger";

// Connection errors
try {
  await connect({ timeout: 1000 });
} catch (error) {
  if (error.message.includes("[9601]")) {
    console.error("Inspector not available - is --inspect enabled?");
  } else if (error.message.includes("[9613]")) {
    console.error("Connection timed out");
  }
}

// Breakpoint errors
try {
  await setBreakpoint("file:///src/missing.ts", 100);
} catch (error) {
  if (error.message.includes("[9614]")) {
    console.error("Invalid breakpoint location");
  } else if (error.message.includes("[9602]")) {
    console.error("Not connected to debugger");
  }
}

// Evaluation errors
try {
  const result = await evaluate("nonexistent.property");
} catch (error) {
  if (error.message.includes("[9606]")) {
    console.error("Evaluation failed:", error);
  }
}
```

## Best Practices

### 1. Always Clean Up Event Listeners

```typescript
// Good: Store and call cleanup function
const cleanup = onPaused((event) => {
  console.log("Paused");
});

// Later
cleanup();

// Bad: No cleanup (memory leak)
onPaused((event) => {
  console.log("Paused");
});
```

### 2. Handle Connection State

```typescript
// Good: Check connection before operations
if (await isConnected()) {
  await setBreakpoint("file:///src/main.ts", 42);
} else {
  await connect();
  await setBreakpoint("file:///src/main.ts", 42);
}

// Bad: Assume always connected
await setBreakpoint("file:///src/main.ts", 42);  // May throw [9602]
```

### 3. Use Conditional Breakpoints for Efficiency

```typescript
// Good: Condition in breakpoint (evaluated by V8)
await setBreakpoint("file:///src/loop.ts", 10, {
  condition: "i === 1000"  // Only pause once
});

// Bad: Manual condition check (pauses 1000 times)
await setBreakpoint("file:///src/loop.ts", 10);
onPaused(async (event) => {
  const i = await evaluate("i", event.call_frames[0].call_frame_id);
  if (i.value === 1000) {
    // Do something
  }
  await resume();
});
```

### 4. Fetch Only Needed Properties

```typescript
// Good: Fetch specific properties
const props = await getProperties(objectId);
const neededProps = props.filter(p =>
  ["name", "email", "role"].includes(p.name)
);

// Bad: Fetch all properties of large objects
const allProps = await getProperties(largeObjectId);  // Slow for huge objects
```

### 5. Use 0-Based Line Numbers Correctly

```typescript
// Good: Remember 0-based indexing
const editorLine = 42;  // Line 42 in editor (1-based)
await setBreakpoint("file:///src/main.ts", editorLine - 1);  // Line 41 (0-based)

// Bad: Direct use of editor line number
await setBreakpoint("file:///src/main.ts", 42);  // Will be line 43 in editor!
```

## Common Pitfalls

### Pitfall 1: Forgetting Async/Await

```typescript
// Wrong: Missing await
const bp = setBreakpoint("file:///src/main.ts", 42);  // Returns Promise
console.log(bp.id);  // undefined - bp is a Promise!

// Correct: Use await
const bp = await setBreakpoint("file:///src/main.ts", 42);
console.log(bp.id);  // Correct breakpoint ID
```

### Pitfall 2: Not Handling V8 Breakpoint Adjustment

```typescript
// Request breakpoint at line 10 (comment line)
const bp = await setBreakpoint("file:///src/main.ts", 10);

// V8 may adjust to nearest executable statement
console.log(`Requested: 10, Actual: ${bp.location.line_number}`);
// Output: "Requested: 10, Actual: 12" (adjusted to next code line)
```

### Pitfall 3: Blocking in Pause Handler

```typescript
// Wrong: Blocking operation in pause handler
onPaused(async (event) => {
  while (true) {
    // This blocks the event loop!
    await someSlowOperation();
  }
});

// Correct: Resume to allow execution to continue
onPaused(async (event) => {
  console.log("Paused");
  await resume();  // Always resume or step
});
```

### Pitfall 4: Incorrect Remote Object Inspection

```typescript
// Wrong: Trying to access value directly
const localScope = frame.scope_chain.find(s => s.type === "local");
console.log(localScope.object.value);  // undefined for complex objects!

// Correct: Use getProperties for complex objects
if (localScope?.object.object_id) {
  const props = await getProperties(localScope.object.object_id);
  console.log(props);
}
```

## Performance Considerations

### WebSocket Latency

Each debugger operation requires round-trip WebSocket communication (~1ms localhost):

```typescript
// Slow: 3 round trips
const frames = await getCallFrames();  // ~1ms
const scopes = await getScopeChain(frames[0].call_frame_id);  // ~1ms
const props = await getProperties(scopes[0].object.object_id);  // ~1ms
// Total: ~3ms

// Faster: Batch operations when possible
const cleanup = onPaused(async (event) => {
  // Use event.call_frames instead of getCallFrames()
  const topFrame = event.call_frames[0];
  const scopes = topFrame.scope_chain;  // Already included!

  if (scopes[0].object.object_id) {
    const props = await getProperties(scopes[0].object.object_id);  // Only 1 round trip
  }
});
```

### Large Object Inspection

Fetching properties of large objects can be slow:

```typescript
// Slow: Fetch all properties of huge object
const allProps = await getProperties(hugeArrayId);  // May take 100ms+

// Faster: Fetch only needed properties
const allProps = await getProperties(hugeArrayId);
const needed = allProps.slice(0, 10);  // Only use first 10
```

### Event Frequency

High-frequency events can overwhelm listeners:

```typescript
// Problematic: Pause in tight loop
await setBreakpoint("file:///src/loop.ts", 5);  // Inside loop
onPaused(async (event) => {
  console.log("Paused");  // Will fire 1000+ times!
  await resume();
});

// Better: Use conditional breakpoint
await setBreakpoint("file:///src/loop.ts", 5, {
  condition: "i % 100 === 0"  // Only pause every 100 iterations
});
```

## Troubleshooting

### "Not connected" Errors

**Problem:** `Error [9602]: Not connected to debugger`

**Solutions:**
```typescript
// Ensure connection before operations
if (!await isConnected()) {
  await connect();
}

// Or use try-catch
try {
  await setBreakpoint("file:///src/main.ts", 42);
} catch (error) {
  if (error.message.includes("[9602]")) {
    await connect();
    await setBreakpoint("file:///src/main.ts", 42);
  }
}
```

### "Connection failed" Errors

**Problem:** `Error [9601]: Failed to connect to inspector`

**Causes:**
- V8 Inspector not enabled (missing `--inspect` flag)
- Inspector running on different port
- Inspector already connected by another client

**Solutions:**
```typescript
// Check inspector is enabled
// Run with: deno run --inspect script.ts

// Try different port
await connect({ url: "ws://localhost:9230" });

// Check for existing connections
// Close Chrome DevTools if open
```

### Breakpoint Not Hit

**Problem:** Breakpoint set but never triggers

**Common Causes:**
1. **Wrong URL format**
   ```typescript
   // Wrong: Relative path
   await setBreakpoint("src/main.ts", 42);

   // Correct: Absolute file:// URL
   await setBreakpoint("file:///path/to/src/main.ts", 42);
   ```

2. **Code never executed**
   ```typescript
   // Breakpoint in dead code won't trigger
   if (false) {
     console.log("This never runs");  // Breakpoint here won't hit
   }
   ```

3. **Line number off-by-one**
   ```typescript
   // Remember: 0-based indexing
   await setBreakpoint("file:///src/main.ts", 41);  // Line 42 in editor
   ```

### Evaluation Failures

**Problem:** `Error [9606]: Evaluation failed`

**Causes:**
- Syntax error in expression
- Variable doesn't exist in scope
- Exception thrown during evaluation

**Solutions:**
```typescript
try {
  const result = await evaluate("nonexistent.property");
} catch (error) {
  console.error("Evaluation failed:", error.message);
  // Try simpler expression or check variable exists
}
```

## See Also

- [ext_devtools](./ext-devtools) - DevTools frontend integration
- [ext_trace](./ext-trace) - Application tracing and profiling
- [Architecture](/docs/architecture) - Forge system architecture
- [Chrome DevTools Protocol](https://chromedevtools.github.io/devtools-protocol/)
- [V8 Inspector Protocol](https://chromedevtools.github.io/devtools-protocol/v8/)
