// runtime:shell module - TypeScript wrapper for Deno core ops

// Deno.core type declaration
declare const Deno: {
  core: {
    ops: {
      op_shell_open_external(url: string): Promise<void>;
      op_shell_open_path(path: string): Promise<void>;
      op_shell_show_item_in_folder(path: string): Promise<void>;
      op_shell_move_to_trash(path: string): Promise<void>;
      op_shell_beep(): void;
      op_shell_get_file_icon(path: string, size: number | null): Promise<FileIconResult>;
      op_shell_get_default_app(pathOrExtension: string): Promise<DefaultAppResult>;
    };
  };
};

export interface FileIconResult {
  data: string;
  width: number;
  height: number;
}

export interface DefaultAppResult {
  name: string | null;
  path: string | null;
  identifier: string | null;
}

const core = Deno.core;

/**
 * Information about a file's icon
 */
export interface FileIcon {
  /** Base64-encoded PNG data of the icon */
  data: string;
  /** Width in pixels */
  width: number;
  /** Height in pixels */
  height: number;
}

/**
 * Information about a default application
 */
export interface DefaultAppInfo {
  /** Application name */
  name: string | null;
  /** Application path */
  path: string | null;
  /** Bundle identifier (macOS) or program ID (Windows) */
  identifier: string | null;
}

/**
 * Open a URL in the default browser.
 * @param url - The URL to open (must start with http://, https://, or mailto:)
 */
export async function openExternal(url: string): Promise<void> {
  return await core.ops.op_shell_open_external(url);
}

/**
 * Open a file or folder with the default application.
 * @param path - Path to the file or folder to open
 */
export async function openPath(path: string): Promise<void> {
  return await core.ops.op_shell_open_path(path);
}

/**
 * Show a file in its containing folder (Finder on macOS, Explorer on Windows).
 * @param path - Path to the file to reveal
 */
export async function showItemInFolder(path: string): Promise<void> {
  return await core.ops.op_shell_show_item_in_folder(path);
}

/**
 * Move a file or folder to the trash/recycle bin.
 * @param path - Path to the file or folder to trash
 */
export async function moveToTrash(path: string): Promise<void> {
  return await core.ops.op_shell_move_to_trash(path);
}

/**
 * Play the system beep sound.
 */
export function beep(): void {
  return core.ops.op_shell_beep();
}

/**
 * Get the icon for a file type.
 * @param path - Path to the file or extension (e.g., ".txt")
 * @param size - Icon size in pixels (default: 32)
 * @returns File icon information with base64-encoded PNG data
 */
export async function getFileIcon(
  path: string,
  size: number = 32
): Promise<FileIcon> {
  const result = await core.ops.op_shell_get_file_icon(path, size);
  return {
    data: result.data,
    width: result.width,
    height: result.height,
  };
}

/**
 * Get the default application for a file type.
 * @param pathOrExtension - File path or extension (e.g., ".txt" or "/path/to/file.txt")
 * @returns Information about the default application
 */
export async function getDefaultApp(
  pathOrExtension: string
): Promise<DefaultAppInfo> {
  const result = await core.ops.op_shell_get_default_app(pathOrExtension);
  return {
    name: result.name,
    path: result.path,
    identifier: result.identifier,
  };
}

// Convenience aliases
export { openExternal as open };
export { moveToTrash as trash };