# ext_fs

Filesystem operations for Forge applications.

## Overview

`ext_fs` is a Forge extension that provides comprehensive cross-platform filesystem operations with capability-based security and async I/O. It enables Forge applications to:

- Read and write files (text and binary formats)
- Manage directory structures with recursive operations
- Watch filesystem changes in real-time with async iteration
- Create and resolve symbolic links
- Access detailed file metadata and timestamps
- Create temporary files and directories securely
- Perform atomic file operations with permission control

The extension wraps Rust's Tokio async filesystem primitives and exposes them to TypeScript via the `runtime:fs` module, providing a secure and ergonomic API for all filesystem needs.

## TypeScript Usage

### Basic File Operations

```typescript
import { readTextFile, writeTextFile, exists } from "runtime:fs";

// Read a text file
const config = await readTextFile("./config.json");
const data = JSON.parse(config);

// Write a text file
await writeTextFile("./output.txt", "Hello, World!");

// Check if file exists
if (await exists("./data.json")) {
  const content = await readTextFile("./data.json");
  console.log("File contents:", content);
}
```

### Binary File Operations

```typescript
import { readBytes, writeBytes } from "runtime:fs";

// Read binary data
const imageData = await readBytes("./image.png");
const buffer = new Uint8Array(imageData);
console.log(`Image size: ${buffer.length} bytes`);

// Write binary data
const data = new Uint8Array([0x89, 0x50, 0x4E, 0x47]); // PNG header
await writeBytes("./output.bin", data);

// Process binary file
const pdfData = await readBytes("./document.pdf");
await processPDF(pdfData);
```

### Directory Management

```typescript
import { readDir, mkdir, remove, stat } from "runtime:fs";

// List directory contents
const entries = await readDir("./data");
for (const entry of entries) {
  const type = entry.isFile ? "file" : "directory";
  console.log(`${entry.name} (${type})`);
}

// Create nested directories
await mkdir("./data/cache/images", { recursive: true });

// Remove directory and all contents
await remove("./temp", { recursive: true });

// Get file statistics
const stats = await stat("./config.json");
console.log(`Size: ${stats.size} bytes, Read-only: ${stats.readonly}`);
```

### File Watching

```typescript
import { watch } from "runtime:fs";

// Watch directory for changes
const watcher = await watch("./data");
try {
  for await (const event of watcher) {
    console.log(`Event: ${event.kind}`);
    console.log(`Files: ${event.paths.join(", ")}`);
  }
} finally {
  // Always clean up resources
  await watcher.close();
}
```

### Symbolic Links

```typescript
import { symlink, readLink, realPath } from "runtime:fs";

// Create a symbolic link
await symlink("/var/log/myapp", "./logs");

// Read where symlink points
const target = await readLink("./logs");
console.log(`Points to: ${target}`); // "/var/log/myapp"

// Resolve to canonical absolute path
const canonical = await realPath("./logs");
console.log(`Canonical: ${canonical}`); // "/var/log/myapp"
```

### Temporary Files

```typescript
import { tempFile, tempDir, writeTextFile, remove } from "runtime:fs";

// Create temp file for processing
const temp = await tempFile("process-", ".json");
try {
  await writeTextFile(temp.path, JSON.stringify({ data: "test" }));
  const result = await processFile(temp.path);
  console.log(`Processed: ${result}`);
} finally {
  await remove(temp.path); // Clean up
}

// Create temp directory for batch work
const tempDirectory = await tempDir("build-");
try {
  await mkdir(`${tempDirectory.path}/output`);
  await processFiles(tempDirectory.path);
} finally {
  await remove(tempDirectory.path, { recursive: true });
}
```

### Appending to Files

```typescript
import { appendTextFile, appendBytes } from "runtime:fs";

// Append to log file
const timestamp = new Date().toISOString();
await appendTextFile("./app.log", `${timestamp} - Application started\n`);

// Append binary data
const encoder = new TextEncoder();
const logEntry = encoder.encode("Binary log entry\n");
await appendBytes("./binary.log", logEntry);
```

## Permissions

### Manifest Configuration

Filesystem operations require explicit permissions in `manifest.app.toml`:

```toml
[permissions.fs]
# Allow reading specific paths (glob patterns supported)
read = [
  "./data/**",           # All files under ./data/
  "./config/*.json",     # All JSON files in ./config/
  "/etc/hosts"           # Specific absolute path
]

# Allow writing specific paths
write = [
  "./data/**",           # All files under ./data/
  "./logs/*.log",        # Log files only
  "/tmp/**"              # System temp directory
]
```

### Development vs Production

- **Development mode** (`forge dev`): All filesystem paths allowed by default
- **Production mode** (bundled apps): Only paths in manifest allowlist can be accessed
- Permission violations throw error **3001** (`PermissionDenied`)

### Permission Scopes

- **Read operations**: `readTextFile`, `readBytes`, `stat`, `readDir`, `exists`, `watch`, `readLink`, `metadata`, `realPath`
- **Write operations**: `writeTextFile`, `writeBytes`, `mkdir`, `remove`, `rename`, `copy`, `symlink`, `appendTextFile`, `appendBytes`, `tempFile`, `tempDir`
- **Combined operations**: `copy` and `rename` require read permission for source and write permission for destination

## Error Codes

All errors include structured error codes for programmatic handling:

| Code | Error Type | Meaning | How to Fix |
|------|------------|---------|------------|
| **3000** | `Io` | Generic I/O error during filesystem operation | Check path exists, verify permissions, check disk space, retry operation |
| **3001** | `PermissionDenied` | Path not in manifest allowlist | Add path pattern to `[permissions.fs]` in manifest.app.toml |
| **3002** | `NotFound` | File or directory not found | Verify path exists, check for typos, create parent directories |
| **3003** | `AlreadyExists` | File or directory already exists | Use different path or remove existing file first |
| **3004** | `IsDirectory` | Path is a directory (expected file) | Use directory-specific operation or check path |
| **3005** | `IsFile` | Path is a file (expected directory) | Use file-specific operation or check path |
| **3006** | `Watch` | File watch setup or operation error | Check path exists, verify read permissions, ensure not too many watchers |
| **3007** | `InvalidWatchId` | Invalid or closed watch ID | Verify watcher wasn't closed, check for race conditions |
| **3008** | `Symlink` | Symbolic link creation or resolution error | Check target exists, verify symlink permissions (Windows: admin/dev mode) |
| **3009** | `TempError` | Temporary file/directory creation error | Check temp directory writable, verify disk space available |

### Error Handling Example

```typescript
try {
  const content = await readTextFile("./config.json");
} catch (error) {
  if (error.message.includes("[3002]")) {
    console.error("Config file not found. Creating default...");
    await writeTextFile("./config.json", JSON.stringify(defaultConfig));
  } else if (error.message.includes("[3001]")) {
    console.error("Permission denied. Update manifest.app.toml");
  } else {
    console.error("Unexpected error:", error);
  }
}
```

## Common Patterns

### Pattern 1: Safe File Read with Fallback

```typescript
async function readConfig<T>(path: string, defaultValue: T): Promise<T> {
  try {
    const content = await readTextFile(path);
    return JSON.parse(content);
  } catch (error) {
    if (error.message.includes("[3002]")) {
      // File not found, use default
      return defaultValue;
    }
    throw error; // Re-throw other errors
  }
}

const config = await readConfig("./config.json", { theme: "dark" });
```

### Pattern 2: Atomic File Write with Backup

```typescript
async function atomicWrite(path: string, content: string): Promise<void> {
  const tempPath = `${path}.tmp`;
  const backupPath = `${path}.bak`;

  // Write to temp file first
  await writeTextFile(tempPath, content);

  // Backup existing file if it exists
  if (await exists(path)) {
    await copy(path, backupPath);
  }

  // Atomic rename (replaces original)
  await rename(tempPath, path);

  // Clean up backup on success
  if (await exists(backupPath)) {
    await remove(backupPath);
  }
}
```

### Pattern 3: Recursive Directory Processing

```typescript
async function processDirectory(path: string, callback: (file: string) => Promise<void>): Promise<void> {
  const entries = await readDir(path);

  for (const entry of entries) {
    const fullPath = `${path}/${entry.name}`;

    if (entry.isFile) {
      await callback(fullPath);
    } else if (entry.isDirectory) {
      await processDirectory(fullPath, callback); // Recurse
    }
  }
}

// Use it
await processDirectory("./src", async (file) => {
  if (file.endsWith(".ts")) {
    console.log(`Processing: ${file}`);
    const content = await readTextFile(file);
    // ... process TypeScript file
  }
});
```

### Pattern 4: File Watcher with Debounce

```typescript
class DebouncedWatcher {
  private timeout: number | null = null;

  async watch(path: string, callback: (event: WatchEvent) => void, debounceMs = 300): Promise<void> {
    const watcher = await watch(path);

    try {
      for await (const event of watcher) {
        // Clear existing timeout
        if (this.timeout !== null) {
          clearTimeout(this.timeout);
        }

        // Set new timeout
        this.timeout = setTimeout(() => {
          callback(event);
          this.timeout = null;
        }, debounceMs);
      }
    } finally {
      await watcher.close();
    }
  }
}

const watcher = new DebouncedWatcher();
await watcher.watch("./config", (event) => {
  console.log(`Config changed: ${event.paths.join(", ")}`);
  reloadConfig();
});
```

### Pattern 5: Batch File Operations with Progress

```typescript
async function copyFiles(sources: string[], destDir: string, onProgress?: (current: number, total: number) => void): Promise<void> {
  // Ensure destination directory exists
  await mkdir(destDir, { recursive: true });

  for (let i = 0; i < sources.length; i++) {
    const src = sources[i];
    const filename = src.split("/").pop()!;
    const dest = `${destDir}/${filename}`;

    await copy(src, dest);

    if (onProgress) {
      onProgress(i + 1, sources.length);
    }
  }
}

// Use it
const files = ["./a.txt", "./b.txt", "./c.txt"];
await copyFiles(files, "./backup", (current, total) => {
  console.log(`Progress: ${current}/${total} (${Math.round(current/total*100)}%)`);
});
```

## Platform Notes

### macOS & Linux (Unix)

- **Permissions**: Unix permission bits available via `metadata()` function
- **Symlinks**: Full support with no restrictions
- **File Watching**: Uses FSEvents (macOS) or inotify (Linux) for efficient monitoring
- **Timestamps**: All timestamps (created, modified, accessed) fully supported
- **Case Sensitivity**:
  - Linux: Usually case-sensitive
  - macOS: Case-insensitive by default (APFS can be case-sensitive)

### Windows

- **Permissions**: Unix-style permission bits not available; use `readonly` property
- **Symlinks**: Supported but directory symlinks require:
  - Administrator privileges, OR
  - Developer Mode enabled in Windows settings
- **File Watching**: Uses ReadDirectoryChangesW API
- **Timestamps**: All timestamps fully supported
- **Path Separators**: Forward slashes (`/`) automatically converted to backslashes (`\`)
- **Case Sensitivity**: Usually case-insensitive (NTFS can be configured for case-sensitivity)

### Cross-Platform Best Practices

1. **Path Separators**: Always use forward slashes (`/`) in code
   ```typescript
   // Good - works everywhere
   const path = "./data/config.json";

   // Bad - only works on Windows
   const path = ".\\data\\config.json";
   ```

2. **Symlink Handling**: Check platform before creating directory symlinks
   ```typescript
   if (Deno.build.os === "windows") {
     console.warn("Directory symlinks may require admin privileges on Windows");
   }
   await symlink(target, link);
   ```

3. **Timestamp Availability**: Always check for null timestamps
   ```typescript
   const meta = await metadata("./file.txt");
   if (meta.createdAt) {
     console.log(`Created: ${new Date(meta.createdAt)}`);
   } else {
     console.log("Creation time not available");
   }
   ```

## Testing

### Running Tests

```bash
# Run all extension tests
cargo test -p ext_fs

# Run with debug logging
RUST_LOG=ext_fs=debug cargo test -p ext_fs -- --nocapture

# Run specific test
cargo test -p ext_fs test_fs_read_write
```

### Testing Considerations

1. **Permissions**: Tests use `PermissiveFsChecker` (allow all operations)
2. **Temp Files**: Tests clean up temp files/directories after execution
3. **Platform**: Some tests are conditionally compiled for Unix/Windows
4. **File Watching**: Watch tests use short timeouts to avoid hanging

### Writing New Tests

```rust
#[tokio::test]
async fn test_custom_fs_operation() {
    let mut state = OpState::new();
    init_fs_state(&mut state, None);

    // Your test logic here
}
```

## Architecture

### State Management

The extension maintains filesystem state in Deno's `OpState`:

```rust
pub struct FsWatchState {
    pub watchers: HashMap<String, WatchEntry>,  // Active file watchers
    pub next_id: u64,                            // ID generator
}

pub struct WatchEntry {
    pub receiver: mpsc::Receiver<FileEvent>,     // Event channel
    pub watcher: notify::RecommendedWatcher,     // Platform watcher
}
```

### Async I/O

All filesystem operations use `tokio::fs` for non-blocking async I/O:
- File reads/writes are fully async (no blocking the event loop)
- Directory operations stream results incrementally
- File watching uses async channels for event delivery

### File Watching Implementation

- Uses `notify` crate with platform-optimal backends
- Events buffered in 64-capacity channel
- Watchers stored in `OpState` with unique string IDs
- Must explicitly call `close()` to clean up resources

### Security Model

```rust
pub trait FsCapabilityChecker: Send + Sync {
    fn check_read(&self, path: &str) -> Result<(), String>;
    fn check_write(&self, path: &str) -> Result<(), String>;
}
```

Runtime initializes with custom checker:

```rust
init_fs_state(
    &mut op_state,
    Some(Arc::new(ManifestFsChecker::new(manifest)))
);
```

## Build System Integration

### Code Generation

This extension uses `forge-weld` for TypeScript binding generation:

```rust
// In build.rs
ExtensionBuilder::new("runtime_fs", "runtime:fs")
    .ts_path("ts/init.ts")
    .ops(&[
        "op_fs_read_text",
        "op_fs_write_text",
        "op_fs_read_bytes",
        "op_fs_write_bytes",
        "op_fs_stat",
        "op_fs_read_dir",
        "op_fs_mkdir",
        "op_fs_remove",
        "op_fs_rename",
        "op_fs_copy",
        "op_fs_exists",
        "op_fs_watch",
        "op_fs_watch_next",
        "op_fs_watch_close",
        "op_fs_symlink",
        "op_fs_read_link",
        "op_fs_append_text",
        "op_fs_append_bytes",
        "op_fs_metadata",
        "op_fs_real_path",
        "op_fs_temp_file",
        "op_fs_temp_dir",
    ])
    .generate_sdk_module("sdk")
    .use_inventory_types()
    .build()
```

### Build Artifacts

- **Generated SDK**: `sdk/runtime.fs.ts` (auto-generated from `ts/init.ts`)
- **Extension Module**: `OUT_DIR/extension.rs` (included at compile time)

### Rebuilding

```bash
# Trigger regeneration of TypeScript SDK
cargo build -p ext_fs

# Generate Rust docs
cargo doc -p ext_fs --no-deps --open
```

## See Also

- **TypeScript SDK**: [`sdk/runtime.fs.ts`](../../sdk/runtime.fs.ts) - Generated SDK with full JSDoc
- **TypeScript Source**: [`ts/init.ts`](./ts/init.ts) - Source for SDK generation
- **Astro Docs**: `site/src/content/docs/crates/ext-fs.md` - User-facing documentation
- **API Docs**: https://docs.rs/ext_fs - Published Rust documentation
- **Examples**: [`examples/`](../../examples/) - Sample Forge applications using filesystem operations

## Contributing

When adding new operations:

1. Add `#[weld_op]` macro **before** `#[op2]` (required for code generation)
2. Update `build.rs` to include new op in `.ops(&[...])` list
3. Add TypeScript wrapper in `ts/init.ts` with comprehensive JSDoc
4. Add tests in `tests` module
5. Rebuild to regenerate SDK: `cargo build -p ext_fs`
6. Update this README and Astro docs

## License

See [LICENSE](../../LICENSE) in the repository root.
