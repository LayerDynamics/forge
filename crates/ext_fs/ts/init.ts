/**
 * @module runtime:fs
 *
 * Filesystem operations for Forge applications.
 *
 * This module provides comprehensive file and directory operations including reading,
 * writing, watching, and managing filesystem entities. All operations respect
 * capability-based permissions defined in manifest.app.toml.
 *
 * ## Features
 * - File I/O (text and binary)
 * - Directory operations (create, read, remove)
 * - File watching with async iteration
 * - Symbolic link management
 * - File metadata and statistics
 * - Temporary file/directory creation
 * - Cross-platform path resolution
 *
 * ## Permissions
 * Requires `fs` permissions in manifest.app.toml:
 * ```toml
 * [permissions.fs]
 * read = ["./data/**", "./config.json"]
 * write = ["./data/**", "./logs/*.log"]
 * ```
 *
 * ## Error Codes
 * - 3000: I/O error during filesystem operation
 * - 3001: Permission denied by capability system
 * - 3002: File or directory not found
 * - 3003: File or directory already exists
 * - 3004: Path is a directory (expected file)
 * - 3005: Path is a file (expected directory)
 * - 3006: File watch error
 * - 3007: Invalid watch ID
 * - 3008: Symbolic link error
 * - 3009: Temporary file/directory creation error
 *
 * @example
 * ```typescript
 * import { readTextFile, writeTextFile } from "runtime:fs";
 *
 * // Read a text file
 * const content = await readTextFile("./data/config.json");
 * console.log(content);
 *
 * // Write a text file
 * await writeTextFile("./output.txt", "Hello, World!");
 * ```
 */

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

/**
 * File or directory statistics and metadata.
 */
export interface FileStat {
  /** Whether the path is a regular file */
  isFile: boolean;
  /** Whether the path is a directory */
  isDirectory: boolean;
  /** Whether the path is a symbolic link */
  isSymlink: boolean;
  /** Size of the file in bytes */
  size: number;
  /** Last modification time as Unix timestamp (milliseconds), or null if unavailable */
  mtime: number | null;
  /** Last access time as Unix timestamp (milliseconds), or null if unavailable */
  atime: number | null;
  /** Creation/birth time as Unix timestamp (milliseconds), or null if unavailable */
  birthtime: number | null;
  /** Whether the file is read-only */
  readonly: boolean;
}

/**
 * Entry in a directory listing.
 */
export interface DirEntry {
  /** Name of the file or directory (without path) */
  name: string;
  /** Whether this entry is a regular file */
  isFile: boolean;
  /** Whether this entry is a directory */
  isDirectory: boolean;
  /** Whether this entry is a symbolic link */
  isSymlink: boolean;
}

/**
 * Options for directory creation.
 */
export interface MkdirOptions {
  /**
   * Create parent directories if they don't exist.
   * @default false
   */
  recursive?: boolean;
}

/**
 * Options for file/directory removal.
 */
export interface RemoveOptions {
  /**
   * Remove directory and all its contents recursively.
   * Required for non-empty directories.
   * @default false
   */
  recursive?: boolean;
}

/**
 * Event emitted by a file watcher.
 */
export interface WatchEvent {
  /**
   * Type of filesystem event.
   * Common values: "create", "modify", "remove", "rename"
   */
  kind: string;
  /**
   * Paths affected by this event.
   * May contain one or more paths depending on the event type.
   */
  paths: string[];
}

/**
 * Handle to an active file watcher.
 *
 * Provides methods for receiving filesystem events and closing the watcher.
 *
 * @example
 * ```typescript
 * const watcher = await watch("./data");
 *
 * // Use async iteration
 * for await (const event of watcher) {
 *   console.log(`${event.kind}: ${event.paths.join(", ")}`);
 * }
 *
 * // Or manually poll for events
 * const event = await watcher.next();
 * if (event) {
 *   console.log(event);
 * }
 *
 * // Close when done
 * await watcher.close();
 * ```
 */
export interface Watcher {
  /** Internal watcher ID */
  readonly id: number;
  /**
   * Get the next filesystem event, or null if watcher closed.
   * @returns Next event or null
   */
  next(): Promise<WatchEvent | null>;
  /**
   * Async iterator for watching events.
   * Completes when watcher is closed.
   */
  [Symbol.asyncIterator](): AsyncGenerator<WatchEvent, void, unknown>;
  /**
   * Stop watching and clean up resources.
   */
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
 * Extended file metadata with timestamps and permissions.
 *
 * Provides more detailed information than FileStat, including
 * precise timestamps and permission bits.
 */
export interface FileMetadata {
  /** Whether the path is a regular file */
  isFile: boolean;
  /** Whether the path is a directory */
  isDir: boolean;
  /** Whether the path is a symbolic link */
  isSymlink: boolean;
  /** Size of the file in bytes */
  size: number;
  /** Whether the file is read-only */
  readonly: boolean;
  /** Creation time as Unix timestamp (milliseconds), or null if unavailable */
  createdAt: number | null;
  /** Last modification time as Unix timestamp (milliseconds), or null if unavailable */
  modifiedAt: number | null;
  /** Last access time as Unix timestamp (milliseconds), or null if unavailable */
  accessedAt: number | null;
  /**
   * Unix permission bits, or null if unavailable.
   * On Unix: standard permission bits (e.g., 0o755)
   * On Windows: null
   */
  permissions: number | null;
}

/**
 * Information about a created temporary file.
 */
export interface TempFileInfo {
  /** Absolute path to the temporary file */
  path: string;
}

/**
 * Information about a created temporary directory.
 */
export interface TempDirInfo {
  /** Absolute path to the temporary directory */
  path: string;
}

const core = Deno.core;

/**
 * Reads the entire contents of a file as a UTF-8 string.
 *
 * @param path - Path to the file to read
 * @returns File contents as string
 *
 * @throws Error (3000) if I/O error occurs
 * @throws Error (3001) if permission denied
 * @throws Error (3002) if file not found
 * @throws Error (3004) if path is a directory
 *
 * @example
 * ```typescript
 * const config = await readTextFile("./config.json");
 * const data = JSON.parse(config);
 * ```
 *
 * @example
 * ```typescript
 * // Read with error handling
 * try {
 *   const content = await readTextFile("./data.txt");
 *   console.log(content);
 * } catch (error) {
 *   if (error.message.includes("[3002]")) {
 *     console.error("File not found");
 *   }
 * }
 * ```
 */
export async function readTextFile(path: string): Promise<string> {
  return await core.ops.op_fs_read_text(path);
}

/**
 * Writes a string to a file, creating it if it doesn't exist.
 *
 * If the file already exists, its contents are completely replaced.
 *
 * @param path - Path to the file to write
 * @param content - String content to write
 *
 * @throws Error (3000) if I/O error occurs
 * @throws Error (3001) if permission denied
 * @throws Error (3005) if path is a directory
 *
 * @example
 * ```typescript
 * await writeTextFile("./output.txt", "Hello, World!");
 * ```
 *
 * @example
 * ```typescript
 * // Write JSON data
 * const data = { name: "Forge", version: "0.1.0" };
 * await writeTextFile("./data.json", JSON.stringify(data, null, 2));
 * ```
 */
export async function writeTextFile(path: string, content: string): Promise<void> {
  return await core.ops.op_fs_write_text(path, content);
}

/**
 * Reads the entire contents of a file as binary data.
 *
 * @param path - Path to the file to read
 * @returns File contents as Uint8Array
 *
 * @throws Error (3000) if I/O error occurs
 * @throws Error (3001) if permission denied
 * @throws Error (3002) if file not found
 * @throws Error (3004) if path is a directory
 *
 * @example
 * ```typescript
 * const bytes = await readBytes("./image.png");
 * console.log(`Read ${bytes.length} bytes`);
 * ```
 *
 * @example
 * ```typescript
 * // Read and process binary data
 * const data = await readBytes("./data.bin");
 * const view = new DataView(data.buffer);
 * const header = view.getUint32(0, true);
 * ```
 */
export async function readBytes(path: string): Promise<Uint8Array> {
  const bytes = await core.ops.op_fs_read_bytes(path);
  return new Uint8Array(bytes);
}

/**
 * Writes binary data to a file, creating it if it doesn't exist.
 *
 * If the file already exists, its contents are completely replaced.
 *
 * @param path - Path to the file to write
 * @param content - Binary data to write
 *
 * @throws Error (3000) if I/O error occurs
 * @throws Error (3001) if permission denied
 * @throws Error (3005) if path is a directory
 *
 * @example
 * ```typescript
 * const data = new Uint8Array([0x48, 0x65, 0x6C, 0x6C, 0x6F]); // "Hello"
 * await writeBytes("./data.bin", data);
 * ```
 *
 * @example
 * ```typescript
 * // Write structured binary data
 * const buffer = new ArrayBuffer(8);
 * const view = new DataView(buffer);
 * view.setUint32(0, 0x12345678, true);
 * await writeBytes("./header.bin", new Uint8Array(buffer));
 * ```
 */
export async function writeBytes(path: string, content: Uint8Array): Promise<void> {
  return await core.ops.op_fs_write_bytes(path, Array.from(content));
}

/**
 * Gets file or directory statistics.
 *
 * @param path - Path to the file or directory
 * @returns Statistics including type, size, and timestamps
 *
 * @throws Error (3000) if I/O error occurs
 * @throws Error (3001) if permission denied
 * @throws Error (3002) if path not found
 *
 * @example
 * ```typescript
 * const info = await stat("./file.txt");
 * console.log(`Size: ${info.size} bytes`);
 * console.log(`Modified: ${new Date(info.mtime!)}`);
 * if (info.isDirectory) {
 *   console.log("It's a directory");
 * }
 * ```
 */
export async function stat(path: string): Promise<FileStat> {
  return await core.ops.op_fs_stat(path);
}

/**
 * Reads the contents of a directory.
 *
 * Returns an array of entries in the directory, not including "." and "..".
 * Does not recurse into subdirectories.
 *
 * @param path - Path to the directory
 * @returns Array of directory entries
 *
 * @throws Error (3000) if I/O error occurs
 * @throws Error (3001) if permission denied
 * @throws Error (3002) if directory not found
 * @throws Error (3005) if path is a file
 *
 * @example
 * ```typescript
 * const entries = await readDir("./data");
 * for (const entry of entries) {
 *   console.log(`${entry.name} (${entry.isFile ? "file" : "dir"})`);
 * }
 * ```
 *
 * @example
 * ```typescript
 * // List only files
 * const files = (await readDir("./data"))
 *   .filter(e => e.isFile)
 *   .map(e => e.name);
 * ```
 */
export async function readDir(path: string): Promise<DirEntry[]> {
  return await core.ops.op_fs_read_dir(path);
}

/**
 * Creates a directory.
 *
 * @param path - Path where directory should be created
 * @param opts - Options including recursive creation
 *
 * @throws Error (3000) if I/O error occurs
 * @throws Error (3001) if permission denied
 * @throws Error (3002) if parent directory not found (when recursive=false)
 * @throws Error (3003) if directory already exists
 *
 * @example
 * ```typescript
 * // Create single directory
 * await mkdir("./data");
 * ```
 *
 * @example
 * ```typescript
 * // Create nested directories
 * await mkdir("./data/logs/2024", { recursive: true });
 * ```
 */
export async function mkdir(path: string, opts: MkdirOptions = {}): Promise<void> {
  return await core.ops.op_fs_mkdir(path, opts);
}

/**
 * Removes a file or directory.
 *
 * @param path - Path to remove
 * @param opts - Options including recursive removal for directories
 *
 * @throws Error (3000) if I/O error occurs
 * @throws Error (3001) if permission denied
 * @throws Error (3002) if path not found
 * @throws Error (3005) if path is a file when removing directory
 *
 * @example
 * ```typescript
 * // Remove a file
 * await remove("./temp.txt");
 * ```
 *
 * @example
 * ```typescript
 * // Remove directory and all contents
 * await remove("./old-data", { recursive: true });
 * ```
 */
export async function remove(path: string, opts: RemoveOptions = {}): Promise<void> {
  return await core.ops.op_fs_remove(path, opts);
}

/**
 * Renames or moves a file or directory.
 *
 * @param from - Current path
 * @param to - New path
 *
 * @throws Error (3000) if I/O error occurs
 * @throws Error (3001) if permission denied
 * @throws Error (3002) if source path not found
 * @throws Error (3003) if destination already exists
 *
 * @example
 * ```typescript
 * // Rename file
 * await rename("./old-name.txt", "./new-name.txt");
 * ```
 *
 * @example
 * ```typescript
 * // Move file to different directory
 * await rename("./temp/file.txt", "./data/file.txt");
 * ```
 */
export async function rename(from: string, to: string): Promise<void> {
  return await core.ops.op_fs_rename(from, to);
}

/**
 * Copies a file or directory.
 *
 * @param from - Source path
 * @param to - Destination path
 *
 * @throws Error (3000) if I/O error occurs
 * @throws Error (3001) if permission denied
 * @throws Error (3002) if source not found
 * @throws Error (3003) if destination already exists
 *
 * @example
 * ```typescript
 * // Copy file
 * await copy("./source.txt", "./backup.txt");
 * ```
 *
 * @example
 * ```typescript
 * // Copy to different location
 * await copy("./config.json", "./backups/config-2024.json");
 * ```
 */
export async function copy(from: string, to: string): Promise<void> {
  return await core.ops.op_fs_copy(from, to);
}

/**
 * Checks if a file or directory exists.
 *
 * @param path - Path to check
 * @returns true if path exists, false otherwise
 *
 * @example
 * ```typescript
 * if (await exists("./config.json")) {
 *   console.log("Config file found");
 * } else {
 *   console.log("Config file missing");
 * }
 * ```
 *
 * @example
 * ```typescript
 * // Check before creating
 * if (!(await exists("./data"))) {
 *   await mkdir("./data");
 * }
 * ```
 */
export async function exists(path: string): Promise<boolean> {
  return await core.ops.op_fs_exists(path);
}

/**
 * Watches a file or directory for changes.
 *
 * Returns a Watcher that can be used with async iteration or manual polling.
 * The watcher emits events for file creation, modification, deletion, and renaming.
 *
 * **Important:** Always call `watcher.close()` when done to clean up resources.
 *
 * @param path - Path to watch (file or directory)
 * @returns Watcher for receiving filesystem events
 *
 * @throws Error (3000) if I/O error occurs
 * @throws Error (3001) if permission denied
 * @throws Error (3002) if path not found
 * @throws Error (3006) if watch setup fails
 *
 * @example
 * ```typescript
 * // Watch with async iteration
 * const watcher = await watch("./data");
 * try {
 *   for await (const event of watcher) {
 *     console.log(`${event.kind}: ${event.paths.join(", ")}`);
 *     if (event.kind === "modify") {
 *       console.log("File was modified");
 *     }
 *   }
 * } finally {
 *   await watcher.close();
 * }
 * ```
 *
 * @example
 * ```typescript
 * // Manual polling
 * const watcher = await watch("./config.json");
 * while (true) {
 *   const event = await watcher.next();
 *   if (!event) break; // Watcher closed
 *   console.log("Config changed:", event);
 *   // Reload config...
 * }
 * await watcher.close();
 * ```
 *
 * @example
 * ```typescript
 * // Watch with timeout
 * const watcher = await watch("./data");
 * const timeout = setTimeout(async () => {
 *   console.log("Watch timeout");
 *   await watcher.close();
 * }, 60000);
 *
 * for await (const event of watcher) {
 *   console.log(event);
 * }
 * clearTimeout(timeout);
 * ```
 */
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
 * Creates a symbolic link pointing to a target path.
 *
 * The symlink can point to a file or directory. On Windows, directory symlinks
 * require administrator privileges or Developer Mode enabled.
 *
 * @param target - The target path the symlink points to (can be relative or absolute)
 * @param path - The path where the symlink will be created
 *
 * @throws Error (3000) if I/O error occurs
 * @throws Error (3001) if permission denied
 * @throws Error (3003) if symlink path already exists
 * @throws Error (3008) if symbolic link creation fails
 *
 * @example
 * ```typescript
 * // Create a symlink to a file
 * await symlink("./data/original.txt", "./data/link.txt");
 * ```
 *
 * @example
 * ```typescript
 * // Create a symlink to a directory
 * await symlink("/var/log/app", "./logs");
 * ```
 *
 * @example
 * ```typescript
 * // Create relative symlink
 * await symlink("../shared/config.json", "./config.json");
 * const target = await readLink("./config.json");
 * console.log(`Points to: ${target}`);
 * ```
 */
export async function symlink(target: string, path: string): Promise<void> {
  return await core.ops.op_fs_symlink(target, path);
}

/**
 * Reads the target path of a symbolic link.
 *
 * Returns the path that the symbolic link points to. The returned path may be
 * relative or absolute depending on how the symlink was created. Use `realPath()`
 * to resolve to an absolute canonical path.
 *
 * @param path - Path to the symbolic link
 * @returns The target path the symlink points to
 *
 * @throws Error (3000) if I/O error occurs
 * @throws Error (3001) if permission denied
 * @throws Error (3002) if symlink not found
 * @throws Error (3005) if path is not a symbolic link
 * @throws Error (3008) if symbolic link read fails
 *
 * @example
 * ```typescript
 * // Read a symlink target
 * const target = await readLink("./config.json");
 * console.log(`Symlink points to: ${target}`);
 * ```
 *
 * @example
 * ```typescript
 * // Check if path is a symlink before reading
 * const stats = await stat("./data");
 * if (stats.isSymlink) {
 *   const target = await readLink("./data");
 *   console.log(`Symlink target: ${target}`);
 * }
 * ```
 *
 * @example
 * ```typescript
 * // Resolve symlink to canonical path
 * const target = await readLink("./logs");
 * const canonical = await realPath(target);
 * console.log(`Canonical path: ${canonical}`);
 * ```
 */
export async function readLink(path: string): Promise<string> {
  return await core.ops.op_fs_read_link(path);
}

/**
 * Appends text content to a file, creating it if it doesn't exist.
 *
 * The content is encoded as UTF-8 and appended to the end of the file. If the
 * file doesn't exist, it is created. This is more efficient than reading,
 * concatenating, and writing for log files or incremental data.
 *
 * @param path - The file path to append to
 * @param content - The text content to append (UTF-8 encoded)
 *
 * @throws Error (3000) if I/O error occurs
 * @throws Error (3001) if permission denied
 * @throws Error (3004) if path is a directory
 *
 * @example
 * ```typescript
 * // Append to a log file
 * const timestamp = new Date().toISOString();
 * await appendTextFile("./app.log", `${timestamp} - Application started\n`);
 * ```
 *
 * @example
 * ```typescript
 * // Append multiple entries
 * for (const entry of logEntries) {
 *   await appendTextFile("./data.log", `${entry}\n`);
 * }
 * ```
 *
 * @example
 * ```typescript
 * // Create file if it doesn't exist
 * const exists = await exists("./notes.txt");
 * if (!exists) {
 *   await appendTextFile("./notes.txt", "First note\n");
 * } else {
 *   await appendTextFile("./notes.txt", "Additional note\n");
 * }
 * ```
 */
export async function appendTextFile(path: string, content: string): Promise<void> {
  return await core.ops.op_fs_append_text(path, content);
}

/**
 * Appends binary data to a file, creating it if it doesn't exist.
 *
 * The bytes are appended to the end of the file without any encoding. If the
 * file doesn't exist, it is created. Useful for binary logs, incremental data
 * files, or appending to serialized formats.
 *
 * @param path - The file path to append to
 * @param content - The bytes to append
 *
 * @throws Error (3000) if I/O error occurs
 * @throws Error (3001) if permission denied
 * @throws Error (3004) if path is a directory
 *
 * @example
 * ```typescript
 * // Append binary data to a file
 * const data = new Uint8Array([0x48, 0x65, 0x6C, 0x6C, 0x6F]);
 * await appendBytes("./data.bin", data);
 * ```
 *
 * @example
 * ```typescript
 * // Append serialized data
 * const encoder = new TextEncoder();
 * const message = encoder.encode("Log entry\n");
 * await appendBytes("./binary.log", message);
 * ```
 *
 * @example
 * ```typescript
 * // Incrementally build a binary file
 * for (const chunk of dataChunks) {
 *   await appendBytes("./output.dat", chunk);
 * }
 * ```
 */
export async function appendBytes(path: string, content: Uint8Array): Promise<void> {
  return await core.ops.op_fs_append_bytes(path, Array.from(content));
}

/**
 * Retrieves extended file metadata including detailed timestamps and permissions.
 *
 * This function returns more detailed information than `stat()`, including
 * creation time, modification time, access time, and platform-specific permissions.
 * Use this when you need precise timestamp information or permission details.
 *
 * @param path - The path to get metadata for (file or directory)
 * @returns Extended metadata including creation/modification/access times and permissions
 *
 * @throws Error (3000) if I/O error occurs
 * @throws Error (3001) if permission denied
 * @throws Error (3002) if path not found
 *
 * @example
 * ```typescript
 * // Get detailed file metadata
 * const meta = await metadata("./config.json");
 * console.log(`Created: ${new Date(meta.createdAt)}`);
 * console.log(`Modified: ${new Date(meta.modifiedAt)}`);
 * console.log(`Permissions: ${meta.permissions}`);
 * ```
 *
 * @example
 * ```typescript
 * // Check if file was recently modified
 * const meta = await metadata("./data.json");
 * const hourAgo = Date.now() - (60 * 60 * 1000);
 * if (meta.modifiedAt > hourAgo) {
 *   console.log("File was modified in the last hour");
 * }
 * ```
 *
 * @example
 * ```typescript
 * // Compare creation and modification times
 * const meta = await metadata("./document.txt");
 * if (meta.modifiedAt > meta.createdAt) {
 *   const diffMs = meta.modifiedAt - meta.createdAt;
 *   console.log(`File edited ${diffMs}ms after creation`);
 * }
 * ```
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
 * Resolves a path to its canonical, absolute form by resolving all symbolic links
 * and relative path components.
 *
 * This function returns the "real" path by resolving:
 * - Symbolic links (recursively)
 * - Relative components (., ..)
 * - Converting to absolute path
 *
 * The resolved path is guaranteed to be canonical and absolute.
 *
 * @param path - The path to resolve (can be relative or absolute)
 * @returns The canonical absolute path
 *
 * @throws Error (3000) if I/O error occurs
 * @throws Error (3001) if permission denied
 * @throws Error (3002) if path not found
 * @throws Error (3008) if symbolic link resolution fails
 *
 * @example
 * ```typescript
 * // Resolve relative path to absolute
 * const absolute = await realPath("./config.json");
 * console.log(absolute); // "/Users/username/app/config.json"
 * ```
 *
 * @example
 * ```typescript
 * // Resolve symlink to target
 * await symlink("/var/log/app", "./logs");
 * const target = await realPath("./logs");
 * console.log(target); // "/var/log/app"
 * ```
 *
 * @example
 * ```typescript
 * // Resolve complex path with .. and symlinks
 * const canonical = await realPath("../shared/../../config/app.conf");
 * console.log(canonical); // "/etc/config/app.conf"
 * ```
 */
export async function realPath(path: string): Promise<string> {
  return await core.ops.op_fs_real_path(path);
}

/**
 * Creates a temporary file in the system's temp directory.
 *
 * The file is created with a unique name and persists until explicitly deleted.
 * Unlike some temp file APIs, this does NOT automatically delete the file when
 * the process exits - you must remove it manually.
 *
 * The file is created with read/write permissions for the current user only.
 *
 * @param prefix - Optional prefix for the file name (default: "forge-")
 * @param suffix - Optional suffix for the file name (e.g., ".txt", ".json")
 * @returns Information about the created temp file (path)
 *
 * @throws Error (3000) if I/O error occurs
 * @throws Error (3009) if temporary file creation fails
 *
 * @example
 * ```typescript
 * // Create a temp file with default naming
 * const temp = await tempFile();
 * await writeTextFile(temp.path, "temporary data");
 * // ... use the file ...
 * await remove(temp.path); // Clean up
 * ```
 *
 * @example
 * ```typescript
 * // Create temp file with prefix and suffix
 * const temp = await tempFile("myapp-", ".json");
 * console.log(temp.path); // e.g., "/tmp/myapp-abc123.json"
 * await writeTextFile(temp.path, JSON.stringify({ data: "test" }));
 * ```
 *
 * @example
 * ```typescript
 * // Use temp file for processing
 * const temp = await tempFile("process-", ".dat");
 * try {
 *   await writeBytes(temp.path, processedData);
 *   const result = await externalTool(temp.path);
 *   return result;
 * } finally {
 *   await remove(temp.path);
 * }
 * ```
 */
export async function tempFile(prefix?: string, suffix?: string): Promise<TempFileInfo> {
  const result = await core.ops.op_fs_temp_file(prefix ?? null, suffix ?? null);
  return { path: result.path };
}

/**
 * Creates a temporary directory in the system's temp directory.
 *
 * The directory is created with a unique name and persists until explicitly deleted.
 * Unlike some temp directory APIs, this does NOT automatically delete the directory
 * when the process exits - you must remove it manually with `remove()`.
 *
 * The directory is created with read/write/execute permissions for the current user only.
 *
 * @param prefix - Optional prefix for the directory name (default: "forge-")
 * @returns Information about the created temp directory (path)
 *
 * @throws Error (3000) if I/O error occurs
 * @throws Error (3009) if temporary directory creation fails
 *
 * @example
 * ```typescript
 * // Create a temp directory
 * const temp = await tempDir();
 * await writeTextFile(`${temp.path}/data.txt`, "content");
 * // ... use the directory ...
 * await remove(temp.path, { recursive: true }); // Clean up
 * ```
 *
 * @example
 * ```typescript
 * // Create temp directory with prefix
 * const temp = await tempDir("build-");
 * console.log(temp.path); // e.g., "/tmp/build-xyz789"
 * await mkdir(`${temp.path}/output`);
 * ```
 *
 * @example
 * ```typescript
 * // Use temp directory for batch processing
 * const temp = await tempDir("batch-");
 * try {
 *   for (const file of inputFiles) {
 *     const outputPath = `${temp.path}/${file.name}`;
 *     await processFile(file, outputPath);
 *   }
 *   await packageResults(temp.path);
 * } finally {
 *   await remove(temp.path, { recursive: true });
 * }
 * ```
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
