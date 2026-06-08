---
title: "runtime:app"
description: Application lifecycle management, metadata, and system integration for Forge applications.
slug: docs/api/runtime-app
---

The `runtime:app` module provides application lifecycle management, metadata access, and system integration capabilities for Forge applications.

> **Implementation**: TypeScript types are auto-generated from Rust via [forge-weld](/docs/crates/forge-weld). See [ext_app](/docs/crates/ext-app) for implementation details.

## Features

**Lifecycle Control**:
- Graceful shutdown and force exit
- Application relaunch
- Single instance management

**Metadata Access**:
- Version, name, and identifier
- System locale information
- Special path resolution (home, appData, documents, etc.)
- Packaged/development mode detection

**System Integration**:
- Window visibility control (focus, hide, show)
- Dock/taskbar badge count (macOS, Windows, Linux)
- Windows App User Model ID (taskbar grouping)

---

## Lifecycle Operations

### quit()

Quit the application gracefully.

Triggers cleanup handlers before exiting. This is the preferred way to exit your application as it allows resources to be properly cleaned up.

```typescript
import { quit } from "runtime:app";

// Graceful exit with cleanup
await quit();
```

**Use in menu:**

```typescript
import { quit } from "runtime:app";
import { menu } from "runtime:window";

menu.setAppMenu([
  {
    label: "File",
    submenu: [
      {
        label: "Quit",
        accelerator: "CmdOrCtrl+Q",
        click: async () => await quit()
      }
    ]
  }
]);
```

**Throws:**
- Error [8300] if quit fails

### exit(exitCode?)

Force exit the application immediately.

No cleanup handlers are called. Use this when you need to terminate immediately, such as in error conditions.

```typescript
import { exit } from "runtime:app";

// Force exit with default code (0)
exit();

// Force exit with error code
exit(1);
```

**Use in fatal error handler:**

```typescript
window.addEventListener("unhandledrejection", (event) => {
  console.error("Fatal error:", event.reason);
  exit(1);
});
```

**Parameters:**
- `exitCode` - Exit code (default: 0). Non-zero codes indicate errors.

**Throws:**
- Error [8301] if exit fails

### relaunch()

Relaunch the application.

The current instance will exit and a new instance will start. This is useful for applying updates or resetting application state.

```typescript
import { relaunch } from "runtime:app";

// Relaunch after update
async function applyUpdate() {
  await downloadUpdate();
  await installUpdate();
  await relaunch(); // Restart with new version
}
```

**Use in settings:**

```typescript
import { relaunch } from "runtime:app";

async function resetToDefaults() {
  await clear();  // Clear all storage
  await relaunch(); // Restart app
}
```

**Throws:**
- Error [8302] if relaunch fails

---

## Metadata

### getVersion()

Get the application version.

Returns the version string from `manifest.app.toml`.

```typescript
import { getVersion } from "runtime:app";

const version = getVersion();
console.log(`App version: ${version}`); // => "1.0.0"
```

**Use in about dialog:**

```typescript
import { getVersion, getName } from "runtime:app";

function showAboutDialog() {
  alert(`${getName()} v${getVersion()}\n© 2025 Your Company`);
}
```

**Returns:** Version string (e.g., "1.0.0", "2.1.0-beta")

**Throws:**
- Error [8303] if metadata cannot be retrieved

### getName()

Get the application name.

Returns the app name from `manifest.app.toml`.

```typescript
import { getName } from "runtime:app";

const appName = getName();
console.log(`App name: ${appName}`); // => "My Forge App"
```

**Use in window title:**

```typescript
import { getName } from "runtime:app";
import { createWindow } from "runtime:window";

const window = await createWindow({
  title: `${getName()} - Document Editor`,
  width: 800,
  height: 600
});
```

**Returns:** App name string

**Throws:**
- Error [8303] if metadata cannot be retrieved

### getIdentifier()

Get the application identifier (bundle ID).

Returns the unique identifier from `manifest.app.toml` (e.g., "com.example.app").

```typescript
import { getIdentifier } from "runtime:app";

const bundleId = getIdentifier();
console.log(`Bundle ID: ${bundleId}`); // => "com.example.myapp"
```

**Use in logging:**

```typescript
import { getIdentifier, getVersion } from "runtime:app";

function initLogger() {
  console.log(`Starting ${getIdentifier()} v${getVersion()}`);
}
```

**Returns:** Bundle identifier string (e.g., "com.example.app")

**Throws:**
- Error [8303] if metadata cannot be retrieved

### isPackaged()

Check if the application is running in packaged mode.

Returns `true` when running as a bundled application (`.app`, `.exe`, etc.), `false` when running in development mode (`forge dev`).

```typescript
import { isPackaged } from "runtime:app";

if (isPackaged()) {
  console.log("Running in production mode");
} else {
  console.log("Running in development mode");
}
```

**Use for conditional features:**

```typescript
import { isPackaged } from "runtime:app";

// Enable dev tools only in development
const showDevTools = !isPackaged();

const window = await createWindow({
  title: "My App",
  devtools: showDevTools
});
```

**Use for different paths:**

```typescript
import { isPackaged } from "runtime:app";

const resourcePath = isPackaged()
  ? getPath("resources")
  : "./resources";
```

**Returns:** `true` if bundled, `false` if in development

**Throws:**
- Error [8303] if state cannot be determined

### getLocale()

Get the system locale information.

Returns information about the user's system language and region.

```typescript
import { getLocale } from "runtime:app";

const locale = getLocale();
console.log(`Language: ${locale.language}`);    // => "en"
console.log(`Country: ${locale.country}`);      // => "US"
console.log(`Full locale: ${locale.locale}`);   // => "en-US"
```

**Use for internationalization:**

```typescript
import { getLocale } from "runtime:app";

const { language } = getLocale();
const translations = await import(`./i18n/${language}.json`);
```

**Returns:** `LocaleInfo` object with:
- `language` - Language code (e.g., "en", "es", "fr")
- `country` - Country code or `null` (e.g., "US", "GB", "MX")
- `locale` - Full locale string (e.g., "en-US", "es-MX", "fr-FR")

**Throws:**
- Error [8303] if locale cannot be determined

---

## Special Paths

### getPath(pathType)

Get a special system or application path.

Returns platform-appropriate paths for common directories.

```typescript
import { getPath } from "runtime:app";

// Common paths
const home = getPath("home");         // User's home directory
const appData = getPath("appData");   // Application data
const documents = getPath("documents"); // Documents folder
const downloads = getPath("downloads"); // Downloads folder
const desktop = getPath("desktop");    // Desktop folder
```

**All supported path types:**

```typescript
const paths = {
  home: getPath("home"),         // ~/
  appData: getPath("appData"),   // Platform app data dir
  documents: getPath("documents"), // ~/Documents
  downloads: getPath("downloads"), // ~/Downloads
  desktop: getPath("desktop"),    // ~/Desktop
  music: getPath("music"),        // ~/Music
  pictures: getPath("pictures"),  // ~/Pictures
  videos: getPath("videos"),      // ~/Videos
  temp: getPath("temp"),          // Temp directory
  exe: getPath("exe"),            // Executable path
  resources: getPath("resources"), // App resources
  logs: getPath("logs"),          // Logs directory
  cache: getPath("cache")         // Cache directory
};
```

**Use for config file:**

```typescript
import { getPath } from "runtime:app";
import { exists, readTextFile, writeTextFile } from "runtime:fs";

const configPath = `${getPath("appData")}/config.json`;

async function loadConfig() {
  if (await exists(configPath)) {
    const content = await readTextFile(configPath);
    return JSON.parse(content);
  }
  return getDefaultConfig();
}

async function saveConfig(config: Config) {
  await writeTextFile(configPath, JSON.stringify(config, null, 2));
}
```

**Parameters:**
- `pathType` - Type of path to retrieve (see PathType below)

**Returns:** Absolute path string

**Throws:**
- Error [8304] if path cannot be determined
- Error [8311] if path type is invalid

**PathType values:**
- `"home"` - User's home directory
- `"appData"` - Application data directory
- `"documents"` - Documents folder
- `"downloads"` - Downloads folder
- `"desktop"` - Desktop folder
- `"music"` - Music folder
- `"pictures"` - Pictures folder
- `"videos"` - Videos folder
- `"temp"` - Temporary directory
- `"exe"` - Executable path
- `"resources"` - Application resources
- `"logs"` - Logs directory
- `"cache"` - Cache directory

---

## Single Instance Management

### requestSingleInstanceLock()

Request a single instance lock.

Prevents multiple instances of the application from running simultaneously. Returns `true` if the lock was acquired, `false` if another instance holds it.

```typescript
import { requestSingleInstanceLock, focus } from "runtime:app";

// Ensure only one instance runs
if (!await requestSingleInstanceLock()) {
  console.log("App is already running");
  exit(0);
}
```

**With focus on existing instance:**

```typescript
import { requestSingleInstanceLock, focus, exit } from "runtime:app";

const gotLock = await requestSingleInstanceLock();

if (!gotLock) {
  // Another instance is running - focus it
  await focus();
  exit(0);
} else {
  // We're the only instance - start normally
  startApp();
}
```

**Returns:** `true` if lock acquired, `false` if another instance holds it

**Throws:**
- Error [8305] if lock operation fails

### releaseSingleInstanceLock()

Release the single instance lock.

Call this before exiting to allow another instance to start. Usually not necessary as the lock is automatically released on exit.

```typescript
import { releaseSingleInstanceLock } from "runtime:app";

// Release lock before exit
await releaseSingleInstanceLock();
```

**Throws:**
- Error [8305] if lock operation fails

---

## Window Control

### focus()

Bring the application to the foreground.

Makes the app's windows visible and focused. Useful when another instance tries to launch.

```typescript
import { focus } from "runtime:app";

// Bring app to foreground
await focus();
```

**Use with single instance:**

```typescript
import { requestSingleInstanceLock, focus } from "runtime:app";

if (!await requestSingleInstanceLock()) {
  await focus(); // Focus the existing instance
  exit(0);
}
```

**Throws:**
- Error [8306] if focus fails

### hide()

Hide all application windows.

On macOS, this hides the application (⌘H). On other platforms, it minimizes all windows.

```typescript
import { hide } from "runtime:app";

// Hide all windows
await hide();
```

**Use in menu:**

```typescript
import { hide } from "runtime:app";
import { menu } from "runtime:window";

menu.setAppMenu([
  {
    label: "Window",
    submenu: [
      {
        label: "Hide",
        accelerator: "CmdOrCtrl+H",
        click: async () => await hide()
      }
    ]
  }
]);
```

**Throws:**
- Error [8307] if hide fails

### show()

Show all application windows.

Restores hidden or minimized windows.

```typescript
import { show } from "runtime:app";

// Show all windows
await show();
```

**Use after processing:**

```typescript
import { hide, show } from "runtime:app";

// Hide during long operation
await hide();
await performLongTask();
await show(); // Show when done
```

**Throws:**
- Error [8308] if show fails

---

## System Integration

### setBadgeCount(count?)

Set the dock/taskbar badge count.

Displays a badge with a number on the app icon. Useful for showing unread counts, notifications, etc.

- **macOS**: Shows badge on dock icon
- **Windows**: Shows badge on taskbar icon
- **Linux**: Shows badge on app indicator (Unity, GNOME)

```typescript
import { setBadgeCount } from "runtime:app";

// Set badge to 5
await setBadgeCount(5);

// Clear badge
await setBadgeCount(null);
await setBadgeCount(0);
await setBadgeCount();
```

**Use for notifications:**

```typescript
import { setBadgeCount } from "runtime:app";

let unreadCount = 0;

async function addNotification() {
  unreadCount++;
  await setBadgeCount(unreadCount);
}

async function clearNotifications() {
  unreadCount = 0;
  await setBadgeCount(0);
}
```

**Parameters:**
- `count` - Badge count to display (number), or `null`/`undefined` to clear

**Throws:**
- Error [8309] if badge operation fails
- Error [8313] if not supported on platform

### setUserModelId(appId)

Set the Windows App User Model ID.

Used for taskbar grouping on Windows. This ensures your app's windows group correctly in the taskbar.

**Windows only** - No effect on other platforms.

```typescript
import { setUserModelId, getIdentifier } from "runtime:app";

// Set Windows taskbar grouping ID
setUserModelId(getIdentifier());
```

**Use during initialization:**

```typescript
import { setUserModelId } from "runtime:app";

// Windows-specific setup
if (navigator.platform.startsWith("Win")) {
  setUserModelId("com.example.myapp");
}
```

**Parameters:**
- `appId` - Application User Model ID (e.g., "com.example.app")

**Throws:**
- Error [8310] if operation fails

---

## Type Definitions

```typescript
interface LocaleInfo {
  /** Language code (e.g., "en") */
  language: string;
  /** Country code (e.g., "US") or null */
  country: string | null;
  /** Full locale string (e.g., "en-US") */
  locale: string;
}

type PathType =
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
```

---

## Lifecycle Hooks

Intercept app operations with before/after/error hooks:

### onBefore(opName, handler)

Execute before an operation:

```typescript
import { onBefore } from "runtime:app";

onBefore("quit", (args) => {
  console.log("App is quitting...");
  // Optionally throw to prevent quit
});
```

### onAfter(opName, handler)

Execute after successful operation:

```typescript
import { onAfter } from "runtime:app";

onAfter("quit", (result, args) => {
  console.log("App quit successfully");
});
```

### onError(opName, handler)

Execute when operation fails:

```typescript
import { onError } from "runtime:app";

onError("quit", (error, args) => {
  console.error("Failed to quit:", error);
});
```

### removeAllHooks(opName?)

Remove all hooks for an operation (or all operations if no name provided):

```typescript
import { removeAllHooks } from "runtime:app";

// Remove all hooks for specific operation
removeAllHooks("quit");

// Remove all hooks for all operations
removeAllHooks();
```

**Supported operations:**
`quit`, `exit`, `relaunch`, `getVersion`, `getName`, `getIdentifier`, `getPath`, `isPackaged`, `getLocale`, `requestSingleInstanceLock`, `releaseSingleInstanceLock`, `focus`, `hide`, `show`, `setBadgeCount`, `setUserModelId`

---

## Handler System

Register custom named handlers for app operations:

### registerHandler(name, handler)

Register a named handler:

```typescript
import { registerHandler } from "runtime:app";

registerHandler("checkUpdates", async () => {
  const currentVersion = getVersion();
  const latestVersion = await fetchLatestVersion();
  return latestVersion !== currentVersion;
});
```

### invokeHandler(name, ...args)

Invoke a handler by name:

```typescript
import { invokeHandler } from "runtime:app";

const hasUpdate = await invokeHandler("checkUpdates");
if (hasUpdate) {
  console.log("Update available!");
}
```

### listHandlers()

List all registered handlers:

```typescript
import { listHandlers } from "runtime:app";

const handlers = listHandlers();
console.log("Registered handlers:", handlers);
```

### hasHandler(name)

Check if a handler exists:

```typescript
import { hasHandler } from "runtime:app";

if (hasHandler("checkUpdates")) {
  const hasUpdate = await invokeHandler("checkUpdates");
}
```

### removeHandler(name)

Unregister a handler:

```typescript
import { removeHandler } from "runtime:app";

removeHandler("checkUpdates");
```

---

## Error Handling

All operations throw on error:

```typescript
import { quit } from "runtime:app";

try {
  await quit();
} catch (error) {
  if (error.message.includes("8300")) {
    console.log("Failed to quit gracefully");
  }
}
```

---

## Error Codes

| Code | Error | Description |
|------|-------|-------------|
| `8300` | QuitFailed | Failed to quit application |
| `8301` | ExitFailed | Failed to exit application |
| `8302` | RelaunchFailed | Failed to relaunch application |
| `8303` | InfoFailed | Failed to get app info/metadata |
| `8304` | PathFailed | Failed to get requested path |
| `8305` | LockFailed | Single instance lock operation failed |
| `8306` | FocusFailed | Failed to focus application |
| `8307` | HideFailed | Failed to hide application |
| `8308` | ShowFailed | Failed to show application |
| `8309` | BadgeFailed | Failed to set badge count |
| `8310` | UserModelIdFailed | Failed to set Windows User Model ID |
| `8311` | InvalidPathType | Invalid path type specified |
| `8312` | PermissionDenied | Permission denied for operation |
| `8313` | NotSupported | Operation not supported on this platform |
| `8314` | NotInitialized | App state not initialized |

---

## Complete Example

```typescript
import {
  getName,
  getVersion,
  getIdentifier,
  getLocale,
  getPath,
  isPackaged,
  requestSingleInstanceLock,
  focus,
  exit,
  quit,
  setBadgeCount
} from "runtime:app";
import { createWindow, menu } from "runtime:window";

// Single instance check
if (!await requestSingleInstanceLock()) {
  console.log("App already running");
  await focus(); // Focus existing instance
  exit(0);
}

// Initialize app
const appName = getName();
const version = getVersion();
const locale = getLocale();

console.log(`Starting ${appName} v${version}`);
console.log(`Locale: ${locale.locale}`);
console.log(`Packaged: ${isPackaged()}`);

// Create main window
const window = await createWindow({
  title: `${appName} - Main Window`,
  width: 900,
  height: 600
});

// Set up app menu
menu.setAppMenu([
  {
    label: "File",
    submenu: [
      {
        label: `About ${appName}`,
        click: () => {
          alert(`${appName}\nVersion: ${version}\nID: ${getIdentifier()}`);
        }
      },
      { type: "separator" },
      {
        label: "Quit",
        accelerator: "CmdOrCtrl+Q",
        click: async () => await quit()
      }
    ]
  },
  {
    label: "View",
    submenu: [
      {
        label: "Dev Tools",
        accelerator: "CmdOrCtrl+Shift+I",
        click: () => window.openDevTools(),
        visible: !isPackaged() // Only show in development
      }
    ]
  }
]);

// Set badge for notifications
let notificationCount = 0;
async function addNotification() {
  notificationCount++;
  await setBadgeCount(notificationCount);
}

// Save state on quit
onBefore("quit", async () => {
  const configPath = `${getPath("appData")}/config.json`;
  await saveConfig(configPath, {
    version,
    lastRun: new Date().toISOString(),
    windowBounds: window.getBounds()
  });
});

console.log(`${appName} started successfully`);
```

## Convenience Exports

The module also exports convenience aliases:

```typescript
import { version, name, identifier } from "runtime:app";

// Equivalent to:
// const version = getVersion();
// const name = getName();
// const identifier = getIdentifier();
```
