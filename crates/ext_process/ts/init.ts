// host:process module - TypeScript wrapper for Deno core ops

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

interface SpawnOptions {
  args?: string[];
  cwd?: string;
  env?: Record<string, string>;
  stdin?: "piped" | "inherit" | "null";
  stdout?: "piped" | "inherit" | "null";
  stderr?: "piped" | "inherit" | "null";
}

interface SpawnResult {
  id: string;
  pid: number;
}

interface WaitResult {
  success: boolean;
  code: number | null;
  signal: string | null;
}

interface RawProcessStatus {
  running: boolean;
  exit_code?: number | null;
  signal?: string | null;
}

interface ProcessStatus {
  running: boolean;
  exitCode?: number;
  signal?: string;
}

interface ReadOutput {
  data: string | null;
  eof: boolean;
}

interface StdioIterator {
  [Symbol.asyncIterator](): AsyncGenerator<string, void, unknown>;
}

interface ProcessHandle {
  readonly id: string;
  readonly pid: number;
  kill(signal?: string): Promise<boolean>;
  wait(): Promise<WaitResult>;
  status(): Promise<ProcessStatus>;
  writeStdin(data: string): Promise<void>;
  readStdout(): Promise<ReadOutput>;
  readStderr(): Promise<ReadOutput>;
  stdout: StdioIterator;
  stderr: StdioIterator;
}

const core = Deno.core;

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

export async function kill(handle: string, signal?: string): Promise<boolean> {
  return await core.ops.op_process_kill(handle, signal);
}

export async function wait(handle: string): Promise<WaitResult> {
  return await core.ops.op_process_wait(handle);
}

export async function status(handle: string): Promise<ProcessStatus> {
  const raw = await core.ops.op_process_status(handle);
  return {
    running: raw.running,
    exitCode: raw.exit_code ?? undefined,
    signal: raw.signal ?? undefined,
  };
}

export async function writeStdin(handle: string, data: string): Promise<void> {
  return await core.ops.op_process_write_stdin(handle, data);
}

export async function readStdout(handle: string): Promise<ReadOutput> {
  return await core.ops.op_process_read_stdout(handle);
}

export async function readStderr(handle: string): Promise<ReadOutput> {
  return await core.ops.op_process_read_stderr(handle);
}
