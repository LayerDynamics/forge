// host:fs module - TypeScript wrapper for Deno core ops

// Deno.core type declaration
declare const Deno: {
  core: {
    ops: {
      op_fs_read_text(path: string): Promise<string>;
      op_fs_write_text(path: string, content: string): Promise<void>;
      op_fs_read_bytes(path: string): Promise<number[]>;
      op_fs_write_bytes(path: string, content: number[]): Promise<void>;
      op_fs_stat(path: string): Promise<FileStat>;
      op_fs_read_dir(path: string): Promise<DirEntry[]>;
      op_fs_mkdir(path: string, opts: MkdirOptions): Promise<void>;
      op_fs_remove(path: string, opts: RemoveOptions): Promise<void>;
      op_fs_rename(from: string, to: string): Promise<void>;
      op_fs_copy(from: string, to: string): Promise<void>;
      op_fs_exists(path: string): Promise<boolean>;
      op_fs_watch(path: string): Promise<number>;
      op_fs_watch_next(watchId: number): Promise<WatchEvent | null>;
      op_fs_watch_close(watchId: number): Promise<void>;
    };
  };
};

interface FileStat {
  isFile: boolean;
  isDirectory: boolean;
  isSymlink: boolean;
  size: number;
  mtime: number | null;
  atime: number | null;
  birthtime: number | null;
  readonly: boolean;
}

interface DirEntry {
  name: string;
  isFile: boolean;
  isDirectory: boolean;
  isSymlink: boolean;
}

interface MkdirOptions {
  recursive?: boolean;
}

interface RemoveOptions {
  recursive?: boolean;
}

interface WatchEvent {
  kind: string;
  paths: string[];
}

interface Watcher {
  id: number;
  next(): Promise<WatchEvent | null>;
  [Symbol.asyncIterator](): AsyncGenerator<WatchEvent, void, unknown>;
  close(): Promise<void>;
}

const core = Deno.core;

export async function readTextFile(path: string): Promise<string> {
  return await core.ops.op_fs_read_text(path);
}

export async function writeTextFile(path: string, content: string): Promise<void> {
  return await core.ops.op_fs_write_text(path, content);
}

export async function readBytes(path: string): Promise<Uint8Array> {
  const bytes = await core.ops.op_fs_read_bytes(path);
  return new Uint8Array(bytes);
}

export async function writeBytes(path: string, content: Uint8Array): Promise<void> {
  return await core.ops.op_fs_write_bytes(path, Array.from(content));
}

export async function stat(path: string): Promise<FileStat> {
  return await core.ops.op_fs_stat(path);
}

export async function readDir(path: string): Promise<DirEntry[]> {
  return await core.ops.op_fs_read_dir(path);
}

export async function mkdir(path: string, opts: MkdirOptions = {}): Promise<void> {
  return await core.ops.op_fs_mkdir(path, opts);
}

export async function remove(path: string, opts: RemoveOptions = {}): Promise<void> {
  return await core.ops.op_fs_remove(path, opts);
}

export async function rename(from: string, to: string): Promise<void> {
  return await core.ops.op_fs_rename(from, to);
}

export async function copy(from: string, to: string): Promise<void> {
  return await core.ops.op_fs_copy(from, to);
}

export async function exists(path: string): Promise<boolean> {
  return await core.ops.op_fs_exists(path);
}

export async function watch(path: string): Promise<Watcher> {
  const watchId = await core.ops.op_fs_watch(path);
  return {
    id: watchId,
    async next(): Promise<WatchEvent | null> {
      return await core.ops.op_fs_watch_next(watchId);
    },
    async *[Symbol.asyncIterator](): AsyncGenerator<WatchEvent, void, unknown> {
      while (true) {
        const event = await core.ops.op_fs_watch_next(watchId);
        if (event === null) break;
        yield event;
      }
    },
    async close(): Promise<void> {
      return await core.ops.op_fs_watch_close(watchId);
    }
  };
}

// Legacy aliases
export const readFile = readBytes;
export const writeFile = writeBytes;
export const watchFile = watch;
