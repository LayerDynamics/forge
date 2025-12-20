/**
 * @module runtime:process
 *
 * Process spawning and management for Forge applications.
 *
 * This module provides APIs for spawning child processes, managing their lifecycle,
 * and communicating via standard I/O streams. It supports both one-shot commands
 * and long-running processes with bidirectional communication.
 *
 * ## Features
 * - Spawn child processes with configurable stdio
 * - Bidirectional communication via stdin/stdout/stderr
 * - Process lifecycle management (kill, wait, status)
 * - Async iteration over stdout/stderr streams
 * - Cross-platform signal handling
 *
 * ## Permissions
 * Requires `process.spawn` permission in manifest.app.toml:
 * ```toml
 * [permissions.process]
 * spawn = ["allowed_binary", "/path/to/script"]
 * ```
 *
 * ## Error Codes
 * - 4000: I/O error during process operations
 * - 4001: Permission denied to spawn process
 * - 4002: Process binary not found
 * - 4003: Failed to spawn process
 * - 4004: Process already exited
 * - 4005: Operation timeout
 * - 4006: Invalid process handle
 * - 4007: Process stdin is closed
 * - 4008: Process output not captured (stdout/stderr not piped)
 * - 4009: Too many processes spawned
 *
 * @example
 * ```typescript
 * import { spawn } from "runtime:process";
 *
 * // Spawn a simple command
 * const proc = await spawn("echo", { args: ["Hello, World!"] });
 *
 * // Read output
 * for await (const line of proc.stdout) {
 *   console.log(line);
 * }
 *
 * // Wait for completion
 * const result = await proc.wait();
 * console.log(`Exit code: ${result.code}`);
 * ```
 */

declare const Deno: {
  core: {
    ops: {
      op_process_spawn(binary: string, opts: SpawnOptions | undefined): Promise<SpawnResult>;
      op_process_kill(handle: string, signal: string | undefined): Promise<boolean>;
      op_process_wait(handle: string): Promise<WaitResult>;
      op_process_status(handle: string): Promise<RawProcessStatus>;
      op_process_write_stdin(handle: string, data: string): Promise<void>;
      op_process_read_stdout(handle: string): Promise<ReadOutput>;
      op_process_read_stderr(handle: string): Promise<ReadOutput>;
    };
  };
};

/**
 * Configuration options for spawning a process.
 */
export interface SpawnOptions {
  /**
   * Command-line arguments to pass to the process.
   *
   * @example
   * ```typescript
   * { args: ["--verbose", "--output=file.txt"] }
   * ```
   */
  args?: string[];

  /**
   * Working directory for the spawned process.
   * Defaults to the parent process's working directory.
   *
   * @example
   * ```typescript
   * { cwd: "/path/to/project" }
   * ```
   */
  cwd?: string;

  /**
   * Environment variables for the spawned process.
   * If not specified, inherits the parent process's environment.
   *
   * @example
   * ```typescript
   * { env: { "NODE_ENV": "production", "API_KEY": "secret" } }
   * ```
   */
  env?: Record<string, string>;

  /**
   * How to handle standard input.
   * - "piped": Create a pipe for programmatic writing
   * - "inherit": Inherit from parent process
   * - "null": Discard input (no stdin available)
   *
   * @default "null"
   */
  stdin?: "piped" | "inherit" | "null";

  /**
   * How to handle standard output.
   * - "piped": Capture output for programmatic reading
   * - "inherit": Output to parent's stdout
   * - "null": Discard output
   *
   * @default "piped"
   */
  stdout?: "piped" | "inherit" | "null";

  /**
   * How to handle standard error.
   * - "piped": Capture errors for programmatic reading
   * - "inherit": Output to parent's stderr
   * - "null": Discard errors
   *
   * @default "piped"
   */
  stderr?: "piped" | "inherit" | "null";
}

/**
 * Result of spawning a process (internal).
 * @internal
 */
export interface SpawnResult {
  /** Internal process handle identifier */
  id: string;
  /** Operating system process ID */
  pid: number;
}

/**
 * Result of waiting for a process to complete.
 */
export interface WaitResult {
  /**
   * Whether the process exited successfully (exit code 0).
   */
  success: boolean;

  /**
   * Exit code of the process, or null if terminated by signal.
   */
  code: number | null;

  /**
   * Signal that terminated the process (e.g., "SIGTERM", "SIGKILL"),
   * or null if exited normally.
   *
   * @platform Platform-specific signal names may vary
   */
  signal: string | null;
}

/**
 * Raw process status from Rust (internal).
 * @internal
 */
export interface RawProcessStatus {
  running: boolean;
  exit_code?: number | null;
  signal?: string | null;
}

/**
 * Current status of a running or completed process.
 */
export interface ProcessStatus {
  /**
   * Whether the process is still running.
   */
  running: boolean;

  /**
   * Exit code of the process if it has exited, otherwise undefined.
   */
  exitCode?: number;

  /**
   * Signal that terminated the process (e.g., "SIGTERM"),
   * or undefined if still running or exited normally.
   */
  signal?: string;
}

/**
 * Output read from a process stream (stdout or stderr).
 */
export interface ReadOutput {
  /**
   * Data read from the stream, or null if no data available.
   * Returns chunks of output as they become available.
   */
  data: string | null;

  /**
   * Whether the end of the stream has been reached.
   * When true, no more data will be available.
   */
  eof: boolean;
}

/**
 * Async iterator for process output streams.
 * Allows using `for await...of` to read output line by line.
 *
 * @example
 * ```typescript
 * for await (const line of proc.stdout) {
 *   console.log("Output:", line);
 * }
 * ```
 */
export interface StdioIterator {
  [Symbol.asyncIterator](): AsyncGenerator<string, void, unknown>;
}

/**
 * Handle to a spawned child process.
 *
 * Provides methods for interacting with the process, reading output,
 * writing input, and managing its lifecycle.
 *
 * @example
 * ```typescript
 * const proc = await spawn("node", {
 *   args: ["script.js"],
 *   stdin: "piped",
 *   stdout: "piped"
 * });
 *
 * // Write to stdin
 * await proc.writeStdin("input data\n");
 *
 * // Read from stdout
 * for await (const line of proc.stdout) {
 *   console.log(line);
 * }
 *
 * // Wait for completion
 * const result = await proc.wait();
 * ```
 */
export interface ProcessHandle {
  /**
   * Internal process handle identifier.
   * Used for low-level operations.
   */
  readonly id: string;

  /**
   * Operating system process ID (PID).
   * Can be used with external tools to monitor or manage the process.
   */
  readonly pid: number;

  /**
   * Terminates the process with an optional signal.
   *
   * @param signal - Signal to send (e.g., "SIGTERM", "SIGKILL").
   *                 Default behavior is platform-specific.
   * @returns True if the signal was sent successfully
   * @throws Error (4006) if process handle is invalid
   *
   * @example
   * ```typescript
   * // Graceful termination
   * await proc.kill("SIGTERM");
   *
   * // Force kill
   * await proc.kill("SIGKILL");
   * ```
   */
  kill(signal?: string): Promise<boolean>;

  /**
   * Waits for the process to complete and returns its exit status.
   * This is a blocking operation that resolves when the process exits.
   *
   * @returns Exit information including code and signal
   * @throws Error (4006) if process handle is invalid
   *
   * @example
   * ```typescript
   * const result = await proc.wait();
   * if (result.success) {
   *   console.log("Process succeeded");
   * } else {
   *   console.error("Process failed with code:", result.code);
   * }
   * ```
   */
  wait(): Promise<WaitResult>;

  /**
   * Checks the current status of the process without blocking.
   *
   * @returns Current process status
   * @throws Error (4006) if process handle is invalid
   *
   * @example
   * ```typescript
   * const status = await proc.status();
   * if (status.running) {
   *   console.log("Process still running");
   * } else {
   *   console.log("Process exited with code:", status.exitCode);
   * }
   * ```
   */
  status(): Promise<ProcessStatus>;

  /**
   * Writes data to the process's standard input.
   *
   * @param data - Data to write to stdin
   * @throws Error (4007) if stdin is closed
   * @throws Error (4008) if stdin was not configured as "piped"
   * @throws Error (4006) if process handle is invalid
   *
   * @example
   * ```typescript
   * const proc = await spawn("cat", { stdin: "piped" });
   * await proc.writeStdin("Hello, process!\n");
   * ```
   */
  writeStdin(data: string): Promise<void>;

  /**
   * Reads available data from the process's standard output.
   * Returns immediately with available data or indicates EOF.
   *
   * @returns Output data and EOF status
   * @throws Error (4008) if stdout was not configured as "piped"
   * @throws Error (4006) if process handle is invalid
   *
   * @example
   * ```typescript
   * const output = await proc.readStdout();
   * if (output.data) {
   *   console.log("Received:", output.data);
   * }
   * if (output.eof) {
   *   console.log("No more output");
   * }
   * ```
   */
  readStdout(): Promise<ReadOutput>;

  /**
   * Reads available data from the process's standard error.
   * Returns immediately with available data or indicates EOF.
   *
   * @returns Error output data and EOF status
   * @throws Error (4008) if stderr was not configured as "piped"
   * @throws Error (4006) if process handle is invalid
   *
   * @example
   * ```typescript
   * const errors = await proc.readStderr();
   * if (errors.data) {
   *   console.error("Process error:", errors.data);
   * }
   * ```
   */
  readStderr(): Promise<ReadOutput>;

  /**
   * Async iterator for reading standard output line by line.
   * Automatically handles EOF and completes when the stream closes.
   *
   * @throws Error (4008) if stdout was not configured as "piped"
   *
   * @example
   * ```typescript
   * for await (const line of proc.stdout) {
   *   console.log("Output:", line);
   * }
   * // Loop completes when process closes stdout
   * ```
   */
  stdout: StdioIterator;

  /**
   * Async iterator for reading standard error line by line.
   * Automatically handles EOF and completes when the stream closes.
   *
   * @throws Error (4008) if stderr was not configured as "piped"
   *
   * @example
   * ```typescript
   * for await (const line of proc.stderr) {
   *   console.error("Error:", line);
   * }
   * ```
   */
  stderr: StdioIterator;
}

const core = Deno.core;

/**
 * Spawns a new child process and returns a handle for interacting with it.
 *
 * The process is spawned asynchronously. Use the returned handle to:
 * - Read output from stdout/stderr
 * - Write input to stdin
 * - Wait for completion or check status
 * - Terminate the process
 *
 * @param binary - Path to the executable or command name (must be in PATH)
 * @param opts - Optional configuration for stdio, environment, working directory
 * @returns Handle to the spawned process with lifecycle management methods
 *
 * @throws Error (4001) if permission denied to spawn the binary
 * @throws Error (4002) if binary not found
 * @throws Error (4003) if failed to spawn process
 * @throws Error (4009) if too many processes already spawned
 *
 * @example
 * ```typescript
 * // Simple command execution
 * const proc = await spawn("echo", { args: ["Hello, World!"] });
 * for await (const line of proc.stdout) {
 *   console.log(line); // "Hello, World!"
 * }
 * await proc.wait();
 * ```
 *
 * @example
 * ```typescript
 * // Interactive process with stdin/stdout
 * const proc = await spawn("python3", {
 *   args: ["-i"],
 *   stdin: "piped",
 *   stdout: "piped",
 *   cwd: "/path/to/project"
 * });
 *
 * await proc.writeStdin("print('Hello from Python')\n");
 * const output = await proc.readStdout();
 * console.log(output.data);
 * ```
 *
 * @example
 * ```typescript
 * // Custom environment variables
 * const proc = await spawn("node", {
 *   args: ["script.js"],
 *   env: {
 *     "NODE_ENV": "production",
 *     "API_KEY": "secret123"
 *   }
 * });
 * ```
 *
 * @see {@link ProcessHandle} for available operations on spawned processes
 */
export async function spawn(binary: string, opts?: SpawnOptions): Promise<ProcessHandle> {
  const result = await core.ops.op_process_spawn(binary, opts);
  const handle = result.id;

  return {
    id: handle,
    pid: result.pid,

    async kill(signal?: string): Promise<boolean> {
      return await core.ops.op_process_kill(handle, signal);
    },

    async wait(): Promise<WaitResult> {
      return await core.ops.op_process_wait(handle);
    },

    async status(): Promise<ProcessStatus> {
      const raw = await core.ops.op_process_status(handle);
      return {
        running: raw.running,
        exitCode: raw.exit_code ?? undefined,
        signal: raw.signal ?? undefined,
      };
    },

    async writeStdin(data: string): Promise<void> {
      return await core.ops.op_process_write_stdin(handle, data);
    },

    async readStdout(): Promise<ReadOutput> {
      return await core.ops.op_process_read_stdout(handle);
    },

    async readStderr(): Promise<ReadOutput> {
      return await core.ops.op_process_read_stderr(handle);
    },

    stdout: {
      async *[Symbol.asyncIterator](): AsyncGenerator<string, void, unknown> {
        while (true) {
          const output = await core.ops.op_process_read_stdout(handle);
          if (output.eof) break;
          if (output.data !== null) yield output.data;
        }
      },
    },

    stderr: {
      async *[Symbol.asyncIterator](): AsyncGenerator<string, void, unknown> {
        while (true) {
          const output = await core.ops.op_process_read_stderr(handle);
          if (output.eof) break;
          if (output.data !== null) yield output.data;
        }
      },
    },
  };
}

/**
 * Terminates a process by its handle ID.
 *
 * This is a low-level function. Prefer using `ProcessHandle.kill()` from
 * the handle returned by `spawn()`.
 *
 * @param handle - Internal process handle identifier
 * @param signal - Optional signal name (e.g., "SIGTERM", "SIGKILL")
 * @returns True if the signal was sent successfully
 *
 * @throws Error (4006) if process handle is invalid
 *
 * @example
 * ```typescript
 * const proc = await spawn("long-running-task");
 * // Later, kill by handle ID
 * await kill(proc.id, "SIGTERM");
 * ```
 */
export async function kill(handle: string, signal?: string): Promise<boolean> {
  return await core.ops.op_process_kill(handle, signal);
}

/**
 * Waits for a process to complete by its handle ID.
 *
 * This is a low-level function. Prefer using `ProcessHandle.wait()` from
 * the handle returned by `spawn()`.
 *
 * @param handle - Internal process handle identifier
 * @returns Exit information including code and signal
 *
 * @throws Error (4006) if process handle is invalid
 *
 * @example
 * ```typescript
 * const proc = await spawn("build-script");
 * const result = await wait(proc.id);
 * console.log("Build finished with code:", result.code);
 * ```
 */
export async function wait(handle: string): Promise<WaitResult> {
  return await core.ops.op_process_wait(handle);
}

/**
 * Checks the status of a process by its handle ID.
 *
 * This is a low-level function. Prefer using `ProcessHandle.status()` from
 * the handle returned by `spawn()`.
 *
 * @param handle - Internal process handle identifier
 * @returns Current process status
 *
 * @throws Error (4006) if process handle is invalid
 *
 * @example
 * ```typescript
 * const proc = await spawn("server");
 * const status = await status(proc.id);
 * if (!status.running) {
 *   console.error("Server crashed with code:", status.exitCode);
 * }
 * ```
 */
export async function status(handle: string): Promise<ProcessStatus> {
  const raw = await core.ops.op_process_status(handle);
  return {
    running: raw.running,
    exitCode: raw.exit_code ?? undefined,
    signal: raw.signal ?? undefined,
  };
}

/**
 * Writes data to a process's standard input by handle ID.
 *
 * This is a low-level function. Prefer using `ProcessHandle.writeStdin()` from
 * the handle returned by `spawn()`.
 *
 * @param handle - Internal process handle identifier
 * @param data - Data to write to stdin
 *
 * @throws Error (4007) if stdin is closed
 * @throws Error (4008) if stdin was not configured as "piped"
 * @throws Error (4006) if process handle is invalid
 *
 * @example
 * ```typescript
 * const proc = await spawn("grep", { args: ["pattern"], stdin: "piped" });
 * await writeStdin(proc.id, "line1\nline2\npattern line\n");
 * ```
 */
export async function writeStdin(handle: string, data: string): Promise<void> {
  return await core.ops.op_process_write_stdin(handle, data);
}

/**
 * Reads data from a process's standard output by handle ID.
 *
 * This is a low-level function. Prefer using `ProcessHandle.readStdout()` or
 * iterating over `ProcessHandle.stdout` from the handle returned by `spawn()`.
 *
 * @param handle - Internal process handle identifier
 * @returns Output data and EOF status
 *
 * @throws Error (4008) if stdout was not configured as "piped"
 * @throws Error (4006) if process handle is invalid
 *
 * @example
 * ```typescript
 * const proc = await spawn("ls", { stdout: "piped" });
 * const output = await readStdout(proc.id);
 * console.log(output.data);
 * ```
 */
export async function readStdout(handle: string): Promise<ReadOutput> {
  return await core.ops.op_process_read_stdout(handle);
}

/**
 * Reads data from a process's standard error by handle ID.
 *
 * This is a low-level function. Prefer using `ProcessHandle.readStderr()` or
 * iterating over `ProcessHandle.stderr` from the handle returned by `spawn()`.
 *
 * @param handle - Internal process handle identifier
 * @returns Error output data and EOF status
 *
 * @throws Error (4008) if stderr was not configured as "piped"
 * @throws Error (4006) if process handle is invalid
 *
 * @example
 * ```typescript
 * const proc = await spawn("command", { stderr: "piped" });
 * const errors = await readStderr(proc.id);
 * if (errors.data) {
 *   console.error("Command error:", errors.data);
 * }
 * ```
 */
export async function readStderr(handle: string): Promise<ReadOutput> {
  return await core.ops.op_process_read_stderr(handle);
}


// ============================================================================
// Extensibility API (auto-generated)
// ============================================================================

/** Registry of operations with their argument and result types */
interface OpRegistry {
  spawn: { args: []; result: void };
  kill: { args: []; result: void };
  wait: { args: []; result: void };
  status: { args: []; result: void };
  writeStdin: { args: []; result: void };
  readStdout: { args: []; result: void };
  readStderr: { args: []; result: void };
}

/** Extract argument types for an operation */
type OpArgs<T extends keyof OpRegistry> = OpRegistry[T]['args'];

/** Extract result type for an operation */
type OpResult<T extends keyof OpRegistry> = OpRegistry[T]['result'];

/** Valid operation names for this extension */
type OpName = "spawn" | "kill" | "wait" | "status" | "writeStdin" | "readStdout" | "readStderr";

/** Hook callback types */
type BeforeHookCallback<T extends OpName> = (args: OpArgs<T>) => void | Promise<void>;
type AfterHookCallback<T extends OpName> = (result: OpResult<T>, args: OpArgs<T>) => void | Promise<void>;
type ErrorHookCallback<T extends OpName> = (error: Error, args: OpArgs<T>) => void | Promise<void>;

/** Internal hook storage */
const _hooks = {
  before: new Map<OpName, Set<BeforeHookCallback<OpName>>>(),
  after: new Map<OpName, Set<AfterHookCallback<OpName>>>(),
  error: new Map<OpName, Set<ErrorHookCallback<OpName>>>(),
};

/**
 * Register a callback to be called before an operation executes.
 * @param opName - The name of the operation to hook
 * @param callback - Function called with the operation arguments
 * @returns Unsubscribe function to remove the hook
 */
export function onBefore<T extends OpName>(
  opName: T,
  callback: BeforeHookCallback<T>
): () => void {
  if (!_hooks.before.has(opName)) {
    _hooks.before.set(opName, new Set());
  }
  _hooks.before.get(opName)!.add(callback as BeforeHookCallback<OpName>);
  return () => _hooks.before.get(opName)?.delete(callback as BeforeHookCallback<OpName>);
}

/**
 * Register a callback to be called after an operation completes successfully.
 * @param opName - The name of the operation to hook
 * @param callback - Function called with the result and original arguments
 * @returns Unsubscribe function to remove the hook
 */
export function onAfter<T extends OpName>(
  opName: T,
  callback: AfterHookCallback<T>
): () => void {
  if (!_hooks.after.has(opName)) {
    _hooks.after.set(opName, new Set());
  }
  _hooks.after.get(opName)!.add(callback as AfterHookCallback<OpName>);
  return () => _hooks.after.get(opName)?.delete(callback as AfterHookCallback<OpName>);
}

/**
 * Register a callback to be called when an operation throws an error.
 * @param opName - The name of the operation to hook
 * @param callback - Function called with the error and original arguments
 * @returns Unsubscribe function to remove the hook
 */
export function onError<T extends OpName>(
  opName: T,
  callback: ErrorHookCallback<T>
): () => void {
  if (!_hooks.error.has(opName)) {
    _hooks.error.set(opName, new Set());
  }
  _hooks.error.get(opName)!.add(callback as ErrorHookCallback<OpName>);
  return () => _hooks.error.get(opName)?.delete(callback as ErrorHookCallback<OpName>);
}

/** Internal: Invoke before hooks for an operation */
async function _invokeBeforeHooks<T extends OpName>(opName: T, args: OpArgs<T>): Promise<void> {
  const hooks = _hooks.before.get(opName);
  if (hooks) {
    for (const hook of hooks) {
      await hook(args);
    }
  }
}

/** Internal: Invoke after hooks for an operation */
async function _invokeAfterHooks<T extends OpName>(opName: T, result: OpResult<T>, args: OpArgs<T>): Promise<void> {
  const hooks = _hooks.after.get(opName);
  if (hooks) {
    for (const hook of hooks) {
      await hook(result, args);
    }
  }
}

/** Internal: Invoke error hooks for an operation */
async function _invokeErrorHooks<T extends OpName>(opName: T, error: Error, args: OpArgs<T>): Promise<void> {
  const hooks = _hooks.error.get(opName);
  if (hooks) {
    for (const hook of hooks) {
      await hook(error, args);
    }
  }
}

/**
 * Remove all hooks for a specific operation or all operations.
 * @param opName - Optional: specific operation to clear hooks for
 */
export function removeAllHooks(opName?: OpName): void {
  if (opName) {
    _hooks.before.delete(opName);
    _hooks.after.delete(opName);
    _hooks.error.delete(opName);
  } else {
    _hooks.before.clear();
    _hooks.after.clear();
    _hooks.error.clear();
  }
}

/** Handler function type */
type HandlerFn = (...args: unknown[]) => unknown | Promise<unknown>;

/** Internal handler storage */
const _handlers = new Map<string, HandlerFn>();

/**
 * Register a custom handler that can be invoked by name.
 * @param name - Unique name for the handler
 * @param handler - Handler function to register
 * @throws Error if a handler with the same name already exists
 */
export function registerHandler(name: string, handler: HandlerFn): void {
  if (_handlers.has(name)) {
    throw new Error(`Handler '${name}' already registered`);
  }
  _handlers.set(name, handler);
}

/**
 * Invoke a registered handler by name.
 * @param name - Name of the handler to invoke
 * @param args - Arguments to pass to the handler
 * @returns The handler's return value
 * @throws Error if no handler with the given name exists
 */
export async function invokeHandler(name: string, ...args: unknown[]): Promise<unknown> {
  const handler = _handlers.get(name);
  if (!handler) {
    throw new Error(`Handler '${name}' not found`);
  }
  return await handler(...args);
}

/**
 * List all registered handler names.
 * @returns Array of handler names
 */
export function listHandlers(): string[] {
  return Array.from(_handlers.keys());
}

/**
 * Remove a registered handler.
 * @param name - Name of the handler to remove
 * @returns true if the handler was removed, false if it didn't exist
 */
export function removeHandler(name: string): boolean {
  return _handlers.delete(name);
}

/**
 * Check if a handler is registered.
 * @param name - Name of the handler to check
 * @returns true if the handler exists
 */
export function hasHandler(name: string): boolean {
  return _handlers.has(name);
}

