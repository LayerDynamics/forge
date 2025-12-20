# ext_shell - Shell Integration and Command Execution Extension

**Runtime Module**: `runtime:shell`

Comprehensive shell integration and command execution for Forge applications. Provides system integration (open URLs/files, trash operations, system sounds) and full-featured shell command execution with cross-platform support.

## Overview

The `ext_shell` extension bridges Forge applications with the operating system shell and desktop environment through two major categories of functionality:

**Key Features:**
- 🌐 **URL/File Opening**: Launch URLs in browser, files in default apps
- 📁 **File Manager Integration**: Reveal files in Finder/Explorer
- 🗑️ **Safe Deletion**: Move to trash with recovery option
- 🔊 **System Sounds**: Play beep/alert sounds
- 🐚 **Shell Execution**: Run commands with full syntax support
- 🌍 **Environment Management**: Read/write environment variables
- 🔍 **Path Resolution**: Find executables in PATH
- ⚙️ **Process Management**: Kill background processes with signals

## TypeScript Usage

### System Integration

#### Opening URLs and Files

```typescript
import { openExternal, openPath, showItemInFolder } from "runtime:shell";

// Open URL in default browser
await openExternal("https://github.com/myproject");

// Open mailto link
await openExternal("mailto:support@example.com?subject=Help");

// Open file with default application
await openPath("./document.pdf");
await openPath("./presentation.pptx");

// Open folder in file manager
await openPath("./downloads");

// Reveal file in file manager (Finder/Explorer)
await showItemInFolder("~/Downloads/report.xlsx");
```

#### Safe File Deletion

```typescript
import { moveToTrash } from "runtime:shell";

// Move file to trash/recycle bin (recoverable)
await moveToTrash("./temp/cache.tmp");

// Delete multiple files safely
const oldFiles = ["cache.tmp", "old-data.db", "temp.log"];
for (const file of oldFiles) {
  await moveToTrash(file);
}

// With user confirmation
const confirmDelete = confirm("Move to trash?");
if (confirmDelete) {
  await moveToTrash("./old-project");
}
```

#### System Sounds and Icons

```typescript
import { beep, getFileIcon, getDefaultApp } from "runtime:shell";

// Play system beep sound
beep();

// Beep when task completes
await longRunningTask();
beep();

// Get file icon (platform-dependent, may not be supported)
try {
  const icon = await getFileIcon(".pdf", 64);
  const img = document.createElement("img");
  img.src = `data:image/png;base64,${icon.data}`;
  document.body.appendChild(img);
} catch (err) {
  console.log("Icon retrieval not supported");
}

// Query default application
const app = await getDefaultApp(".txt");
console.log(`Text files open with: ${app.name}`);
console.log(`App path: ${app.path}`);
```

### Shell Execution

#### Basic Command Execution

```typescript
import { execute } from "runtime:shell";

// Simple command
const result = await execute("echo hello");
console.log(result.stdout); // "hello\n"
console.log(result.code);   // 0

// List files
const result = await execute("ls -la");
console.log(result.stdout);

// Check exit codes
const result = await execute("test -f ./missing.txt");
if (result.code !== 0) {
  console.log("File does not exist");
}
```

#### Advanced Shell Syntax

```typescript
import { execute } from "runtime:shell";

// Pipes
const result = await execute("ls -la | grep .ts | wc -l");
console.log(`TypeScript files: ${result.stdout.trim()}`);

// Logical operators
const result = await execute("mkdir temp && cd temp && touch file.txt");

// Conditional execution
const result = await execute("npm test || npm run test:fallback");

// Redirections
const result = await execute("command > output.txt 2>&1");

// Environment variables
const result = await execute("echo $PATH && echo $HOME");
```

#### Execution Options

```typescript
import { execute } from "runtime:shell";

// Change working directory
const result = await execute("npm install", {
  cwd: "/path/to/project"
});

// Set environment variables
const result = await execute("npm test", {
  env: {
    NODE_ENV: "test",
    CI: "true",
    DEBUG: "*"
  }
});

// Set timeout
const result = await execute("long-running-command", {
  timeout: 30000  // 30 seconds
});

// Provide stdin input
const result = await execute("grep error", {
  stdin: "line 1\nerror on line 2\nline 3"
});
console.log(result.stdout); // "error on line 2\n"

// Combine all options
const result = await execute("make build", {
  cwd: "./project",
  env: { BUILD_ENV: "production" },
  timeout: 60000,
  stdin: null
});
```

### Environment Management

```typescript
import { getEnv, setEnv, unsetEnv, getAllEnv } from "runtime:shell";

// Get single variable
const home = getEnv("HOME");
const path = getEnv("PATH");

// Get with fallback
const nodeEnv = getEnv("NODE_ENV") ?? "development";

// Set variables
setEnv("RUST_LOG", "debug");
setEnv("API_KEY", "secret-123");

// Remove variable
setEnv("SECRET_KEY", "temp");
// ... use it ...
unsetEnv("SECRET_KEY");

// Get all environment variables
const env = getAllEnv();
console.log(`Total variables: ${Object.keys(env).length}`);
for (const [key, value] of Object.entries(env)) {
  console.log(`${key}=${value}`);
}

// Pass modified environment to command
const env = getAllEnv();
const result = await execute("command", {
  env: { ...env, CUSTOM_VAR: "value" }
});
```

### Working Directory Management

```typescript
import { cwd, chdir, execute } from "runtime:shell";

// Get current directory
const current = cwd();
console.log(`Working directory: ${current}`);

// Change directory
chdir("/path/to/project");

// Save and restore directory
const original = cwd();
chdir("/tmp");
// ... do work ...
chdir(original);

// Execute in specific directory (recommended)
const result = await execute("npm install", {
  cwd: "/path/to/project"  // Don't need chdir()
});
```

### Path Resolution

```typescript
import { which, execute } from "runtime:shell";

// Find executable path
const nodePath = which("node");
console.log(`Node.js is at: ${nodePath}`);

// Check if command exists
if (which("git")) {
  console.log("Git is installed");
  await execute("git --version");
} else {
  console.log("Git not found in PATH");
}

// Verify tool availability before use
const tools = ["node", "npm", "git", "cargo"];
const missing = tools.filter(tool => !which(tool));

if (missing.length > 0) {
  console.error(`Missing required tools: ${missing.join(", ")}`);
  process.exit(1);
} else {
  await execute("npm install && cargo build");
}
```

### Process Management

```typescript
import { kill } from "runtime:shell";

// Graceful termination (SIGTERM)
const handle = await spawn("long-running-server");
// ... later ...
await kill(handle);

// Force kill (SIGKILL)
await kill(handle, "SIGKILL");

// Send interrupt (SIGINT - like Ctrl+C)
await kill(handle, "SIGINT");

// Send quit signal
await kill(handle, "SIGQUIT");
```

## Permissions

Shell operations require permissions in `manifest.app.toml`:

```toml
[permissions.shell]
execute = true          # Allow shell command execution
open_external = true    # Allow opening URLs/files
trash = true            # Allow moving to trash
```

In **development mode** (`forge dev`), all permissions are automatically granted. In **production mode**, operations are strictly checked against the manifest configuration.

## Error Codes

Shell operations may throw errors with codes 8200-8214:

| Code | Error | Description |
|------|-------|-------------|
| 8200 | OpenExternalFailed | Failed to open external URL |
| 8201 | OpenPathFailed | Failed to open path with default app |
| 8202 | ShowItemFailed | Failed to show item in folder |
| 8203 | TrashFailed | Failed to move to trash |
| 8204 | BeepFailed | Failed to play system beep |
| 8205 | IconFailed | Failed to get file icon |
| 8206 | DefaultAppFailed | Failed to get default app |
| 8207 | InvalidPath | Invalid path provided |
| 8208 | PermissionDenied | Shell operation not permitted |
| 8209 | NotSupported | Operation not supported on platform |
| 8210 | ParseError | Shell command syntax error |
| 8211 | ExecutionFailed | Command execution failed |
| 8212 | Timeout | Command timed out |
| 8213 | ProcessKilled | Process was killed |
| 8214 | InvalidHandle | Invalid process handle |

### Error Handling

```typescript
import { openExternal, execute } from "runtime:shell";

try {
  await openExternal("https://example.com");
} catch (err) {
  if (err.code === 8200) {
    console.error("Failed to open URL");
  } else if (err.code === 8208) {
    console.error("Permission denied");
  }
}

// Handle command execution errors
const result = await execute("risky-command");
if (result.code !== 0) {
  console.error(`Command failed with code ${result.code}`);
  console.error(`Error output: ${result.stderr}`);
}

// Timeout handling
try {
  await execute("very-long-command", { timeout: 5000 });
} catch (err) {
  if (err.code === 8212) {
    console.error("Command timed out after 5 seconds");
  }
}
```

## Shell Syntax Support

The `execute()` function provides comprehensive shell syntax support:

### Pipes and Operators

```typescript
// Pipes
execute("cmd1 | cmd2 | cmd3")

// Logical AND (only runs cmd2 if cmd1 succeeds)
execute("cmd1 && cmd2")

// Logical OR (only runs cmd2 if cmd1 fails)
execute("cmd1 || cmd2")

// Sequential (always runs both)
execute("cmd1; cmd2")

// Background
execute("long-running-cmd &")
```

### Redirections

```typescript
// Output redirection
execute("echo hello > file.txt")

// Append
execute("echo world >> file.txt")

// Stderr redirection
execute("command 2> errors.txt")

// Combine stdout and stderr
execute("command 2>&1 | tee output.log")

// Input redirection
execute("sort < unsorted.txt > sorted.txt")
```

### Variables and Expansion

```typescript
// Environment variables
execute("echo $HOME && echo $PATH")

// Braced expansion
execute("echo ${HOME}/documents")

// Command substitution (backticks)
execute("echo Current: `pwd`")
```

### Quoting

```typescript
// Single quotes (literal)
execute("echo 'Hello $USER'")  // Prints: Hello $USER

// Double quotes (expansion)
execute('echo "Hello $USER"')  // Prints: Hello john

// Mixed quoting
execute(`echo "Path: '$PATH'"`)
```

### Globs

```typescript
// Match files
execute("ls *.ts")           // All TypeScript files
execute("ls file[0-9].txt")  // file0.txt through file9.txt
execute("ls **/*.js")        // All JS files recursively
```

## Built-in Commands

Cross-platform built-in commands (no external binaries required):

| Category | Commands |
|----------|----------|
| File Operations | `cat`, `cp`, `mv`, `rm`, `mkdir`, `ls` |
| Navigation | `cd`, `pwd` |
| Output | `echo` |
| Environment | `export`, `unset` |
| Utilities | `sleep`, `which`, `exit` |
| Piping | `head`, `xargs` |

Built-ins provide consistent behavior across platforms and don't require external dependencies.

## Common Patterns

### 1. Build Automation

```typescript
import { execute, which } from "runtime:shell";

async function buildProject() {
  // Verify tools
  const requiredTools = ["npm", "cargo"];
  const missing = requiredTools.filter(t => !which(t));
  if (missing.length > 0) {
    throw new Error(`Missing tools: ${missing.join(", ")}`);
  }

  // Install dependencies
  await execute("npm install", { cwd: "./frontend" });
  await execute("cargo build --release", { cwd: "./backend" });

  console.log("Build complete!");
}
```

### 2. File Management Automation

```typescript
import { moveToTrash, showItemInFolder } from "runtime:shell";

async function cleanupOldFiles(directory: string, daysOld: number) {
  const cutoff = Date.now() - (daysOld * 24 * 60 * 60 * 1000);
  const result = await execute(`find "${directory}" -type f -mtime +${daysOld}`);

  const oldFiles = result.stdout.trim().split("\n").filter(Boolean);

  for (const file of oldFiles) {
    await moveToTrash(file);
  }

  console.log(`Cleaned up ${oldFiles.length} files`);
}
```

### 3. Development Workflow

```typescript
import { execute, getEnv, setEnv } from "runtime:shell";

async function runTests() {
  // Set test environment
  setEnv("NODE_ENV", "test");
  setEnv("CI", "true");

  // Run tests with timeout
  try {
    const result = await execute("npm test", {
      timeout: 60000,  // 1 minute
      env: {
        NODE_ENV: "test",
        FORCE_COLOR: "1"
      }
    });

    if (result.code === 0) {
      console.log("✅ Tests passed!");
    } else {
      console.error("❌ Tests failed!");
      console.error(result.stderr);
    }
  } catch (err) {
    if (err.code === 8212) {
      console.error("⏱️ Tests timed out");
    }
  }
}
```

### 4. Cross-Platform Scripting

```typescript
import { execute, getAllEnv } from "runtime:shell";

async function detectPlatform(): Promise<string> {
  const env = getAllEnv();

  if (env.OS?.includes("Windows")) {
    return "windows";
  } else if (env.HOME?.startsWith("/Users")) {
    return "macos";
  } else {
    return "linux";
  }
}

async function platformSpecificCommand() {
  const platform = await detectPlatform();

  switch (platform) {
    case "windows":
      await execute("dir");
      break;
    case "macos":
    case "linux":
      await execute("ls -la");
      break;
  }
}
```

### 5. Interactive User Actions

```typescript
import { openExternal, showItemInFolder, beep } from "runtime:shell";

async function handleDownload(url: string, savePath: string) {
  // Download file (using external tool or fetch)
  await execute(`curl -o "${savePath}" "${url}"`);

  // Alert user
  beep();

  // Offer to reveal file
  const reveal = confirm("Download complete! Show in folder?");
  if (reveal) {
    await showItemInFolder(savePath);
  }

  // Offer to open
  const open = confirm("Open file?");
  if (open) {
    await openPath(savePath);
  }
}
```

## Platform Support

### System Integration

| Operation | macOS | Windows | Linux | Notes |
|-----------|-------|---------|-------|-------|
| openExternal | ✅ | ✅ | ✅ | Uses system default browser |
| openPath | ✅ | ✅ | ✅ | Opens with default app |
| showItemInFolder | ✅ | ✅ | ⚠️ | Linux uses dbus or fallback |
| moveToTrash | ✅ | ✅ | ✅ | Trash / Recycle Bin / freedesktop |
| beep | ✅ | ✅ | ⚠️ | Linux tries paplay, falls back to bell |
| getFileIcon | ❌ | ❌ | ❌ | Requires native bindings |
| getDefaultApp | ✅ | ✅ | ✅ | Platform-specific implementations |

✅ = Full support, ⚠️ = Partial/fallback support, ❌ = Not implemented

### Shell Execution

All shell execution operations work consistently across platforms:
- **macOS/Linux**: Uses sh-compatible shell
- **Windows**: Uses cmd.exe compatible commands
- **Built-ins**: Provide cross-platform consistency

## Testing

### Unit Tests

The extension includes comprehensive unit tests:

```bash
cargo test -p ext_shell
```

### Integration Tests

```typescript
import { execute, which, getEnv } from "runtime:shell";

// Test basic execution
const result = await execute("echo test");
console.assert(result.code === 0);
console.assert(result.stdout === "test\n");

// Test which
const nodePath = which("node");
console.assert(nodePath !== null);

// Test environment
setEnv("TEST_VAR", "value");
console.assert(getEnv("TEST_VAR") === "value");
unsetEnv("TEST_VAR");
console.assert(getEnv("TEST_VAR") === null);
```

## Build System Integration

### Extension Registration

This extension is registered in `forge-runtime` as a **Tier 1 (SimpleState)** extension:

```rust
ExtensionDescriptor {
    id: "runtime_shell",
    init_fn: ExtensionInitFn::SimpleState(|state| {
        init_shell_state(state, None);
    }),
    tier: ExtensionTier::SimpleState,
    required: false,
}
```

### Code Generation

The TypeScript bindings are generated via `forge-weld` from Rust ops:

```rust
// In build.rs
ExtensionBuilder::new("runtime_shell", "runtime:shell")
    .ts_path("ts/init.ts")
    .ops(&[
        "op_shell_open_external",
        "op_shell_open_path",
        "op_shell_show_item_in_folder",
        "op_shell_move_to_trash",
        "op_shell_beep",
        "op_shell_get_file_icon",
        "op_shell_get_default_app",
        "op_shell_execute",
        "op_shell_kill",
        "op_shell_cwd",
        "op_shell_set_cwd",
        "op_shell_get_env",
        "op_shell_set_env",
        "op_shell_unset_env",
        "op_shell_get_all_env",
        "op_shell_which",
    ])
    .use_inventory_types()
    .generate_sdk_module("../../sdk")
    .build()
```

This generates:
- `sdk/runtime.shell.ts` - TypeScript SDK module
- `ts/init.ts` → `init.js` - JavaScript shim for Deno runtime

## Implementation Details

### State Management

The extension maintains:
- **Capability Checker**: Permission validation via `ShellCapabilityChecker` trait
- **Process Registry**: Track spawned processes in `SpawnedProcessState`
- **Shell State**: Per-execution context with cwd and environment

### Shell Parser

Custom shell parser handles:
- Tokenization with quote handling
- Glob expansion
- Variable substitution
- Pipe and redirection parsing
- Command sequence parsing

### Platform-Specific Code

```rust
#[cfg(target_os = "macos")]
{
    // macOS-specific implementation
    Command::new("open").args(["-R", &path]).spawn()?;
}

#[cfg(target_os = "windows")]
{
    // Windows-specific implementation
    Command::new("explorer").args(["/select,", &path]).spawn()?;
}

#[cfg(target_os = "linux")]
{
    // Linux-specific implementation with fallback
    let result = Command::new("dbus-send").args([...]).spawn();
    if result.is_err() {
        // Fallback to opening parent folder
    }
}
```

## Security Considerations

- **Command Injection**: Commands are parsed via custom parser, not passed to shell directly
- **Path Validation**: All paths checked for existence before operations
- **URL Validation**: URLs must start with http://, https://, or mailto:
- **Capability Checks**: Every operation verifies permissions before execution
- **Timeout Protection**: Commands can be terminated if they exceed timeout

## See Also

- [`ext_process`](../ext_process/README.md) - Process spawning with full I/O control
- [`ext_fs`](../ext_fs/README.md) - Filesystem operations
- [`ext_path`](../ext_path/README.md) - Path manipulation
- [Rust documentation](https://docs.rs/ext_shell) - Full Rust API docs
