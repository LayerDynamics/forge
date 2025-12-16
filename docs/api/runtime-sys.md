# runtime:sys API Reference

The `runtime:sys` module provides system-level operations including environment, clipboard, notifications, and system information.

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
console.log(sysInfo.platform);  // "darwin", "win32", "linux"
console.log(sysInfo.cpu_count); // 8
```

**Returns:**

```typescript
interface SystemInfo {
  os: string;
  arch: string;
  hostname: string | null;
  platform: string;
  cpu_count: number;
}
```

### powerInfo()

Get battery/power information:

```typescript
import { powerInfo } from "runtime:sys";

const power = await powerInfo();

if (power.has_battery) {
  const battery = power.batteries[0];
  console.log(`Battery: ${battery.charge_percent}%`);
  console.log(`Status: ${battery.state}`);  // "charging", "discharging", etc.
  console.log(`AC Connected: ${power.ac_connected}`);
}
```

**Returns:**

```typescript
interface PowerInfo {
  has_battery: boolean;
  batteries: BatteryInfo[];
  ac_connected: boolean;
}

interface BatteryInfo {
  charge_percent: number;
  state: "charging" | "discharging" | "full" | "empty" | "unknown";
  time_to_full_secs?: number;
  time_to_empty_secs?: number;
  health_percent?: number;
  cycle_count?: number;
  temperature_celsius?: number;
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
  powerInfo
} from "runtime:sys";
import { writeTextFile } from "runtime:fs";

// System diagnostics
async function getDiagnostics() {
  const sysInfo = info();
  const power = await powerInfo();

  const report = {
    system: {
      os: sysInfo.os,
      arch: sysInfo.arch,
      hostname: sysInfo.hostname,
      cpus: sysInfo.cpu_count
    },
    power: power.has_battery ? {
      batteryPercent: power.batteries[0]?.charge_percent,
      charging: power.ac_connected
    } : null,
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
  const text = `${sysInfo.os} ${sysInfo.arch} (${sysInfo.cpu_count} CPUs)`;

  await clipboard.write(text);
  await notify("Copied", "System info copied to clipboard");
}
```
