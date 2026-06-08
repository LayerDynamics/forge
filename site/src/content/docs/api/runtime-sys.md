---
title: "runtime:sys"
description: System-level operations including environment, clipboard, notifications, and system information.
slug: docs/api/runtime-sys
---

The `runtime:sys` module provides system-level operations including environment, clipboard, notifications, and system information.

> **Implementation**: TypeScript types are auto-generated from Rust via [forge-weld](/docs/crates/forge-weld). See [ext_sys](/docs/crates/ext-sys) for implementation details.

## Capabilities

Some operations require capability declarations:

```toml
[capabilities.sys]
clipboard = true
notifications = true
```

---

## System Information

### info()

Get system information (synchronous):

```typescript
import { info } from "runtime:sys";

const sysInfo = info();
console.log(sysInfo.os);        // "macos", "windows", "linux"
console.log(sysInfo.arch);      // "x86_64", "aarch64"
console.log(sysInfo.hostname);  // "my-computer" or null
console.log(sysInfo.cpuCount);  // 8
```

**Returns:**

```typescript
interface SystemInfo {
  os: string;
  arch: string;
  hostname: string | null;
  cpuCount: number;
}
```

### powerInfo()

Get battery/power information:

```typescript
import { powerInfo } from "runtime:sys";

const power = await powerInfo();

console.log(power.state);       // "charging" | "discharging" | "full" | "empty" | "unknown"
console.log(power.percentage);  // Battery % (null if unavailable)
console.log(power.timeToFull);  // Seconds until full (or null)
console.log(power.timeToEmpty); // Seconds until empty (or null)
```

**Returns:**

```typescript
interface PowerInfo {
  state: "charging" | "discharging" | "full" | "empty" | "unknown";
  percentage: number | null;
  timeToFull: number | null;
  timeToEmpty: number | null;
}
```

---

## Environment

### getEnv(key)

Get an environment variable:

```typescript
import { getEnv } from "runtime:sys";

const home = getEnv("HOME");
const path = getEnv("PATH");
const custom = getEnv("MY_VAR");  // null if not set
```

### setEnv(key, value)

Set an environment variable:

```typescript
import { setEnv } from "runtime:sys";

setEnv("MY_APP_DEBUG", "true");
```

### getAllEnv() and deleteEnv(key)

Read or remove environment variables in bulk:

```typescript
import { getAllEnv, deleteEnv } from "runtime:sys";

const all = getAllEnv();         // { PATH: "...", HOME: "...", ... }
deleteEnv("MY_APP_DEBUG");       // Removes a variable
```

### cwd()

Get the current working directory:

```typescript
import { cwd } from "runtime:sys";

const currentDir = cwd();
console.log(currentDir);  // "/Users/name/projects/myapp"
```

### homeDir()

Get the user's home directory:

```typescript
import { homeDir } from "runtime:sys";

const home = homeDir();
console.log(home);  // "/Users/name" or "C:\Users\name" or null
```

### tempDir()

Get the system's temporary directory:

```typescript
import { tempDir } from "runtime:sys";

const temp = tempDir();
console.log(temp);  // "/tmp" or "C:\Users\name\AppData\Local\Temp"
```

### locale()

Get locale information:

```typescript
import { locale } from "runtime:sys";

const loc = locale();
console.log(loc.language); // "en"
console.log(loc.locale);   // "en-US"
```

### appPaths()

Get common application directories (platform-specific):

```typescript
import { appPaths } from "runtime:sys";

const paths = appPaths();
console.log(paths.documents); // e.g., "/Users/alex/Documents"
console.log(paths.cache);     // e.g., "/Users/alex/Library/Caches/MyApp"
```

---

## Clipboard

**Requires capability:** `capabilities.sys.clipboard = true`

### clipboard.read()

Read text from the clipboard:

```typescript
import { clipboard } from "runtime:sys";

const text = await clipboard.read();
console.log("Clipboard contains:", text);
```

### clipboard.write(text)

Write text to the clipboard:

```typescript
import { clipboard } from "runtime:sys";

await clipboard.write("Hello, World!");
```

---

## Notifications

**Requires capability:** `capabilities.sys.notifications = true`

### notify(title, body?)

Show a simple system notification:

```typescript
import { notify } from "runtime:sys";

await notify("Download Complete", "Your file has been downloaded.");
await notify("Alert");  // Title only
```

### notifyExt(options)

Show a notification with extended options:

```typescript
import { notifyExt } from "runtime:sys";

await notifyExt({
  title: "New Message",
  body: "You have a new message from John",
  subtitle: "Messages",
  sound: true
});
```

**Options:**

```typescript
interface NotifyOptions {
  title: string;
  body?: string;
  subtitle?: string;
  icon?: string;   // Path to an icon image
  sound?: boolean;
}
```

---

## Complete Example

```typescript
import {
  info,
  homeDir,
  clipboard,
  notify,
  powerInfo,
  locale,
  appPaths,
  getAllEnv
} from "runtime:sys";
import { writeTextFile } from "runtime:fs";

// System diagnostics
async function getDiagnostics() {
  const sysInfo = info();
  const loc = locale();
  const paths = appPaths();
  const power = await powerInfo();

  const report = {
    system: {
      os: sysInfo.os,
      arch: sysInfo.arch,
      hostname: sysInfo.hostname,
      cpus: sysInfo.cpuCount,
      locale: loc.locale
    },
    paths,
    power: {
      state: power.state,
      percentage: power.percentage,
      timeToFull: power.timeToFull,
      timeToEmpty: power.timeToEmpty
    },
    timestamp: new Date().toISOString()
  };

  return report;
}

// Save diagnostics to file
async function saveDiagnostics() {
  const report = await getDiagnostics();
  const path = `${homeDir()}/diagnostics.json`;

  await writeTextFile(path, JSON.stringify(report, null, 2));
  await notify("Diagnostics Saved", `Report saved to ${path}`);
}

// Copy system info to clipboard
async function copySystemInfo() {
  const sysInfo = info();
  const text = `${sysInfo.os} ${sysInfo.arch} (${sysInfo.cpuCount} CPUs)`;

  await clipboard.write(text);
  await notify("Copied", "System info copied to clipboard");
}
```

---

## Error Codes

System operations use structured error codes for precise error handling:

| Code | Name | Description |
|------|------|-------------|
| 2000 | Io | Generic I/O error |
| 2001 | PermissionDenied | Operation not allowed by capabilities |
| 2002 | NotSupported | Feature not supported on this platform |
| 2003 | Clipboard | Clipboard access failed |
| 2004 | Notification | Notification delivery failed |
| 2005 | Power | Battery/power information unavailable |

### Error Handling Example

```typescript
import { clipboard, notify } from "runtime:sys";

try {
  const text = await clipboard.read();
} catch (error) {
  if (error.message.includes("[2001]")) {
    console.error("Clipboard access denied - check capabilities.sys.clipboard");
  } else if (error.message.includes("[2003]")) {
    console.error("Clipboard error - may be empty or inaccessible");
  }
}

try {
  await notify("Hello", "World");
} catch (error) {
  if (error.message.includes("[2001]")) {
    console.error("Notifications denied - check capabilities.sys.notifications");
  }
}
```
