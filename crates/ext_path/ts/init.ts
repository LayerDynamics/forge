/**
 * @module runtime:path
 *
 * Cross-platform path manipulation utilities for Forge applications.
 *
 * This module provides functions for working with filesystem paths in a
 * platform-independent way. All operations handle forward slashes on Unix
 * and backslashes on Windows automatically.
 *
 * ## Features
 * - Join path segments with correct separators
 * - Extract directory names and basenames
 * - Get file extensions
 * - Parse paths into components
 * - Cross-platform path normalization
 *
 * ## Platform Behavior
 * - **Unix (macOS/Linux)**: Uses forward slashes (`/`)
 * - **Windows**: Uses backslashes (`\`)
 * - Operations automatically use platform-appropriate separators
 *
 * ## No Permissions Required
 * Path operations are pure string manipulation and don't require filesystem
 * permissions. They work with any path string, whether or not it exists.
 *
 * @example
 * ```typescript
 * import { join, dirname, basename, extname } from "runtime:path";
 *
 * // Join path segments
 * const configPath = join("./data", "config.json");
 *
 * // Extract components
 * const dir = dirname("/usr/local/bin/node");  // "/usr/local/bin"
 * const file = basename("/usr/local/bin/node"); // "node"
 * const ext = extname("file.txt");              // ".txt"
 * ```
 */

/**
 * Components of a parsed path.
 */
interface PathParts {
  /** Directory path (empty string if no directory) */
  dir: string;
  /** Base filename including extension */
  base: string;
  /** File extension including the dot (empty string if no extension) */
  ext: string;
}

declare const Deno: {
  core: {
    ops: {
      op_path_join(base: string, segments: string[]): string;
      op_path_dirname(path: string): string;
      op_path_basename(path: string): string;
      op_path_extname(path: string): string;
      op_path_parts(path: string): PathParts;
    };
  };
};

const { core } = Deno;
const ops = {
  join: core.ops.op_path_join,
  dirname: core.ops.op_path_dirname,
  basename: core.ops.op_path_basename,
  extname: core.ops.op_path_extname,
  parts: core.ops.op_path_parts,
};

/**
 * Joins path segments into a single path using platform-appropriate separators.
 *
 * On Unix systems, uses forward slashes (`/`). On Windows, uses backslashes (`\`).
 * Automatically normalizes redundant separators and handles relative path components.
 *
 * @param base - The base path to start from
 * @param segments - Additional path segments to append
 * @returns Combined path with platform-appropriate separators
 *
 * @example
 * ```typescript
 * // Unix: "./data/config.json"
 * // Windows: ".\\data\\config.json"
 * const path = join("./data", "config.json");
 * ```
 *
 * @example
 * ```typescript
 * // Build nested paths
 * const imagePath = join("./assets", "images", "logo.png");
 * // Unix: "./assets/images/logo.png"
 * // Windows: ".\\assets\\images\\logo.png"
 * ```
 *
 * @example
 * ```typescript
 * // Join with absolute paths
 * const binPath = join("/usr", "local", "bin", "node");
 * // Unix: "/usr/local/bin/node"
 * ```
 */
export function join(base: string, ...segments: string[]): string {
  return ops.join(base, segments);
}

/**
 * Extracts the directory path from a file path.
 *
 * Returns everything before the final path separator. If there is no directory
 * component, returns an empty string.
 *
 * @param path - The path to extract the directory from
 * @returns The directory portion of the path, or empty string if none
 *
 * @example
 * ```typescript
 * const dir = dirname("/usr/local/bin/node");
 * console.log(dir); // "/usr/local/bin"
 * ```
 *
 * @example
 * ```typescript
 * const dir = dirname("./data/config.json");
 * console.log(dir); // "./data"
 * ```
 *
 * @example
 * ```typescript
 * // No directory component
 * const dir = dirname("file.txt");
 * console.log(dir); // ""
 * ```
 */
export function dirname(path: string): string {
  return ops.dirname(path);
}

/**
 * Extracts the final component of a path (filename with extension).
 *
 * Returns the last segment of the path after the final separator. If the path
 * ends with a separator, returns an empty string.
 *
 * @param path - The path to extract the basename from
 * @returns The filename portion of the path, or empty string if none
 *
 * @example
 * ```typescript
 * const file = basename("/usr/local/bin/node");
 * console.log(file); // "node"
 * ```
 *
 * @example
 * ```typescript
 * const file = basename("./data/config.json");
 * console.log(file); // "config.json"
 * ```
 *
 * @example
 * ```typescript
 * // Path with no directory
 * const file = basename("readme.md");
 * console.log(file); // "readme.md"
 * ```
 */
export function basename(path: string): string {
  return ops.basename(path);
}

/**
 * Extracts the file extension from a path.
 *
 * Returns the extension including the leading dot. If there is no extension,
 * returns an empty string. Only considers the portion after the last dot in
 * the basename.
 *
 * @param path - The path to extract the extension from
 * @returns The file extension including the dot, or empty string if none
 *
 * @example
 * ```typescript
 * const ext = extname("file.txt");
 * console.log(ext); // ".txt"
 * ```
 *
 * @example
 * ```typescript
 * const ext = extname("archive.tar.gz");
 * console.log(ext); // ".gz" (only last extension)
 * ```
 *
 * @example
 * ```typescript
 * // No extension
 * const ext = extname("README");
 * console.log(ext); // ""
 * ```
 *
 * @example
 * ```typescript
 * // Hidden file (dot prefix is not an extension)
 * const ext = extname(".gitignore");
 * console.log(ext); // ""
 * ```
 */
export function extname(path: string): string {
  return ops.extname(path);
}

/**
 * Parses a path into its directory, basename, and extension components.
 *
 * This is a convenience function that combines `dirname()`, `basename()`, and
 * `extname()` in a single operation.
 *
 * @param path - The path to parse
 * @returns Object with `dir`, `base`, and `ext` properties
 *
 * @example
 * ```typescript
 * const parts = parts("/usr/local/bin/node");
 * console.log(parts.dir);  // "/usr/local/bin"
 * console.log(parts.base); // "node"
 * console.log(parts.ext);  // ""
 * ```
 *
 * @example
 * ```typescript
 * const parts = parts("./data/config.json");
 * console.log(parts.dir);  // "./data"
 * console.log(parts.base); // "config.json"
 * console.log(parts.ext);  // ".json"
 * ```
 *
 * @example
 * ```typescript
 * // Use to build modified paths
 * const original = "./images/photo.jpg";
 * const p = parts(original);
 * const thumbnail = join(p.dir, `thumb_${p.base}`);
 * console.log(thumbnail); // "./images/thumb_photo.jpg"
 * ```
 */
export function parts(path: string): PathParts {
  return ops.parts(path);
}
