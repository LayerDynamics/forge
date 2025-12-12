// host:sys module - Deno API for system operations
// This is the single source of truth for the host:sys SDK

// Type definitions
export interface SystemInfo {
  os: string;
  arch: string;
  hostname: string | null;
  platform: string;
  cpu_count: number;
}

export interface NotifyOptions {
  title: string;
  body?: string;
  subtitle?: string;
  sound?: boolean;
}

export interface Clipboard {
  /** Read text from the system clipboard */
  read(): Promise<string>;
  /** Write text to the system clipboard */
  write(text: string): Promise<void>;
}

export interface PowerInfo {
  has_battery: boolean;
  batteries: BatteryInfo[];
  ac_connected: boolean;
}

export interface BatteryInfo {
  charge_percent: number;
  state: "charging" | "discharging" | "full" | "empty" | "unknown";
  time_to_full_secs?: number;
  time_to_empty_secs?: number;
  health_percent?: number;
  cycle_count?: number;
  temperature_celsius?: number;
}

// Deno.core.ops type declaration
declare const Deno: {
  core: {
    ops: {
      op_sys_info(): SystemInfo;
      op_sys_env_get(key: string): string | null;
      op_sys_env_set(key: string, value: string): void;
      op_sys_cwd(): string;
      op_sys_home_dir(): string | null;
      op_sys_temp_dir(): string;
      op_sys_clipboard_read(): Promise<string>;
      op_sys_clipboard_write(text: string): Promise<void>;
      op_sys_notify(title: string, body: string): Promise<void>;
      op_sys_notify_ext(opts: NotifyOptions): Promise<void>;
      op_sys_power_info(): Promise<PowerInfo>;
    };
  };
};

/**
 * Get system information (OS, architecture, hostname, CPU count).
 *
 * @returns System information object
 *
 * @example
 * ```ts
 * import { info } from "host:sys";
 *
 * const sysInfo = info();
 * console.log(`Running on ${sysInfo.os} (${sysInfo.arch})`);
 * console.log(`CPU cores: ${sysInfo.cpu_count}`);
 * ```
 */
export function info(): SystemInfo {
  const result = Deno.core.ops.op_sys_info();
  // Normalize snake_case to camelCase
  return {
    ...result,
    cpuCount: (result as any).cpu_count,
  } as unknown as SystemInfo;
}

/**
 * Get an environment variable value.
 *
 * @param key - The environment variable name
 * @returns The value or null if not set
 *
 * @example
 * ```ts
 * import { getEnv } from "host:sys";
 *
 * const home = getEnv("HOME");
 * ```
 */
export function getEnv(key: string): string | null {
  return Deno.core.ops.op_sys_env_get(key);
}

/**
 * Set an environment variable.
 *
 * @param key - The environment variable name
 * @param value - The value to set
 *
 * @example
 * ```ts
 * import { setEnv } from "host:sys";
 *
 * setEnv("MY_VAR", "my_value");
 * ```
 */
export function setEnv(key: string, value: string): void {
  Deno.core.ops.op_sys_env_set(key, value);
}

/**
 * Get the current working directory.
 *
 * @returns The current working directory path
 */
export function cwd(): string {
  return Deno.core.ops.op_sys_cwd();
}

/**
 * Get the user's home directory.
 *
 * @returns The home directory path or null if not available
 */
export function homeDir(): string | null {
  return Deno.core.ops.op_sys_home_dir();
}

/**
 * Get the system's temporary directory.
 *
 * @returns The temp directory path
 */
export function tempDir(): string {
  return Deno.core.ops.op_sys_temp_dir();
}

/**
 * Clipboard operations.
 * Requires `sys.clipboard = true` in manifest permissions.
 *
 * @example
 * ```ts
 * import { clipboard } from "host:sys";
 *
 * // Read from clipboard
 * const text = await clipboard.read();
 *
 * // Write to clipboard
 * await clipboard.write("Hello, clipboard!");
 * ```
 */
export const clipboard: Clipboard = {
  async read(): Promise<string> {
    return await Deno.core.ops.op_sys_clipboard_read();
  },

  async write(text: string): Promise<void> {
    return await Deno.core.ops.op_sys_clipboard_write(text);
  },
};

/**
 * Show a system notification.
 * Requires `sys.notify = true` in manifest permissions.
 *
 * @param title - The notification title
 * @param body - The notification body text
 *
 * @example
 * ```ts
 * import { notify } from "host:sys";
 *
 * await notify("Download Complete", "Your file has finished downloading.");
 * ```
 */
export async function notify(title: string, body: string = ""): Promise<void> {
  return await Deno.core.ops.op_sys_notify(title, body);
}

/**
 * Show a system notification with extended options.
 * Requires `sys.notify = true` in manifest permissions.
 *
 * @param opts - Notification options
 *
 * @example
 * ```ts
 * import { notifyExt } from "host:sys";
 *
 * await notifyExt({
 *   title: "New Message",
 *   body: "You have a new message from Alice",
 *   subtitle: "Messages",
 *   sound: true,
 * });
 * ```
 */
export async function notifyExt(opts: NotifyOptions): Promise<void> {
  return await Deno.core.ops.op_sys_notify_ext(opts);
}

/**
 * Get power/battery information.
 * Requires `sys.power = true` in manifest permissions.
 *
 * @returns Power information including battery status and AC connection
 *
 * @example
 * ```ts
 * import { powerInfo } from "host:sys";
 *
 * const power = await powerInfo();
 * console.log(`Battery: ${power.has_battery}`);
 * if (power.batteries.length > 0) {
 *   const batt = power.batteries[0];
 *   console.log(`Charge: ${batt.charge_percent}%`);
 *   console.log(`State: ${batt.state}`);
 * }
 * ```
 */
export async function powerInfo(): Promise<PowerInfo> {
  return await Deno.core.ops.op_sys_power_info();
}

// Re-export types for convenience
export type { SystemInfo, NotifyOptions, Clipboard, PowerInfo, BatteryInfo };
