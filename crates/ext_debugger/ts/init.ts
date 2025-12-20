/**
 * @module runtime:debugger
 *
 * V8 Inspector Protocol debugger extension for comprehensive debugging capabilities.
 *
 * Provides a complete debugging API accessible from TypeScript for breakpoint management,
 * execution control, stack inspection, expression evaluation, and script analysis using
 * the V8 Inspector Protocol (Chrome DevTools Protocol).
 *
 * ## Features
 *
 * ### Connection Management
 * - Connect to V8 inspector via WebSocket (local or remote)
 * - Check connection status and debugger state
 * - Automatic debugger domain and runtime enable
 * - Configurable timeout and connection URL
 *
 * ### Breakpoint Management
 * - Set breakpoints by URL and line number
 * - Conditional breakpoints with JavaScript expressions
 * - Enable/disable breakpoints without removing them
 * - Remove individual or all breakpoints
 * - List all active breakpoints with hit counts
 * - Configure exception pause behavior (none, uncaught, all)
 * - Automatic breakpoint resolution to actual locations
 *
 * ### Execution Control
 * - Pause and resume execution
 * - Step over, into, and out of functions
 * - Continue to specific location
 * - Track execution state (paused, running)
 *
 * ### Stack Inspection
 * - Get complete call stack when paused
 * - Access call frame details (function name, location, URL)
 * - Inspect scope chains (global, local, closure, etc.)
 * - Access `this` object and return values
 *
 * ### Object Inspection
 * - Get object properties via remote object IDs
 * - Filter own vs inherited properties
 * - Access getters/setters and property descriptors
 * - Inspect complex objects with previews
 *
 * ### Expression Evaluation
 * - Evaluate JavaScript expressions globally
 * - Evaluate in specific call frame context
 * - Access local variables and scope during evaluation
 * - Modify variable values in scope
 *
 * ### Script Management
 * - Get source code of parsed scripts
 * - List all loaded scripts with metadata
 * - Access script URLs and source maps
 * - Track script parsing events
 *
 * ### Event Handling
 * - Listen for pause events (breakpoint, exception, step)
 * - Subscribe to script parsed events
 * - Receive breakpoint resolution notifications
 * - Event-based async architecture
 *
 * ## Error Codes (9600-9614)
 *
 * | Code | Error | Description |
 * |------|-------|-------------|
 * | 9600 | Generic | Generic debugger error |
 * | 9601 | ConnectionFailed | Failed to connect to inspector |
 * | 9602 | NotConnected | Not connected to inspector |
 * | 9603 | BreakpointFailed | Breakpoint operation failed |
 * | 9604 | InvalidFrameId | Invalid call frame ID |
 * | 9605 | InvalidScopeId | Invalid scope ID |
 * | 9606 | EvaluationFailed | Expression evaluation failed |
 * | 9607 | SourceNotFound | Script source not found |
 * | 9608 | StepFailed | Step operation failed |
 * | 9609 | PauseFailed | Pause operation failed |
 * | 9610 | ResumeFailed | Resume operation failed |
 * | 9611 | ProtocolError | V8 protocol error |
 * | 9612 | NotEnabled | Inspector not enabled |
 * | 9613 | Timeout | Operation timeout |
 * | 9614 | InvalidLocation | Invalid breakpoint location |
 *
 * ## V8 Inspector Protocol
 *
 * This extension uses the V8 Inspector Protocol (Chrome DevTools Protocol) for all debugging
 * operations. The protocol provides:
 * - Full control over JavaScript execution
 * - Complete introspection of runtime state
 * - Standardized messaging format
 * - WebSocket-based communication
 *
 * ## Connection
 *
 * Connect to the V8 inspector (usually at ws://127.0.0.1:9229) to enable debugging:
 *
 * ```typescript
 * import { connect } from "runtime:debugger";
 *
 * const status = await connect();
 * console.log(`Connected: ${status.connected}, Enabled: ${status.enabled}`);
 * ```
 *
 * ## Thread Safety
 *
 * All operations are async and thread-safe. The debugger state is managed internally with
 * proper synchronization.
 *
 * ## Performance
 *
 * - Breakpoints have minimal overhead when not hit
 * - Expression evaluation is fast for simple expressions
 * - Object inspection requires protocol round-trips
 * - Event receivers use broadcast channels for efficiency
 *
 * @example
 * ```typescript
 * import * as debugger from "runtime:debugger";
 *
 * // Connect and set breakpoint
 * await debugger.connect();
 * const bp = await debugger.setBreakpoint("file:///src/main.ts", 42);
 *
 * // Listen for pause events
 * const cleanup = debugger.onPaused(async (event) => {
 *   console.log(`Paused: ${event.reason}`);
 *   const frames = event.call_frames;
 *   console.log(`At: ${frames[0].function_name}`);
 *
 *   // Inspect variables
 *   const scopes = frames[0].scope_chain;
 *   for (const scope of scopes) {
 *     if (scope.object.object_id) {
 *       const props = await debugger.getProperties(scope.object.object_id);
 *       console.log("Variables:", props.map(p => p.name));
 *     }
 *   }
 *
 *   // Resume execution
 *   await debugger.resume();
 * });
 *
 * // Later: disconnect
 * cleanup();
 * await debugger.disconnect();
 * ```
 */

// ============================================================================
// Type Definitions
// ============================================================================

/**
 * Extension metadata and version information.
 *
 * @example
 * ```typescript
 * import { info } from "runtime:debugger";
 *
 * const metadata = info();
 * console.log(`${metadata.name} v${metadata.version} - ${metadata.status}`);
 * // Output: ext_debugger v0.1.0-alpha.1 - active
 * ```
 */
export interface ExtensionInfo {
  /** Extension name (always "ext_debugger") */
  name: string;
  /** Extension version from Cargo.toml */
  version: string;
  /** Extension status (typically "active") */
  status: string;
}

/**
 * Source code location in a script.
 *
 * Identifies a specific position in JavaScript code using script ID, line, and column.
 * Line and column numbers are 0-based as per V8 Inspector Protocol.
 *
 * @example
 * ```typescript
 * const location: Location = {
 *   script_id: "42",
 *   line_number: 10,  // Line 11 in editor (0-based)
 *   column_number: 5   // Column 6 in editor (0-based)
 * };
 * ```
 */
export interface Location {
  /** Script identifier from V8 */
  script_id: string;
  /** Line number (0-based, so line 1 in editor is line_number: 0) */
  line_number: number;
  /** Column number (0-based, optional) */
  column_number?: number;
}

/**
 * Breakpoint information with location and hit statistics.
 *
 * Represents an active breakpoint set in the debugger. V8 may adjust the actual
 * location from the requested location (e.g., moving to next executable line).
 *
 * @example
 * ```typescript
 * const bp: Breakpoint = {
 *   id: "1:10:0",
 *   location: { script_id: "42", line_number: 10, column_number: 0 },
 *   condition: "x > 10",
 *   hit_count: 5,
 *   enabled: true
 * };
 * ```
 */
export interface Breakpoint {
  /** Unique breakpoint identifier assigned by V8 */
  id: string;
  /** Actual location (may differ from requested due to V8 adjustment to nearest executable line) */
  location: Location;
  /** Conditional expression (breakpoint only triggers when condition evaluates to true) */
  condition?: string;
  /** Number of times this breakpoint has been hit */
  hit_count: number;
  /** Whether breakpoint is currently enabled */
  enabled: boolean;
}

/**
 * Options for setting a breakpoint.
 *
 * Allows configuring conditional breakpoints and column-specific breakpoints.
 *
 * @example
 * ```typescript
 * // Conditional breakpoint
 * const opts1: BreakpointOptions = {
 *   condition: "user.role === 'admin'"
 * };
 *
 * // Column-specific breakpoint
 * const opts2: BreakpointOptions = {
 *   column_number: 15,
 *   condition: "data.length > 100"
 * };
 * ```
 */
export interface BreakpointOptions {
  /** JavaScript expression that must evaluate to true for breakpoint to trigger */
  condition?: string;
  /** Specific column number for precise breakpoint placement (0-based) */
  column_number?: number;
}

/**
 * Scope types in JavaScript execution.
 *
 * Represents different kinds of scopes that can appear in a scope chain when execution
 * is paused. Each scope type contains different kinds of variables and has different rules.
 *
 * - `"global"`: Global scope containing global variables
 * - `"local"`: Local function scope with function parameters and local variables
 * - `"closure"`: Closure scope containing variables captured from outer function
 * - `"catch"`: Catch block scope containing the exception variable
 * - `"block"`: Block scope (let/const) within a function
 * - `"script"`: Script-level scope for module scripts
 * - `"eval"`: Eval scope for code evaluated via eval()
 * - `"module"`: ES module scope
 * - `"wasmExpressionStack"`: WebAssembly expression stack
 */
export type ScopeType =
  | "global"
  | "local"
  | "closure"
  | "catch"
  | "block"
  | "script"
  | "eval"
  | "module"
  | "wasmExpressionStack";

/**
 * Scope in the call stack containing variables.
 *
 * Represents a single scope in the scope chain of a call frame. Access the scope's
 * variables by getting properties of the `object` using its `object_id`.
 *
 * @example
 * ```typescript
 * import { getCallFrames, getProperties } from "runtime:debugger";
 *
 * const frames = await getCallFrames();
 * const localScope = frames[0].scope_chain.find(s => s.type === "local");
 *
 * if (localScope && localScope.object.object_id) {
 *   const variables = await getProperties(localScope.object.object_id);
 *   for (const v of variables) {
 *     console.log(`${v.name}: ${v.value?.value}`);
 *   }
 * }
 * ```
 */
export interface Scope {
  /** Type of scope (global, local, closure, etc.) */
  type: ScopeType;
  /** Remote object representing the scope (contains variables as properties) */
  object: RemoteObject;
  /** Scope name (typically the function name for closures) */
  name?: string;
  /** Source location where scope begins */
  start_location?: Location;
  /** Source location where scope ends */
  end_location?: Location;
}

/**
 * Remote object reference from the V8 runtime.
 *
 * Represents a JavaScript object in the debugged runtime. For primitives (number, string, boolean),
 * the value is provided directly. For complex objects (arrays, objects, functions), use the
 * `object_id` to fetch properties via `getProperties()`.
 *
 * **Object Types:**
 * - `"object"`: Regular object, array, function, or null
 * - `"function"`: Function object
 * - `"undefined"`: Undefined value
 * - `"string"`: String primitive
 * - `"number"`: Number primitive
 * - `"boolean"`: Boolean primitive
 * - `"symbol"`: Symbol primitive
 * - `"bigint"`: BigInt primitive
 *
 * **Subtypes** (when type is "object"):
 * - `"array"`, `"null"`, `"regexp"`, `"date"`, `"map"`, `"set"`, `"weakmap"`, `"weakset"`,
 *   `"iterator"`, `"generator"`, `"error"`, `"proxy"`, `"promise"`, `"typedarray"`, etc.
 *
 * @example
 * ```typescript
 * // Primitive value
 * const primitiveObj: RemoteObject = {
 *   type: "number",
 *   value: 42,
 *   description: "42"
 * };
 *
 * // Complex object (requires getProperties to inspect)
 * const complexObj: RemoteObject = {
 *   type: "object",
 *   subtype: "array",
 *   class_name: "Array",
 *   object_id: "obj-123",
 *   description: "Array(5)",
 *   preview: { ... }  // Optional preview of first few properties
 * };
 *
 * if (complexObj.object_id) {
 *   const props = await getProperties(complexObj.object_id);
 *   console.log("Properties:", props);
 * }
 * ```
 */
export interface RemoteObject {
  /** Object type (object, function, undefined, string, number, boolean, symbol, bigint) */
  type: string;
  /** Object subtype when type is "object" (array, null, regexp, date, map, set, error, promise, etc.) */
  subtype?: string;
  /** Class/constructor name (e.g., "Array", "Object", "MyClass") */
  class_name?: string;
  /** Primitive value (only present for primitives and null) */
  value?: any;
  /** Unique object ID for fetching properties (only for complex objects) */
  object_id?: string;
  /** Human-readable description (e.g., "Array(3)", "function foo()", "[object Object]") */
  description?: string;
  /** Optional preview showing first few properties without full fetch */
  preview?: ObjectPreview;
}

/** Object property preview */
export interface ObjectPreview {
  type: string;
  subtype?: string;
  description?: string;
  overflow: boolean;
  properties: PropertyPreview[];
  entries?: EntryPreview[];
}

export interface PropertyPreview {
  name: string;
  type: string;
  subtype?: string;
  value?: string;
  value_preview?: ObjectPreview;
}

export interface EntryPreview {
  key?: ObjectPreview;
  value: ObjectPreview;
}

/**
 * Call frame in the execution stack.
 *
 * Represents a single frame in the call stack when execution is paused. Contains
 * the function name, location, scope chain, and the `this` binding. Use the
 * `call_frame_id` for operations like expression evaluation in this frame's context.
 *
 * @example
 * ```typescript
 * import { getCallFrames, evaluate } from "runtime:debugger";
 *
 * const frames = await getCallFrames();
 * const topFrame = frames[0];
 *
 * console.log(`Stopped in: ${topFrame.function_name}`);
 * console.log(`At: ${topFrame.url}:${topFrame.location.line_number}`);
 * console.log(`Scopes: ${topFrame.scope_chain.map(s => s.type).join(", ")}`);
 *
 * // Evaluate expression in this frame's context
 * const result = await evaluate("localVar + 10", topFrame.call_frame_id);
 * console.log("Result:", result.value);
 * ```
 */
export interface CallFrame {
  /** Unique identifier for this call frame (used for evaluate, setVariableValue) */
  call_frame_id: string;
  /** Name of the function (empty string for anonymous functions) */
  function_name: string;
  /** Location in source code where execution is paused */
  location: Location;
  /** Script URL (file:///path/to/file.ts or internal V8 URLs) */
  url: string;
  /** Scope chain from innermost (local) to outermost (global) */
  scope_chain: Scope[];
  /** The `this` binding for this function */
  this_object: RemoteObject;
  /** Return value if paused on function return statement */
  return_value?: RemoteObject;
}

/**
 * Property descriptor for an object property.
 *
 * Describes a property retrieved via `getProperties()`. Can be a data property
 * (with value) or an accessor property (with get/set functions).
 *
 * @example
 * ```typescript
 * import { getProperties } from "runtime:debugger";
 *
 * const props = await getProperties(objectId, true);
 * for (const prop of props) {
 *   if (prop.value) {
 *     console.log(`${prop.name}: ${prop.value.value}`);
 *   } else if (prop.get || prop.set) {
 *     console.log(`${prop.name}: [getter/setter]`);
 *   }
 * }
 * ```
 */
export interface PropertyDescriptor {
  /** Property name (or index for array elements) */
  name: string;
  /** Property value (for data properties) */
  value?: RemoteObject;
  /** Whether the property value can be changed */
  writable?: boolean;
  /** Getter function (for accessor properties) */
  get?: RemoteObject;
  /** Setter function (for accessor properties) */
  set?: RemoteObject;
  /** Whether the property can be deleted or have its attributes changed */
  configurable: boolean;
  /** Whether the property shows up in for-in loops and Object.keys() */
  enumerable: boolean;
  /** Whether this is an own property (not inherited from prototype) */
  is_own?: boolean;
  /** Symbol key if the property is keyed by a symbol */
  symbol?: RemoteObject;
}

/**
 * Reasons for execution pause.
 *
 * Indicates why the debugger paused execution. Use this to determine how to
 * respond to a pause event.
 *
 * - `"breakpoint"`: Hit a breakpoint
 * - `"step"`: Completed a step operation (stepOver, stepInto, stepOut)
 * - `"exception"`: Exception thrown
 * - `"debuggerStatement"`: Hit a `debugger;` statement in code
 * - `"ambiguous"`: Ambiguous pause reason
 * - `"assert"`: Assertion failed
 * - `"instrumentation"`: Instrumentation breakpoint
 * - `"oom"`: Out of memory
 * - `"promiseRejection"`: Unhandled promise rejection
 * - `"other"`: Other reason
 */
export type PauseReason =
  | "breakpoint"
  | "step"
  | "exception"
  | "debuggerStatement"
  | "ambiguous"
  | "assert"
  | "instrumentation"
  | "oom"
  | "promiseRejection"
  | "other";

/**
 * Event data when execution is paused.
 *
 * Delivered via `onPaused()` event listener when the debugger pauses execution.
 * Contains the reason for pausing, call stack, and any breakpoints that were hit.
 *
 * @example
 * ```typescript
 * import { onPaused, resume } from "runtime:debugger";
 *
 * const cleanup = onPaused(async (event) => {
 *   console.log(`Paused due to: ${event.reason}`);
 *
 *   if (event.reason === "breakpoint" && event.hit_breakpoints) {
 *     console.log(`Hit breakpoints: ${event.hit_breakpoints.join(", ")}`);
 *   }
 *
 *   // Print call stack
 *   for (const frame of event.call_frames) {
 *     console.log(`  ${frame.function_name} at ${frame.url}:${frame.location.line_number}`);
 *   }
 *
 *   // Auto-resume after logging
 *   await resume();
 * });
 *
 * // Later: stop listening
 * cleanup();
 * ```
 */
export interface PausedEvent {
  /** Reason execution was paused */
  reason: PauseReason;
  /** Call stack at pause point (frames[0] is the topmost/current frame) */
  call_frames: CallFrame[];
  /** IDs of breakpoints that were hit (only for reason: "breakpoint") */
  hit_breakpoints?: string[];
  /** Additional data about the pause (exception details, instrumentation data, etc.) */
  data?: any;
  /** Async stack trace for async operations */
  async_stack_trace?: any;
}

/**
 * Parsed script information.
 *
 * Represents a JavaScript/TypeScript script loaded and parsed by V8. Use the
 * `script_id` to fetch source code via `getScriptSource()`. Scripts are reported
 * via `onScriptParsed()` events as they're loaded.
 *
 * @example
 * ```typescript
 * import { listScripts, getScriptSource } from "runtime:debugger";
 *
 * const scripts = await listScripts();
 * const appScripts = scripts.filter(s => s.url.startsWith("file:///"));
 *
 * for (const script of appScripts) {
 *   console.log(`${script.url} (${script.length} bytes)`);
 *   const source = await getScriptSource(script.script_id);
 *   console.log(`First line: ${source.split("\n")[0]}`);
 * }
 * ```
 */
export interface ScriptInfo {
  /** Unique script identifier assigned by V8 */
  script_id: string;
  /** Script URL (file:///path, http://, or internal V8 URLs) */
  url: string;
  /** Source map URL for mapping compiled code back to source */
  source_map_url?: string;
  /** Starting line number of script in source file */
  start_line: number;
  /** Starting column number */
  start_column: number;
  /** Ending line number */
  end_line: number;
  /** Ending column number */
  end_column: number;
  /** Content hash for cache validation */
  hash: string;
  /** Script length in characters */
  length: number;
  /** Execution context ID this script belongs to */
  execution_context_id?: number;
}

/**
 * Result of a step operation.
 *
 * Returned by `stepOver()`, `stepInto()`, and `stepOut()` to indicate if the
 * step succeeded and optionally provide the pause event if execution stopped.
 *
 * @example
 * ```typescript
 * import { stepOver } from "runtime:debugger";
 *
 * const result = await stepOver();
 * if (result.success) {
 *   console.log("Step completed");
 *   if (result.paused_event) {
 *     console.log(`Stopped at: ${result.paused_event.call_frames[0].function_name}`);
 *   }
 * }
 * ```
 */
export interface StepResult {
  /** Whether the step operation succeeded */
  success: boolean;
  /** Pause event if execution stopped after stepping (e.g., hit another breakpoint) */
  paused_event?: PausedEvent;
}

/**
 * Options for connecting to the V8 inspector.
 *
 * Configure the WebSocket URL and connection timeout when connecting to the debugger.
 *
 * @example
 * ```typescript
 * import { connect } from "runtime:debugger";
 *
 * // Connect to default local inspector
 * await connect();
 *
 * // Connect to specific URL with custom timeout
 * await connect({
 *   url: "ws://192.168.1.100:9229",
 *   timeout_ms: 10000
 * });
 * ```
 */
export interface ConnectOptions {
  /** WebSocket URL to connect to (default: "ws://127.0.0.1:9229") */
  url?: string;
  /** Connection timeout in milliseconds (default: 5000) */
  timeout_ms?: number;
}

/**
 * Connection and debugger status.
 *
 * Indicates whether connected to the inspector, whether the debugger domain is
 * enabled, and the current execution state.
 *
 * @example
 * ```typescript
 * import { isConnected } from "runtime:debugger";
 *
 * const status = await isConnected();
 * console.log(`Connected: ${status.connected}`);
 * console.log(`Enabled: ${status.enabled}`);
 * console.log(`Paused: ${status.paused}`);
 * ```
 */
export interface ConnectionStatus {
  /** Whether connected to the V8 inspector via WebSocket */
  connected: boolean;
  /** Whether the debugger domain is enabled in V8 */
  enabled: boolean;
  /** Whether execution is currently paused */
  paused: boolean;
  /** WebSocket URL if connected */
  url?: string;
}

/**
 * Exception pause behavior configuration.
 *
 * Controls when the debugger should pause on exceptions:
 * - `"none"`: Never pause on exceptions (default)
 * - `"uncaught"`: Pause only on uncaught exceptions
 * - `"all"`: Pause on all exceptions (caught and uncaught)
 *
 * @example
 * ```typescript
 * import { setPauseOnExceptions } from "runtime:debugger";
 *
 * // Pause on all exceptions for debugging
 * await setPauseOnExceptions("all");
 *
 * // Only pause on uncaught exceptions
 * await setPauseOnExceptions("uncaught");
 *
 * // Don't pause on exceptions
 * await setPauseOnExceptions("none");
 * ```
 */
export type ExceptionPauseState = "none" | "uncaught" | "all";

/** Breakpoint resolved event */
export interface BreakpointResolved {
  breakpoint_id: string;
  location: Location;
}

// ============================================================================
// Deno Core Bindings
// ============================================================================

declare const Deno: {
  core: {
    ops: {
      op_debugger_info(): ExtensionInfo;
      op_debugger_connect(options: ConnectOptions): Promise<ConnectionStatus>;
      op_debugger_disconnect(): Promise<boolean>;
      op_debugger_is_connected(): Promise<ConnectionStatus>;
      op_debugger_set_breakpoint(
        url: string,
        line_number: number,
        options: BreakpointOptions
      ): Promise<Breakpoint>;
      op_debugger_remove_breakpoint(breakpoint_id: string): Promise<boolean>;
      op_debugger_remove_all_breakpoints(): Promise<number>;
      op_debugger_list_breakpoints(): Promise<Breakpoint[]>;
      op_debugger_enable_breakpoint(breakpoint_id: string): Promise<boolean>;
      op_debugger_disable_breakpoint(breakpoint_id: string): Promise<boolean>;
      op_debugger_pause(): Promise<boolean>;
      op_debugger_resume(): Promise<boolean>;
      op_debugger_step_over(): Promise<StepResult>;
      op_debugger_step_into(): Promise<StepResult>;
      op_debugger_step_out(): Promise<StepResult>;
      op_debugger_continue_to_location(
        script_id: string,
        line_number: number,
        column_number?: number
      ): Promise<boolean>;
      op_debugger_get_call_frames(): Promise<CallFrame[]>;
      op_debugger_get_scope_chain(call_frame_id: string): Promise<Scope[]>;
      op_debugger_get_properties(
        object_id: string,
        own_properties: boolean
      ): Promise<PropertyDescriptor[]>;
      op_debugger_evaluate(
        expression: string,
        call_frame_id?: string
      ): Promise<RemoteObject>;
      op_debugger_set_variable_value(
        scope_number: number,
        variable_name: string,
        new_value: any,
        call_frame_id: string
      ): Promise<boolean>;
      op_debugger_get_script_source(script_id: string): Promise<string>;
      op_debugger_list_scripts(): Promise<ScriptInfo[]>;
      op_debugger_set_pause_on_exceptions(
        state: ExceptionPauseState
      ): Promise<boolean>;
      op_debugger_create_pause_receiver(): number;
      op_debugger_receive_pause_event(rid: number): Promise<PausedEvent>;
      op_debugger_create_script_receiver(): number;
      op_debugger_receive_script_event(rid: number): Promise<ScriptInfo>;
    };
  };
};

const { core } = Deno;
const ops = core.ops;

// ============================================================================
// Public API
// ============================================================================

/**
 * Get extension metadata and version information.
 *
 * Returns static information about the debugger extension including name, version,
 * and status. This is a synchronous operation.
 *
 * @returns Extension metadata
 *
 * @example
 * ```typescript
 * import { info } from "runtime:debugger";
 *
 * const metadata = info();
 * console.log(`Debugger extension: ${metadata.name} v${metadata.version}`);
 * console.log(`Status: ${metadata.status}`);
 * // Output:
 * // Debugger extension: ext_debugger v0.1.0-alpha.1
 * // Status: active
 * ```
 */
export function info(): ExtensionInfo {
  return ops.op_debugger_info();
}

/**
 * Connect to the V8 inspector via WebSocket.
 *
 * Establishes a WebSocket connection to the V8 Inspector Protocol endpoint and
 * enables both the Debugger and Runtime domains. Must be called before any other
 * debugging operations.
 *
 * @param options - Connection options (URL and timeout)
 * @returns Connection status indicating success
 *
 * @throws Error [9601] if connection fails (invalid URL, network error, timeout)
 * @throws Error [9611] if protocol initialization fails
 *
 * @example
 * ```typescript
 * import { connect } from "runtime:debugger";
 *
 * // Connect to default local inspector (ws://127.0.0.1:9229)
 * const status = await connect();
 * console.log(`Connected: ${status.connected}, Enabled: ${status.enabled}`);
 *
 * // Connect to specific URL with custom timeout
 * const status2 = await connect({
 *   url: "ws://192.168.1.100:9229",
 *   timeout_ms: 10000  // 10 second timeout
 * });
 *
 * // Connect to remote debugger
 * const status3 = await connect({
 *   url: "ws://remote-server:9229",
 *   timeout_ms: 5000
 * });
 * ```
 */
export async function connect(
  options: ConnectOptions = {}
): Promise<ConnectionStatus> {
  return await ops.op_debugger_connect(options);
}

/**
 * Disconnect from the V8 inspector.
 *
 * Closes the WebSocket connection and disables the debugger. All breakpoints remain
 * set in V8 but will not be accessible until reconnecting. Event listeners will
 * stop receiving events.
 *
 * @returns True if disconnected successfully
 *
 * @example
 * ```typescript
 * import { connect, disconnect } from "runtime:debugger";
 *
 * await connect();
 * // ... perform debugging ...
 * const success = await disconnect();
 * console.log(`Disconnected: ${success}`);
 * ```
 */
export async function disconnect(): Promise<boolean> {
  return await ops.op_debugger_disconnect();
}

/**
 * Check current connection status.
 *
 * Returns the current state of the debugger connection including whether connected
 * to inspector, whether debugger domain is enabled, and whether execution is paused.
 *
 * @returns Connection and debugger status
 *
 * @example
 * ```typescript
 * import { isConnected, connect } from "runtime:debugger";
 *
 * // Check before connecting
 * let status = await isConnected();
 * console.log(`Connected: ${status.connected}`);  // false
 *
 * // Connect and check again
 * await connect();
 * status = await isConnected();
 * console.log(`Connected: ${status.connected}`);  // true
 * console.log(`Enabled: ${status.enabled}`);      // true
 * console.log(`Paused: ${status.paused}`);        // false
 * ```
 */
export async function isConnected(): Promise<ConnectionStatus> {
  return await ops.op_debugger_is_connected();
}

/**
 * Set a breakpoint at a specific URL and line number.
 *
 * Creates a breakpoint that will pause execution when the specified line is reached.
 * V8 may adjust the location to the nearest executable statement. Supports conditional
 * breakpoints that only pause when the condition evaluates to true.
 *
 * **Note:** Line numbers are 0-based (line 1 in editor = lineNumber 0).
 *
 * @param url - Script URL (must match script URL exactly, e.g., "file:///src/main.ts")
 * @param lineNumber - Line number (0-based)
 * @param options - Optional breakpoint configuration (condition, column)
 * @returns The created breakpoint with V8-assigned ID and actual location
 *
 * @throws Error [9602] if not connected to inspector
 * @throws Error [9603] if breakpoint creation fails
 * @throws Error [9614] if location is invalid
 *
 * @example
 * ```typescript
 * import { setBreakpoint } from "runtime:debugger";
 *
 * // Simple breakpoint at line 42 (line 43 in editor)
 * const bp1 = await setBreakpoint("file:///src/main.ts", 42);
 * console.log(`Breakpoint set: ${bp1.id} at line ${bp1.location.line_number}`);
 *
 * // Conditional breakpoint (only pause when user.role === "admin")
 * const bp2 = await setBreakpoint("file:///src/auth.ts", 100, {
 *   condition: "user.role === 'admin'"
 * });
 *
 * // Column-specific breakpoint for precise control
 * const bp3 = await setBreakpoint("file:///src/utils.ts", 25, {
 *   column_number: 15,
 *   condition: "data.length > 1000"
 * });
 * ```
 */
export async function setBreakpoint(
  url: string,
  lineNumber: number,
  options: BreakpointOptions = {}
): Promise<Breakpoint> {
  return await ops.op_debugger_set_breakpoint(url, lineNumber, options);
}

/**
 * Remove a breakpoint by ID.
 *
 * Deletes the breakpoint from V8. Execution will no longer pause at this location.
 *
 * @param breakpointId - Unique breakpoint ID returned from setBreakpoint()
 * @returns True if removed successfully
 *
 * @throws Error [9602] if not connected to inspector
 * @throws Error [9603] if breakpoint not found or removal fails
 *
 * @example
 * ```typescript
 * import { setBreakpoint, removeBreakpoint } from "runtime:debugger";
 *
 * const bp = await setBreakpoint("file:///src/main.ts", 42);
 * console.log(`Breakpoint created: ${bp.id}`);
 *
 * // Later: remove the breakpoint
 * const removed = await removeBreakpoint(bp.id);
 * console.log(`Breakpoint removed: ${removed}`);
 * ```
 */
export async function removeBreakpoint(breakpointId: string): Promise<boolean> {
  return await ops.op_debugger_remove_breakpoint(breakpointId);
}

/**
 * Remove all breakpoints.
 *
 * Clears all breakpoints from V8 in a single operation. Useful for resetting
 * debugging state.
 *
 * @returns Number of breakpoints removed
 *
 * @throws Error [9602] if not connected to inspector
 *
 * @example
 * ```typescript
 * import { setBreakpoint, removeAllBreakpoints } from "runtime:debugger";
 *
 * // Set multiple breakpoints
 * await setBreakpoint("file:///src/main.ts", 10);
 * await setBreakpoint("file:///src/main.ts", 20);
 * await setBreakpoint("file:///src/utils.ts", 5);
 *
 * // Clear all at once
 * const count = await removeAllBreakpoints();
 * console.log(`Removed ${count} breakpoints`);  // 3
 * ```
 */
export async function removeAllBreakpoints(): Promise<number> {
  return await ops.op_debugger_remove_all_breakpoints();
}

/**
 * List all active breakpoints.
 *
 * Returns all breakpoints currently set in the debugger, including their locations,
 * conditions, hit counts, and enabled state.
 *
 * @returns Array of all breakpoints
 *
 * @example
 * ```typescript
 * import { listBreakpoints } from "runtime:debugger";
 *
 * const breakpoints = await listBreakpoints();
 * for (const bp of breakpoints) {
 *   console.log(`${bp.id}: ${bp.location.script_id}:${bp.location.line_number}`);
 *   console.log(`  Hits: ${bp.hit_count}, Enabled: ${bp.enabled}`);
 *   if (bp.condition) {
 *     console.log(`  Condition: ${bp.condition}`);
 *   }
 * }
 * ```
 */
export async function listBreakpoints(): Promise<Breakpoint[]> {
  return await ops.op_debugger_list_breakpoints();
}

/**
 * Enable a previously disabled breakpoint.
 *
 * Re-enables a breakpoint without removing and recreating it. The breakpoint will
 * pause execution again when hit.
 *
 * @param breakpointId - ID of breakpoint to enable
 * @returns True if enabled successfully
 *
 * @throws Error [9602] if not connected to inspector
 * @throws Error [9603] if breakpoint not found
 *
 * @example
 * ```typescript
 * import { setBreakpoint, disableBreakpoint, enableBreakpoint } from "runtime:debugger";
 *
 * const bp = await setBreakpoint("file:///src/main.ts", 42);
 * await disableBreakpoint(bp.id);  // Temporarily disable
 * // ... some operations ...
 * await enableBreakpoint(bp.id);    // Re-enable
 * ```
 */
export async function enableBreakpoint(breakpointId: string): Promise<boolean> {
  return await ops.op_debugger_enable_breakpoint(breakpointId);
}

/**
 * Disable a breakpoint without removing it.
 *
 * Temporarily disables a breakpoint. Execution will not pause at this location
 * until the breakpoint is re-enabled.
 *
 * @param breakpointId - ID of breakpoint to disable
 * @returns True if disabled successfully
 *
 * @throws Error [9602] if not connected to inspector
 * @throws Error [9603] if breakpoint not found
 *
 * @example
 * ```typescript
 * import { setBreakpoint, disableBreakpoint } from "runtime:debugger";
 *
 * const bp = await setBreakpoint("file:///src/main.ts", 42);
 * // Temporarily disable without removing
 * await disableBreakpoint(bp.id);
 * ```
 */
export async function disableBreakpoint(
  breakpointId: string
): Promise<boolean> {
  return await ops.op_debugger_disable_breakpoint(breakpointId);
}

/**
 * Pause execution at the next statement.
 *
 * Requests V8 to pause execution as soon as possible. Execution will stop at the next
 * statement and a `PausedEvent` will be delivered to `onPaused()` listeners.
 *
 * @returns True if pause requested successfully
 *
 * @throws Error [9602] if not connected to inspector
 * @throws Error [9609] if pause request fails
 *
 * @example
 * ```typescript
 * import { pause, onPaused } from "runtime:debugger";
 *
 * onPaused((event) => {
 *   console.log("Execution paused!");
 *   console.log("Call stack:", event.call_frames.map(f => f.function_name));
 * });
 *
 * // Pause execution
 * await pause();
 * ```
 */
export async function pause(): Promise<boolean> {
  return await ops.op_debugger_pause();
}

/**
 * Resume execution from paused state.
 *
 * Continues execution after pausing at a breakpoint or from a step operation.
 * The program will run until the next breakpoint, exception, or explicit pause.
 *
 * @returns True if resume successful
 *
 * @throws Error [9602] if not connected to inspector
 * @throws Error [9610] if resume fails
 *
 * @example
 * ```typescript
 * import { pause, resume, onPaused } from "runtime:debugger";
 *
 * onPaused(async (event) => {
 *   console.log(`Paused at: ${event.call_frames[0].function_name}`);
 *   // Inspect state, then resume
 *   await resume();
 * });
 *
 * await pause();  // Will trigger onPaused, which then resumes
 * ```
 */
export async function resume(): Promise<boolean> {
  return await ops.op_debugger_resume();
}

/**
 * Step over the current line (execute and move to next line).
 *
 * Executes the current statement and pauses at the next statement in the same function.
 * If the current line contains a function call, executes the entire function without
 * stepping into it.
 *
 * **Must be paused** before calling this function.
 *
 * @returns Step result with success status
 *
 * @throws Error [9602] if not connected to inspector
 * @throws Error [9608] if step operation fails
 *
 * @example
 * ```typescript
 * import { stepOver, onPaused } from "runtime:debugger";
 *
 * onPaused(async (event) => {
 *   if (event.reason === "breakpoint") {
 *     console.log("At breakpoint, stepping over...");
 *     const result = await stepOver();
 *     console.log(`Step completed: ${result.success}`);
 *   }
 * });
 * ```
 */
export async function stepOver(): Promise<StepResult> {
  return await ops.op_debugger_step_over();
}

/**
 * Step into the current statement (enter function calls).
 *
 * If the current statement is a function call, enters the function and pauses at
 * its first statement. If not a function call, behaves like stepOver().
 *
 * **Must be paused** before calling this function.
 *
 * @returns Step result with success status
 *
 * @throws Error [9602] if not connected to inspector
 * @throws Error [9608] if step operation fails
 *
 * @example
 * ```typescript
 * import { stepInto, onPaused } from "runtime:debugger";
 *
 * onPaused(async (event) => {
 *   // Step into function calls to debug them
 *   console.log("Stepping into...");
 *   await stepInto();
 * });
 * ```
 */
export async function stepInto(): Promise<StepResult> {
  return await ops.op_debugger_step_into();
}

/**
 * Step out of the current function (return to caller).
 *
 * Continues execution until the current function returns, then pauses at the
 * statement in the caller function.
 *
 * **Must be paused** before calling this function.
 *
 * @returns Step result with success status
 *
 * @throws Error [9602] if not connected to inspector
 * @throws Error [9608] if step operation fails
 *
 * @example
 * ```typescript
 * import { stepOut, onPaused } from "runtime:debugger";
 *
 * onPaused(async (event) => {
 *   const frame = event.call_frames[0];
 *   console.log(`In: ${frame.function_name}, stepping out...`);
 *   await stepOut();
 * });
 * ```
 */
export async function stepOut(): Promise<StepResult> {
  return await ops.op_debugger_step_out();
}

/**
 * Continue execution until a specific location is reached.
 *
 * Resumes execution but automatically pauses when the specified line in the specified
 * script is reached. Useful for "run to cursor" functionality.
 *
 * @param scriptId - Script ID to target
 * @param lineNumber - Line number to pause at (0-based)
 * @param columnNumber - Optional column number for precise location
 * @returns True if operation succeeded
 *
 * @throws Error [9602] if not connected to inspector
 *
 * @example
 * ```typescript
 * import { continueToLocation, getCallFrames } from "runtime:debugger";
 *
 * const frames = await getCallFrames();
 * const currentScript = frames[0].location.script_id;
 *
 * // Run until line 100 in current script
 * await continueToLocation(currentScript, 100);
 * ```
 */
export async function continueToLocation(
  scriptId: string,
  lineNumber: number,
  columnNumber?: number
): Promise<boolean> {
  return await ops.op_debugger_continue_to_location(
    scriptId,
    lineNumber,
    columnNumber
  );
}

/**
 * Get the current call stack when execution is paused.
 *
 * Returns all call frames from the top (current function) to the bottom (global).
 * Each frame contains the function name, location, scope chain, and `this` binding.
 *
 * **Must be paused** to get meaningful results.
 *
 * @returns Array of call frames (frames[0] is the topmost/current frame)
 *
 * @example
 * ```typescript
 * import { getCallFrames, onPaused } from "runtime:debugger";
 *
 * onPaused(async (event) => {
 *   const frames = await getCallFrames();
 *   console.log("Call stack:");
 *   for (const frame of frames) {
 *     console.log(`  ${frame.function_name} at ${frame.url}:${frame.location.line_number}`);
 *   }
 * });
 * ```
 */
export async function getCallFrames(): Promise<CallFrame[]> {
  return await ops.op_debugger_get_call_frames();
}

/**
 * Get the scope chain for a specific call frame.
 *
 * Returns all scopes accessible from the given call frame, from innermost (local)
 * to outermost (global). Use the scope objects' `object_id` to fetch variables.
 *
 * @param callFrameId - Call frame ID from a CallFrame
 * @returns Array of scopes in the scope chain
 *
 * @throws Error [9604] if call frame ID is invalid
 *
 * @example
 * ```typescript
 * import { getCallFrames, getScopeChain } from "runtime:debugger";
 *
 * const frames = await getCallFrames();
 * const scopes = await getScopeChain(frames[0].call_frame_id);
 *
 * console.log("Scopes:");
 * for (const scope of scopes) {
 *   console.log(`  ${scope.type}: ${scope.name || "(unnamed)"}`);
 * }
 * ```
 */
export async function getScopeChain(callFrameId: string): Promise<Scope[]> {
  return await ops.op_debugger_get_scope_chain(callFrameId);
}

/**
 * Get properties of a remote object.
 *
 * Fetches all properties of an object using its remote object ID. Can filter to
 * show only own properties (not inherited) or include prototype chain.
 *
 * @param objectId - Remote object ID from a RemoteObject
 * @param ownProperties - If true, only return own properties (default: true)
 * @returns Array of property descriptors
 *
 * @throws Error [9602] if not connected to inspector
 *
 * @example
 * ```typescript
 * import { getProperties, evaluate } from "runtime:debugger";
 *
 * // Evaluate to get an object
 * const obj = await evaluate("{ name: 'Alice', age: 30, role: 'admin' }");
 *
 * if (obj.object_id) {
 *   // Get all properties
 *   const props = await getProperties(obj.object_id, true);
 *   for (const prop of props) {
 *     console.log(`${prop.name}: ${prop.value?.value}`);
 *   }
 *   // Output:
 *   // name: Alice
 *   // age: 30
 *   // role: admin
 * }
 * ```
 */
export async function getProperties(
  objectId: string,
  ownProperties: boolean = true
): Promise<PropertyDescriptor[]> {
  return await ops.op_debugger_get_properties(objectId, ownProperties);
}

/**
 * Evaluate a JavaScript expression.
 *
 * Executes arbitrary JavaScript code either globally or in the context of a specific
 * call frame. Can access variables in scope and produce side effects.
 *
 * @param expression - JavaScript expression to evaluate
 * @param callFrameId - Optional call frame ID for evaluation context (accesses local variables)
 * @returns Result as a remote object
 *
 * @throws Error [9602] if not connected to inspector
 * @throws Error [9606] if evaluation throws an exception
 * @throws Error [9604] if call frame ID is invalid
 *
 * @example
 * ```typescript
 * import { evaluate, getCallFrames } from "runtime:debugger";
 *
 * // Global evaluation
 * const result1 = await evaluate("1 + 2 + 3");
 * console.log(result1.value);  // 6
 *
 * // Evaluate in call frame context (when paused)
 * const frames = await getCallFrames();
 * const result2 = await evaluate("localVar * 2", frames[0].call_frame_id);
 * console.log(result2.value);  // Value of localVar * 2
 *
 * // Access and modify global state
 * const result3 = await evaluate("globalCounter++");
 * console.log(`Counter: ${result3.value}`);
 *
 * // Complex expressions
 * const result4 = await evaluate(`
 *   users.filter(u => u.role === 'admin')
 *        .map(u => u.name)
 *        .join(', ')
 * `);
 * console.log(`Admins: ${result4.value}`);
 * ```
 */
export async function evaluate(
  expression: string,
  callFrameId?: string
): Promise<RemoteObject> {
  return await ops.op_debugger_evaluate(expression, callFrameId);
}

/**
 * Set a variable value in a specific scope during debugging.
 *
 * Allows modifying local variables, closure variables, or global variables while
 * execution is paused. Useful for testing different code paths or correcting state
 * during debugging sessions.
 *
 * **Scope Numbers**: Index into the scope chain (0 = innermost local scope, higher = outer scopes)
 *
 * @param scopeNumber - Index in the scope chain (0 = local, 1+ = closure/global)
 * @param variableName - Name of the variable to modify
 * @param newValue - New value to assign (JavaScript primitive or object)
 * @param callFrameId - Call frame ID from pause event or getCallFrames()
 * @returns `true` if variable was successfully set
 *
 * @throws Error [9602] if not connected to inspector
 * @throws Error [9604] if call frame ID is invalid
 * @throws Error [9612] if variable name doesn't exist in scope
 *
 * @example
 * ```typescript
 * import { setVariableValue, onPaused, getCallFrames, getScopeChain } from "runtime:debugger";
 *
 * // When paused at breakpoint
 * const cleanup = onPaused(async (event) => {
 *   const frames = event.call_frames;
 *   const topFrame = frames[0];
 *
 *   // Get scope chain to find variable
 *   const scopes = topFrame.scope_chain;
 *   console.log("Scopes:", scopes.map((s, i) => `${i}: ${s.type}`));
 *
 *   // Modify local variable (scope 0)
 *   await setVariableValue(0, "counter", 100, topFrame.call_frame_id);
 *
 *   // Modify closure variable (scope 1)
 *   await setVariableValue(1, "config", { debug: true }, topFrame.call_frame_id);
 *
 *   // Modify global variable
 *   const globalScopeIndex = scopes.findIndex(s => s.type === "global");
 *   await setVariableValue(globalScopeIndex, "appState", "testing", topFrame.call_frame_id);
 * });
 * ```
 */
export async function setVariableValue(
  scopeNumber: number,
  variableName: string,
  newValue: any,
  callFrameId: string
): Promise<boolean> {
  return await ops.op_debugger_set_variable_value(
    scopeNumber,
    variableName,
    newValue,
    callFrameId
  );
}

/**
 * Retrieve the source code of a parsed script by its ID.
 *
 * Fetches the complete source text of a script that has been loaded into the runtime.
 * Useful for displaying source code in custom debugging UIs or analyzing dynamically
 * loaded scripts.
 *
 * **Script IDs** are obtained from:
 * - `ScriptInfo.script_id` from `listScripts()`
 * - `onScriptParsed()` event callback
 * - `CallFrame.location.script_id` from pause events
 *
 * @param scriptId - Unique script identifier from V8 inspector
 * @returns Complete source code of the script
 *
 * @throws Error [9602] if not connected to inspector
 * @throws Error [9613] if script ID is not found
 *
 * @example
 * ```typescript
 * import { getScriptSource, listScripts, onPaused } from "runtime:debugger";
 *
 * // Get source of all scripts
 * const scripts = await listScripts();
 * for (const script of scripts) {
 *   if (script.url.startsWith("file://")) {
 *     const source = await getScriptSource(script.script_id);
 *     console.log(`${script.url}:\n${source.slice(0, 200)}...`);
 *   }
 * }
 *
 * // Get source of currently executing script when paused
 * const cleanup = onPaused(async (event) => {
 *   const frame = event.call_frames[0];
 *   const scriptId = frame.location.script_id;
 *   const source = await getScriptSource(scriptId);
 *
 *   // Show context around current line
 *   const lines = source.split("\n");
 *   const currentLine = frame.location.line_number;
 *   const start = Math.max(0, currentLine - 3);
 *   const end = Math.min(lines.length, currentLine + 4);
 *
 *   console.log("Context:");
 *   for (let i = start; i < end; i++) {
 *     const marker = i === currentLine ? ">" : " ";
 *     console.log(`${marker} ${i + 1}: ${lines[i]}`);
 *   }
 * });
 * ```
 */
export async function getScriptSource(scriptId: string): Promise<string> {
  return await ops.op_debugger_get_script_source(scriptId);
}

/**
 * List all scripts that have been parsed and loaded into the runtime.
 *
 * Returns information about every script currently loaded in the JavaScript runtime,
 * including application code, internal modules, and dynamically evaluated code.
 *
 * **Returned Scripts Include:**
 * - Application entry point and imported modules
 * - Deno runtime internals (ext:* modules)
 * - Dynamically loaded code (import(), eval())
 * - Internal V8 scripts
 *
 * **Filtering Recommendations:**
 * - Filter by URL prefix to focus on app code: `script.url.startsWith("file://")`
 * - Exclude internal modules: `!script.url.startsWith("ext:")`
 *
 * @returns Array of all loaded scripts with metadata
 *
 * @throws Error [9602] if not connected to inspector
 *
 * @example
 * ```typescript
 * import { listScripts } from "runtime:debugger";
 *
 * // List all application scripts (exclude internals)
 * const scripts = await listScripts();
 * const appScripts = scripts.filter(s =>
 *   s.url.startsWith("file://") && !s.url.includes("node_modules")
 * );
 *
 * console.log("Application scripts:");
 * for (const script of appScripts) {
 *   console.log(`  ${script.url}`);
 *   console.log(`    ID: ${script.script_id}`);
 *   console.log(`    Lines: ${script.start_line}-${script.end_line}`);
 *   console.log(`    Hash: ${script.hash}`);
 * }
 * ```
 *
 * @example
 * ```typescript
 * // Find script by URL pattern
 * import { listScripts, setBreakpoint } from "runtime:debugger";
 *
 * const scripts = await listScripts();
 * const mainScript = scripts.find(s => s.url.endsWith("/main.ts"));
 *
 * if (mainScript) {
 *   // Set breakpoint in found script
 *   await setBreakpoint(mainScript.url, 10);
 *   console.log(`Breakpoint set in ${mainScript.url}`);
 * }
 * ```
 */
export async function listScripts(): Promise<ScriptInfo[]> {
  return await ops.op_debugger_list_scripts();
}

/**
 * Configure when the debugger should pause on exceptions.
 *
 * Controls whether execution pauses when exceptions are thrown. Useful for debugging
 * error handling code or finding the source of uncaught exceptions.
 *
 * **States:**
 * - `"none"`: Never pause on exceptions (default)
 * - `"uncaught"`: Pause only on exceptions that aren't caught by try/catch
 * - `"all"`: Pause on every exception, even if caught
 *
 * **Use Cases:**
 * - `"none"`: Normal execution, let error handlers work
 * - `"uncaught"`: Find exceptions that crash the app
 * - `"all"`: Debug exception handling logic, trace error propagation
 *
 * @param state - Exception pause mode: "none", "uncaught", or "all"
 * @returns `true` if configuration was successfully updated
 *
 * @throws Error [9602] if not connected to inspector
 *
 * @example
 * ```typescript
 * import { setPauseOnExceptions, connect, onPaused } from "runtime:debugger";
 *
 * await connect();
 *
 * // Debug all exceptions (including caught ones)
 * await setPauseOnExceptions("all");
 *
 * const cleanup = onPaused((event) => {
 *   if (event.reason === "exception" || event.reason === "promiseRejection") {
 *     console.log("Exception caught:", event.data);
 *     // Inspect exception object
 *     if (event.data?.exception?.description) {
 *       console.log("Error:", event.data.exception.description);
 *     }
 *   }
 * });
 * ```
 *
 * @example
 * ```typescript
 * // Only pause on uncaught exceptions
 * await setPauseOnExceptions("uncaught");
 *
 * try {
 *   throw new Error("This won't pause (caught)");
 * } catch (e) {
 *   console.log("Caught:", e);
 * }
 *
 * throw new Error("This WILL pause (uncaught)");
 * ```
 *
 * @example
 * ```typescript
 * // Disable exception pausing
 * await setPauseOnExceptions("none");
 *
 * // Now exceptions won't pause, even if uncaught
 * throw new Error("Won't pause debugger");
 * ```
 */
export async function setPauseOnExceptions(
  state: ExceptionPauseState
): Promise<boolean> {
  return await ops.op_debugger_set_pause_on_exceptions(state);
}

/**
 * Register a listener for debugger pause events.
 *
 * Invoked whenever execution pauses due to a breakpoint, step operation, manual pause,
 * exception, or other pause condition. The callback receives complete stack trace,
 * pause reason, and additional context data.
 *
 * **Event Sources:**
 * - Breakpoint hit (conditional or unconditional)
 * - Step operation completion (stepOver, stepInto, stepOut)
 * - Manual pause via `pause()`
 * - Exception thrown (if `setPauseOnExceptions()` enabled)
 * - Debugger statement execution
 * - Other V8 pause reasons
 *
 * **Callback Execution:**
 * - Runs asynchronously when pause occurs
 * - Receives `PausedEvent` with full context
 * - Can call debugger APIs (evaluate, getProperties, etc.)
 * - Multiple listeners are supported
 *
 * **Cleanup:**
 * - Call returned function to stop listening
 * - Cleanup is automatic if listener throws error
 * - Important to cleanup to avoid memory leaks
 *
 * @param callback - Function called when execution pauses (receives PausedEvent)
 * @returns Cleanup function to stop listening
 *
 * @example
 * ```typescript
 * import { onPaused, setBreakpoint, resume } from "runtime:debugger";
 *
 * // Basic pause handler with stack trace
 * const cleanup = onPaused((event) => {
 *   console.log(`Paused: ${event.reason}`);
 *
 *   // Print stack trace
 *   for (const frame of event.call_frames) {
 *     console.log(`  at ${frame.function_name} (${frame.url}:${frame.location.line_number})`);
 *   }
 * });
 *
 * // Set breakpoint and wait for pause
 * await setBreakpoint("file:///src/main.ts", 42);
 *
 * // Later: stop listening
 * cleanup();
 * ```
 *
 * @example
 * ```typescript
 * // Automatic variable inspection on pause
 * import { onPaused, getProperties, resume } from "runtime:debugger";
 *
 * const cleanup = onPaused(async (event) => {
 *   console.log(`Paused at ${event.call_frames[0].url}:${event.call_frames[0].location.line_number}`);
 *
 *   // Inspect local variables
 *   const topFrame = event.call_frames[0];
 *   const localScope = topFrame.scope_chain.find(s => s.type === "local");
 *
 *   if (localScope?.object.object_id) {
 *     const vars = await getProperties(localScope.object.object_id);
 *     console.log("Local variables:");
 *     for (const prop of vars) {
 *       if (prop.value) {
 *         console.log(`  ${prop.name} = ${prop.value.description}`);
 *       }
 *     }
 *   }
 *
 *   // Auto-resume after inspection
 *   await resume();
 * });
 * ```
 *
 * @example
 * ```typescript
 * // Handle different pause reasons
 * import { onPaused } from "runtime:debugger";
 *
 * const cleanup = onPaused((event) => {
 *   switch (event.reason) {
 *     case "breakpoint":
 *       console.log("Hit breakpoint");
 *       break;
 *     case "exception":
 *       console.log("Exception:", event.data?.exception?.description);
 *       break;
 *     case "debugCommand":
 *       console.log("Manual pause or step");
 *       break;
 *     case "promiseRejection":
 *       console.log("Unhandled promise rejection");
 *       break;
 *     default:
 *       console.log("Other pause:", event.reason);
 *   }
 * });
 * ```
 */
export function onPaused(callback: (event: PausedEvent) => void): () => void {
  let active = true;
  const rid = ops.op_debugger_create_pause_receiver();

  (async () => {
    while (active) {
      try {
        const event = await ops.op_debugger_receive_pause_event(rid);
        if (active) {
          callback(event);
        }
      } catch (err) {
        if (active) {
          console.error("Pause event listener error:", err);
        }
        break;
      }
    }
  })();

  return () => {
    active = false;
  };
}

/**
 * Register a listener for script parsed events.
 *
 * Invoked whenever a new script is loaded and parsed by the V8 runtime. This includes
 * the initial application load, dynamic imports, eval() calls, and module loading.
 * Useful for tracking code loading, setting breakpoints in dynamically loaded code,
 * or analyzing the module dependency graph.
 *
 * **Event Sources:**
 * - Application startup (main entry point)
 * - Dynamic imports (`import()`, `require()`)
 * - Eval execution (`eval()`, `new Function()`)
 * - Internal runtime modules (ext:* modules)
 * - Inline scripts from HTML
 *
 * **Use Cases:**
 * - Auto-set breakpoints in newly loaded scripts
 * - Track module loading order and dependencies
 * - Monitor dynamic code generation
 * - Build code coverage tracking
 * - Analyze bundle composition
 *
 * **Timing:**
 * - Event fires immediately after V8 parses the script
 * - Before the script executes its top-level code
 * - Safe to call `setBreakpoint()` with the script URL
 *
 * @param callback - Function called when a script is parsed (receives ScriptInfo)
 * @returns Cleanup function to stop listening
 *
 * @example
 * ```typescript
 * import { onScriptParsed, setBreakpoint } from "runtime:debugger";
 *
 * // Track all loaded scripts
 * const cleanup = onScriptParsed((script) => {
 *   console.log(`Script loaded: ${script.url}`);
 *   console.log(`  ID: ${script.script_id}`);
 *   console.log(`  Lines: ${script.start_line}-${script.end_line}`);
 * });
 *
 * // Later: stop listening
 * cleanup();
 * ```
 *
 * @example
 * ```typescript
 * // Auto-set breakpoints in application code
 * import { onScriptParsed, setBreakpoint } from "runtime:debugger";
 *
 * const cleanup = onScriptParsed(async (script) => {
 *   // Only app code (exclude internals)
 *   if (script.url.startsWith("file://") && !script.url.includes("node_modules")) {
 *     console.log(`App script loaded: ${script.url}`);
 *
 *     // Auto-breakpoint on first line of new modules
 *     if (script.url.endsWith(".ts") || script.url.endsWith(".js")) {
 *       await setBreakpoint(script.url, 0);
 *       console.log(`  => Breakpoint set at line 0`);
 *     }
 *   }
 * });
 * ```
 *
 * @example
 * ```typescript
 * // Build module dependency graph
 * import { onScriptParsed } from "runtime:debugger";
 *
 * const moduleGraph = new Map<string, ScriptInfo>();
 *
 * const cleanup = onScriptParsed((script) => {
 *   moduleGraph.set(script.url, script);
 *
 *   // Filter to app modules
 *   const appModules = Array.from(moduleGraph.values())
 *     .filter(s => s.url.startsWith("file://"));
 *
 *   console.log(`Loaded ${appModules.length} application modules`);
 * });
 * ```
 *
 * @example
 * ```typescript
 * // Monitor dynamic imports
 * import { onScriptParsed } from "runtime:debugger";
 *
 * let loadedCount = 0;
 *
 * const cleanup = onScriptParsed((script) => {
 *   loadedCount++;
 *   console.log(`[${loadedCount}] ${script.url}`);
 *
 *   // Detect eval/dynamic code
 *   if (!script.url || script.url === "") {
 *     console.warn("  WARNING: Dynamic/eval code detected");
 *   }
 * });
 * ```
 */
export function onScriptParsed(
  callback: (script: ScriptInfo) => void
): () => void {
  let active = true;
  const rid = ops.op_debugger_create_script_receiver();

  (async () => {
    while (active) {
      try {
        const script = await ops.op_debugger_receive_script_event(rid);
        if (active) {
          callback(script);
        }
      } catch (err) {
        if (active) {
          console.error("Script event listener error:", err);
        }
        break;
      }
    }
  })();

  return () => {
    active = false;
  };
}

// ============================================================================
// Convenience Exports
// ============================================================================

/**
 * Default export containing all debugger operations.
 *
 * Provides a convenient namespace for importing all debugger functionality at once.
 * Use named imports for better tree-shaking in production builds.
 *
 * **Included Operations:**
 * - **Connection**: `info`, `connect`, `disconnect`, `isConnected`
 * - **Breakpoints**: `setBreakpoint`, `removeBreakpoint`, `removeAllBreakpoints`, `listBreakpoints`, `enableBreakpoint`, `disableBreakpoint`
 * - **Execution**: `pause`, `resume`, `stepOver`, `stepInto`, `stepOut`, `continueToLocation`
 * - **Inspection**: `getCallFrames`, `getScopeChain`, `getProperties`, `evaluate`, `setVariableValue`
 * - **Scripts**: `getScriptSource`, `listScripts`
 * - **Configuration**: `setPauseOnExceptions`
 * - **Events**: `onPaused`, `onScriptParsed`
 *
 * @example
 * ```typescript
 * // Default import (namespace style)
 * import debugger from "runtime:debugger";
 *
 * await debugger.connect();
 * await debugger.setBreakpoint("file:///src/main.ts", 42);
 * const frames = await debugger.getCallFrames();
 * ```
 *
 * @example
 * ```typescript
 * // Recommended: Named imports (better tree-shaking)
 * import { connect, setBreakpoint, onPaused } from "runtime:debugger";
 *
 * await connect();
 * await setBreakpoint("file:///src/main.ts", 42);
 * const cleanup = onPaused((event) => {
 *   console.log("Paused:", event.reason);
 * });
 * ```
 */
export default {
  info,
  connect,
  disconnect,
  isConnected,
  setBreakpoint,
  removeBreakpoint,
  removeAllBreakpoints,
  listBreakpoints,
  enableBreakpoint,
  disableBreakpoint,
  pause,
  resume,
  stepOver,
  stepInto,
  stepOut,
  continueToLocation,
  getCallFrames,
  getScopeChain,
  getProperties,
  evaluate,
  setVariableValue,
  getScriptSource,
  listScripts,
  setPauseOnExceptions,
  onPaused,
  onScriptParsed,
};
