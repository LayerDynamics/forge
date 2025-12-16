// runtime:fs module - TypeScript wrapper for Deno core ops

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
      // Enhanced operations
      op_fs_symlink(target: string, path: string): Promise<void>;
      op_fs_read_link(path: string): Promise<string>;
      op_fs_append_text(path: string, content: string): Promise<void>;
      op_fs_append_bytes(path: string, content: number[]): Promise<void>;
      op_fs_metadata(path: string): Promise<FileMetadataResult>;
      op_fs_real_path(path: string): Promise<string>;
      op_fs_temp_file(prefix: string | null, suffix: string | null): Promise<TempFileResult>;
      op_fs_temp_dir(prefix: string | null): Promise<TempDirResult>;
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

// Enhanced types
interface FileMetadataResult {
  is_file: boolean;
  is_dir: boolean;
  is_symlink: boolean;
  size: number;
  readonly: boolean;
  created_at: number | null;
  modified_at: number | null;
  accessed_at: number | null;
  permissions: number | null;
}

interface TempFileResult {
  path: string;
}

interface TempDirResult {
  path: string;
}

/**
 * Extended file metadata with timestamps
 */
export interface FileMetadata {
  isFile: boolean;
  isDir: boolean;
  isSymlink: boolean;
  size: number;
  readonly: boolean;
  createdAt: number | null;
  modifiedAt: number | null;
  accessedAt: number | null;
  permissions: number | null;
}

/**
 * Temporary file information
 */
export interface TempFileInfo {
  path: string;
}

/**
 * Temporary directory information
 */
export interface TempDirInfo {
  path: string;
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

// ============================================================================
// Enhanced Operations
// ============================================================================

/**
 * Create a symbolic link.
 * @param target - The target path the symlink points to
 * @param path - The path where the symlink will be created
 */
export async function symlink(target: string, path: string): Promise<void> {
  return await core.ops.op_fs_symlink(target, path);
}

/**
 * Read the target of a symbolic link.
 * @param path - Path to the symbolic link
 * @returns The target path the symlink points to
 */
export async function readLink(path: string): Promise<string> {
  return await core.ops.op_fs_read_link(path);
}

/**
 * Append text to a file. Creates the file if it doesn't exist.
 * @param path - The file path to append to
 * @param content - The text content to append
 */
export async function appendTextFile(path: string, content: string): Promise<void> {
  return await core.ops.op_fs_append_text(path, content);
}

/**
 * Append bytes to a file. Creates the file if it doesn't exist.
 * @param path - The file path to append to
 * @param content - The bytes to append
 */
export async function appendBytes(path: string, content: Uint8Array): Promise<void> {
  return await core.ops.op_fs_append_bytes(path, Array.from(content));
}

/**
 * Get extended file metadata including timestamps.
 * @param path - The path to get metadata for
 * @returns Extended metadata including creation/modification times
 */
export async function metadata(path: string): Promise<FileMetadata> {
  const result = await core.ops.op_fs_metadata(path);
  return {
    isFile: result.is_file,
    isDir: result.is_dir,
    isSymlink: result.is_symlink,
    size: result.size,
    readonly: result.readonly,
    createdAt: result.created_at,
    modifiedAt: result.modified_at,
    accessedAt: result.accessed_at,
    permissions: result.permissions,
  };
}

/**
 * Resolve a path to its canonical, absolute form (resolving symlinks).
 * @param path - The path to resolve
 * @returns The canonical absolute path
 */
export async function realPath(path: string): Promise<string> {
  return await core.ops.op_fs_real_path(path);
}

/**
 * Create a temporary file that persists until explicitly deleted.
 * @param prefix - Optional prefix for the file name
 * @param suffix - Optional suffix for the file name
 * @returns Information about the created temp file
 */
export async function tempFile(prefix?: string, suffix?: string): Promise<TempFileInfo> {
  const result = await core.ops.op_fs_temp_file(prefix ?? null, suffix ?? null);
  return { path: result.path };
}

/**
 * Create a temporary directory that persists until explicitly deleted.
 * @param prefix - Optional prefix for the directory name
 * @returns Information about the created temp directory
 */
export async function tempDir(prefix?: string): Promise<TempDirInfo> {
  const result = await core.ops.op_fs_temp_dir(prefix ?? null);
  return { path: result.path };
}

// Legacy aliases
export const readFile = readBytes;
export const writeFile = writeBytes;
export const watchFile = watch;
export const appendFile = appendBytes;
export const createSymlink = symlink;
