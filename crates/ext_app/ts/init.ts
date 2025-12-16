// runtime:app module - TypeScript wrapper for Deno core ops

// Deno.core type declaration
declare const Deno: {
  core: {
    ops: {
      op_app_quit(): Promise<void>;
      op_app_exit(exitCode: number): void;
      op_app_relaunch(): Promise<void>;
      op_app_get_version(): string;
      op_app_get_name(): string;
      op_app_get_identifier(): string;
      op_app_get_path(pathType: string): string;
      op_app_is_packaged(): boolean;
      op_app_get_locale(): LocaleResult;
      op_app_request_single_instance_lock(): Promise<boolean>;
      op_app_release_single_instance_lock(): Promise<void>;
      op_app_focus(): Promise<void>;
      op_app_hide(): Promise<void>;
      op_app_show(): Promise<void>;
      op_app_set_badge_count(count: number | null): Promise<void>;
      op_app_set_user_model_id(appId: string): void;
    };
  };
};

interface LocaleResult {
  language: string;
  country: string | null;
  locale: string;
}

const core = Deno.core;

/**
 * System locale information
 */
export interface LocaleInfo {
  /** Language code (e.g., "en") */
  language: string;
  /** Country code (e.g., "US") */
  country: string | null;
  /** Full locale string (e.g., "en-US") */
  locale: string;
}

/**
 * Types of special paths that can be requested
 */
export type PathType =
  | "home"
  | "appData"
  | "documents"
  | "downloads"
  | "desktop"
  | "music"
  | "pictures"
  | "videos"
  | "temp"
  | "exe"
  | "resources"
  | "logs"
  | "cache";

/**
 * Quit the application gracefully.
 * Triggers cleanup handlers before exiting.
 */
export async function quit(): Promise<void> {
  return await core.ops.op_app_quit();
}

/**
 * Force exit the application immediately.
 * No cleanup handlers are called.
 * @param exitCode - Exit code (default: 0)
 */
export function exit(exitCode: number = 0): void {
  return core.ops.op_app_exit(exitCode);
}

/**
 * Relaunch the application.
 * The current instance will exit and a new instance will start.
 */
export async function relaunch(): Promise<void> {
  return await core.ops.op_app_relaunch();
}

/**
 * Get the application version.
 * @returns Version string from manifest.app.toml
 */
export function getVersion(): string {
  return core.ops.op_app_get_version();
}

/**
 * Get the application name.
 * @returns App name from manifest.app.toml
 */
export function getName(): string {
  return core.ops.op_app_get_name();
}

/**
 * Get the application identifier.
 * @returns App identifier (e.g., "com.example.app")
 */
export function getIdentifier(): string {
  return core.ops.op_app_get_identifier();
}

/**
 * Get a special system or application path.
 * @param pathType - Type of path to retrieve
 * @returns The requested path
 */
export function getPath(pathType: PathType): string {
  return core.ops.op_app_get_path(pathType);
}

/**
 * Check if the application is running in packaged mode.
 * @returns true if running as a bundled application, false in development
 */
export function isPackaged(): boolean {
  return core.ops.op_app_is_packaged();
}

/**
 * Get the system locale information.
 * @returns Locale information including language and country codes
 */
export function getLocale(): LocaleInfo {
  const result = core.ops.op_app_get_locale();
  return {
    language: result.language,
    country: result.country,
    locale: result.locale,
  };
}

/**
 * Request a single instance lock.
 * Prevents multiple instances of the application from running.
 * @returns true if the lock was acquired, false if another instance holds it
 */
export async function requestSingleInstanceLock(): Promise<boolean> {
  return await core.ops.op_app_request_single_instance_lock();
}

/**
 * Release the single instance lock.
 * Call this before exiting to allow another instance to start.
 */
export async function releaseSingleInstanceLock(): Promise<void> {
  return await core.ops.op_app_release_single_instance_lock();
}

/**
 * Bring the application to the foreground.
 * Makes the app's windows visible and focused.
 */
export async function focus(): Promise<void> {
  return await core.ops.op_app_focus();
}

/**
 * Hide all application windows.
 * On macOS, this hides the application. On other platforms, minimizes windows.
 */
export async function hide(): Promise<void> {
  return await core.ops.op_app_hide();
}

/**
 * Show all application windows.
 * Restores hidden or minimized windows.
 */
export async function show(): Promise<void> {
  return await core.ops.op_app_show();
}

/**
 * Set the dock/taskbar badge count.
 * @param count - Badge count to display, or null/undefined to clear
 */
export async function setBadgeCount(count?: number | null): Promise<void> {
  return await core.ops.op_app_set_badge_count(count ?? null);
}

/**
 * Set the Windows App User Model ID.
 * Used for taskbar grouping on Windows.
 * @param appId - Application user model ID
 */
export function setUserModelId(appId: string): void {
  return core.ops.op_app_set_user_model_id(appId);
}

// Convenience exports
export const version = getVersion;
export const name = getName;
export const identifier = getIdentifier;
