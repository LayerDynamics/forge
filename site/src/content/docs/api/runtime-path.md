---
title: "runtime:path"
description: "Cross-platform path manipulation utilities for Forge applications"
slug: api/runtime-path
---

Cross-platform path manipulation utilities for Forge applications. All operations handle forward slashes on Unix and backslashes on Windows automatically.

> **Implementation**: TypeScript types are auto-generated from Rust via [forge-weld](/docs/crates/forge-weld). See [ext_path](/docs/crates/ext-path) for implementation details.

## Features

- Join path segments with correct separators
- Extract directory names and basenames
- Get file extensions
- Parse paths into components
- Cross-platform path normalization

## Platform Behavior

| Platform | Path Separator |
|----------|---------------|
| Unix (macOS/Linux) | Forward slash (`/`) |
| Windows | Backslash (`\`) |

Operations automatically use platform-appropriate separators.

## No Permissions Required

Path operations are pure string manipulation and don't require filesystem permissions. They work with any path string, whether or not it exists on disk.

## Import

```typescript
import { join, dirname, basename, extname, parts } from "runtime:path";
```

## API Reference

### join(base, ...segments)

Joins path segments into a single path using platform-appropriate separators.

Automatically normalizes redundant separators and handles relative path components.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `base` | `string` | The base path to start from |
| `...segments` | `string[]` | Additional path segments to append |

**Returns:** `string` - Combined path with platform-appropriate separators

**Example:**

```typescript
import { join } from "runtime:path";

// Basic path joining
const configPath = join("./data", "config.json");
// Unix: "./data/config.json"
// Windows: ".\\data\\config.json"

// Multiple segments
const imagePath = join("./assets", "images", "logo.png");
// Unix: "./assets/images/logo.png"
// Windows: ".\\assets\\images\\logo.png"

// Absolute paths
const binPath = join("/usr", "local", "bin", "node");
// Unix: "/usr/local/bin/node"

// Building dynamic paths
function getLogPath(date: string): string {
  return join("./logs", date, "app.log");
}
```

---

### dirname(path)

Extracts the directory path from a file path.

Returns everything before the final path separator. If there is no directory component, returns an empty string.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `path` | `string` | The path to extract the directory from |

**Returns:** `string` - The directory portion of the path, or empty string if none

**Example:**

```typescript
import { dirname } from "runtime:path";

// Extract directory from absolute path
const dir = dirname("/usr/local/bin/node");
console.log(dir); // "/usr/local/bin"

// Relative path
const relDir = dirname("./data/config.json");
console.log(relDir); // "./data"

// No directory component
const noDir = dirname("file.txt");
console.log(noDir); // ""

// Get parent directory of current file
function getAssetDir(filePath: string): string {
  return dirname(filePath);
}
```

---

### basename(path)

Extracts the final component of a path (filename with extension).

Returns the last segment of the path after the final separator. If the path ends with a separator, returns an empty string.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `path` | `string` | The path to extract the basename from |

**Returns:** `string` - The filename portion of the path, or empty string if none

**Example:**

```typescript
import { basename } from "runtime:path";

// Get filename from path
const file = basename("/usr/local/bin/node");
console.log(file); // "node"

// With extension
const config = basename("./data/config.json");
console.log(config); // "config.json"

// Just filename
const readme = basename("readme.md");
console.log(readme); // "readme.md"

// Use for display
function displayFilename(path: string): void {
  console.log(`File: ${basename(path)}`);
}
```

---

### extname(path)

Extracts the file extension from a path.

Returns the extension including the leading dot. If there is no extension, returns an empty string. Only considers the portion after the last dot in the basename.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `path` | `string` | The path to extract the extension from |

**Returns:** `string` - The file extension including the dot, or empty string if none

**Example:**

```typescript
import { extname } from "runtime:path";

// Simple extension
const ext = extname("file.txt");
console.log(ext); // ".txt"

// Compound extension (only last)
const tarExt = extname("archive.tar.gz");
console.log(tarExt); // ".gz" (only last extension)

// No extension
const noExt = extname("README");
console.log(noExt); // ""

// Hidden files (dot prefix is not an extension)
const hidden = extname(".gitignore");
console.log(hidden); // ""

// Filter files by extension
function isImageFile(path: string): boolean {
  const ext = extname(path).toLowerCase();
  return [".png", ".jpg", ".jpeg", ".gif", ".webp"].includes(ext);
}

function isTypeScriptFile(path: string): boolean {
  const ext = extname(path);
  return ext === ".ts" || ext === ".tsx";
}
```

---

### parts(path)

Parses a path into its directory, basename, and extension components.

This is a convenience function that combines `dirname()`, `basename()`, and `extname()` in a single operation.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `path` | `string` | The path to parse |

**Returns:** `PathParts` - Object with `dir`, `base`, and `ext` properties

**Example:**

```typescript
import { parts, join } from "runtime:path";

// Parse a full path
const p = parts("/usr/local/bin/node");
console.log(p.dir);  // "/usr/local/bin"
console.log(p.base); // "node"
console.log(p.ext);  // ""

// Parse path with extension
const config = parts("./data/config.json");
console.log(config.dir);  // "./data"
console.log(config.base); // "config.json"
console.log(config.ext);  // ".json"

// Build modified paths
function createThumbnailPath(imagePath: string): string {
  const p = parts(imagePath);
  return join(p.dir, `thumb_${p.base}`);
}

const original = "./images/photo.jpg";
const thumbnail = createThumbnailPath(original);
console.log(thumbnail); // "./images/thumb_photo.jpg"

// Change file extension
function changeExtension(path: string, newExt: string): string {
  const p = parts(path);
  const nameWithoutExt = p.base.slice(0, -p.ext.length || undefined);
  return join(p.dir, nameWithoutExt + newExt);
}

const tsFile = changeExtension("./src/app.ts", ".js");
console.log(tsFile); // "./src/app.js"
```

## Type Definitions

```typescript
/**
 * Components of a parsed path.
 */
interface PathParts {
  /** Directory path (empty string if no directory) */
  dir: string;

  /** Base filename including extension */
  base: string;

  /** File extension including the dot (empty string if no extension) */
  ext: string;
}
```

## Lifecycle Hooks

Path operations support the standard extensibility hooks for observing and extending behavior.

### onBefore(opName, callback)

Register a callback to be called before an operation executes.

```typescript
import { onBefore } from "runtime:path";

const unsubscribe = onBefore("join", (args) => {
  console.log("Joining paths:", args);
});

// Later, remove the hook
unsubscribe();
```

### onAfter(opName, callback)

Register a callback to be called after an operation completes successfully.

```typescript
import { onAfter } from "runtime:path";

onAfter("dirname", (result, args) => {
  console.log(`dirname(${args}) = ${result}`);
});
```

### onError(opName, callback)

Register a callback to be called when an operation throws an error.

```typescript
import { onError } from "runtime:path";

onError("parts", (error, args) => {
  console.error("Path parsing failed:", error.message);
});
```

### removeAllHooks(opName?)

Remove all hooks for a specific operation or all operations.

```typescript
import { removeAllHooks } from "runtime:path";

// Remove hooks for specific operation
removeAllHooks("join");

// Remove all hooks
removeAllHooks();
```

**Available operation names:** `"join"`, `"dirname"`, `"basename"`, `"extname"`, `"parts"`

## Handler System

Register custom handlers that can be invoked by name.

### registerHandler(name, handler)

```typescript
import { registerHandler } from "runtime:path";

registerHandler("normalize", (path: string) => {
  return path.replace(/\/+/g, "/");
});
```

### invokeHandler(name, ...args)

```typescript
import { invokeHandler } from "runtime:path";

const normalized = await invokeHandler("normalize", "foo//bar//baz");
console.log(normalized); // "foo/bar/baz"
```

### listHandlers()

```typescript
import { listHandlers } from "runtime:path";

const handlers = listHandlers();
console.log(handlers); // ["normalize", ...]
```

### hasHandler(name) / removeHandler(name)

```typescript
import { hasHandler, removeHandler } from "runtime:path";

if (hasHandler("normalize")) {
  removeHandler("normalize");
}
```

## Complete Example

```typescript
import {
  join,
  dirname,
  basename,
  extname,
  parts
} from "runtime:path";
import { readText, writeText, readDir } from "runtime:fs";

/**
 * File organizer that sorts files by extension
 */
async function organizeFiles(sourceDir: string, destDir: string) {
  const entries = await readDir(sourceDir);

  for (const entry of entries) {
    if (!entry.isFile) continue;

    const sourcePath = join(sourceDir, entry.name);
    const ext = extname(entry.name).slice(1).toLowerCase() || "other";

    // Create category folder based on extension
    const category = getCategory(ext);
    const destPath = join(destDir, category, entry.name);

    console.log(`Moving ${basename(sourcePath)} to ${category}/`);

    // Read and write to new location
    const content = await readText(sourcePath);
    await writeText(destPath, content);
  }
}

function getCategory(ext: string): string {
  const categories: Record<string, string[]> = {
    images: ["png", "jpg", "jpeg", "gif", "webp", "svg"],
    documents: ["pdf", "doc", "docx", "txt", "md"],
    code: ["ts", "js", "tsx", "jsx", "rs", "py"],
    data: ["json", "xml", "yaml", "yml", "csv"],
  };

  for (const [category, extensions] of Object.entries(categories)) {
    if (extensions.includes(ext)) {
      return category;
    }
  }
  return "other";
}

/**
 * Create backup with timestamp
 */
function createBackupPath(originalPath: string): string {
  const p = parts(originalPath);
  const timestamp = new Date().toISOString().replace(/[:.]/g, "-");
  const nameWithoutExt = p.base.slice(0, -p.ext.length || undefined);

  return join(p.dir, `${nameWithoutExt}.backup-${timestamp}${p.ext}`);
}

// Usage
const backupPath = createBackupPath("./data/important.json");
console.log(backupPath);
// "./data/important.backup-2024-01-15T10-30-00-000Z.json"

/**
 * Resolve relative import paths
 */
function resolveImport(fromFile: string, importPath: string): string {
  if (importPath.startsWith(".")) {
    // Relative import - resolve from file's directory
    const fromDir = dirname(fromFile);
    return join(fromDir, importPath);
  }
  // Absolute or module import
  return importPath;
}

// Example
const resolved = resolveImport("./src/components/Button.tsx", "../utils/helpers");
console.log(resolved); // "./src/utils/helpers"
```

## Best Practices

### Use `join()` for Building Paths

Always use `join()` instead of string concatenation to ensure cross-platform compatibility:

```typescript
// Good - cross-platform
const path = join("./data", "users", "config.json");

// Bad - platform-specific
const path = "./data/users/config.json"; // Breaks on Windows
```

### Parse Once, Use Multiple Times

When you need multiple components, use `parts()` instead of calling individual functions:

```typescript
// Good - single parse operation
const p = parts(filePath);
const newPath = join(p.dir, "processed_" + p.base);

// Less efficient - parses path multiple times
const dir = dirname(filePath);
const base = basename(filePath);
const newPath = join(dir, "processed_" + base);
```

### Handle Empty Returns

Path functions may return empty strings for edge cases:

```typescript
const dir = dirname("file.txt");
if (dir) {
  // Has directory component
  console.log("In directory:", dir);
} else {
  // File is in current directory
  console.log("In current directory");
}
```
