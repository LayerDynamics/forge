# host:fs API Reference

The `host:fs` module provides file system operations with capability-based access control.

## Capabilities

File system access must be declared in `manifest.app.toml`:

```toml
[capabilities.fs]
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
import { readTextFile } from "host:fs";

const content = await readTextFile("./config.json");
const config = JSON.parse(content);
```

### readBytes(path)

Read a file as raw bytes:

```typescript
import { readBytes } from "host:fs";

const data = await readBytes("./image.png");
// Returns: Uint8Array
```

---

## Writing Files

### writeTextFile(path, content)

Write UTF-8 text to a file:

```typescript
import { writeTextFile } from "host:fs";

await writeTextFile("./output.txt", "Hello, World!");
```

### writeBytes(path, content)

Write raw bytes to a file:

```typescript
import { writeBytes } from "host:fs";

const data = new Uint8Array([0x48, 0x65, 0x6c, 0x6c, 0x6f]);
await writeBytes("./binary.dat", data);
```

---

## Directory Operations

### readDir(path)

Read directory contents:

```typescript
import { readDir } from "host:fs";

const entries = await readDir("./src");
for (const entry of entries) {
  console.log(entry.name, entry.is_file ? "file" : "dir");
}
```

**Returns:**

```typescript
interface DirEntry {
  name: string;
  is_file: boolean;
  is_dir: boolean;
}
```

### mkdir(path, options?)

Create a directory:

```typescript
import { mkdir } from "host:fs";

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
import { stat } from "host:fs";

const info = await stat("./file.txt");
console.log(info.size, info.is_file, info.readonly);
```

**Returns:**

```typescript
interface FileStat {
  is_file: boolean;
  is_dir: boolean;
  size: number;
  readonly: boolean;
}
```

### exists(path)

Check if a path exists:

```typescript
import { exists } from "host:fs";

if (await exists("./config.json")) {
  // Load config
}
```

### remove(path, options?)

Remove a file or directory:

```typescript
import { remove } from "host:fs";

// Remove file
await remove("./temp.txt");

// Remove directory recursively
await remove("./cache", { recursive: true });
```

### rename(from, to)

Rename or move a file/directory:

```typescript
import { rename } from "host:fs";

await rename("./old-name.txt", "./new-name.txt");
await rename("./file.txt", "./archive/file.txt");
```

### copy(from, to)

Copy a file:

```typescript
import { copy } from "host:fs";

await copy("./source.txt", "./destination.txt");
```

---

## File Watching

### watch(path)

Watch a file or directory for changes:

```typescript
import { watch } from "host:fs";

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
  id: string;
  next(): Promise<FileEvent | null>;
  [Symbol.asyncIterator](): AsyncIterableIterator<FileEvent>;
  close(): Promise<void>;
}
```

---

## Error Handling

All operations throw on error:

```typescript
import { readTextFile } from "host:fs";

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
} from "host:fs";
import { homeDir } from "host:sys";

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
