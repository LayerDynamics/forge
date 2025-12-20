// forge:bundler module - TypeScript wrapper for bundling ops

// Deno.core type declaration
declare const Deno: {
  core: {
    ops: {
      op_bundler_info(): ExtensionInfo;
      op_bundler_icon_create(options?: IconCreateOptions): Uint8Array;
      op_bundler_icon_validate(data: Uint8Array): IconValidation;
      op_bundler_icon_resize(data: Uint8Array, options: IconResizeOptions): Uint8Array;
      op_bundler_manifest_parse(content: string): AppManifest;
      op_bundler_sanitize_name(name: string): string;
      op_bundler_platform_info(): PlatformInfo;
      op_bundler_icon_requirements(platform: string): IconResizeOptions[];
      op_bundler_set_app_dir(path: string): void;
      op_bundler_get_app_dir(): string | null;
      op_bundler_set_build_config(config: BuildConfig): void;
      op_bundler_get_build_config(): BuildConfig | null;
      op_bundler_path_info(path: string): PathInfo;
      op_bundler_path_join(components: string[]): string;
      op_bundler_manifest_path(appDir: string): string;
      op_bundler_cache_manifest(path: string, manifest: AppManifest): void;
      op_bundler_get_cached_manifest(path: string): AppManifest | null;
    };
  };
};

// ============================================================================
// Types
// ============================================================================

export interface ExtensionInfo {
  name: string;
  version: string;
  capabilities: string[];
}

export interface IconCreateOptions {
  /** Size in pixels (default: 1024) */
  size?: number;
  /** Primary color (hex, default: "#3C5AB8") */
  color?: string;
}

export interface IconValidation {
  width: number;
  height: number;
  isSquare: boolean;
  meetsMinimum: boolean;
  meetsRecommended: boolean;
  hasTransparency: boolean;
  warnings: string[];
  errors: string[];
}

export interface IconResizeOptions {
  /** Target width */
  width: number;
  /** Target height */
  height: number;
}

export interface AppManifest {
  name: string;
  identifier: string;
  version: string;
  icon?: string;
}

export interface PlatformInfo {
  os: string;
  arch: string;
  bundleFormat: string;
  supported: boolean;
}

export type BundleFormat =
  | "App"      // macOS .app bundle
  | "Dmg"      // macOS .dmg disk image
  | "Pkg"      // macOS .pkg installer
  | "Msix"     // Windows .msix package
  | "AppImage" // Linux AppImage
  | "Tarball"  // Compressed tarball
  | "Zip";     // ZIP archive

export interface BuildConfig {
  /** App directory path */
  appDir: string;
  /** Output directory for bundle */
  outputDir?: string;
  /** Target bundle format */
  format?: BundleFormat;
  /** Whether to sign the bundle */
  sign?: boolean;
  /** Code signing identity (platform-specific) */
  signingIdentity?: string;
}

export interface PathInfo {
  /** The full path */
  path: string;
  /** Whether the path exists */
  exists: boolean;
  /** Whether it's a directory */
  isDir: boolean;
  /** Whether it's a file */
  isFile: boolean;
  /** File extension if any */
  extension?: string;
  /** File name without path */
  fileName?: string;
  /** Parent directory */
  parent?: string;
}

// ============================================================================
// Constants
// ============================================================================

/** Minimum recommended icon size */
export const MIN_ICON_SIZE = 512;

/** Optimal icon size for best quality across all platforms */
export const RECOMMENDED_ICON_SIZE = 1024;

// ============================================================================
// Core ops access
// ============================================================================

const core = Deno.core;

// ============================================================================
// Functions
// ============================================================================

/**
 * Get extension information
 */
export function info(): ExtensionInfo {
  return core.ops.op_bundler_info();
}

/**
 * Create a placeholder icon
 * @param options - Icon creation options
 * @returns PNG image bytes
 */
export function iconCreate(options?: IconCreateOptions): Uint8Array {
  return core.ops.op_bundler_icon_create(options);
}

/**
 * Validate an icon
 * @param data - PNG image bytes
 * @returns Validation result with dimensions, warnings, and errors
 */
export function iconValidate(data: Uint8Array): IconValidation {
  return core.ops.op_bundler_icon_validate(data);
}

/**
 * Resize an icon to specified dimensions
 * @param data - PNG image bytes
 * @param options - Resize options with target dimensions
 * @returns Resized PNG image bytes
 */
export function iconResize(data: Uint8Array, options: IconResizeOptions): Uint8Array {
  return core.ops.op_bundler_icon_resize(data, options);
}

/**
 * Parse a manifest.app.toml file content
 * @param content - TOML content string
 * @returns Parsed app manifest
 */
export function manifestParse(content: string): AppManifest {
  return core.ops.op_bundler_manifest_parse(content);
}

/**
 * Sanitize a name for use as executable/identifier
 * @param name - Original name
 * @returns Sanitized name (lowercase, alphanumeric with hyphens)
 */
export function sanitizeName(name: string): string {
  return core.ops.op_bundler_sanitize_name(name);
}

/**
 * Get platform-specific bundle information
 * @returns Platform info including OS, architecture, and bundle format
 */
export function platformInfo(): PlatformInfo {
  return core.ops.op_bundler_platform_info();
}

/**
 * Get icon requirements for a specific platform
 * @param platform - Platform name ("macos", "windows", or "linux")
 * @returns Array of required icon sizes
 */
export function iconRequirements(platform: string): IconResizeOptions[] {
  return core.ops.op_bundler_icon_requirements(platform);
}

/**
 * Set the current app directory for bundling operations
 * @param path - Path to app directory
 */
export function setAppDir(path: string): void {
  core.ops.op_bundler_set_app_dir(path);
}

/**
 * Get the current app directory
 * @returns App directory path or null if not set
 */
export function getAppDir(): string | null {
  return core.ops.op_bundler_get_app_dir();
}

/**
 * Set build configuration
 * @param config - Build configuration
 */
export function setBuildConfig(config: BuildConfig): void {
  core.ops.op_bundler_set_build_config(config);
}

/**
 * Get current build configuration
 * @returns Build configuration or null if not set
 */
export function getBuildConfig(): BuildConfig | null {
  return core.ops.op_bundler_get_build_config();
}

/**
 * Analyze a path and return information about it
 * @param path - Path to analyze
 * @returns Path information including existence and type
 */
export function pathInfo(path: string): PathInfo {
  return core.ops.op_bundler_path_info(path);
}

/**
 * Join path components
 * @param components - Array of path components
 * @returns Joined path string
 */
export function pathJoin(...components: string[]): string {
  return core.ops.op_bundler_path_join(components);
}

/**
 * Get the manifest path for an app directory
 * @param appDir - App directory path
 * @returns Path to manifest.app.toml
 */
export function manifestPath(appDir: string): string {
  return core.ops.op_bundler_manifest_path(appDir);
}

/**
 * Cache a parsed manifest for later retrieval
 * @param path - Manifest file path
 * @param manifest - Parsed manifest
 */
export function cacheManifest(path: string, manifest: AppManifest): void {
  core.ops.op_bundler_cache_manifest(path, manifest);
}

/**
 * Get a cached manifest
 * @param path - Manifest file path
 * @returns Cached manifest or null if not cached
 */
export function getCachedManifest(path: string): AppManifest | null {
  return core.ops.op_bundler_get_cached_manifest(path);
}
