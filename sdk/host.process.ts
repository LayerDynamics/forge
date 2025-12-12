// host:process module - Deno API for child process management
// This is the single source of truth for the host:process SDK

// Type definitions
export interface SpawnOptions {
  /** Arguments to pass to the command */
  args?: string[];
  /** Environment variables to set */
  env?: Record<string, string>;
  /** Working directory for the process */
  cwd?: string;
  /** How to handle stdout: "piped", "inherit", or "null" */
  stdout?: "piped" | "inherit" | "null";
  /** How to handle stderr: "piped", "inherit", or "null" */
  stderr?: "piped" | "inherit" | "null";
  /** How to handle stdin: "piped", "inherit", or "null" */
  stdin?: "piped" | "inherit" | "null";
}

export interface ProcessHandle {
  /** Internal handle ID */
  id: string;
  /** Operating system process ID */
  pid: number;
}

export interface ProcessStatus {
  /** Whether the process is still running */
  running: boolean;
  /** Exit code if process has exited */
  exitCode?: number;
  /** Signal that killed the process (Unix only) */
  signal?: string;
}

export interface ProcessOutput {
  /** Line of output data, or null if EOF */
  data: string | null;
  /** Whether end of stream has been reached */
  eof: boolean;
}

export interface ChildProcess extends ProcessHandle {
  /** Kill the process with an optional signal (default: SIGTERM) */
  kill(signal?: string): Promise<void>;
  /** Wait for the process to exit and return the exit code */
  wait(): Promise<number>;
  /** Get the current process status */
  status(): Promise<ProcessStatus>;
  /** Write to stdin (requires stdin: "piped") */
  writeStdin(data: string): Promise<void>;
  /** Read a line from stdout (requires stdout: "piped") */
  readStdout(): Promise<ProcessOutput>;
  /** Read a line from stderr (requires stderr: "piped") */
  readStderr(): Promise<ProcessOutput>;
  /** Async iterator for stdout lines */
  stdout: AsyncIterable<string>;
  /** Async iterator for stderr lines */
  stderr: AsyncIterable<string>;
}

// Internal type for spawn result from Rust
interface SpawnResult {
  id: string;
  pid: number;
}

// Internal type for process output from Rust
interface RawProcessOutput {
  data: string | null;
  eof: boolean;
}

// Internal type for process status from Rust
interface RawProcessStatus {
  running: boolean;
  exit_code: number | null;
  signal: string | null;
}

// Deno.core.ops type declaration
declare const Deno: {
  core: {
    ops: {
      op_process_spawn(binary: string, opts?: SpawnOptions): Promise<SpawnResult>;
      op_process_kill(handle: string, signal?: string): Promise<void>;
      op_process_wait(handle: string): Promise<number>;
      op_process_status(handle: string): Promise<RawProcessStatus>;
      op_process_write_stdin(handle: string, data: string): Promise<void>;
      op_process_read_stdout(handle: string): Promise<RawProcessOutput>;
      op_process_read_stderr(handle: string): Promise<RawProcessOutput>;
    };
  };
};

/**
 * Spawn a child process.
 * Requires process permissions in the manifest.
 *
 * @param binary - The command/binary to execute
 * @param opts - Spawn options (args, env, cwd, stdio config)
 * @returns A ChildProcess handle for managing the process
 *
 * @example
 * ```ts
 * import { spawn } from "host:process";
 *
 * // Simple spawn and wait
 * const proc = await spawn("ls", { args: ["-la"] });
 * const exitCode = await proc.wait();
 *
 * // Spawn with piped output
 * const grep = await spawn("grep", {
 *   args: ["pattern"],
 *   stdin: "piped",
 *   stdout: "piped",
 * });
 * await grep.writeStdin("line1\npattern here\nline3\n");
 * for await (const line of grep.stdout) {
 *   console.log("Match:", line);
 * }
 * ```
 */
export async function spawn(binary: string, opts?: SpawnOptions): Promise<ChildProcess> {
  const result = await Deno.core.ops.op_process_spawn(binary, opts);
  const handle = result.id;

  return {
    id: handle,
    pid: result.pid,

    async kill(signal?: string): Promise<void> {
      return await Deno.core.ops.op_process_kill(handle, signal);
    },

    async wait(): Promise<number> {
      return await Deno.core.ops.op_process_wait(handle);
    },

    async status(): Promise<ProcessStatus> {
      const raw = await Deno.core.ops.op_process_status(handle);
      return {
        running: raw.running,
        exitCode: raw.exit_code ?? undefined,
        signal: raw.signal ?? undefined,
      };
    },

    async writeStdin(data: string): Promise<void> {
      return await Deno.core.ops.op_process_write_stdin(handle, data);
    },

    async readStdout(): Promise<ProcessOutput> {
      return await Deno.core.ops.op_process_read_stdout(handle);
    },

    async readStderr(): Promise<ProcessOutput> {
      return await Deno.core.ops.op_process_read_stderr(handle);
    },

    stdout: {
      async *[Symbol.asyncIterator](): AsyncIterator<string> {
        while (true) {
          const output = await Deno.core.ops.op_process_read_stdout(handle);
          if (output.eof) break;
          if (output.data !== null) yield output.data;
        }
      },
    },

    stderr: {
      async *[Symbol.asyncIterator](): AsyncIterator<string> {
        while (true) {
          const output = await Deno.core.ops.op_process_read_stderr(handle);
          if (output.eof) break;
          if (output.data !== null) yield output.data;
        }
      },
    },
  };
}

/**
 * Kill a process by handle ID.
 *
 * @param handle - The process handle ID
 * @param signal - Signal to send (Unix: "SIGTERM", "SIGKILL", "SIGINT", etc.)
 *
 * @example
 * ```ts
 * import { spawn, kill } from "host:process";
 *
 * const proc = await spawn("sleep", { args: ["100"] });
 * await kill(proc.id, "SIGTERM");
 * ```
 */
export async function kill(handle: string, signal?: string): Promise<void> {
  return await Deno.core.ops.op_process_kill(handle, signal);
}

/**
 * Wait for a process to exit.
 *
 * @param handle - The process handle ID
 * @returns The exit code
 */
export async function wait(handle: string): Promise<number> {
  return await Deno.core.ops.op_process_wait(handle);
}

/**
 * Get the status of a process.
 *
 * @param handle - The process handle ID
 * @returns Process status information
 */
export async function status(handle: string): Promise<ProcessStatus> {
  const raw = await Deno.core.ops.op_process_status(handle);
  return {
    running: raw.running,
    exitCode: raw.exit_code ?? undefined,
    signal: raw.signal ?? undefined,
  };
}

/**
 * Write data to a process's stdin.
 *
 * @param handle - The process handle ID
 * @param data - The string data to write
 */
export async function writeStdin(handle: string, data: string): Promise<void> {
  return await Deno.core.ops.op_process_write_stdin(handle, data);
}

/**
 * Read a line from a process's stdout.
 *
 * @param handle - The process handle ID
 * @returns Output containing the line data and EOF status
 */
export async function readStdout(handle: string): Promise<ProcessOutput> {
  return await Deno.core.ops.op_process_read_stdout(handle);
}

/**
 * Read a line from a process's stderr.
 *
 * @param handle - The process handle ID
 * @returns Output containing the line data and EOF status
 */
export async function readStderr(handle: string): Promise<ProcessOutput> {
  return await Deno.core.ops.op_process_read_stderr(handle);
}

// Re-export types for convenience
export type { SpawnOptions, ProcessHandle, ProcessStatus, ProcessOutput, ChildProcess };
