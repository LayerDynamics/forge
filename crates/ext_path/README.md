# ext_path - Path Manipulation Extension

**Runtime Module**: `runtime:path`

Cross-platform path manipulation utilities for Forge applications. This extension provides pure string-based path operations that work consistently across all platforms without requiring filesystem access or permissions.

## Overview

The `ext_path` extension offers essential path manipulation functions that handle platform-specific path separators automatically. All operations are pure string transformations - they never access the filesystem and require no permissions configuration.

**Key Features:**
- 🔀 **Join Segments**: Combine path components with platform-appropriate separators
- 📂 **Extract Components**: Get directory names, basenames, and extensions
- 🔍 **Parse Paths**: Split paths into structured components
- 🌍 **Cross-Platform**: Consistent behavior on Unix and Windows
- ⚡ **Pure Functions**: No filesystem access, no permissions required
- 🛡️ **Safe Operations**: Never fail or throw errors, return empty strings for missing components

## TypeScript Usage

### Basic Path Manipulation

```typescript
import { join, dirname, basename, extname } from "runtime:path";

// Join path segments - automatically uses correct separators
const configPath = join("./data", "config.json");
// Unix: "./data/config.json"
// Windows: ".\\data\\config.json"

// Extract components
const fullPath = "/usr/local/bin/node";
const dir = dirname(fullPath);   // "/usr/local/bin"
const file = basename(fullPath); // "node"

// Get file extension
const ext = extname("document.pdf"); // ".pdf"
```

### Building Nested Paths

```typescript
import { join } from "runtime:path";

// Build multi-level directory structures
const imagePath = join("./assets", "images", "logo.png");
// Unix: "./assets/images/logo.png"
// Windows: ".\\assets\\images\\logo.png"

// Combine absolute and relative paths
const binPath = join("/usr", "local", "bin", "node");
// Unix: "/usr/local/bin/node"
```

### Parsing Complete Paths

```typescript
import { parts } from "runtime:path";

const p = parts("./data/logs/app.log");
console.log(p.dir);  // "./data/logs"
console.log(p.base); // "app.log"
console.log(p.ext);  // ".log"
```

### Building Modified Paths

```typescript
import { parts, join } from "runtime:path";

// Create thumbnail from image path
const original = "./images/photo.jpg";
const p = parts(original);
const thumbnail = join(p.dir, `thumb_${p.base}`);
console.log(thumbnail); // "./images/thumb_photo.jpg"

// Change file extension
const sourcePath = "./src/module.ts";
const parts = parts(sourcePath);
const baseName = parts.base.slice(0, -parts.ext.length); // Remove extension
const jsPath = join(parts.dir, `${baseName}.js`);
console.log(jsPath); // "./src/module.js"
```

## API Reference

### `join(base, ...segments): string`

Joins path segments into a single path using platform-appropriate separators.

**Parameters:**
- `base: string` - The base path to start from
- `...segments: string[]` - Additional path segments to append

**Returns:** Combined path with platform-appropriate separators

**Examples:**
```typescript
join("./data", "config.json")           // "./data/config.json"
join("/usr", "local", "bin")            // "/usr/local/bin"
join("C:\\Program Files", "MyApp")      // "C:\\Program Files\\MyApp" (Windows)
```

### `dirname(path): string`

Extracts the directory path from a file path.

**Parameters:**
- `path: string` - The path to extract the directory from

**Returns:** The directory portion of the path, or empty string if none

**Examples:**
```typescript
dirname("/usr/local/bin/node")   // "/usr/local/bin"
dirname("./data/config.json")    // "./data"
dirname("file.txt")              // "" (no directory)
dirname("/path/to/")             // "/path/to" (trailing separator removed)
```

### `basename(path): string`

Extracts the final component of a path (filename with extension).

**Parameters:**
- `path: string` - The path to extract the basename from

**Returns:** The filename portion of the path, or empty string if none

**Examples:**
```typescript
basename("/usr/local/bin/node")  // "node"
basename("./data/config.json")   // "config.json"
basename("readme.md")            // "readme.md"
basename("/path/to/")            // "" (ends with separator)
```

### `extname(path): string`

Extracts the file extension from a path.

**Parameters:**
- `path: string` - The path to extract the extension from

**Returns:** The file extension including the dot, or empty string if none

**Examples:**
```typescript
extname("file.txt")           // ".txt"
extname("archive.tar.gz")     // ".gz" (only last extension)
extname("README")             // "" (no extension)
extname(".gitignore")         // "" (dot prefix is not an extension)
extname(".config.json")       // ".json" (has real extension)
```

### `parts(path): PathParts`

Parses a path into its directory, basename, and extension components.

**Parameters:**
- `path: string` - The path to parse

**Returns:** Object with `dir`, `base`, and `ext` properties

**Examples:**
```typescript
parts("/usr/local/bin/node")
// { dir: "/usr/local/bin", base: "node", ext: "" }

parts("./data/config.json")
// { dir: "./data", base: "config.json", ext: ".json" }

parts("file.txt")
// { dir: "", base: "file.txt", ext: ".txt" }
```

## Common Patterns

### 1. Building Dynamic File Paths

```typescript
import { join } from "runtime:path";

function getLogPath(appName: string, date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');

  return join("./logs", appName, `${year}-${month}-${day}.log`);
}

const logPath = getLogPath("myapp", new Date());
// "./logs/myapp/2025-12-19.log"
```

### 2. Validating File Extensions

```typescript
import { extname } from "runtime:path";

function isImageFile(path: string): boolean {
  const ext = extname(path).toLowerCase();
  return ['.jpg', '.jpeg', '.png', '.gif', '.webp'].includes(ext);
}

console.log(isImageFile("photo.jpg"));  // true
console.log(isImageFile("document.pdf")); // false
```

### 3. Generating Output Paths

```typescript
import { parts, join } from "runtime:path";

function getOutputPath(inputPath: string, suffix: string): string {
  const p = parts(inputPath);
  const baseName = p.base.slice(0, -p.ext.length);
  return join(p.dir, `${baseName}${suffix}${p.ext}`);
}

const output = getOutputPath("./video.mp4", ".compressed");
console.log(output); // "./video.compressed.mp4"
```

### 4. Path Component Extraction

```typescript
import { dirname, basename, extname } from "runtime:path";

function analyzeFile(path: string) {
  return {
    directory: dirname(path),
    filename: basename(path),
    extension: extname(path),
    nameWithoutExt: basename(path).slice(0, -extname(path).length)
  };
}

const info = analyzeFile("./docs/guide.md");
// {
//   directory: "./docs",
//   filename: "guide.md",
//   extension: ".md",
//   nameWithoutExt: "guide"
// }
```

### 5. Cross-Platform Path Handling

```typescript
import { join } from "runtime:path";

// Always use join() instead of manual concatenation
// ❌ Wrong - breaks on Windows
const wrongPath = "./data" + "/" + "config.json";

// ✅ Correct - works on all platforms
const correctPath = join("./data", "config.json");

// join() handles platform differences automatically
// Unix:    "./data/config.json"
// Windows: ".\\data\\config.json"
```

## Edge Cases

### Empty Components

All functions return empty strings when the requested component doesn't exist:

```typescript
dirname("file.txt");      // "" (no directory)
extname("README");        // "" (no extension)
basename("/path/to/");    // "" (ends with separator)

parts("file.txt");
// { dir: "", base: "file.txt", ext: ".txt" }
```

### Hidden Files

Dot prefixes are not treated as extensions:

```typescript
extname(".gitignore");    // "" (dot prefix, not extension)
extname(".config.json");  // ".json" (has real extension after name)

basename(".gitignore");   // ".gitignore"
```

### Multiple Extensions

Only the last extension is extracted:

```typescript
extname("archive.tar.gz");  // ".gz" (not ".tar.gz")

// To handle multiple extensions, use string operations:
const path = "archive.tar.gz";
const parts = path.split('.');
const allExtensions = parts.slice(1).join('.'); // "tar.gz"
```

### Absolute vs Relative Paths

```typescript
dirname("/usr/local/bin");    // "/usr/local"
dirname("./local/bin");       // "./local"
dirname("local/bin");         // "local"

// Root directory
dirname("/");                 // "/"
dirname("C:\\");             // "C:\\" (Windows)
```

## Platform Notes

### Path Separators

- **Unix (macOS/Linux)**: Uses forward slashes (`/`)
- **Windows**: Uses backslashes (`\`)

The extension automatically handles platform differences:

```typescript
// Unix
join("usr", "local", "bin")  // "usr/local/bin"

// Windows
join("C:", "Program Files")  // "C:\\Program Files"
```

### Best Practices

1. **Always use `join()`** instead of manual string concatenation
2. **Never hardcode separators** - let the extension handle them
3. **Use forward slashes in literals** - they work on all platforms as input
4. **Don't assume separators** - use dirname/basename instead of split()

```typescript
// ✅ Good - cross-platform
const path = join(baseDir, "data", "config.json");

// ❌ Bad - breaks on Windows
const path = `${baseDir}/data/config.json`;
```

## No Permissions Required

Unlike filesystem operations (`runtime:fs`), path manipulation functions:
- ✅ Don't access the filesystem
- ✅ Don't require permissions in `manifest.app.toml`
- ✅ Work with any path string, whether it exists or not
- ✅ Are pure string transformations
- ✅ Never fail or throw errors

## Testing

### Unit Tests

```typescript
import { join, dirname, basename, extname, parts } from "runtime:path";

function testPathOperations() {
  // Test join
  console.assert(join("a", "b") !== "a/b" || "a\\b", "join handles separators");

  // Test dirname
  console.assert(dirname("/a/b/c") === "/a/b", "dirname extracts directory");
  console.assert(dirname("file.txt") === "", "dirname returns empty for no dir");

  // Test basename
  console.assert(basename("/a/b/c.txt") === "c.txt", "basename extracts file");
  console.assert(basename("/a/b/") === "", "basename returns empty for trailing sep");

  // Test extname
  console.assert(extname("file.txt") === ".txt", "extname extracts extension");
  console.assert(extname(".gitignore") === "", "extname handles hidden files");

  // Test parts
  const p = parts("/a/b/c.txt");
  console.assert(p.dir === "/a/b", "parts.dir correct");
  console.assert(p.base === "c.txt", "parts.base correct");
  console.assert(p.ext === ".txt", "parts.ext correct");
}

testPathOperations();
```

### Integration with runtime:fs

```typescript
import { join } from "runtime:path";
import { readTextFile } from "runtime:fs";

async function loadConfig(appDir: string): Promise<object> {
  const configPath = join(appDir, "config.json");
  const content = await readTextFile(configPath);
  return JSON.parse(content);
}
```

## Build System Integration

### Extension Registration

This extension is registered in `forge-runtime` as a **Tier 0 (ExtensionOnly)** extension - it requires no state initialization and is always available.

### Code Generation

The TypeScript bindings are generated via `forge-weld` from Rust ops:

```rust
// In build.rs
ExtensionBuilder::new("runtime_path", "runtime:path")
    .ts_path("ts/init.ts")
    .ops(&[
        "op_path_join",
        "op_path_dirname",
        "op_path_basename",
        "op_path_extname",
        "op_path_parts"
    ])
    .use_inventory_types()
    .generate_sdk_module("../../sdk")
    .build()
```

This generates:
- `sdk/runtime.path.ts` - TypeScript SDK module
- `ts/init.ts` → `init.js` - JavaScript shim for Deno runtime

## Implementation Details

### Rust Standard Library

The extension uses Rust's `std::path::Path` and `std::path::PathBuf` for all operations:

```rust
fn path_join(base: String, segments: Vec<String>) -> String {
    let mut pb = PathBuf::from(base);
    for seg in segments {
        pb.push(seg);
    }
    pb.to_string_lossy().to_string()
}
```

This ensures:
- ✅ Correct platform-specific behavior
- ✅ Proper handling of separators
- ✅ Edge case handling (empty paths, trailing separators)
- ✅ UTF-8 string conversion with `to_string_lossy()`

### Pure Functions

All operations are stateless:
- No `OpState` required
- No capability checks
- No async operations
- Instant execution

### Error Handling

Path operations never fail. Missing components return empty strings:

```typescript
dirname("file.txt")     // "" instead of Error
extname("README")       // "" instead of Error
basename("/path/to/")   // "" instead of Error
```

## See Also

- [`ext_fs`](../ext_fs/README.md) - Filesystem operations (read, write, stat)
- [`ext_process`](../ext_process/README.md) - Process spawning with working directories
- [Rust std::path documentation](https://doc.rust-lang.org/std/path/index.html)

## Examples

See examples using path operations:
- `examples/text-editor/` - File path manipulation for editor
- `examples/todo-app/` - Storage path construction
- `examples/system-monitor/` - Log file path building
