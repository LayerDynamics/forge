// host:fs module - Deno API for filesystem operations
// This is the single source of truth for the host:fs SDK

// Type definitions
export interface FileStat {
  is_file: boolean;
  is_dir: boolean;
  size: number;
  readonly: boolean;
}

export interface DirEntry {
  name: string;
  is_file: boolean;
  is_dir: boolean;
}

export interface MkdirOptions {
  recursive?: boolean;
}

export interface RemoveOptions {
  recursive?: boolean;
}

export interface FileEvent {
  kind: string;
  paths: string[];
}

export interface FileWatcher {
  id: string;
  /** Get the next file event (blocks until event occurs) */
  next(): Promise<FileEvent | null>;
  /** Async iterator for file events */
  [Symbol.asyncIterator](): AsyncIterableIterator<FileEvent>;
  /** Close the watcher and stop receiving events */
  close(): Promise<void>;
}

// Deno.core.ops type declaration
declare const Deno: {
  core: {
    ops: {
      op_fs_read_text(path: string): Promise<string>;
      op_fs_write_text(path: string, content: string): Promise<void>;
      op_fs_read_bytes(path: string): Promise<number[]>;
      op_fs_write_bytes(path: string, content: number[]): Promise<void>;
      op_fs_stat(path: string): Promise<FileStat>;
      op_fs_read_dir(path: string): Promise<DirEntry[]>;
      op_fs_mkdir(path: string, options: MkdirOptions): Promise<void>;
      op_fs_remove(path: string, options: RemoveOptions): Promise<void>;
      op_fs_rename(from: string, to: string): Promise<void>;
      op_fs_copy(from: string, to: string): Promise<void>;
      op_fs_exists(path: string): Promise<boolean>;
      op_fs_watch(path: string): Promise<string>;
      op_fs_watch_next(watchId: string): Promise<FileEvent | null>;
      op_fs_watch_close(watchId: string): Promise<void>;
    };
  };
};

/**
 * Read a text file from the filesystem.
 * Subject to manifest permissions (fs.read).
 *
 * @param path - The file path to read
 * @returns Promise resolving to the file contents
 *
 * @example
 * ```ts
 * import { readTextFile } from "host:fs";
 *
 * const content = await readTextFile("./config.json");
 * const config = JSON.parse(content);
 * ```
 */
export async function readTextFile(path: string): Promise<string> {
  return await Deno.core.ops.op_fs_read_text(path);
}

/**
 * Write text to a file.
 * Subject to manifest permissions (fs.write).
 *
 * @param path - The file path to write
 * @param content - The text content to write
 *
 * @example
 * ```ts
 * import { writeTextFile } from "host:fs";
 *
 * await writeTextFile("./output.txt", "Hello, World!");
 * ```
 */
export async function writeTextFile(path: string, content: string): Promise<void> {
  return await Deno.core.ops.op_fs_write_text(path, content);
}

/**
 * Read a file as raw bytes.
 *
 * @param path - The file path to read
 * @returns Promise resolving to the file contents as Uint8Array
 */
export async function readBytes(path: string): Promise<Uint8Array> {
  const bytes = await Deno.core.ops.op_fs_read_bytes(path);
  return new Uint8Array(bytes);
}

/**
 * Write raw bytes to a file.
 *
 * @param path - The file path to write
 * @param content - The bytes to write
 */
export async function writeBytes(path: string, content: Uint8Array): Promise<void> {
  return await Deno.core.ops.op_fs_write_bytes(path, Array.from(content));
}

/**
 * Get file/directory stats.
 *
 * @param path - The path to stat
 * @returns Promise resolving to file statistics
 */
export async function stat(path: string): Promise<FileStat> {
  return await Deno.core.ops.op_fs_stat(path);
}

/**
 * Read directory contents.
 *
 * @param path - The directory path to read
 * @returns Promise resolving to array of directory entries
 */
export async function readDir(path: string): Promise<DirEntry[]> {
  return await Deno.core.ops.op_fs_read_dir(path);
}

/**
 * Create a directory.
 *
 * @param path - The directory path to create
 * @param opts - Options (recursive: create parent directories)
 */
export async function mkdir(path: string, opts: MkdirOptions = {}): Promise<void> {
  return await Deno.core.ops.op_fs_mkdir(path, opts);
}

/**
 * Remove a file or directory.
 *
 * @param path - The path to remove
 * @param opts - Options (recursive: remove directories and contents)
 */
export async function remove(path: string, opts: RemoveOptions = {}): Promise<void> {
  return await Deno.core.ops.op_fs_remove(path, opts);
}

/**
 * Rename/move a file or directory.
 *
 * @param from - The source path
 * @param to - The destination path
 */
export async function rename(from: string, to: string): Promise<void> {
  return await Deno.core.ops.op_fs_rename(from, to);
}

/**
 * Copy a file.
 *
 * @param from - The source path
 * @param to - The destination path
 */
export async function copy(from: string, to: string): Promise<void> {
  return await Deno.core.ops.op_fs_copy(from, to);
}

/**
 * Check if a path exists.
 *
 * @param path - The path to check
 * @returns Promise resolving to true if the path exists
 */
export async function exists(path: string): Promise<boolean> {
  return await Deno.core.ops.op_fs_exists(path);
}

/**
 * Watch a file or directory for changes.
 * Returns a FileWatcher that can be iterated or polled for events.
 *
 * @param path - The path to watch
 * @returns Promise resolving to a FileWatcher
 *
 * @example
 * ```ts
 * import { watch } from "host:fs";
 *
 * const watcher = await watch("./src");
 * for await (const event of watcher) {
 *   console.log(`${event.kind}: ${event.paths.join(", ")}`);
 * }
 * ```
 */
export async function watch(path: string): Promise<FileWatcher> {
  const watchId = await Deno.core.ops.op_fs_watch(path);
  return {
    id: watchId,
    async next(): Promise<FileEvent | null> {
      return await Deno.core.ops.op_fs_watch_next(watchId);
    },
    async *[Symbol.asyncIterator](): AsyncIterableIterator<FileEvent> {
      while (true) {
        const event = await Deno.core.ops.op_fs_watch_next(watchId);
        if (event === null) break;
        yield event;
      }
    },
    async close(): Promise<void> {
      return await Deno.core.ops.op_fs_watch_close(watchId);
    }
  };
}

// Legacy aliases for backwards compatibility
export const readFile = readBytes;
export const writeFile = writeBytes;
export const watchFile = watch;
