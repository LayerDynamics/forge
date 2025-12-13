// host:process module - JavaScript wrapper for Deno core ops
const core = Deno.core;

export async function spawn(binary, opts) {
  const result = await core.ops.op_process_spawn(binary, opts);
  const handle = result.id;

  return {
    id: handle,
    pid: result.pid,

    async kill(signal) {
      return await core.ops.op_process_kill(handle, signal);
    },

    async wait() {
      return await core.ops.op_process_wait(handle);
    },

    async status() {
      const raw = await core.ops.op_process_status(handle);
      return {
        running: raw.running,
        exitCode: raw.exit_code ?? undefined,
        signal: raw.signal ?? undefined,
      };
    },

    async writeStdin(data) {
      return await core.ops.op_process_write_stdin(handle, data);
    },

    async readStdout() {
      return await core.ops.op_process_read_stdout(handle);
    },

    async readStderr() {
      return await core.ops.op_process_read_stderr(handle);
    },

    stdout: {
      async *[Symbol.asyncIterator]() {
        while (true) {
          const output = await core.ops.op_process_read_stdout(handle);
          if (output.eof) break;
          if (output.data !== null) yield output.data;
        }
      },
    },

    stderr: {
      async *[Symbol.asyncIterator]() {
        while (true) {
          const output = await core.ops.op_process_read_stderr(handle);
          if (output.eof) break;
          if (output.data !== null) yield output.data;
        }
      },
    },
  };
}

export async function kill(handle, signal) {
  return await core.ops.op_process_kill(handle, signal);
}

export async function wait(handle) {
  return await core.ops.op_process_wait(handle);
}

export async function status(handle) {
  const raw = await core.ops.op_process_status(handle);
  return {
    running: raw.running,
    exitCode: raw.exit_code ?? undefined,
    signal: raw.signal ?? undefined,
  };
}

export async function writeStdin(handle, data) {
  return await core.ops.op_process_write_stdin(handle, data);
}

export async function readStdout(handle) {
  return await core.ops.op_process_read_stdout(handle);
}

export async function readStderr(handle) {
  return await core.ops.op_process_read_stderr(handle);
}
