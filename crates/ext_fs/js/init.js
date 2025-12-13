// host:fs module - JavaScript wrapper for Deno core ops
const core = Deno.core;

export async function readTextFile(path) {
  return await core.ops.op_fs_read_text(path);
}

export async function writeTextFile(path, content) {
  return await core.ops.op_fs_write_text(path, content);
}

export async function readBytes(path) {
  const bytes = await core.ops.op_fs_read_bytes(path);
  return new Uint8Array(bytes);
}

export async function writeBytes(path, content) {
  return await core.ops.op_fs_write_bytes(path, Array.from(content));
}

export async function stat(path) {
  return await core.ops.op_fs_stat(path);
}

export async function readDir(path) {
  return await core.ops.op_fs_read_dir(path);
}

export async function mkdir(path, opts = {}) {
  return await core.ops.op_fs_mkdir(path, opts);
}

export async function remove(path, opts = {}) {
  return await core.ops.op_fs_remove(path, opts);
}

export async function rename(from, to) {
  return await core.ops.op_fs_rename(from, to);
}

export async function copy(from, to) {
  return await core.ops.op_fs_copy(from, to);
}

export async function exists(path) {
  return await core.ops.op_fs_exists(path);
}

export async function watch(path) {
  const watchId = await core.ops.op_fs_watch(path);
  return {
    id: watchId,
    async next() {
      return await core.ops.op_fs_watch_next(watchId);
    },
    async *[Symbol.asyncIterator]() {
      while (true) {
        const event = await core.ops.op_fs_watch_next(watchId);
        if (event === null) break;
        yield event;
      }
    },
    async close() {
      return await core.ops.op_fs_watch_close(watchId);
    }
  };
}

// Legacy aliases
export const readFile = readBytes;
export const writeFile = writeBytes;
export const watchFile = watch;
