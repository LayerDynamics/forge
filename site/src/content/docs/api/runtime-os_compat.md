---
title: "runtime:os_compat"
description: "Operating system compatibility information for cross-platform Forge applications"
slug: api/runtime-os_compat
---

Operating system compatibility utilities for Forge applications. Query runtime environment information including OS type, architecture, and platform-specific path separators.

> **Implementation**: TypeScript types are auto-generated from Rust via [forge-weld](/docs/crates/forge-weld). See [ext_os_compat](/docs/crates/ext-os-compat) for implementation details.

## Features

- Operating system detection (Windows, macOS, Linux)
- CPU architecture information (x64, arm64, etc.)
- OS family classification (unix, windows)
- Platform-specific path separators
- Environment variable separators
- Temporary directory paths
- Home directory detection

## Import

```typescript
import {
  // Functions
  info,
  pathSep,
  // Types
  type OsInfo,
} from "runtime:os_compat";
```

## API Reference

### info()

Get comprehensive operating system information.

**Returns:** `OsInfo`

**Example:**

```typescript
import { info } from "runtime:os_compat";

const osInfo = info();
console.log(`OS: ${osInfo.os}`);         // "darwin", "windows", "linux"
console.log(`Arch: ${osInfo.arch}`);     // "x86_64", "aarch64"
console.log(`Family: ${osInfo.family}`); // "unix", "windows"
```

---

### pathSep()

Get the platform-specific path separator.

**Returns:** `string` - `"/"` on Unix-like systems, `"\\"` on Windows

**Example:**

```typescript
import { pathSep } from "runtime:os_compat";

const sep = pathSep();
console.log(`Path separator: ${sep}`);

// Build cross-platform paths
const path = ["home", "user", "documents"].join(sep);
```

## Type Definitions

### OsInfo

Complete operating system information.

```typescript
interface OsInfo {
  /** Operating system name: "darwin", "windows", "linux" */
  os: string;

  /** CPU architecture: "x86_64", "aarch64", etc. */
  arch: string;

  /** OS family: "unix" or "windows" */
  family: string;

  /** Path separator: "/" or "\\" */
  path_sep: string;

  /** Environment variable separator: ":" (Unix) or ";" (Windows) */
  env_sep: string;

  /** System temporary directory path */
  tmp_dir: string;

  /** User home directory path, or null if not available */
  home_dir: string | null;
}
```

## Platform Values

### Operating System (`os`)

| Value | Description |
|-------|-------------|
| `"darwin"` | macOS |
| `"windows"` | Windows |
| `"linux"` | Linux |

### Architecture (`arch`)

| Value | Description |
|-------|-------------|
| `"x86_64"` | 64-bit Intel/AMD |
| `"aarch64"` | 64-bit ARM (Apple Silicon, ARM servers) |
| `"x86"` | 32-bit Intel/AMD |
| `"arm"` | 32-bit ARM |

### Family (`family`)

| Value | Platforms |
|-------|-----------|
| `"unix"` | macOS, Linux, BSD |
| `"windows"` | Windows |

### Path Separators

| Platform | `path_sep` | `env_sep` |
|----------|------------|-----------|
| Unix | `/` | `:` |
| Windows | `\` | `;` |

## Lifecycle Hooks

### onBefore(opName, callback)

```typescript
import { onBefore } from "runtime:os_compat";

const unsubscribe = onBefore("compatInfo", () => {
  console.log("Querying OS info...");
});
```

### onAfter(opName, callback)

```typescript
import { onAfter } from "runtime:os_compat";

onAfter("compatInfo", () => {
  console.log("OS info retrieved");
});
```

**Available operation names:** `"compatInfo"`, `"compatPathSep"`

## Complete Examples

### Cross-Platform Configuration

```typescript
import { info } from "runtime:os_compat";

function getConfigDir(): string {
  const { os, home_dir } = info();

  if (!home_dir) {
    throw new Error("Home directory not available");
  }

  switch (os) {
    case "darwin":
      return `${home_dir}/Library/Application Support/MyApp`;
    case "windows":
      return `${home_dir}\\AppData\\Roaming\\MyApp`;
    case "linux":
      return `${home_dir}/.config/myapp`;
    default:
      return `${home_dir}/.myapp`;
  }
}

const configDir = getConfigDir();
console.log(`Config directory: ${configDir}`);
```

### Platform-Specific Features

```typescript
import { info } from "runtime:os_compat";

function getPlatformFeatures(): string[] {
  const { os, family } = info();
  const features: string[] = [];

  if (family === "unix") {
    features.push("unix-permissions", "symlinks", "signals");
  }

  if (os === "darwin") {
    features.push("applescript", "dock", "notarization");
  } else if (os === "windows") {
    features.push("registry", "msix", "taskbar");
  } else if (os === "linux") {
    features.push("xdg", "appimage", "desktop-entries");
  }

  return features;
}

console.log("Platform features:", getPlatformFeatures());
```

### Environment Path Builder

```typescript
import { info } from "runtime:os_compat";

function buildEnvPath(dirs: string[]): string {
  const { env_sep } = info();
  return dirs.join(env_sep);
}

// Build PATH-style environment variable
const newPath = buildEnvPath([
  "/usr/local/bin",
  "/usr/bin",
  "/bin",
]);

console.log(`PATH: ${newPath}`);
// Unix: "/usr/local/bin:/usr/bin:/bin"
// Windows: "/usr/local/bin;/usr/bin;/bin"
```

### Architecture-Specific Binary Selection

```typescript
import { info } from "runtime:os_compat";

function getBinaryName(baseName: string): string {
  const { os, arch } = info();

  let suffix = "";

  // OS-specific extension
  if (os === "windows") {
    suffix = ".exe";
  }

  // Architecture suffix for multi-arch distributions
  const archSuffix = arch === "aarch64" ? "-arm64" : "-x64";

  return `${baseName}${archSuffix}${suffix}`;
}

const binary = getBinaryName("myapp");
// macOS ARM: "myapp-arm64"
// macOS Intel: "myapp-x64"
// Windows ARM: "myapp-arm64.exe"
// Windows Intel: "myapp-x64.exe"
```

### Platform Information Logger

```typescript
import { info } from "runtime:os_compat";

function logPlatformInfo(): void {
  const osInfo = info();

  console.log("=== Platform Information ===");
  console.log(`Operating System: ${osInfo.os}`);
  console.log(`Architecture: ${osInfo.arch}`);
  console.log(`Family: ${osInfo.family}`);
  console.log(`Path Separator: "${osInfo.path_sep}"`);
  console.log(`Env Separator: "${osInfo.env_sep}"`);
  console.log(`Temp Directory: ${osInfo.tmp_dir}`);
  console.log(`Home Directory: ${osInfo.home_dir ?? "(not available)"}`);
}

logPlatformInfo();
```

### Conditional Platform Logic

```typescript
import { info } from "runtime:os_compat";

class PlatformUtils {
  private osInfo = info();

  get isMac(): boolean {
    return this.osInfo.os === "darwin";
  }

  get isWindows(): boolean {
    return this.osInfo.os === "windows";
  }

  get isLinux(): boolean {
    return this.osInfo.os === "linux";
  }

  get isUnix(): boolean {
    return this.osInfo.family === "unix";
  }

  get isArm(): boolean {
    return this.osInfo.arch === "aarch64" || this.osInfo.arch === "arm";
  }

  get tempDir(): string {
    return this.osInfo.tmp_dir;
  }

  get homeDir(): string {
    if (!this.osInfo.home_dir) {
      throw new Error("Home directory not available");
    }
    return this.osInfo.home_dir;
  }

  joinPath(...parts: string[]): string {
    return parts.join(this.osInfo.path_sep);
  }
}

// Usage
const platform = new PlatformUtils();

if (platform.isMac) {
  console.log("Running on macOS");
  if (platform.isArm) {
    console.log("Apple Silicon detected");
  }
}

const tempFile = platform.joinPath(platform.tempDir, "myapp", "cache.tmp");
console.log(`Temp file: ${tempFile}`);
```

## Best Practices

### Cache OS Info

```typescript
// Good - query once and cache
const osInfo = info();
function getOs() { return osInfo.os; }

// Avoid - querying repeatedly in hot paths
function processItem() {
  if (info().os === "windows") { // Called for each item
    // ...
  }
}
```

### Use Family for Broad Compatibility

```typescript
import { info } from "runtime:os_compat";

const { family } = info();

// Good - covers macOS, Linux, BSD, etc.
if (family === "unix") {
  // Unix-like behavior
}

// More specific when needed
const { os } = info();
if (os === "darwin") {
  // macOS-specific behavior
}
```

### Handle Missing Home Directory

```typescript
import { info } from "runtime:os_compat";

const { home_dir, tmp_dir } = info();

// Fallback strategy
const userDir = home_dir ?? tmp_dir;
const configPath = `${userDir}/.myapp/config.json`;
```
