/**
 * @module runtime:shell
 *
 * Shell integration and command execution for Forge applications.
 *
 * This module provides two main categories of functionality:
 * 1. **System Integration** - Open URLs/files, interact with file manager, system sounds
 * 2. **Shell Execution** - Execute shell commands with full syntax support
 *
 * ## Features
 *
 * ### System Integration
 * - Open URLs in default browser
 * - Open files/folders with default applications
 * - Reveal files in file manager (Finder/Explorer)
 * - Move files to trash/recycle bin
 * - Play system beep sound
 * - Get file icons (limited platform support)
 * - Query default applications for file types
 *
 * ### Shell Execution
 * - Execute shell commands with full syntax support
 * - Pipes, redirections, variables, globs
 * - Environment variable management
 * - Working directory control
 * - Command path resolution (which)
 * - Process management and timeouts
 *
 * ## Error Codes
 *
 * Shell operations may throw errors with these codes:
 * - `8200` - Failed to open external URL
 * - `8201` - Failed to open path
 * - `8202` - Failed to show item in folder
 * - `8203` - Failed to move to trash
 * - `8204` - Failed to play beep
 * - `8205` - Failed to get file icon
 * - `8206` - Failed to get default app
 * - `8207` - Invalid path provided
 * - `8208` - Permission denied
 * - `8209` - Operation not supported on this platform
 * - `8210` - Shell command parse error
 * - `8211` - Shell command execution failed
 * - `8212` - Shell command timed out
 * - `8213` - Shell process was killed
 * - `8214` - Invalid shell handle
 *
 * ## Permissions
 *
 * Shell operations require permissions in `manifest.app.toml`:
 *
 * ```toml
 * [permissions.shell]
 * execute = true              # Allow shell command execution
 * open_external = true        # Allow opening URLs/files
 * ```
 *
 * ## Platform Support
 *
 * - **macOS**: Full support for all operations
 * - **Windows**: Full support with platform-specific implementations
 * - **Linux**: Full support with freedesktop.org standards
 *
 * Some operations (like getFileIcon) may have limited platform support.
 */

// Deno.core type declaration
declare const Deno: {
  core: {
    ops: {
      // System integration ops
      op_shell_open_external(url: string): Promise<void>;
      op_shell_open_path(path: string): Promise<void>;
      op_shell_show_item_in_folder(path: string): Promise<void>;
      op_shell_move_to_trash(path: string): Promise<void>;
      op_shell_beep(): void;
      op_shell_get_file_icon(path: string, size: number | null): Promise<FileIconResult>;
      op_shell_get_default_app(pathOrExtension: string): Promise<DefaultAppResult>;
      // Shell execution ops
      op_shell_execute(command: string, options: ExecuteOptionsInternal | null): Promise<ExecuteOutputResult>;
      op_shell_kill(handleId: number, signal: string | null): Promise<void>;
      op_shell_cwd(): string;
      op_shell_set_cwd(path: string): void;
      op_shell_get_env(name: string): string | null;
      op_shell_set_env(name: string, value: string): void;
      op_shell_unset_env(name: string): void;
      op_shell_get_all_env(): Record<string, string>;
      op_shell_which(command: string): string | null;
    };
  };
};

// Internal types for ops
interface FileIconResult {
  data: string;
  width: number;
  height: number;
}

interface DefaultAppResult {
  name: string | null;
  path: string | null;
  identifier: string | null;
}

interface ExecuteOptionsInternal {
  cwd?: string;
  env?: Record<string, string>;
  timeout_ms?: number;
  stdin?: string;
}

interface ExecuteOutputResult {
  code: number;
  stdout: string;
  stderr: string;
}

const core = Deno.core;

// ============================================================================
// System Integration Types
// ============================================================================

/**
 * Information about a file's icon.
 *
 * Contains base64-encoded PNG data and dimensions. Icon retrieval
 * may not be supported on all platforms.
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
 * Information about a default application for a file type.
 *
 * Platform-specific fields:
 * - **macOS**: `path` and `name` are usually available, `identifier` is bundle ID
 * - **Windows**: `identifier` is the ProgID from registry
 * - **Linux**: `identifier` is the .desktop file name
 */
export interface DefaultAppInfo {
  /** Application name (e.g., "TextEdit", "notepad") */
  name: string | null;
  /** Full path to application (e.g., "/Applications/TextEdit.app") */
  path: string | null;
  /** Bundle identifier (macOS), ProgID (Windows), or .desktop file (Linux) */
  identifier: string | null;
}

// ============================================================================
// Shell Execution Types
// ============================================================================

/**
 * Options for shell command execution.
 *
 * All fields are optional. Commands execute in the current working directory
 * with inherited environment variables by default.
 */
export interface ExecuteOptions {
  /** Working directory for the command. Defaults to current directory. */
  cwd?: string;
  /** Environment variables to set or override. Merged with inherited environment. */
  env?: Record<string, string>;
  /** Timeout in milliseconds. Zero or undefined means no timeout. */
  timeout?: number;
  /** Input to send to stdin. If not provided, stdin is inherited. */
  stdin?: string;
}

/**
 * Result of shell command execution.
 *
 * Contains exit code and captured output from both stdout and stderr.
 * Non-zero exit codes indicate command failure but are not thrown as errors.
 */
export interface ExecuteOutput {
  /** Exit code of the command (0 = success, non-zero = failure) */
  code: number;
  /** Standard output captured from the command */
  stdout: string;
  /** Standard error captured from the command */
  stderr: string;
}

/**
 * Handle for a spawned process.
 *
 * Used to manage and kill background processes. The handle becomes
 * invalid after the process exits.
 */
export interface SpawnHandle {
  /** Unique process ID for this spawned process */
  id: number;
}

// ============================================================================
// System Integration Functions
// ============================================================================

/**
 * Opens a URL in the default web browser.
 *
 * Supports HTTP, HTTPS, and mailto URLs. The URL is validated before
 * attempting to open. Works on all platforms using system defaults.
 *
 * @param url - The URL to open (must start with http://, https://, or mailto:)
 *
 * @throws Error [8200] if opening the URL fails
 * @throws Error [8207] if URL format is invalid
 * @throws Error [8208] if permission denied
 *
 * @example
 * ```typescript
 * // Open a website
 * await openExternal("https://github.com");
 * ```
 *
 * @example
 * ```typescript
 * // Open email client
 * await openExternal("mailto:support@example.com?subject=Help");
 * ```
 *
 * @example
 * ```typescript
 * // Handle errors
 * try {
 *   await openExternal("https://example.com");
 * } catch (err) {
 *   console.error("Failed to open URL:", err);
 * }
 * ```
 */
export async function openExternal(url: string): Promise<void> {
  return await core.ops.op_shell_open_external(url);
}

/**
 * Opens a file or folder with its default application.
 *
 * Uses the system's default application for the file type. For folders,
 * opens in the file manager (Finder on macOS, Explorer on Windows).
 *
 * @param path - Path to the file or folder to open
 *
 * @throws Error [8201] if opening the path fails
 * @throws Error [8207] if path doesn't exist
 * @throws Error [8208] if permission denied
 *
 * @example
 * ```typescript
 * // Open a text file in default editor
 * await openPath("./README.md");
 * ```
 *
 * @example
 * ```typescript
 * // Open a folder in file manager
 * await openPath("./documents");
 * ```
 *
 * @example
 * ```typescript
 * // Open an image in default viewer
 * await openPath("/Users/me/Pictures/photo.jpg");
 * ```
 */
export async function openPath(path: string): Promise<void> {
  return await core.ops.op_shell_open_path(path);
}

/**
 * Reveals a file in its containing folder.
 *
 * Opens the file manager (Finder on macOS, Explorer on Windows, file manager on Linux)
 * and selects/highlights the specified file. Useful for "Show in Folder" functionality.
 *
 * Platform behavior:
 * - **macOS**: Uses `open -R` to reveal in Finder
 * - **Windows**: Uses `explorer /select,` to select in Explorer
 * - **Linux**: Attempts dbus-send, falls back to opening parent folder
 *
 * @param path - Path to the file to reveal
 *
 * @throws Error [8202] if showing the item fails
 * @throws Error [8207] if path doesn't exist
 * @throws Error [8208] if permission denied
 *
 * @example
 * ```typescript
 * // Show a downloaded file in Downloads folder
 * await showItemInFolder("~/Downloads/document.pdf");
 * ```
 *
 * @example
 * ```typescript
 * // Reveal a generated file
 * const outputPath = "./build/app.exe";
 * await showItemInFolder(outputPath);
 * ```
 */
export async function showItemInFolder(path: string): Promise<void> {
  return await core.ops.op_shell_show_item_in_folder(path);
}

/**
 * Moves a file or folder to the trash/recycle bin.
 *
 * Safely deletes items by moving them to the system trash instead of
 * permanent deletion. Items can be recovered from trash by the user.
 *
 * Platform behavior:
 * - **macOS**: Moves to Trash
 * - **Windows**: Moves to Recycle Bin
 * - **Linux**: Moves to freedesktop.org Trash
 *
 * @param path - Path to the file or folder to trash
 *
 * @throws Error [8203] if moving to trash fails
 * @throws Error [8207] if path doesn't exist
 * @throws Error [8208] if permission denied
 *
 * @example
 * ```typescript
 * // Delete a temporary file safely
 * await moveToTrash("./temp/cache.tmp");
 * ```
 *
 * @example
 * ```typescript
 * // Delete with confirmation
 * const confirmDelete = confirm("Move to trash?");
 * if (confirmDelete) {
 *   await moveToTrash("./old-data.db");
 * }
 * ```
 *
 * @example
 * ```typescript
 * // Delete multiple files
 * const files = ["file1.txt", "file2.txt", "file3.txt"];
 * for (const file of files) {
 *   await moveToTrash(file);
 * }
 * ```
 */
export async function moveToTrash(path: string): Promise<void> {
  return await core.ops.op_shell_move_to_trash(path);
}

/**
 * Plays the system beep sound.
 *
 * Triggers the default system alert sound. Useful for notifications
 * or drawing user attention. Respects user's system volume settings.
 *
 * Platform behavior:
 * - **macOS**: Uses AppleScript `beep` command
 * - **Windows**: Uses PowerShell console beep (800Hz, 200ms)
 * - **Linux**: Attempts to play freedesktop bell sound, falls back to console bell
 *
 * @throws Error [8204] if playing beep fails
 *
 * @example
 * ```typescript
 * // Alert user when task completes
 * await longRunningTask();
 * beep();
 * ```
 *
 * @example
 * ```typescript
 * // Beep on error
 * try {
 *   await riskyOperation();
 * } catch (err) {
 *   beep();
 *   console.error("Operation failed:", err);
 * }
 * ```
 */
export function beep(): void {
  return core.ops.op_shell_beep();
}

/**
 * Retrieves the icon for a file type.
 *
 * Returns base64-encoded PNG data for the system icon associated with
 * the file type. Icon size can be specified in pixels.
 *
 * **Note**: This feature requires platform-specific native bindings and
 * may throw "not supported" errors on some platforms.
 *
 * @param path - Path to file or file extension (e.g., ".txt")
 * @param size - Icon size in pixels (default: 32)
 * @returns File icon with base64 PNG data and dimensions
 *
 * @throws Error [8205] if getting icon fails
 * @throws Error [8208] if permission denied
 * @throws Error [8209] if not supported on current platform
 *
 * @example
 * ```typescript
 * // Get icon for a file type
 * try {
 *   const icon = await getFileIcon(".pdf", 64);
 *   const img = document.createElement("img");
 *   img.src = `data:image/png;base64,${icon.data}`;
 *   document.body.appendChild(img);
 * } catch (err) {
 *   console.log("Icon retrieval not supported:", err);
 * }
 * ```
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
 * Queries the default application for a file type.
 *
 * Returns information about which application the system will use to
 * open files of the given type. Can accept either a file path or
 * a file extension.
 *
 * Returned fields vary by platform:
 * - **macOS**: Returns app path and name, bundle identifier
 * - **Windows**: Returns ProgID from registry
 * - **Linux**: Returns .desktop file name via xdg-mime
 *
 * @param pathOrExtension - File path or extension (e.g., ".txt" or "/path/to/file.txt")
 * @returns Information about the default application (fields may be null)
 *
 * @throws Error [8206] if query fails
 * @throws Error [8208] if permission denied
 *
 * @example
 * ```typescript
 * // Query default app for text files
 * const app = await getDefaultApp(".txt");
 * console.log(`Default text editor: ${app.name}`);
 * console.log(`Path: ${app.path}`);
 * ```
 *
 * @example
 * ```typescript
 * // Query default app for a specific file
 * const app = await getDefaultApp("./document.pdf");
 * if (app.name) {
 *   console.log(`Will open with: ${app.name}`);
 * } else {
 *   console.log("No default app configured");
 * }
 * ```
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

// ============================================================================
// Shell Execution Functions
// ============================================================================

/**
 * Executes a shell command and waits for completion.
 *
 * Provides full shell syntax support including pipes, redirections,
 * variables, quoting, and globs. Commands execute in a cross-platform
 * shell environment.
 *
 * **Supported Shell Syntax:**
 * - Pipes: `cmd1 | cmd2 | cmd3`
 * - Logical operators: `cmd1 && cmd2`, `cmd1 || cmd2`
 * - Sequences: `cmd1; cmd2; cmd3`
 * - Redirections: `cmd > file`, `cmd 2>&1`, `cmd < input`
 * - Variables: `$VAR`, `${VAR}`, `$PATH`
 * - Quoting: `'literal'`, `"expansion $VAR"`, `` `backticks` ``
 * - Globs: `*.ts`, `**\/*.js`, `file[0-9].txt`
 * - Background: `cmd &`
 *
 * **Built-in Commands:**
 * echo, cd, pwd, ls, cat, cp, mv, rm, mkdir, export, unset, exit, sleep, which
 *
 * @param command - Shell command string to execute
 * @param options - Execution options (cwd, env, timeout, stdin)
 * @returns Execution result with exit code, stdout, and stderr
 *
 * @throws Error [8210] if command syntax is invalid
 * @throws Error [8211] if command execution fails
 * @throws Error [8212] if command times out
 * @throws Error [8208] if permission denied
 *
 * @example
 * ```typescript
 * // Simple command
 * const result = await execute("echo hello");
 * console.log(result.stdout); // "hello\n"
 * console.log(result.code);   // 0
 * ```
 *
 * @example
 * ```typescript
 * // With pipes and redirection
 * const result = await execute("ls -la | grep .ts | wc -l");
 * console.log(`TypeScript files: ${result.stdout.trim()}`);
 * ```
 *
 * @example
 * ```typescript
 * // With options
 * const result = await execute("npm test", {
 *   cwd: "/path/to/project",
 *   timeout: 30000,  // 30 second timeout
 *   env: {
 *     NODE_ENV: "test",
 *     CI: "true"
 *   }
 * });
 * ```
 *
 * @example
 * ```typescript
 * // With stdin input
 * const result = await execute("grep error", {
 *   stdin: "line 1\nerror on line 2\nline 3"
 * });
 * console.log(result.stdout); // "error on line 2\n"
 * ```
 *
 * @example
 * ```typescript
 * // Handle errors and exit codes
 * const result = await execute("command-that-might-fail");
 * if (result.code !== 0) {
 *   console.error("Command failed with code:", result.code);
 *   console.error("Error output:", result.stderr);
 * }
 * ```
 *
 * @example
 * ```typescript
 * // Logical operators
 * const result = await execute("mkdir temp && cd temp && touch file.txt");
 * // Only creates file if mkdir and cd both succeed
 * ```
 */
export async function execute(
  command: string,
  options?: ExecuteOptions
): Promise<ExecuteOutput> {
  const opts: ExecuteOptionsInternal | null = options ? {
    cwd: options.cwd,
    env: options.env,
    timeout_ms: options.timeout,
    stdin: options.stdin,
  } : null;

  const result = await core.ops.op_shell_execute(command, opts);
  return {
    code: result.code,
    stdout: result.stdout,
    stderr: result.stderr,
  };
}

/**
 * Kills a spawned process.
 *
 * Sends a signal to terminate a background process. Supports various
 * signal types for graceful or forceful termination.
 *
 * **Available Signals:**
 * - `SIGTERM` (default) - Graceful termination, allows cleanup
 * - `SIGKILL` or `9` - Forceful termination, cannot be caught
 * - `SIGINT` or `2` - Interrupt (like Ctrl+C)
 * - `SIGQUIT` or `3` - Quit with core dump
 *
 * @param handle - Process handle from spawn()
 * @param signal - Signal to send (default: "SIGTERM")
 *
 * @throws Error [8214] if handle is invalid or process already exited
 * @throws Error [8208] if permission denied
 *
 * @example
 * ```typescript
 * // Graceful termination
 * const handle = await spawn("long-running-server");
 * // ... later ...
 * await kill(handle); // Sends SIGTERM
 * ```
 *
 * @example
 * ```typescript
 * // Force kill if not responding
 * await kill(handle, "SIGKILL");
 * ```
 *
 * @example
 * ```typescript
 * // Send interrupt signal
 * await kill(handle, "SIGINT");
 * ```
 */
export async function kill(handle: SpawnHandle, signal?: string): Promise<void> {
  return await core.ops.op_shell_kill(handle.id, signal ?? null);
}

/**
 * Gets the current working directory.
 *
 * Returns the absolute path of the current working directory for shell
 * operations. This is the directory where relative paths are resolved from.
 *
 * @returns Current working directory path
 *
 * @example
 * ```typescript
 * const current = cwd();
 * console.log(`Working directory: ${current}`);
 * ```
 *
 * @example
 * ```typescript
 * // Save and restore working directory
 * const original = cwd();
 * chdir("/tmp");
 * // ... do work ...
 * chdir(original);
 * ```
 */
export function cwd(): string {
  return core.ops.op_shell_cwd();
}

/**
 * Changes the current working directory.
 *
 * Sets the working directory for subsequent shell operations. Relative
 * paths in future commands will be resolved from this directory.
 *
 * @param path - Path to change to (can be relative or absolute)
 *
 * @throws Error [8211] if directory doesn't exist or can't be accessed
 *
 * @example
 * ```typescript
 * // Change to project directory
 * chdir("/path/to/project");
 * const result = await execute("npm install");
 * ```
 *
 * @example
 * ```typescript
 * // Relative path change
 * chdir("../other-project");
 * ```
 *
 * @example
 * ```typescript
 * // Change and execute
 * chdir("./build");
 * const result = await execute("ls -la");
 * ```
 */
export function chdir(path: string): void {
  return core.ops.op_shell_set_cwd(path);
}

/**
 * Gets an environment variable value.
 *
 * Retrieves the value of an environment variable by name. Returns null
 * if the variable is not set.
 *
 * @param name - Environment variable name (case-sensitive)
 * @returns Variable value or null if not set
 *
 * @example
 * ```typescript
 * const home = getEnv("HOME");
 * console.log(`Home directory: ${home}`);
 * ```
 *
 * @example
 * ```typescript
 * const nodeEnv = getEnv("NODE_ENV") ?? "development";
 * console.log(`Environment: ${nodeEnv}`);
 * ```
 *
 * @example
 * ```typescript
 * // Check if variable is set
 * if (getEnv("DEBUG")) {
 *   console.log("Debug mode enabled");
 * }
 * ```
 */
export function getEnv(name: string): string | null {
  return core.ops.op_shell_get_env(name);
}

/**
 * Sets an environment variable.
 *
 * Sets or updates an environment variable for the current process and
 * child processes. Changes affect future shell command executions.
 *
 * @param name - Environment variable name
 * @param value - Variable value to set
 *
 * @example
 * ```typescript
 * // Set environment variable
 * setEnv("NODE_ENV", "production");
 * ```
 *
 * @example
 * ```typescript
 * // Configure for command execution
 * setEnv("RUST_LOG", "debug");
 * const result = await execute("cargo run");
 * ```
 *
 * @example
 * ```typescript
 * // Set API key
 * setEnv("API_KEY", "secret-key-123");
 * const result = await execute("./deploy.sh");
 * ```
 */
export function setEnv(name: string, value: string): void {
  return core.ops.op_shell_set_env(name, value);
}

/**
 * Removes an environment variable.
 *
 * Unsets an environment variable, removing it from the environment.
 * Subsequent getEnv() calls will return null.
 *
 * @param name - Environment variable name to remove
 *
 * @example
 * ```typescript
 * // Remove sensitive variable
 * setEnv("SECRET_KEY", "temp-value");
 * // ... use it ...
 * unsetEnv("SECRET_KEY");
 * ```
 *
 * @example
 * ```typescript
 * // Clear debug flag
 * unsetEnv("DEBUG");
 * ```
 */
export function unsetEnv(name: string): void {
  return core.ops.op_shell_unset_env(name);
}

/**
 * Gets all environment variables.
 *
 * Returns a record containing all environment variables as key-value pairs.
 * Useful for inspecting the full environment or passing to child processes.
 *
 * @returns Object with all environment variables
 *
 * @example
 * ```typescript
 * const env = getAllEnv();
 * console.log(`PATH: ${env.PATH}`);
 * console.log(`Total variables: ${Object.keys(env).length}`);
 * ```
 *
 * @example
 * ```typescript
 * // Pass modified environment to command
 * const env = getAllEnv();
 * const result = await execute("command", {
 *   env: { ...env, CUSTOM_VAR: "value" }
 * });
 * ```
 *
 * @example
 * ```typescript
 * // List all variables
 * const env = getAllEnv();
 * for (const [key, value] of Object.entries(env)) {
 *   console.log(`${key}=${value}`);
 * }
 * ```
 */
export function getAllEnv(): Record<string, string> {
  return core.ops.op_shell_get_all_env();
}

/**
 * Finds the full path to an executable command.
 *
 * Searches the PATH environment variable for the given command and
 * returns its full path. Returns null if the command is not found.
 *
 * Equivalent to the Unix `which` command.
 *
 * @param command - Command name to find (e.g., "node", "git", "cargo")
 * @returns Full path to the executable, or null if not found
 *
 * @example
 * ```typescript
 * const nodePath = which("node");
 * console.log(`Node.js is at: ${nodePath}`);
 * ```
 *
 * @example
 * ```typescript
 * // Check if command exists
 * if (which("git")) {
 *   console.log("Git is installed");
 *   await execute("git --version");
 * } else {
 *   console.log("Git not found in PATH");
 * }
 * ```
 *
 * @example
 * ```typescript
 * // Verify tool availability before use
 * const tools = ["node", "npm", "git", "cargo"];
 * const missing = tools.filter(tool => !which(tool));
 * if (missing.length > 0) {
 *   console.error(`Missing tools: ${missing.join(", ")}`);
 * }
 * ```
 */
export function which(command: string): string | null {
  return core.ops.op_shell_which(command);
}

// ============================================================================
// Convenience Aliases
// ============================================================================

/**
 * Alias for {@link openExternal}.
 * Opens a URL in the default browser.
 */
export { openExternal as open };

/**
 * Alias for {@link moveToTrash}.
 * Moves a file or folder to the trash/recycle bin.
 */
export { moveToTrash as trash };

/**
 * Alias for {@link execute}.
 * Executes a shell command and waits for completion.
 */
export { execute as exec };

/**
 * Alias for {@link execute}.
 * Executes a shell command and waits for completion.
 */
export { execute as run };
