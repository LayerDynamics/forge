// runtime:sys module - TypeScript wrapper for Deno core ops

declare const Deno: {
  core: {
    ops: {
      op_sys_info(): RawSystemInfo;
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
      // Enhanced operations
      op_sys_env_all(): Record<string, string>;
      op_sys_env_delete(key: string): void;
      op_sys_locale(): LocaleInfoResult;
      op_sys_app_paths(): AppPathsResult;
    };
  };
};

interface RawSystemInfo {
  os: string;
  arch: string;
  hostname: string;
  cpu_count: number;
  cpuCount?: number;
}

interface SystemInfo {
  os: string;
  arch: string;
  hostname: string;
  cpuCount: number;
}

interface NotifyOptions {
  title: string;
  body?: string;
  icon?: string;
  sound?: boolean;
}

interface PowerInfo {
  state: string;
  percentage: number | null;
  timeToFull: number | null;
  timeToEmpty: number | null;
}

// Enhanced types
interface LocaleInfoResult {
  language: string;
  country: string | null;
  locale: string;
}

interface AppPathsResult {
  home: string | null;
  documents: string | null;
  downloads: string | null;
  desktop: string | null;
  music: string | null;
  pictures: string | null;
  videos: string | null;
  data: string | null;
  config: string | null;
  cache: string | null;
  runtime: string | null;
}

/**
 * System locale information
 */
export interface LocaleInfo {
  language: string;
  country: string | null;
  locale: string;
}

/**
 * Standard application paths
 */
export interface AppPaths {
  home: string | null;
  documents: string | null;
  downloads: string | null;
  desktop: string | null;
  music: string | null;
  pictures: string | null;
  videos: string | null;
  data: string | null;
  config: string | null;
  cache: string | null;
  runtime: string | null;
}

const core = Deno.core;

export function info(): SystemInfo {
  const result = core.ops.op_sys_info();
  return {
    os: result.os,
    arch: result.arch,
    hostname: result.hostname,
    cpuCount: result.cpu_count,
  };
}

export function getEnv(key: string): string | null {
  return core.ops.op_sys_env_get(key);
}

export function setEnv(key: string, value: string): void {
  core.ops.op_sys_env_set(key, value);
}

export function cwd(): string {
  return core.ops.op_sys_cwd();
}

export function homeDir(): string | null {
  return core.ops.op_sys_home_dir();
}

export function tempDir(): string {
  return core.ops.op_sys_temp_dir();
}

export const clipboard = {
  async read(): Promise<string> {
    return await core.ops.op_sys_clipboard_read();
  },

  async write(text: string): Promise<void> {
    return await core.ops.op_sys_clipboard_write(text);
  },
};

export async function notify(title: string, body: string = ""): Promise<void> {
  return await core.ops.op_sys_notify(title, body);
}

export async function notifyExt(opts: NotifyOptions): Promise<void> {
  return await core.ops.op_sys_notify_ext(opts);
}

export async function powerInfo(): Promise<PowerInfo> {
  return await core.ops.op_sys_power_info();
}

// ============================================================================
// Enhanced Operations
// ============================================================================

/**
 * Get all environment variables.
 * @returns Object with all environment variables
 */
export function getAllEnv(): Record<string, string> {
  return core.ops.op_sys_env_all();
}

/**
 * Delete an environment variable.
 * @param key - The environment variable key to delete
 */
export function deleteEnv(key: string): void {
  return core.ops.op_sys_env_delete(key);
}

/**
 * Get system locale information.
 * @returns Locale information including language and country codes
 */
export function locale(): LocaleInfo {
  return core.ops.op_sys_locale();
}

/**
 * Get standard application paths.
 * @returns Object with standard directory paths
 */
export function appPaths(): AppPaths {
  return core.ops.op_sys_app_paths();
}

// Convenience aliases
export { getAllEnv as envAll };
export { deleteEnv as envDelete };
export { locale as getLocale };
export { appPaths as getPaths };
