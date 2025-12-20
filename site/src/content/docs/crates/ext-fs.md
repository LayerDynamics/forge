---
title: "ext_fs"
description: "Filesystem operations for Forge applications"
slug: crates/ext-fs
---

# Filesystem Operations

Comprehensive filesystem operations with async I/O, file watching, and capability-based security.

## Overview

The `runtime:fs` module enables Forge applications to interact with the filesystem through a secure, cross-platform API. It provides comprehensive file and directory operations, real-time change monitoring, and symbolic link management.

**Key Capabilities:**
- Read and write files (text and binary)
- Create, list, and remove directories (with recursive options)
- Watch files and directories for changes
- Create and resolve symbolic links
- Access detailed file metadata and timestamps
- Create temporary files and directories
- Atomic file operations with permission control

## Installation

Import from the `runtime:fs` module:

```typescript
import { readTextFile, writeTextFile, watch } from "runtime:fs";
```

## Core Concepts

### File Operations

The module provides separate functions for text and binary operations:

```typescript
// Text files (UTF-8)
const config = await readTextFile("./config.json");
await writeTextFile("./output.txt", "Hello, World!");

// Binary files
const imageData = await readBytes("./image.png");
await writeBytes("./output.bin", new Uint8Array([0x89, 0x50, 0x4E, 0x47]));
```

### Async Operations

All filesystem operations are fully asynchronous and non-blocking:

```typescript
// These run in parallel without blocking the event loop
const [config, image, data] = await Promise.all([
  readTextFile("./config.json"),
  readBytes("./image.png"),
  readTextFile("./data.txt")
]);
```

### Permission Model

Filesystem access requires explicit permissions in `manifest.app.toml`:

```toml
[permissions.fs]
read = ["./data/**", "./config.json"]
write = ["./data/**", "./logs/*.log"]
```

In development mode (`forge dev`), all permissions are granted by default.

## API Reference

### File I/O

#### readTextFile(path: string)

Reads the entire contents of a file as a UTF-8 string.

```typescript
const config = await readTextFile("./config.json");
const data = JSON.parse(config);
```

**Throws:** Error 3000 (I/O error), 3001 (permission denied), 3002 (not found), 3004 (is directory)

#### writeTextFile(path: string, content: string)

Writes a string to a file, creating it if it doesn't exist.

```typescript
await writeTextFile("./output.txt", "Hello, World!");
await writeTextFile("./data.json", JSON.stringify(data, null, 2));
```

**Throws:** Error 3000 (I/O error), 3001 (permission denied), 3004 (is directory)

#### readBytes(path: string)

Reads the entire contents of a file as binary data.

```typescript
const imageData = await readBytes("./image.png");
const buffer = new Uint8Array(imageData);
console.log(`Size: ${buffer.length} bytes`);
```

**Throws:** Error 3000 (I/O error), 3001 (permission denied), 3002 (not found), 3004 (is directory)

#### writeBytes(path: string, content: Uint8Array)

Writes binary data to a file, creating it if it doesn't exist.

```typescript
const data = new Uint8Array([0x89, 0x50, 0x4E, 0x47]); // PNG header
await writeBytes("./output.bin", data);
```

**Throws:** Error 3000 (I/O error), 3001 (permission denied), 3004 (is directory)

### Directory Operations

#### readDir(path: string)

Lists the contents of a directory.

```typescript
const entries = await readDir("./data");
for (const entry of entries) {
  console.log(`${entry.name} (${entry.isFile ? "file" : "directory"})`);
}
```

**Returns:** Array of `DirEntry` objects with `name`, `isFile`, `isDirectory` properties.

**Throws:** Error 3000 (I/O error), 3001 (permission denied), 3002 (not found), 3005 (is file)

#### mkdir(path: string, options?: { recursive?: boolean })

Creates a directory.

```typescript
// Create single directory
await mkdir("./cache");

// Create nested directories
await mkdir("./data/cache/images", { recursive: true });
```

**Throws:** Error 3000 (I/O error), 3001 (permission denied), 3003 (already exists)

#### remove(path: string, options?: { recursive?: boolean })

Removes a file or directory.

```typescript
// Remove a file
await remove("./temp.txt");

// Remove directory and all contents
await remove("./temp", { recursive: true });
```

**Throws:** Error 3000 (I/O error), 3001 (permission denied), 3002 (not found)

### File Metadata

#### stat(path: string)

Gets basic file statistics.

```typescript
const stats = await stat("./config.json");
console.log(`Size: ${stats.size} bytes`);
console.log(`Is file: ${stats.isFile}`);
console.log(`Read-only: ${stats.readonly}`);
```

**Returns:** `FileStat` object with `isFile`, `isDirectory`, `isSymlink`, `size`, `mtime`, `atime`, `birthtime`, `readonly`.

**Throws:** Error 3000 (I/O error), 3001 (permission denied), 3002 (not found)

#### exists(path: string)

Checks if a path exists.

```typescript
if (await exists("./config.json")) {
  const config = await readTextFile("./config.json");
  // ... use config
}
```

**Returns:** `true` if path exists, `false` otherwise.

**Throws:** Error 3001 (permission denied)

### File Watching

#### watch(path: string)

Watches a file or directory for changes, returning a watcher that emits events asynchronously.

```typescript
const watcher = await watch("./data");
try {
  for await (const event of watcher) {
    console.log(`${event.kind}: ${event.paths.join(", ")}`);
    // event.kind: "Create", "Modify", "Remove", or "Rename"
  }
} finally {
  await watcher.close(); // Always clean up
}
```

**Important:** Always call `watcher.close()` when done to clean up resources.

**Events:**
- `Create` - File or directory created
- `Modify` - File or directory modified
- `Remove` - File or directory deleted
- `Rename` - File or directory renamed

**Throws:** Error 3000 (I/O error), 3001 (permission denied), 3002 (not found), 3006 (watch error)

### Symbolic Links

#### symlink(target: string, path: string)

Creates a symbolic link.

```typescript
// Create a symlink to a file
await symlink("./data/original.txt", "./data/link.txt");

// Create a symlink to a directory
await symlink("/var/log/app", "./logs");
```

**Platform Note:** On Windows, directory symlinks require administrator privileges or Developer Mode.

**Throws:** Error 3000 (I/O error), 3001 (permission denied), 3003 (already exists), 3008 (symlink error)

#### readLink(path: string)

Reads the target of a symbolic link.

```typescript
const target = await readLink("./logs");
console.log(`Points to: ${target}`); // "/var/log/app"
```

**Throws:** Error 3000 (I/O error), 3001 (permission denied), 3002 (not found), 3005 (not a symlink), 3008 (symlink error)

#### realPath(path: string)

Resolves a path to its canonical, absolute form by resolving all symbolic links.

```typescript
const canonical = await realPath("./logs");
console.log(canonical); // "/var/log/app"
```

**Throws:** Error 3000 (I/O error), 3001 (permission denied), 3002 (not found), 3008 (symlink error)

### Utility Operations

#### copy(from: string, to: string)

Copies a file.

```typescript
await copy("./config.json", "./config.backup.json");
```

**Throws:** Error 3000 (I/O error), 3001 (permission denied), 3002 (source not found)

#### rename(from: string, to: string)

Moves or renames a file or directory.

```typescript
await rename("./old-name.txt", "./new-name.txt");
await rename("./temp", "./backup");
```

**Throws:** Error 3000 (I/O error), 3001 (permission denied), 3002 (source not found)

#### appendTextFile(path: string, content: string)

Appends text to a file, creating it if it doesn't exist.

```typescript
const timestamp = new Date().toISOString();
await appendTextFile("./app.log", `${timestamp} - Application started\n`);
```

**Throws:** Error 3000 (I/O error), 3001 (permission denied), 3004 (is directory)

### Temporary Files

#### tempFile(prefix?: string, suffix?: string)

Creates a temporary file that persists until explicitly deleted.

```typescript
const temp = await tempFile("process-", ".json");
try {
  await writeTextFile(temp.path, JSON.stringify(data));
  await processFile(temp.path);
} finally {
  await remove(temp.path); // Clean up
}
```

**Throws:** Error 3000 (I/O error), 3009 (temp error)

#### tempDir(prefix?: string)

Creates a temporary directory that persists until explicitly deleted.

```typescript
const temp = await tempDir("build-");
try {
  await mkdir(`${temp.path}/output`);
  // ... use directory
} finally {
  await remove(temp.path, { recursive: true }); // Clean up
}
```

**Throws:** Error 3000 (I/O error), 3009 (temp error)

## Error Handling

All errors include structured error codes in the format `[3xxx]`:

```typescript
try {
  const content = await readTextFile("./config.json");
} catch (error) {
  if (error.message.includes("[3002]")) {
    console.error("File not found");
    // Create default config
    await writeTextFile("./config.json", JSON.stringify(defaultConfig));
  } else if (error.message.includes("[3001]")) {
    console.error("Permission denied - check manifest.app.toml");
  } else {
    throw error; // Re-throw unexpected errors
  }
}
```

### Error Code Reference

| Code | Meaning |
|------|---------|
| 3000 | I/O error during operation |
| 3001 | Permission denied by capability system |
| 3002 | File or directory not found |
| 3003 | File or directory already exists |
| 3004 | Path is a directory (expected file) |
| 3005 | Path is a file (expected directory) |
| 3006 | File watch error |
| 3007 | Invalid watch ID |
| 3008 | Symbolic link error |
| 3009 | Temporary file/directory creation error |

## Best Practices

### 1. Always Check if Files Exist

```typescript
// Good
if (await exists("./config.json")) {
  const config = await readTextFile("./config.json");
}

// Bad - throws error if file doesn't exist
const config = await readTextFile("./config.json");
```

### 2. Clean Up File Watchers

```typescript
// Good
const watcher = await watch("./data");
try {
  for await (const event of watcher) {
    handleEvent(event);
  }
} finally {
  await watcher.close(); // Always clean up
}

// Bad - watcher leaks resources
const watcher = await watch("./data");
for await (const event of watcher) {
  handleEvent(event);
}
```

### 3. Use Atomic Writes for Important Files

```typescript
// Write to temp file first, then rename (atomic on most systems)
const tempPath = `${path}.tmp`;
await writeTextFile(tempPath, content);
await rename(tempPath, path);
```

### 4. Handle Recursive Directory Operations Carefully

```typescript
// Create parent directories automatically
await mkdir("./data/cache/images", { recursive: true });

// Remove directories and all contents
await remove("./temp", { recursive: true });
```

### 5. Use Appropriate Permissions

```toml
[permissions.fs]
# Good - specific patterns
read = ["./data/**", "./config/*.json"]
write = ["./data/**", "./logs/*.log"]

# Bad - overly permissive (security risk)
read = ["**"]
write = ["**"]
```

## Common Pitfalls

### Forgetting to Close Watchers

File watchers consume system resources. Always call `close()`:

```typescript
// Wrong
const watcher = await watch("./data");
for await (const event of watcher) {
  console.log(event);
}
// watcher never closed - resource leak!

// Right
const watcher = await watch("./data");
try {
  for await (const event of watcher) {
    console.log(event);
  }
} finally {
  await watcher.close();
}
```

### Not Handling Permission Errors

In production mode, permission errors are common. Always handle error 3001:

```typescript
try {
  await writeTextFile("/etc/hosts", content);
} catch (error) {
  if (error.message.includes("[3001]")) {
    console.error("Add '/etc/hosts' to write permissions in manifest.app.toml");
  } else {
    throw error;
  }
}
```

### Assuming File Exists

Always check before reading or use error handling:

```typescript
// Option 1: Check first
if (await exists("./config.json")) {
  const config = await readTextFile("./config.json");
}

// Option 2: Handle error
try {
  const config = await readTextFile("./config.json");
} catch (error) {
  if (error.message.includes("[3002]")) {
    // File not found, use defaults
    config = defaultConfig;
  }
}
```

### Platform-Specific Path Separators

Always use forward slashes - they work everywhere:

```typescript
// Good - works on all platforms
const path = "./data/config.json";

// Bad - only works on Windows
const path = ".\\data\\config.json";
```

## Platform Notes

### Windows

- Directory symlinks require administrator privileges or Developer Mode
- Unix-style permission bits not available (use `readonly` property)
- Timestamps fully supported

### macOS & Linux

- Full symlink support with no restrictions
- Unix permission bits available via `metadata()` function
- Timestamps fully supported

### Cross-Platform

- Use forward slashes (`/`) in paths (automatically converted on Windows)
- Check for null timestamps when using `metadata()`
- File watching uses platform-optimal backend (FSEvents on macOS, inotify on Linux)

## See Also

- [ext_path](./ext-path.md) - Path manipulation utilities
- [ext_storage](./ext-storage.md) - Persistent key-value storage
- [ext_process](./ext-process.md) - Child process management
- [Permissions Guide](/guides/permissions) - Configuring app permissions
