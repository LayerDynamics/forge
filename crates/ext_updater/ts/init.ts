// runtime:updater module - Application auto-update for Forge apps.
// Supports both GitHub Releases and custom JSON manifest formats.
// Provides check, download, verify, and install functionality.

// ============================================================================
// Deno Core Type Declarations
// ============================================================================

declare const Deno: {
  core: {
    ops: {
      // Legacy operations (backward compatibility)
      op_updater_info(): ExtensionInfo;
      op_updater_echo(message: string): string;
      // Configuration operations
      op_updater_configure_github(
        owner: string,
        repo: string,
        currentVersion: string,
        includePrereleases: boolean
      ): Promise<void>;
      op_updater_configure_custom(
        url: string,
        currentVersion: string,
        includePrereleases: boolean
      ): Promise<void>;
      // Check operations
      op_updater_check(): Promise<UpdateInfo | null>;
      // Download operations
      op_updater_download(): Promise<string>;
      op_updater_download_progress(): UpdateProgress;
      op_updater_cancel(): Promise<void>;
      // Verification operations
      op_updater_verify(): Promise<boolean>;
      // Install operations
      op_updater_install(): Promise<void>;
      // Status operations
      op_updater_status(): UpdaterStatus;
      op_updater_get_current_version(): string;
      op_updater_get_pending_update(): PendingUpdate | null;
    };
  };
};

const { core } = Deno;

// ============================================================================
// Extension Info Types (Legacy)
// ============================================================================

/**
 * Extension information for backward compatibility
 */
export interface ExtensionInfo {
  name: string;
  version: string;
  status: string;
}

// ============================================================================
// Update Source Types
// ============================================================================

/**
 * GitHub Releases update source configuration
 */
export interface GitHubSource {
  type: "github";
  /** Repository owner (e.g., "myorg") */
  owner: string;
  /** Repository name (e.g., "myapp") */
  repo: string;
}

/**
 * Custom JSON manifest update source configuration
 */
export interface CustomSource {
  type: "custom";
  /** URL to the JSON manifest file */
  url: string;
}

/**
 * Update source - either GitHub Releases or custom manifest
 */
export type UpdateSource = GitHubSource | CustomSource;

// ============================================================================
// Core Types
// ============================================================================

/**
 * Configuration for the updater
 */
export interface UpdateConfig {
  /** Update source configuration */
  source: UpdateSource;
  /** Current application version (semver format) */
  currentVersion: string;
  /** Whether to include prerelease versions (default: false) */
  includePrereleases?: boolean;
}

/**
 * Information about an available update
 */
export interface UpdateInfo {
  /** New version string */
  version: string;
  /** Download URL for the current platform */
  download_url: string;
  /** Release notes (if available) */
  release_notes: string | null;
  /** Download size in bytes */
  size_bytes: number;
  /** SHA256 checksum (if available from custom manifest) */
  sha256: string | null;
  /** Publish date (ISO 8601 format, if available) */
  publish_date: string | null;
  /** Whether this is a prerelease version */
  is_prerelease: boolean;
  /** All available assets for download */
  assets: UpdateAsset[];
}

/**
 * Individual downloadable asset
 */
export interface UpdateAsset {
  /** Asset filename */
  name: string;
  /** Download URL */
  url: string;
  /** Size in bytes */
  size_bytes: number;
  /** Content type (MIME) */
  content_type: string | null;
}

/**
 * Download progress information
 */
export interface UpdateProgress {
  /** Bytes downloaded so far */
  downloaded_bytes: number;
  /** Total bytes to download */
  total_bytes: number;
  /** Progress percentage (0-100) */
  percent: number;
  /** Current state */
  state: UpdateState;
}

/**
 * Update state enumeration
 */
export type UpdateState =
  | "idle"
  | "checking"
  | "update_available"
  | "downloading"
  | "verifying"
  | "ready_to_install"
  | "installing"
  | "complete"
  | "failed";

/**
 * Pending update information (downloaded but not installed)
 */
export interface PendingUpdate {
  /** Update information */
  info: UpdateInfo;
  /** Local path to downloaded file */
  local_path: string;
  /** Whether checksum verification passed */
  verified: boolean;
}

/**
 * Current updater status
 */
export interface UpdaterStatus {
  /** Current state */
  state: UpdateState;
  /** Download progress (if downloading) */
  progress: UpdateProgress | null;
  /** Available update info (if any) */
  available_update: UpdateInfo | null;
  /** Error message (if failed) */
  error: string | null;
  /** Whether an update source is configured */
  configured: boolean;
}

/**
 * Custom manifest format for self-hosted updates
 *
 * @example
 * ```json
 * {
 *   "version": "1.2.0",
 *   "platforms": {
 *     "darwin-aarch64": { "url": "https://...", "sha256": "...", "size": 12345 },
 *     "darwin-x64": { "url": "https://...", "sha256": "...", "size": 12345 },
 *     "win32-x64": { "url": "https://...", "sha256": "...", "size": 12345 },
 *     "linux-x64": { "url": "https://...", "sha256": "...", "size": 12345 }
 *   },
 *   "release_notes": "What's new in 1.2.0...",
 *   "publish_date": "2024-12-18T00:00:00Z"
 * }
 * ```
 */
export interface CustomManifest {
  /** Version string (semver format) */
  version: string;
  /** Platform-specific download information */
  platforms: Record<string, PlatformAsset>;
  /** Release notes (optional) */
  release_notes?: string;
  /** Publish date in ISO 8601 format (optional) */
  publish_date?: string;
}

/**
 * Platform-specific asset in custom manifest
 */
export interface PlatformAsset {
  /** Download URL */
  url: string;
  /** SHA256 checksum for verification */
  sha256?: string;
  /** File size in bytes */
  size?: number;
}

// ============================================================================
// Error Codes
// ============================================================================

/**
 * Updater error codes (5000-5099)
 */
export const UpdaterErrorCode = {
  /** Generic updater error */
  GENERIC: 5000,
  /** Failed to check for updates */
  CHECK_FAILED: 5001,
  /** Failed to download update */
  DOWNLOAD_FAILED: 5002,
  /** Package verification failed */
  VERIFICATION_FAILED: 5003,
  /** Failed to install update */
  INSTALL_FAILED: 5004,
  /** No update available */
  NO_UPDATE: 5005,
  /** Network error */
  NETWORK_ERROR: 5006,
  /** Invalid manifest format */
  INVALID_MANIFEST: 5007,
  /** Permission denied */
  PERMISSION_DENIED: 5008,
  /** Update already in progress */
  ALREADY_IN_PROGRESS: 5009,
  /** Update cancelled */
  CANCELLED: 5010,
  /** Not configured */
  NOT_CONFIGURED: 5011,
  /** Invalid version format */
  INVALID_VERSION: 5012,
} as const;

// ============================================================================
// Legacy Operations (Backward Compatibility)
// ============================================================================

/**
 * Get extension information (legacy).
 * @returns Extension info object
 */
export function info(): ExtensionInfo {
  return core.ops.op_updater_info();
}

/**
 * Echo a message back (legacy, for testing).
 * @param message - Message to echo
 * @returns The same message
 */
export function echo(message: string): string {
  return core.ops.op_updater_echo(message);
}

// ============================================================================
// Configuration Functions
// ============================================================================

/**
 * Configure updater with GitHub Releases as the update source.
 *
 * GitHub releases are automatically parsed and the correct platform-specific
 * asset is selected based on filename patterns like:
 * - `myapp-darwin-aarch64.dmg`
 * - `myapp-win32-x64.exe`
 * - `myapp-linux-x64.AppImage`
 *
 * @param config - Configuration options
 * @throws Error if configuration fails
 *
 * @example
 * ```ts
 * import { configureGitHub } from "runtime:updater";
 *
 * configureGitHub({
 *   owner: "myorg",
 *   repo: "myapp",
 *   currentVersion: "1.0.0",
 * });
 * ```
 */
export async function configureGitHub(config: {
  owner: string;
  repo: string;
  currentVersion: string;
  includePrereleases?: boolean;
}): Promise<void> {
  await core.ops.op_updater_configure_github(
    config.owner,
    config.repo,
    config.currentVersion,
    config.includePrereleases ?? false
  );
}

/**
 * Configure updater with a custom JSON manifest as the update source.
 *
 * The manifest should follow the CustomManifest interface format with
 * platform keys like "darwin-aarch64", "darwin-x64", "win32-x64", "linux-x64".
 *
 * @param config - Configuration options
 * @throws Error if URL is invalid
 *
 * @example
 * ```ts
 * import { configureCustom } from "runtime:updater";
 *
 * configureCustom({
 *   url: "https://myapp.com/updates.json",
 *   currentVersion: "1.0.0",
 * });
 * ```
 */
export async function configureCustom(config: {
  url: string;
  currentVersion: string;
  includePrereleases?: boolean;
}): Promise<void> {
  await core.ops.op_updater_configure_custom(
    config.url,
    config.currentVersion,
    config.includePrereleases ?? false
  );
}

/**
 * Configure the updater with the given update source.
 *
 * @param config - Update configuration
 * @throws Error if configuration fails
 *
 * @example
 * ```ts
 * import { configure } from "runtime:updater";
 *
 * // Using GitHub Releases
 * configure({
 *   source: { type: "github", owner: "myorg", repo: "myapp" },
 *   currentVersion: "1.0.0",
 * });
 *
 * // Using custom manifest
 * configure({
 *   source: { type: "custom", url: "https://myapp.com/updates.json" },
 *   currentVersion: "1.0.0",
 * });
 * ```
 */
export async function configure(config: UpdateConfig): Promise<void> {
  if (config.source.type === "github") {
    await configureGitHub({
      owner: config.source.owner,
      repo: config.source.repo,
      currentVersion: config.currentVersion,
      includePrereleases: config.includePrereleases,
    });
  } else {
    await configureCustom({
      url: config.source.url,
      currentVersion: config.currentVersion,
      includePrereleases: config.includePrereleases,
    });
  }
}

// ============================================================================
// Check Functions
// ============================================================================

/**
 * Check for available updates.
 *
 * Fetches the latest release from the configured source and compares
 * versions using semantic versioning. Returns update info if a newer
 * version is available, null otherwise.
 *
 * @returns Update info if available, null otherwise
 * @throws Error if not configured or network fails
 *
 * @example
 * ```ts
 * import { configure, check } from "runtime:updater";
 *
 * configure({
 *   source: { type: "github", owner: "myorg", repo: "myapp" },
 *   currentVersion: "1.0.0",
 * });
 *
 * const update = await check();
 * if (update) {
 *   console.log(`New version available: ${update.version}`);
 *   console.log(`Download size: ${formatBytes(update.size_bytes)}`);
 *   if (update.release_notes) {
 *     console.log(`Release notes: ${update.release_notes}`);
 *   }
 * } else {
 *   console.log("You're running the latest version!");
 * }
 * ```
 */
export async function check(): Promise<UpdateInfo | null> {
  return await core.ops.op_updater_check();
}

// ============================================================================
// Download Functions
// ============================================================================

/**
 * Download the available update.
 *
 * Downloads the update package to a temporary location. Use `getProgress()`
 * to monitor download progress. The download can be cancelled with `cancel()`.
 *
 * @returns Path to the downloaded file
 * @throws Error if no update available, already downloading, or download fails
 *
 * @example
 * ```ts
 * import { check, download, getProgress } from "runtime:updater";
 *
 * const update = await check();
 * if (update) {
 *   // Start download
 *   const downloadPromise = download();
 *
 *   // Monitor progress
 *   const interval = setInterval(() => {
 *     const progress = getProgress();
 *     console.log(`Downloaded: ${progress.percent.toFixed(1)}%`);
 *   }, 500);
 *
 *   const filePath = await downloadPromise;
 *   clearInterval(interval);
 *   console.log(`Downloaded to: ${filePath}`);
 * }
 * ```
 */
export async function download(): Promise<string> {
  return await core.ops.op_updater_download();
}

/**
 * Get current download progress.
 *
 * @returns Progress information including bytes downloaded, total, and percentage
 *
 * @example
 * ```ts
 * import { getProgress } from "runtime:updater";
 *
 * const progress = getProgress();
 * console.log(`Progress: ${progress.percent.toFixed(1)}%`);
 * console.log(`Downloaded: ${progress.downloaded_bytes} / ${progress.total_bytes}`);
 * ```
 */
export function getProgress(): UpdateProgress {
  return core.ops.op_updater_download_progress();
}

/**
 * Cancel an in-progress download.
 *
 * @throws Error if no download in progress
 *
 * @example
 * ```ts
 * import { download, cancel } from "runtime:updater";
 *
 * // Start download
 * const downloadPromise = download();
 *
 * // Cancel after 5 seconds
 * setTimeout(() => {
 *   try {
 *     cancel();
 *     console.log("Download cancelled");
 *   } catch (e) {
 *     console.log("Could not cancel:", e);
 *   }
 * }, 5000);
 * ```
 */
export async function cancel(): Promise<void> {
  await core.ops.op_updater_cancel();
}

// ============================================================================
// Verification Functions
// ============================================================================

/**
 * Verify the downloaded update package.
 *
 * Checks the SHA256 checksum if provided in the update info (custom manifests).
 * GitHub releases don't include checksums, so verification is skipped and
 * returns true automatically.
 *
 * @returns True if verification passed or no checksum available
 * @throws Error if verification fails (checksum mismatch)
 *
 * @example
 * ```ts
 * import { download, verify, install } from "runtime:updater";
 *
 * await download();
 *
 * const isValid = await verify();
 * if (isValid) {
 *   console.log("Package verified successfully");
 *   await install();
 * }
 * ```
 */
export async function verify(): Promise<boolean> {
  return await core.ops.op_updater_verify();
}

// ============================================================================
// Install Functions
// ============================================================================

/**
 * Install the downloaded update.
 *
 * Platform-specific behavior:
 * - **macOS**: Opens .dmg files for user installation, extracts .zip files
 * - **Windows**: Launches .exe, .msi, or .msix installers
 * - **Linux**: Makes .AppImage executable and launches it, or installs .deb/.rpm
 *
 * Note: The application may need to restart after installation.
 *
 * @throws Error if no pending update or installation fails
 *
 * @example
 * ```ts
 * import { check, download, verify, install } from "runtime:updater";
 *
 * const update = await check();
 * if (update) {
 *   console.log(`Installing ${update.version}...`);
 *   await download();
 *   await verify();
 *   await install();
 *   console.log("Update installed! Please restart the application.");
 * }
 * ```
 */
export async function install(): Promise<void> {
  return await core.ops.op_updater_install();
}

// ============================================================================
// Status Functions
// ============================================================================

/**
 * Get current updater status.
 *
 * @returns Complete status information
 *
 * @example
 * ```ts
 * import { getStatus } from "runtime:updater";
 *
 * const status = getStatus();
 * console.log(`State: ${status.state}`);
 * console.log(`Configured: ${status.configured}`);
 *
 * if (status.state === "failed" && status.error) {
 *   console.error(`Error: ${status.error}`);
 * }
 *
 * if (status.available_update) {
 *   console.log(`Available: v${status.available_update.version}`);
 * }
 * ```
 */
export function getStatus(): UpdaterStatus {
  return core.ops.op_updater_status();
}

/**
 * Get the current application version.
 *
 * @returns Current version string
 * @throws Error if not configured
 *
 * @example
 * ```ts
 * import { getCurrentVersion } from "runtime:updater";
 *
 * const version = getCurrentVersion();
 * console.log(`Current version: ${version}`);
 * ```
 */
export function getCurrentVersion(): string {
  return core.ops.op_updater_get_current_version();
}

/**
 * Get pending update information.
 *
 * Returns information about a downloaded update that hasn't been installed yet.
 *
 * @returns Pending update info or null if none
 *
 * @example
 * ```ts
 * import { getPendingUpdate, install } from "runtime:updater";
 *
 * const pending = getPendingUpdate();
 * if (pending) {
 *   console.log(`Pending: v${pending.info.version}`);
 *   console.log(`Location: ${pending.local_path}`);
 *   console.log(`Verified: ${pending.verified}`);
 *
 *   if (pending.verified) {
 *     await install();
 *   }
 * }
 * ```
 */
export function getPendingUpdate(): PendingUpdate | null {
  return core.ops.op_updater_get_pending_update();
}

// ============================================================================
// Convenience Functions
// ============================================================================

/**
 * Check for updates and automatically download if available.
 *
 * @param onProgress - Optional callback for download progress
 * @returns Update info and local path, or null if no update
 *
 * @example
 * ```ts
 * import { checkAndDownload } from "runtime:updater";
 *
 * const result = await checkAndDownload((progress) => {
 *   console.log(`Download progress: ${progress.percent.toFixed(1)}%`);
 * });
 *
 * if (result) {
 *   console.log(`Downloaded v${result.info.version} to ${result.localPath}`);
 * }
 * ```
 */
export async function checkAndDownload(
  onProgress?: (progress: UpdateProgress) => void
): Promise<{ info: UpdateInfo; localPath: string } | null> {
  const update = await check();
  if (!update) {
    return null;
  }

  let progressInterval: number | undefined;
  if (onProgress) {
    progressInterval = setInterval(() => {
      onProgress(getProgress());
    }, 100) as unknown as number;
  }

  try {
    const localPath = await download();
    return { info: update, localPath };
  } finally {
    if (progressInterval !== undefined) {
      clearInterval(progressInterval);
    }
  }
}

/**
 * Perform a full update cycle: check, download, verify, and install.
 *
 * @param callbacks - Optional callbacks for each stage
 * @returns True if update was installed, false if no update available
 *
 * @example
 * ```ts
 * import { fullUpdate } from "runtime:updater";
 *
 * const updated = await fullUpdate({
 *   onCheckComplete: (info) => {
 *     console.log(`Found update: v${info.version}`);
 *   },
 *   onProgress: (progress) => {
 *     console.log(`Downloading: ${progress.percent.toFixed(1)}%`);
 *   },
 *   onVerifyComplete: (verified) => {
 *     console.log(`Verified: ${verified}`);
 *   },
 *   onInstallComplete: () => {
 *     console.log("Installation complete!");
 *   },
 * });
 *
 * if (updated) {
 *   console.log("Please restart the application.");
 * } else {
 *   console.log("Already up to date.");
 * }
 * ```
 */
export async function fullUpdate(callbacks?: {
  onCheckComplete?: (info: UpdateInfo) => void;
  onProgress?: (progress: UpdateProgress) => void;
  onVerifyComplete?: (verified: boolean) => void;
  onInstallComplete?: () => void;
}): Promise<boolean> {
  // Check
  const update = await check();
  if (!update) {
    return false;
  }

  callbacks?.onCheckComplete?.(update);

  // Download
  let progressInterval: number | undefined;
  if (callbacks?.onProgress) {
    progressInterval = setInterval(() => {
      callbacks.onProgress!(getProgress());
    }, 100) as unknown as number;
  }

  try {
    await download();
  } finally {
    if (progressInterval !== undefined) {
      clearInterval(progressInterval);
    }
  }

  // Verify
  const verified = await verify();
  callbacks?.onVerifyComplete?.(verified);

  if (!verified) {
    throw new Error("Update verification failed");
  }

  // Install
  await install();
  callbacks?.onInstallComplete?.();

  return true;
}

/**
 * Format bytes to human-readable string.
 *
 * @param bytes - Number of bytes
 * @returns Formatted string (e.g., "10.5 MB")
 *
 * @example
 * ```ts
 * import { formatBytes } from "runtime:updater";
 *
 * console.log(formatBytes(1024)); // "1.00 KB"
 * console.log(formatBytes(1048576)); // "1.00 MB"
 * console.log(formatBytes(1073741824)); // "1.00 GB"
 * ```
 */
export function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 Bytes";
  const k = 1024;
  const sizes = ["Bytes", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
}

/**
 * Check if the current state indicates an update is available.
 *
 * @returns True if an update is available
 */
export function isUpdateAvailable(): boolean {
  const status = getStatus();
  return status.available_update !== null;
}

/**
 * Check if a download is in progress.
 *
 * @returns True if downloading
 */
export function isDownloading(): boolean {
  return getStatus().state === "downloading";
}

/**
 * Check if an update is ready to install.
 *
 * @returns True if ready to install
 */
export function isReadyToInstall(): boolean {
  return getStatus().state === "ready_to_install";
}

// ============================================================================
// Convenience Aliases
// ============================================================================

export { check as checkForUpdates };
export { download as downloadUpdate };
export { install as installUpdate };
export { getStatus as status };
export { getProgress as progress };
