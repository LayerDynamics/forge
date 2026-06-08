---
title: "runtime:updater"
description: "Application auto-update system for Forge applications"
slug: docs/api/runtime-updater
---

Application auto-update system for Forge applications. Supports GitHub Releases and custom JSON manifest formats with check, download, verify, and install functionality.

> **Implementation**: TypeScript types are auto-generated from Rust via [forge-weld](/docs/crates/forge-weld). See [ext_updater](/docs/crates/ext-updater) for implementation details.

## Features

- Check for updates from GitHub Releases or custom manifests
- Download updates with progress tracking
- SHA256 checksum verification (custom manifests)
- Platform-specific installation handling
- Prerelease version support
- Cancel in-progress downloads
- Full update lifecycle with callbacks

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Application                              │
├─────────────────────────────────────────────────────────────┤
│  configure() → check() → download() → verify() → install()  │
├─────────────────────────────────────────────────────────────┤
│                    runtime:updater                           │
├──────────────────────┬──────────────────────────────────────┤
│    GitHub Releases   │        Custom Manifest                │
│   (Auto-parsing)     │     (JSON with checksums)             │
└──────────────────────┴──────────────────────────────────────┘
```

## Import

```typescript
import {
  // Configuration
  configure,
  configureGitHub,
  configureCustom,
  // Update lifecycle
  check,
  download,
  verify,
  install,
  // Status & Progress
  getStatus,
  getProgress,
  getCurrentVersion,
  getPendingUpdate,
  // Control
  cancel,
  // Convenience
  checkAndDownload,
  fullUpdate,
  formatBytes,
  isUpdateAvailable,
  isDownloading,
  isReadyToInstall,
  // Types
  type UpdateConfig,
  type UpdateInfo,
  type UpdateProgress,
  type UpdaterStatus,
  type UpdateState,
  type PendingUpdate,
  type CustomManifest,
} from "runtime:updater";
```

## Update Sources

### GitHub Releases

GitHub Releases are automatically parsed. Assets are matched by platform-specific filename patterns:

| Platform | Pattern Examples |
|----------|------------------|
| macOS ARM | `*-darwin-aarch64.dmg`, `*-macos-arm64.zip` |
| macOS Intel | `*-darwin-x64.dmg`, `*-macos-x64.zip` |
| Windows | `*-win32-x64.exe`, `*-windows-x64.msi` |
| Linux | `*-linux-x64.AppImage`, `*-linux-x64.deb` |

### Custom JSON Manifest

Self-hosted updates use a JSON manifest with the following structure:

```json
{
  "version": "1.2.0",
  "platforms": {
    "darwin-aarch64": {
      "url": "https://cdn.myapp.com/releases/myapp-1.2.0-darwin-aarch64.dmg",
      "sha256": "abc123...",
      "size": 45678901
    },
    "darwin-x64": {
      "url": "https://cdn.myapp.com/releases/myapp-1.2.0-darwin-x64.dmg",
      "sha256": "def456...",
      "size": 45678901
    },
    "win32-x64": {
      "url": "https://cdn.myapp.com/releases/myapp-1.2.0-win32-x64.exe",
      "sha256": "ghi789...",
      "size": 34567890
    },
    "linux-x64": {
      "url": "https://cdn.myapp.com/releases/myapp-1.2.0-linux-x64.AppImage",
      "sha256": "jkl012...",
      "size": 56789012
    }
  },
  "release_notes": "## What's New\n- Feature A\n- Bug fix B",
  "publish_date": "2024-12-18T00:00:00Z"
}
```

## API Reference

<!-- forge:api -->
<!-- generated from sdk/runtime.updater.ts — edit signatures in the SDK, run `make docs-api` to refresh -->
```typescript
info(): ExtensionInfo
echo(message: string): string
configureGitHub(config:
configureCustom(config:
configure(config: UpdateConfig): Promise<void>
check(): Promise<UpdateInfo | null>
download(): Promise<string>
getProgress(): UpdateProgress
cancel(): Promise<void>
verify(): Promise<boolean>
install(): Promise<void>
getStatus(): UpdaterStatus
getCurrentVersion(): string
getPendingUpdate(): PendingUpdate | null
checkAndDownload( onProgress?: (progress: UpdateProgress) => void ): Promise<
fullUpdate(callbacks?:
formatBytes(bytes: number): string
isUpdateAvailable(): boolean
isDownloading(): boolean
isReadyToInstall(): boolean
checkForUpdates(): Promise<UpdateInfo | null>
downloadUpdate(): Promise<string>
installUpdate(): Promise<void>
status(): UpdaterStatus
progress(): UpdateProgress
```
<!-- /forge:api -->

### Configuration

#### configure(config)

Configure the updater with an update source.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `config` | `UpdateConfig` | Update configuration |

**UpdateConfig:**

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `source` | `UpdateSource` | - | Update source (GitHub or custom) |
| `currentVersion` | `string` | - | Current app version (semver) |
| `includePrereleases` | `boolean` | `false` | Include prerelease versions |

**Throws:** Error if configuration fails

**Example:**

```typescript
import { configure } from "runtime:updater";

// Using GitHub Releases
await configure({
  source: { type: "github", owner: "myorg", repo: "myapp" },
  currentVersion: "1.0.0",
});

// Using custom manifest
await configure({
  source: { type: "custom", url: "https://myapp.com/updates.json" },
  currentVersion: "1.0.0",
  includePrereleases: true,
});
```

---

#### configureGitHub(config)

Configure updater with GitHub Releases as the update source.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `config.owner` | `string` | Repository owner |
| `config.repo` | `string` | Repository name |
| `config.currentVersion` | `string` | Current app version |
| `config.includePrereleases` | `boolean` | Include prereleases (default: false) |

**Example:**

```typescript
import { configureGitHub } from "runtime:updater";

await configureGitHub({
  owner: "myorg",
  repo: "myapp",
  currentVersion: "1.0.0",
});
```

---

#### configureCustom(config)

Configure updater with a custom JSON manifest.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `config.url` | `string` | URL to JSON manifest |
| `config.currentVersion` | `string` | Current app version |
| `config.includePrereleases` | `boolean` | Include prereleases (default: false) |

**Example:**

```typescript
import { configureCustom } from "runtime:updater";

await configureCustom({
  url: "https://myapp.com/updates.json",
  currentVersion: "1.0.0",
});
```

### Update Lifecycle

#### check()

Check for available updates using semantic version comparison.

**Returns:** `Promise<UpdateInfo | null>` - Update info if available, null otherwise

**Throws:** Error if not configured or network fails

**Example:**

```typescript
import { configure, check, formatBytes } from "runtime:updater";

await configure({
  source: { type: "github", owner: "myorg", repo: "myapp" },
  currentVersion: "1.0.0",
});

const update = await check();
if (update) {
  console.log(`New version available: ${update.version}`);
  console.log(`Download size: ${formatBytes(update.size_bytes)}`);
  if (update.release_notes) {
    console.log(`Release notes:\n${update.release_notes}`);
  }
} else {
  console.log("You're running the latest version!");
}
```

---

#### download()

Download the available update to a temporary location.

**Returns:** `Promise<string>` - Path to downloaded file

**Throws:** Error if no update available, already downloading, or download fails

**Example:**

```typescript
import { check, download, getProgress } from "runtime:updater";

const update = await check();
if (update) {
  // Start download
  const downloadPromise = download();

  // Monitor progress
  const interval = setInterval(() => {
    const progress = getProgress();
    console.log(`Downloaded: ${progress.percent.toFixed(1)}%`);
  }, 500);

  const filePath = await downloadPromise;
  clearInterval(interval);
  console.log(`Downloaded to: ${filePath}`);
}
```

---

#### verify()

Verify the downloaded update package using SHA256 checksum.

**Returns:** `Promise<boolean>` - True if verified or no checksum available

**Throws:** Error if checksum mismatch

**Note:** GitHub releases don't include checksums, so verification returns true automatically. Custom manifests with `sha256` fields enable full verification.

**Example:**

```typescript
import { download, verify, install } from "runtime:updater";

await download();

const isValid = await verify();
if (isValid) {
  console.log("Package verified successfully");
  await install();
} else {
  console.error("Verification failed - download may be corrupted");
}
```

---

#### install()

Install the downloaded update.

**Platform-specific behavior:**

| Platform | File Types | Behavior |
|----------|------------|----------|
| macOS | `.dmg` | Opens disk image for user installation |
| macOS | `.zip` | Extracts and replaces app bundle |
| Windows | `.exe`, `.msi` | Launches installer |
| Windows | `.msix` | Uses Windows installer API |
| Linux | `.AppImage` | Makes executable and launches |
| Linux | `.deb`, `.rpm` | Uses package manager |

**Throws:** Error if no pending update or installation fails

**Example:**

```typescript
import { check, download, verify, install } from "runtime:updater";

const update = await check();
if (update) {
  console.log(`Installing ${update.version}...`);
  await download();
  await verify();
  await install();
  console.log("Update installed! Please restart the application.");
}
```

### Status & Progress

#### getStatus()

Get comprehensive updater status.

**Returns:** `UpdaterStatus`

**Example:**

```typescript
import { getStatus } from "runtime:updater";

const status = getStatus();
console.log(`State: ${status.state}`);
console.log(`Configured: ${status.configured}`);

if (status.state === "failed" && status.error) {
  console.error(`Error: ${status.error}`);
}

if (status.available_update) {
  console.log(`Available: v${status.available_update.version}`);
}

if (status.progress) {
  console.log(`Progress: ${status.progress.percent.toFixed(1)}%`);
}
```

---

#### getProgress()

Get current download progress.

**Returns:** `UpdateProgress`

**Example:**

```typescript
import { getProgress, formatBytes } from "runtime:updater";

const progress = getProgress();
console.log(`Progress: ${progress.percent.toFixed(1)}%`);
console.log(`Downloaded: ${formatBytes(progress.downloaded_bytes)} / ${formatBytes(progress.total_bytes)}`);
console.log(`State: ${progress.state}`);
```

---

#### getCurrentVersion()

Get the configured current application version.

**Returns:** `string` - Current version

**Throws:** Error if not configured

**Example:**

```typescript
import { getCurrentVersion } from "runtime:updater";

const version = getCurrentVersion();
console.log(`Current version: ${version}`);
```

---

#### getPendingUpdate()

Get information about a downloaded update that hasn't been installed.

**Returns:** `PendingUpdate | null`

**Example:**

```typescript
import { getPendingUpdate, install } from "runtime:updater";

const pending = getPendingUpdate();
if (pending) {
  console.log(`Pending: v${pending.info.version}`);
  console.log(`Location: ${pending.local_path}`);
  console.log(`Verified: ${pending.verified}`);

  if (pending.verified) {
    const shouldInstall = await promptUser("Install update now?");
    if (shouldInstall) {
      await install();
    }
  }
}
```

### Control

#### cancel()

Cancel an in-progress download.

**Throws:** Error if no download in progress

**Example:**

```typescript
import { download, cancel, isDownloading } from "runtime:updater";

// Start download
const downloadPromise = download();

// Allow user to cancel
cancelButton.onclick = async () => {
  if (isDownloading()) {
    await cancel();
    console.log("Download cancelled");
  }
};

try {
  await downloadPromise;
} catch (e) {
  if (e.message.includes("cancelled")) {
    console.log("Download was cancelled by user");
  }
}
```

### Convenience Functions

#### checkAndDownload(onProgress?)

Check for updates and download if available.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `onProgress` | `(progress: UpdateProgress) => void` | Optional progress callback |

**Returns:** `Promise<{ info: UpdateInfo; localPath: string } | null>`

**Example:**

```typescript
import { checkAndDownload, formatBytes } from "runtime:updater";

const result = await checkAndDownload((progress) => {
  progressBar.value = progress.percent;
  statusText.textContent = `Downloading: ${formatBytes(progress.downloaded_bytes)}`;
});

if (result) {
  console.log(`Downloaded v${result.info.version} to ${result.localPath}`);
}
```

---

#### fullUpdate(callbacks?)

Perform a complete update cycle: check → download → verify → install.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `callbacks.onCheckComplete` | `(info: UpdateInfo) => void` | Called when update found |
| `callbacks.onProgress` | `(progress: UpdateProgress) => void` | Download progress |
| `callbacks.onVerifyComplete` | `(verified: boolean) => void` | Verification result |
| `callbacks.onInstallComplete` | `() => void` | Installation complete |

**Returns:** `Promise<boolean>` - True if update was installed

**Example:**

```typescript
import { configure, fullUpdate, formatBytes } from "runtime:updater";

await configure({
  source: { type: "github", owner: "myorg", repo: "myapp" },
  currentVersion: "1.0.0",
});

const updated = await fullUpdate({
  onCheckComplete: (info) => {
    showNotification(`Update available: v${info.version}`);
  },
  onProgress: (progress) => {
    updateProgressBar(progress.percent);
    updateStatusText(`Downloading: ${formatBytes(progress.downloaded_bytes)}`);
  },
  onVerifyComplete: (verified) => {
    console.log(`Package verified: ${verified}`);
  },
  onInstallComplete: () => {
    showNotification("Update installed successfully!");
  },
});

if (updated) {
  showRestartPrompt();
} else {
  showNotification("Already running the latest version");
}
```

---

#### formatBytes(bytes)

Format byte count to human-readable string.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `bytes` | `number` | Number of bytes |

**Returns:** `string` - Formatted string (e.g., "10.5 MB")

**Example:**

```typescript
import { formatBytes } from "runtime:updater";

console.log(formatBytes(1024));        // "1.00 KB"
console.log(formatBytes(1048576));     // "1.00 MB"
console.log(formatBytes(1073741824));  // "1.00 GB"
```

---

#### isUpdateAvailable()

Check if an update is available.

**Returns:** `boolean`

---

#### isDownloading()

Check if a download is in progress.

**Returns:** `boolean`

---

#### isReadyToInstall()

Check if an update is ready to install.

**Returns:** `boolean`

## Type Definitions

```typescript
/**
 * Update source - GitHub Releases or custom manifest
 */
type UpdateSource = GitHubSource | CustomSource;

interface GitHubSource {
  type: "github";
  /** Repository owner (e.g., "myorg") */
  owner: string;
  /** Repository name (e.g., "myapp") */
  repo: string;
}

interface CustomSource {
  type: "custom";
  /** URL to the JSON manifest file */
  url: string;
}

/**
 * Configuration for the updater
 */
interface UpdateConfig {
  source: UpdateSource;
  currentVersion: string;
  includePrereleases?: boolean;
}

/**
 * Information about an available update
 */
interface UpdateInfo {
  /** New version string */
  version: string;
  /** Download URL for the current platform */
  download_url: string;
  /** Release notes (if available) */
  release_notes: string | null;
  /** Download size in bytes */
  size_bytes: number;
  /** SHA256 checksum (custom manifests only) */
  sha256: string | null;
  /** Publish date (ISO 8601 format) */
  publish_date: string | null;
  /** Whether this is a prerelease */
  is_prerelease: boolean;
  /** All available assets */
  assets: UpdateAsset[];
}

/**
 * Downloadable asset
 */
interface UpdateAsset {
  name: string;
  url: string;
  size_bytes: number;
  content_type: string | null;
}

/**
 * Download progress
 */
interface UpdateProgress {
  downloaded_bytes: number;
  total_bytes: number;
  percent: number;
  state: UpdateState;
}

/**
 * Update state
 */
type UpdateState =
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
 * Pending update (downloaded but not installed)
 */
interface PendingUpdate {
  info: UpdateInfo;
  local_path: string;
  verified: boolean;
}

/**
 * Complete updater status
 */
interface UpdaterStatus {
  state: UpdateState;
  progress: UpdateProgress | null;
  available_update: UpdateInfo | null;
  error: string | null;
  configured: boolean;
}

/**
 * Custom manifest format
 */
interface CustomManifest {
  version: string;
  platforms: Record<string, PlatformAsset>;
  release_notes?: string;
  publish_date?: string;
}

interface PlatformAsset {
  url: string;
  sha256?: string;
  size?: number;
}
```

## Error Codes

| Code | Name | Description |
|------|------|-------------|
| 5000 | `GENERIC` | Generic updater error |
| 5001 | `CHECK_FAILED` | Failed to check for updates |
| 5002 | `DOWNLOAD_FAILED` | Failed to download update |
| 5003 | `VERIFICATION_FAILED` | Package verification failed (checksum mismatch) |
| 5004 | `INSTALL_FAILED` | Failed to install update |
| 5005 | `NO_UPDATE` | No update available |
| 5006 | `NETWORK_ERROR` | Network error during operation |
| 5007 | `INVALID_MANIFEST` | Invalid manifest format |
| 5008 | `PERMISSION_DENIED` | Permission denied |
| 5009 | `ALREADY_IN_PROGRESS` | Update already in progress |
| 5010 | `CANCELLED` | Update cancelled by user |
| 5011 | `NOT_CONFIGURED` | Updater not configured |
| 5012 | `INVALID_VERSION` | Invalid version format |

## Lifecycle Hooks

### onBefore(opName, callback)

```typescript
import { onBefore } from "runtime:updater";

onBefore("check", () => {
  console.log("Checking for updates...");
});

onBefore("download", () => {
  console.log("Starting download...");
});
```

### onAfter(opName, callback)

```typescript
import { onAfter } from "runtime:updater";

onAfter("check", (result) => {
  if (result) {
    analytics.track("update_available", { version: result.version });
  }
});

onAfter("install", () => {
  analytics.track("update_installed");
});
```

### onError(opName, callback)

```typescript
import { onError } from "runtime:updater";

onError("download", (error) => {
  console.error("Download failed:", error.message);
  showErrorNotification("Update download failed. Please try again.");
});

onError("verify", (error) => {
  console.error("Verification failed:", error.message);
  // Offer to retry download
});
```

**Available operation names:** `"info"`, `"echo"`, `"configureGithub"`, `"configureCustom"`, `"check"`, `"download"`, `"downloadProgress"`, `"cancel"`, `"verify"`, `"install"`, `"status"`, `"getCurrentVersion"`, `"getPendingUpdate"`

## Complete Examples

### Basic Update Check on Startup

```typescript
import { configure, check, formatBytes } from "runtime:updater";
import { dialog } from "runtime:window";

async function checkForUpdatesOnStartup() {
  try {
    await configure({
      source: { type: "github", owner: "myorg", repo: "myapp" },
      currentVersion: APP_VERSION,
    });

    const update = await check();
    if (update) {
      const button = await dialog.message({
        kind: "info",
        title: "Update Available",
        message: `Version ${update.version} is available (${formatBytes(update.size_bytes)}). Would you like to download it?`,
        buttons: ["Download", "Later"],
      });

      if (button === 0) {
        // User clicked "Download"
        openUpdateWindow();
      }
    }
  } catch (error) {
    console.error("Update check failed:", error);
    // Silent failure on startup - don't bother user
  }
}

// Check 30 seconds after app start
setTimeout(checkForUpdatesOnStartup, 30000);
```

### Update Manager with UI

```typescript
import {
  configure,
  check,
  download,
  verify,
  install,
  cancel,
  getProgress,
  getStatus,
  formatBytes,
  isDownloading,
} from "runtime:updater";

class UpdateManager {
  private updateInfo: UpdateInfo | null = null;
  private progressInterval: number | null = null;

  async initialize(currentVersion: string) {
    await configure({
      source: { type: "github", owner: "myorg", repo: "myapp" },
      currentVersion,
    });
  }

  async checkForUpdates(): Promise<UpdateInfo | null> {
    this.updateUI("checking");

    try {
      this.updateInfo = await check();

      if (this.updateInfo) {
        this.updateUI("update_available", {
          version: this.updateInfo.version,
          size: formatBytes(this.updateInfo.size_bytes),
          notes: this.updateInfo.release_notes,
        });
      } else {
        this.updateUI("up_to_date");
      }

      return this.updateInfo;
    } catch (error) {
      this.updateUI("error", { message: error.message });
      throw error;
    }
  }

  async startDownload() {
    if (!this.updateInfo) {
      throw new Error("No update available");
    }

    this.updateUI("downloading");

    // Start progress monitoring
    this.progressInterval = setInterval(() => {
      const progress = getProgress();
      this.updateUI("downloading", {
        percent: progress.percent,
        downloaded: formatBytes(progress.downloaded_bytes),
        total: formatBytes(progress.total_bytes),
      });
    }, 200) as unknown as number;

    try {
      await download();
      this.clearProgressInterval();

      // Verify
      this.updateUI("verifying");
      const verified = await verify();

      if (verified) {
        this.updateUI("ready_to_install");
      } else {
        this.updateUI("error", { message: "Verification failed" });
      }
    } catch (error) {
      this.clearProgressInterval();
      this.updateUI("error", { message: error.message });
      throw error;
    }
  }

  async cancelDownload() {
    if (isDownloading()) {
      await cancel();
      this.clearProgressInterval();
      this.updateUI("cancelled");
    }
  }

  async installUpdate() {
    this.updateUI("installing");

    try {
      await install();
      this.updateUI("complete");
      this.promptRestart();
    } catch (error) {
      this.updateUI("error", { message: error.message });
      throw error;
    }
  }

  private clearProgressInterval() {
    if (this.progressInterval) {
      clearInterval(this.progressInterval);
      this.progressInterval = null;
    }
  }

  private updateUI(state: string, data?: Record<string, unknown>) {
    // Send to renderer via IPC
    window.host.send("update:status", { state, ...data });
  }

  private promptRestart() {
    window.host.send("update:restart-prompt");
  }
}

export const updateManager = new UpdateManager();
```

### Custom Manifest Server Setup

```typescript
import { configureCustom, check, fullUpdate } from "runtime:updater";

// For self-hosted updates
async function setupSelfHostedUpdates() {
  await configureCustom({
    url: "https://updates.myapp.com/latest.json",
    currentVersion: APP_VERSION,
  });

  // The manifest at the URL should look like:
  // {
  //   "version": "2.0.0",
  //   "platforms": {
  //     "darwin-aarch64": {
  //       "url": "https://cdn.myapp.com/MyApp-2.0.0-arm64.dmg",
  //       "sha256": "a1b2c3d4e5f6...",
  //       "size": 45000000
  //     },
  //     "darwin-x64": { ... },
  //     "win32-x64": { ... },
  //     "linux-x64": { ... }
  //   },
  //   "release_notes": "# Version 2.0.0\n\n- New feature\n- Bug fixes",
  //   "publish_date": "2024-12-20T00:00:00Z"
  // }
}

// Silent background update
async function silentBackgroundUpdate() {
  await setupSelfHostedUpdates();

  const updated = await fullUpdate({
    onCheckComplete: (info) => {
      console.log(`Found update: ${info.version}`);
    },
    onProgress: (progress) => {
      // Update system tray icon or menu bar
      if (progress.percent % 10 === 0) {
        console.log(`Download: ${progress.percent}%`);
      }
    },
  });

  if (updated) {
    // Show subtle notification
    showNotification({
      title: "Update Ready",
      body: "A new version has been installed. Restart to apply.",
      actions: [{ title: "Restart Now" }],
    });
  }
}
```

### Periodic Update Checks

```typescript
import { configure, check, isUpdateAvailable } from "runtime:updater";

const UPDATE_CHECK_INTERVAL = 4 * 60 * 60 * 1000; // 4 hours

async function setupPeriodicUpdateChecks() {
  await configure({
    source: { type: "github", owner: "myorg", repo: "myapp" },
    currentVersion: APP_VERSION,
  });

  // Check immediately
  await checkAndNotify();

  // Then check periodically
  setInterval(checkAndNotify, UPDATE_CHECK_INTERVAL);
}

async function checkAndNotify() {
  try {
    const update = await check();

    if (update) {
      // Store that we have an update available
      localStorage.setItem("pendingUpdate", JSON.stringify({
        version: update.version,
        checkedAt: Date.now(),
      }));

      // Update app badge/indicator
      updateAppBadge(true);

      // Maybe show notification (but not too often)
      const lastNotified = localStorage.getItem("lastUpdateNotification");
      const dayAgo = Date.now() - 24 * 60 * 60 * 1000;

      if (!lastNotified || parseInt(lastNotified) < dayAgo) {
        showUpdateNotification(update);
        localStorage.setItem("lastUpdateNotification", Date.now().toString());
      }
    }
  } catch (error) {
    // Silent failure for background checks
    console.debug("Periodic update check failed:", error);
  }
}
```

## Best Practices

### Always Configure Before Operations

```typescript
// Good - configure once at startup
await configure({ source: { type: "github", owner: "myorg", repo: "myapp" }, currentVersion: "1.0.0" });

// Then use throughout app lifecycle
const update = await check();
```

### Handle Network Failures Gracefully

```typescript
async function safeCheck() {
  try {
    return await check();
  } catch (error) {
    if (error.code === 5006) {
      // Network error - user might be offline
      console.log("Offline - will check later");
      return null;
    }
    throw error;
  }
}
```

### Verify Before Installing

```typescript
// Good - always verify custom manifest downloads
const verified = await verify();
if (!verified) {
  throw new Error("Update package failed verification");
}
await install();
```

### Allow Users to Cancel

```typescript
// Provide cancel option during long downloads
downloadButton.disabled = true;
cancelButton.disabled = false;

cancelButton.onclick = () => cancel();

try {
  await download();
} finally {
  downloadButton.disabled = false;
  cancelButton.disabled = true;
}
```

### Don't Block App Startup

```typescript
// Good - check in background after app is ready
setTimeout(async () => {
  try {
    await checkForUpdatesOnStartup();
  } catch {
    // Silent failure
  }
}, 5000);

// Bad - blocking startup
await checkForUpdatesOnStartup(); // User waits...
```

## Aliases

The module provides convenient aliases:

```typescript
import {
  checkForUpdates,    // alias for check
  downloadUpdate,     // alias for download
  installUpdate,      // alias for install
  status,             // alias for getStatus
  progress,           // alias for getProgress
} from "runtime:updater";
```
