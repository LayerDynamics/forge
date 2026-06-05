---
title: "runtime:fs"
description: File system operations with capability-based access control.
slug: api/runtime-fs
---

The `runtime:fs` module provides file system operations with capability-based access control.

> **Implementation**: TypeScript types are auto-generated from Rust via [forge-weld](/docs/crates/forge-weld). See [ext_fs](/docs/crates/ext-fs) for implementation details.

## Capabilities

File system access must be declared in `manifest.app.toml` (use either `permissions.fs` or `capabilities.fs`):

```toml
[permissions.fs]
read = ["~/.myapp/*", "./data/*"]
write = ["~/.myapp/*"]
```

Glob patterns supported:
- `*` - matches any characters except `/`
- `**` - matches any characters including `/`
- `~` - expands to user's home directory

---

## Reading Files

### readTextFile(path)

Read a file as UTF-8 text:

```typescript
import { readTextFile } from "runtime:fs";

const content = await readTextFile("./config.json");
const config = JSON.parse(content);
```

### readBytes(path)

Read a file as raw bytes:

```typescript
import { readBytes } from "runtime:fs";

const data = await readBytes("./image.png");
// Returns: Uint8Array
```

---

## Writing Files

### writeTextFile(path, content)

Write UTF-8 text to a file:

```typescript
import { writeTextFile } from "runtime:fs";

await writeTextFile("./output.txt", "Hello, World!");
```

### writeBytes(path, content)

Write raw bytes to a file:

```typescript
import { writeBytes } from "runtime:fs";

const data = new Uint8Array([0x48, 0x65, 0x6c, 0x6c, 0x6f]);
await writeBytes("./binary.dat", data);
```

---

## Directory Operations

### readDir(path)

Read directory contents:

```typescript
import { readDir } from "runtime:fs";

const entries = await readDir("./src");
for (const entry of entries) {
  console.log(entry.name, entry.is_file ? "file" : "dir");
}
```

**Returns:**

```typescript
interface DirEntry {
  name: string;
  isFile: boolean;
  isDirectory: boolean;
  isSymlink: boolean;
}
```

### mkdir(path, options?)

Create a directory:

```typescript
import { mkdir } from "runtime:fs";

// Create single directory
await mkdir("./output");

// Create nested directories
await mkdir("./path/to/nested", { recursive: true });
```

---

## File Operations

### stat(path)

Get file/directory information:

```typescript
import { stat } from "runtime:fs";

const info = await stat("./file.txt");
console.log(info.size, info.is_file, info.readonly);
```

**Returns:**

```typescript
interface FileStat {
  isFile: boolean;
  isDirectory: boolean;
  isSymlink: boolean;
  size: number;
  mtime: number | null;    // modified (ms)
  atime: number | null;    // accessed (ms)
  birthtime: number | null; // created (ms)
  readonly: boolean;
}
```

### exists(path)

Check if a path exists:

```typescript
import { exists } from "runtime:fs";

if (await exists("./config.json")) {
  // Load config
}
```

### remove(path, options?)

Remove a file or directory:

```typescript
import { remove } from "runtime:fs";

// Remove file
await remove("./temp.txt");

// Remove directory recursively
await remove("./cache", { recursive: true });
```

### rename(from, to)

Rename or move a file/directory:

```typescript
import { rename } from "runtime:fs";

await rename("./old-name.txt", "./new-name.txt");
await rename("./file.txt", "./archive/file.txt");
```

### copy(from, to)

Copy a file:

```typescript
import { copy } from "runtime:fs";

await copy("./source.txt", "./destination.txt");
```

---

## Symbolic Links

### symlink(target, path)

Create a symbolic link:

```typescript
import { symlink } from "runtime:fs";

// Create symlink pointing to a file
await symlink("./actual-file.txt", "./link-to-file.txt");

// Create symlink pointing to a directory
await symlink("./actual-dir", "./link-to-dir");
```

### readLink(path)

Read the target of a symbolic link:

```typescript
import { readLink } from "runtime:fs";

const target = await readLink("./my-symlink");
console.log("Link points to:", target);
```

---

## Appending to Files

### appendTextFile(path, content)

Append UTF-8 text to a file:

```typescript
import { appendTextFile } from "runtime:fs";

await appendTextFile("./log.txt", "New log entry\n");
```

### appendBytes(path, content)

Append raw bytes to a file:

```typescript
import { appendBytes } from "runtime:fs";

const data = new Uint8Array([1, 2, 3, 4]);
await appendBytes("./data.bin", data);
```

---

## Advanced Operations

### metadata(path)

Get detailed file metadata including permissions:

```typescript
import { metadata } from "runtime:fs";

const meta = await metadata("./myfile.txt");
console.log({
  isFile: meta.isFile,
  isDir: meta.isDir,
  isSymlink: meta.isSymlink,
  size: meta.size,
  modifiedAt: meta.modifiedAt,
  accessedAt: meta.accessedAt,
  createdAt: meta.createdAt,
  permissions: meta.permissions, // Unix permissions (e.g., 0o644)
});
```

**FileMetadata type:**

```typescript
interface FileMetadata {
  isFile: boolean;
  isDir: boolean;
  isSymlink: boolean;
  size: number;
  readonly: boolean;
  createdAt: number | null;   // Unix timestamp (ms)
  modifiedAt: number | null;  // Unix timestamp (ms)
  accessedAt: number | null;  // Unix timestamp (ms)
  permissions: number | null; // Unix permissions (octal) or null on Windows
}
```

### realPath(path)

Resolve to canonical absolute path (follows symlinks):

```typescript
import { realPath } from "runtime:fs";

const canonical = await realPath("./my-symlink/../file.txt");
console.log("Canonical path:", canonical);
// => "/absolute/path/to/file.txt"
```

### tempFile(prefix?, suffix?)

Create a temporary file:

```typescript
import { tempFile } from "runtime:fs";

const temp = await tempFile("myapp-", ".tmp");
console.log("Temp file:", temp.path);

// Write to temp file
await writeTextFile(temp.path, "temporary data");

// Clean up when done
await remove(temp.path);
```

**TempFileInfo type:**

```typescript
interface TempFileInfo {
  path: string;  // Absolute path to temporary file
}
```

### tempDir(prefix?)

Create a temporary directory:

```typescript
import { tempDir } from "runtime:fs";

const temp = await tempDir("myapp-");
console.log("Temp directory:", temp.path);

// Use temp directory
await writeTextFile(`${temp.path}/data.txt`, "content");

// Clean up when done
await remove(temp.path, { recursive: true });
```

**TempDirInfo type:**

```typescript
interface TempDirInfo {
  path: string;  // Absolute path to temporary directory
}
```

---

## Lifecycle Hooks

Intercept filesystem operations with before/after/error hooks:

### onBefore(opName, handler)

Execute before an operation:

```typescript
import { onBefore } from "runtime:fs";

onBefore("readTextFile", (args) => {
  console.log("Reading file:", args[0]);
  // Optionally modify args or throw to prevent operation
});
```

### onAfter(opName, handler)

Execute after successful operation:

```typescript
import { onAfter } from "runtime:fs";

onAfter("writeTextFile", (args, result) => {
  console.log("Wrote file:", args[0]);
  // Optionally transform result
  return result;
});
```

### onError(opName, handler)

Execute when operation fails:

```typescript
import { onError } from "runtime:fs";

onError("readTextFile", (args, error) => {
  console.error("Failed to read:", args[0], error);
  // Optionally throw different error or return fallback value
});
```

### removeAllHooks(opName?)

Remove all hooks for an operation (or all operations if no name provided):

```typescript
import { removeAllHooks } from "runtime:fs";

// Remove all hooks for specific operation
removeAllHooks("readTextFile");

// Remove all hooks for all operations
removeAllHooks();
```

**Supported operations:**
`readTextFile`, `writeTextFile`, `readBytes`, `writeBytes`, `appendTextFile`, `appendBytes`, `stat`, `mkdir`, `remove`, `rename`, `copy`, `symlink`, `readLink`, `metadata`, `realPath`, `readDir`, `exists`

**Example - Validation:**

```typescript
import { onBefore, writeTextFile } from "runtime:fs";

// Prevent writes to protected directories
onBefore("writeTextFile", (args) => {
  const path = args[0];
  if (path.startsWith("/system/")) {
    throw new Error("Cannot write to system directory");
  }
});

await writeTextFile("/system/config.txt", "data"); // Throws error
```

---

## Handler System

Register custom named handlers for filesystem operations:

### registerHandler(name, handler)

Register a named handler:

```typescript
import { registerHandler } from "runtime:fs";

registerHandler("loadConfig", async (path: string) => {
  const content = await readTextFile(path);
  return JSON.parse(content);
});
```

### invokeHandler(name, ...args)

Invoke a handler by name:

```typescript
import { invokeHandler } from "runtime:fs";

const config = await invokeHandler("loadConfig", "./config.json");
console.log(config);
```

### listHandlers()

List all registered handlers:

```typescript
import { listHandlers } from "runtime:fs";

const handlers = listHandlers();
console.log("Registered handlers:", handlers); // => ["loadConfig", ...]
```

### hasHandler(name)

Check if a handler exists:

```typescript
import { hasHandler } from "runtime:fs";

if (hasHandler("loadConfig")) {
  const config = await invokeHandler("loadConfig", "./config.json");
}
```

### removeHandler(name)

Unregister a handler:

```typescript
import { removeHandler } from "runtime:fs";

removeHandler("loadConfig");
```

**Example - Plugin System:**

```typescript
import { registerHandler, invokeHandler } from "runtime:fs";

// Plugin registers handlers
registerHandler("loadJSON", async (path: string) => {
  const content = await readTextFile(path);
  return JSON.parse(content);
});

registerHandler("saveJSON", async (path: string, data: unknown) => {
  const content = JSON.stringify(data, null, 2);
  await writeTextFile(path, content);
});

// Application uses handlers dynamically
const data = await invokeHandler("loadJSON", "./data.json");
data.lastUpdated = Date.now();
await invokeHandler("saveJSON", "./data.json", data);
```

---

## File Watching

### watch(path)

Watch a file or directory for changes:

```typescript
import { watch } from "runtime:fs";

const watcher = await watch("./src");

// Using async iterator
for await (const event of watcher) {
  console.log(event.kind, event.paths);
}

// Using next() method
while (true) {
  const event = await watcher.next();
  if (!event) break;
  console.log(event);
}

// Clean up
await watcher.close();
```

**Event shape:**

```typescript
interface FileEvent {
  kind: string;    // "create", "modify", "remove", etc.
  paths: string[]; // Affected paths
}
```

**Watcher interface:**

```typescript
interface FileWatcher {
  id: number;
  next(): Promise<FileEvent | null>;
  [Symbol.asyncIterator](): AsyncIterableIterator<FileEvent>;
  close(): Promise<void>;
}
```

---

## Error Handling

All operations throw on error:

```typescript
import { readTextFile } from "runtime:fs";

try {
  const content = await readTextFile("./missing.txt");
} catch (error) {
  if (error.message.includes("not found")) {
    console.log("File does not exist");
  } else if (error.message.includes("permission")) {
    console.log("Access denied - check capabilities");
  }
}
```

---

## Complete Example

```typescript
import {
  readTextFile,
  writeTextFile,
  exists,
  mkdir,
  watch
} from "runtime:fs";
import { homeDir } from "runtime:sys";

// Config file path
const configPath = `${homeDir()}/.myapp/config.json`;

// Ensure directory exists
if (!await exists(`${homeDir()}/.myapp`)) {
  await mkdir(`${homeDir()}/.myapp`);
}

// Load or create config
let config;
if (await exists(configPath)) {
  const content = await readTextFile(configPath);
  config = JSON.parse(content);
} else {
  config = { theme: "dark", fontSize: 14 };
  await writeTextFile(configPath, JSON.stringify(config, null, 2));
}

// Watch for external changes
const watcher = await watch(configPath);
for await (const event of watcher) {
  if (event.kind === "modify") {
    const content = await readTextFile(configPath);
    config = JSON.parse(content);
    console.log("Config reloaded");
  }
}
```
