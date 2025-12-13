// Type definitions for Forge host:* modules
// These types are for editor intellisense and TypeScript compilation

declare module "host:fs" {
  /** File statistics */
  export interface FileStat {
    is_file: boolean;
    is_dir: boolean;
    size: number;
    readonly: boolean;
  }

  /** Directory entry */
  export interface DirEntry {
    name: string;
    is_file: boolean;
    is_dir: boolean;
  }

  /** Options for mkdir */
  export interface MkdirOptions {
    recursive?: boolean;
  }

  /** Options for remove */
  export interface RemoveOptions {
    recursive?: boolean;
  }

  /** File event from watch */
  export interface FileEvent {
    kind: string;
    paths: string[];
  }

  /** Async file watcher */
  export interface FileWatcher {
    /** Get the next file event (blocks until event occurs) */
    next(): Promise<FileEvent | null>;
    /** Close the watcher and stop receiving events */
    close(): Promise<void>;
  }

  /** Read a text file */
  export function readTextFile(path: string): Promise<string>;

  /** Write a text file */
  export function writeTextFile(path: string, content: string): Promise<void>;

  /** Read a file as raw bytes */
  export function readBytes(path: string): Promise<Uint8Array>;

  /** Write raw bytes to a file */
  export function writeBytes(path: string, content: Uint8Array): Promise<void>;

  /** Get file stats */
  export function stat(path: string): Promise<FileStat>;

  /** Read directory contents */
  export function readDir(path: string): Promise<DirEntry[]>;

  /** Create a directory */
  export function mkdir(path: string, opts?: MkdirOptions): Promise<void>;

  /** Remove a file or directory */
  export function remove(path: string, opts?: RemoveOptions): Promise<void>;

  /** Rename/move a file or directory */
  export function rename(from: string, to: string): Promise<void>;

  /** Copy a file */
  export function copy(from: string, to: string): Promise<void>;

  /** Check if a path exists */
  export function exists(path: string): Promise<boolean>;

  /** Watch a path for changes */
  export function watch(path: string): Promise<FileWatcher>;
}

declare module "host:net" {
  /** Options for fetch requests */
  export interface FetchOptions {
    method?: string;
    headers?: Record<string, string>;
    body?: string;
    timeout_ms?: number;
  }

  /** Fetch response with text body */
  export interface FetchResponse {
    status: number;
    statusText: string;
    headers: Record<string, string>;
    body: string;
    url: string;
    ok: boolean;
  }

  /** Fetch response with binary body */
  export interface FetchBytesResponse {
    status: number;
    statusText: string;
    headers: Record<string, string>;
    body: Uint8Array;
    url: string;
    ok: boolean;
  }

  /** Fetch a URL and return response as text */
  export function fetch(url: string, opts?: FetchOptions): Promise<FetchResponse>;

  /** Fetch a URL and return response as raw bytes */
  export function fetchBytes(url: string, opts?: FetchOptions): Promise<FetchBytesResponse>;

  /** Convenience method to fetch and parse JSON */
  export function fetchJson<T = unknown>(url: string, opts?: FetchOptions): Promise<T>;

  /** POST JSON data to a URL */
  export function postJson(url: string, data: unknown, opts?: Omit<FetchOptions, "method" | "body">): Promise<FetchResponse>;
}

declare module "host:sys" {
  /** System information */
  export interface SystemInfo {
    os: string;
    arch: string;
    hostname: string | null;
    platform: string;
    cpu_count: number;
  }

  /** Notification options */
  export interface NotifyOptions {
    title: string;
    body?: string;
    subtitle?: string;
    sound?: boolean;
  }

  /** Clipboard interface */
  export interface Clipboard {
    /** Read text from the system clipboard */
    read(): Promise<string>;
    /** Write text to the system clipboard */
    write(text: string): Promise<void>;
  }

  /** Power/battery information */
  export interface PowerInfo {
    has_battery: boolean;
    batteries: BatteryInfo[];
    ac_connected: boolean;
  }

  /** Battery status information */
  export interface BatteryInfo {
    charge_percent: number;
    state: "charging" | "discharging" | "full" | "empty" | "unknown";
    time_to_full_secs?: number;
    time_to_empty_secs?: number;
    health_percent?: number;
    cycle_count?: number;
    temperature_celsius?: number;
  }

  /** Get system information */
  export function info(): SystemInfo;

  /** Get an environment variable */
  export function getEnv(key: string): string | null;

  /** Set an environment variable */
  export function setEnv(key: string, value: string): void;

  /** Get the current working directory */
  export function cwd(): string;

  /** Get the user's home directory */
  export function homeDir(): string | null;

  /** Get the system's temporary directory */
  export function tempDir(): string;

  /** Clipboard operations */
  export const clipboard: Clipboard;

  /** Show a system notification */
  export function notify(title: string, body?: string): Promise<void>;

  /** Show a system notification with extended options */
  export function notifyExt(opts: NotifyOptions): Promise<void>;

  /** Get power/battery information */
  export function powerInfo(): Promise<PowerInfo>;
}

declare module "host:ui" {
  // Window Types
  export interface OpenWindowOptions {
    url?: string;
    width?: number;
    height?: number;
    title?: string;
    resizable?: boolean;
    decorations?: boolean;
    transparent?: boolean;
    always_on_top?: boolean;
    visible?: boolean;
    channels?: string[];
  }

  export interface WindowEvent {
    windowId: string;
    channel: string;
    payload: unknown;
    eventType?: string;
  }

  export interface Window {
    readonly id: string;
    send(channel: string, payload?: unknown): Promise<void>;
    emit(channel: string, payload?: unknown): Promise<void>;
    events(): AsyncGenerator<WindowEvent, void, unknown>;
    on(channel: string, callback: (payload: unknown) => void): () => void;
    close(): Promise<boolean>;
    setTitle(title: string): Promise<void>;
  }

  // Dialog Types
  export interface FileFilter {
    name: string;
    extensions: string[];
  }

  export interface FileDialogOptions {
    title?: string;
    default_path?: string;
    filters?: FileFilter[];
    multiple?: boolean;
    directory?: boolean;
  }

  export interface MessageDialogOptions {
    title?: string;
    message: string;
    kind?: "info" | "warning" | "error";
    buttons?: string[];
  }

  // Menu Types
  export interface MenuItem {
    id?: string;
    label: string;
    accelerator?: string;
    enabled?: boolean;
    checked?: boolean;
    submenu?: MenuItem[];
    item_type?: "normal" | "checkbox" | "separator";
  }

  export interface MenuEvent {
    menuId: string;
    itemId: string;
    label: string;
  }

  // Tray Types
  export interface TrayOptions {
    icon?: string;
    tooltip?: string;
    menu?: MenuItem[];
  }

  // Window Functions
  export function openWindow(options?: OpenWindowOptions): Promise<Window>;
  export function closeWindow(windowId: string): Promise<boolean>;
  export function setWindowTitle(windowId: string, title: string): void;
  export function sendToWindow(windowId: string, channel: string, payload?: unknown): Promise<void>;
  export function windowEvents(): AsyncGenerator<WindowEvent, void, unknown>;

  // Dialog Functions
  export function showOpenDialog(options?: FileDialogOptions): Promise<string[] | null>;
  export function showSaveDialog(options?: FileDialogOptions): Promise<string | null>;
  export function showMessageDialog(options: MessageDialogOptions): Promise<number>;

  // Menu Functions
  export function setAppMenu(items: MenuItem[]): Promise<boolean>;
  export function showContextMenu(windowId: string | null, items: MenuItem[]): Promise<string | null>;
  export function menuEvents(): AsyncGenerator<MenuEvent, void, unknown>;
  export function onMenu(callback: (event: MenuEvent) => void): () => void;

  // Tray Functions
  export function createTray(options?: TrayOptions): Promise<string>;
  export function updateTray(trayId: string, options: TrayOptions): Promise<boolean>;
  export function destroyTray(trayId: string): Promise<boolean>;
}

declare module "host:process" {
  /** Spawn options for child processes */
  export interface SpawnOptions {
    /** Arguments to pass to the command */
    args?: string[];
    /** Environment variables to set */
    env?: Record<string, string>;
    /** Working directory for the process */
    cwd?: string;
    /** How to handle stdout: "piped", "inherit", or "null" */
    stdout?: "piped" | "inherit" | "null";
    /** How to handle stderr: "piped", "inherit", or "null" */
    stderr?: "piped" | "inherit" | "null";
    /** How to handle stdin: "piped", "inherit", or "null" */
    stdin?: "piped" | "inherit" | "null";
  }

  /** Process handle with basic info */
  export interface ProcessHandle {
    /** Internal handle ID */
    id: string;
    /** Operating system process ID */
    pid: number;
  }

  /** Process status information */
  export interface ProcessStatus {
    /** Whether the process is still running */
    running: boolean;
    /** Exit code if process has exited */
    exitCode?: number;
    /** Signal that killed the process (Unix only) */
    signal?: string;
  }

  /** Output from reading stdout/stderr */
  export interface ProcessOutput {
    /** Line of output data, or null if EOF */
    data: string | null;
    /** Whether end of stream has been reached */
    eof: boolean;
  }

  /** Child process handle with full control */
  export interface ChildProcess extends ProcessHandle {
    /** Kill the process with an optional signal (default: SIGTERM) */
    kill(signal?: string): Promise<void>;
    /** Wait for the process to exit and return the exit code */
    wait(): Promise<number>;
    /** Get the current process status */
    status(): Promise<ProcessStatus>;
    /** Write to stdin (requires stdin: "piped") */
    writeStdin(data: string): Promise<void>;
    /** Read a line from stdout (requires stdout: "piped") */
    readStdout(): Promise<ProcessOutput>;
    /** Read a line from stderr (requires stderr: "piped") */
    readStderr(): Promise<ProcessOutput>;
    /** Async iterator for stdout lines */
    stdout: AsyncIterable<string>;
    /** Async iterator for stderr lines */
    stderr: AsyncIterable<string>;
  }

  /** Spawn a child process */
  export function spawn(binary: string, opts?: SpawnOptions): Promise<ChildProcess>;

  /** Kill a process by handle ID */
  export function kill(handle: string, signal?: string): Promise<void>;

  /** Wait for a process to exit */
  export function wait(handle: string): Promise<number>;

  /** Get the status of a process */
  export function status(handle: string): Promise<ProcessStatus>;

  /** Write data to a process's stdin */
  export function writeStdin(handle: string, data: string): Promise<void>;

  /** Read a line from a process's stdout */
  export function readStdout(handle: string): Promise<ProcessOutput>;

  /** Read a line from a process's stderr */
  export function readStderr(handle: string): Promise<ProcessOutput>;
}

declare module "host:wasm" {
  /** WASM value type */
  export type WasmValueType = "i32" | "i64" | "f32" | "f64";

  /** WASM value with explicit type */
  export interface WasmValue {
    type: WasmValueType;
    value: number;
  }

  /** Export information from a WASM module */
  export interface ExportInfo {
    name: string;
    kind: "function" | "memory" | "table" | "global";
    params?: WasmValueType[];
    results?: WasmValueType[];
  }

  /** WASI configuration for instantiating WASM modules */
  export interface WasiConfig {
    /** Guest path to host path mappings for filesystem access */
    preopens?: Record<string, string>;
    /** Environment variables to expose to the WASM module */
    env?: Record<string, string>;
    /** Command-line arguments for the WASM module */
    args?: string[];
    /** Whether to inherit stdin from the host process */
    inheritStdin?: boolean;
    /** Whether to inherit stdout from the host process */
    inheritStdout?: boolean;
    /** Whether to inherit stderr from the host process */
    inheritStderr?: boolean;
  }

  /** Memory access interface for a WASM instance */
  export interface WasmMemory {
    /** Read bytes from WASM memory */
    read(offset: number, length: number): Promise<Uint8Array>;
    /** Write bytes to WASM memory */
    write(offset: number, data: Uint8Array): Promise<void>;
    /** Get the current memory size in pages (1 page = 64KB) */
    size(): Promise<number>;
    /** Grow memory by the specified number of pages */
    grow(pages: number): Promise<number>;
  }

  /** WASM instance handle with methods for calling functions and accessing memory */
  export interface WasmInstance {
    /** The unique instance ID */
    readonly id: string;
    /** The module ID this instance was created from */
    readonly moduleId: string;
    /** Call an exported WASM function */
    call(name: string, ...args: (number | bigint | WasmValue)[]): Promise<number[]>;
    /** Get list of all exports from this instance */
    getExports(): Promise<ExportInfo[]>;
    /** Memory access interface */
    memory: WasmMemory;
    /** Drop this instance and free resources */
    drop(): Promise<void>;
  }

  /** Compile WASM bytes to a module */
  export function compile(bytes: Uint8Array): Promise<string>;

  /** Compile WASM from a file path */
  export function compileFile(path: string): Promise<string>;

  /** Drop a compiled module and free its resources */
  export function dropModule(moduleId: string): Promise<void>;

  /** Instantiate a compiled WASM module */
  export function instantiate(moduleId: string, wasiConfig?: WasiConfig): Promise<WasmInstance>;

  /** Low-level function call without instance wrapper */
  export function call(instanceId: string, funcName: string, args: WasmValue[]): Promise<WasmValue[]>;

  /** Low-level exports query */
  export function getExports(instanceId: string): Promise<ExportInfo[]>;

  /** Helper functions to create typed WASM values */
  export const types: {
    /** Create an i32 (32-bit integer) value */
    i32(value: number): WasmValue;
    /** Create an i64 (64-bit integer) value */
    i64(value: number | bigint): WasmValue;
    /** Create an f32 (32-bit float) value */
    f32(value: number): WasmValue;
    /** Create an f64 (64-bit float) value */
    f64(value: number): WasmValue;
  };

  /** Low-level memory operations namespace */
  export const memory: {
    read(instanceId: string, offset: number, length: number): Promise<Uint8Array>;
    write(instanceId: string, offset: number, data: Uint8Array): Promise<void>;
    size(instanceId: string): Promise<number>;
    grow(instanceId: string, pages: number): Promise<number>;
  };
}
