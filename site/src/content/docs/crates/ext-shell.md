---
title: "ext_shell - Shell Integration and Execution"
description: Shell integration and command execution for Forge applications
slug: crates/ext-shell
---

The `ext_shell` crate provides comprehensive shell integration and command execution through the `runtime:shell` module. Bridge your Forge applications with the operating system shell and desktop environment.

## Overview

Shell operations fall into two main categories:

**System Integration** - Desktop environment interaction:
- 🌐 Open URLs in default browser
- 📂 Open files/folders with default applications
- 👁️ Reveal files in file manager (Finder/Explorer)
- 🗑️ Move files to trash/recycle bin
- 🔊 Play system beep sounds
- 🖼️ Query file icons and default applications

**Shell Execution** - Full-featured command execution:
- 🐚 Execute shell commands with full syntax support
- 🌍 Manage environment variables
- 📁 Control working directory
- 🔍 Resolve executable paths
- ⚙️ Manage background processes

## Quick Start

```typescript
import {
  openExternal,
  openPath,
  showItemInFolder,
  moveToTrash,
  execute,
  which,
  getEnv
} from "runtime:shell";

// System Integration
await openExternal("https://github.com");
await openPath("./document.pdf");
await showItemInFolder("~/Downloads/file.pdf");
await moveToTrash("./old-file.txt");

// Shell Execution
const result = await execute("ls -la");
console.log(result.stdout);

// Environment & Path Resolution
const home = getEnv("HOME");
const gitPath = which("git");
```

## Module: `runtime:shell`

Import functions from the shell module:

```typescript
import {
  // System Integration
  openExternal,
  openPath,
  showItemInFolder,
  moveToTrash,
  beep,
  getFileIcon,
  getDefaultApp,
  // Shell Execution
  execute,
  kill,
  cwd,
  chdir,
  getEnv,
  setEnv,
  unsetEnv,
  getAllEnv,
  which,
  // Aliases
  open,       // alias for openExternal
  trash,      // alias for moveToTrash
  exec,       // alias for execute
  run         // alias for execute
} from "runtime:shell";
```

## Core Concepts

### System Integration

System integration operations interact with the desktop environment without executing shell commands. They use platform-specific APIs for seamless OS integration.

**Key Characteristics:**
- No command execution involved
- Platform-specific implementations
- Safe for user interaction
- Respects system defaults

### Shell Execution

Shell execution provides a full-featured shell environment with support for pipes, redirections, variables, and globs. Commands execute in a controlled environment with timeout and permission management.

**Supported Syntax:**
- Pipes: `cmd1 | cmd2`
- Logical operators: `cmd1 && cmd2`, `cmd1 || cmd2`
- Redirections: `cmd > file`, `cmd 2>&1`
- Variables: `$VAR`, `${VAR}`
- Globs: `*.ts`, `**/*.js`

### Built-in Commands

Cross-platform built-ins work consistently without external dependencies:
- File operations: `cat`, `cp`, `mv`, `rm`, `mkdir`, `ls`
- Navigation: `cd`, `pwd`
- Environment: `export`, `unset`
- Utilities: `echo`, `sleep`, `which`, `exit`

## API Reference

### System Integration Functions

#### `openExternal(url: string): Promise<void>`

Opens a URL in the default web browser.

**Parameters:**
- `url` - URL to open (must start with `http://`, `https://`, or `mailto:`)

**Throws:**
- Error [8200] if opening fails
- Error [8207] if URL format invalid
- Error [8208] if permission denied

**Examples:**

```typescript
// Open website
await openExternal("https://github.com/myproject");

// Open email client
await openExternal("mailto:support@example.com?subject=Help");

// Handle errors
try {
  await openExternal("https://example.com");
} catch (err) {
  console.error("Failed to open URL:", err);
}
```

#### `openPath(path: string): Promise<void>`

Opens a file or folder with its default application.

**Parameters:**
- `path` - Path to file or folder

**Throws:**
- Error [8201] if opening fails
- Error [8207] if path doesn't exist

**Examples:**

```typescript
// Open file in default app
await openPath("./document.pdf");
await openPath("./presentation.pptx");

// Open folder in file manager
await openPath("./downloads");
```

#### `showItemInFolder(path: string): Promise<void>`

Reveals a file in its containing folder (Finder on macOS, Explorer on Windows).

**Parameters:**
- `path` - Path to file to reveal

**Throws:**
- Error [8202] if operation fails
- Error [8207] if path doesn't exist

**Platform Behavior:**
- **macOS**: Uses `open -R` to reveal in Finder
- **Windows**: Uses `explorer /select,` to select in Explorer
- **Linux**: Attempts dbus-send, falls back to opening parent folder

**Examples:**

```typescript
// Show downloaded file
await showItemInFolder("~/Downloads/report.xlsx");

// Reveal generated output
await showItemInFolder("./build/app.exe");
```

#### `moveToTrash(path: string): Promise<void>`

Moves a file or folder to the trash/recycle bin.

**Parameters:**
- `path` - Path to file or folder

**Throws:**
- Error [8203] if operation fails
- Error [8207] if path doesn't exist

**Examples:**

```typescript
// Delete file safely
await moveToTrash("./temp/cache.tmp");

// Delete with confirmation
const confirm = window.confirm("Move to trash?");
if (confirm) {
  await moveToTrash("./old-project");
}

// Delete multiple files
for (const file of oldFiles) {
  await moveToTrash(file);
}
```

#### `beep(): void`

Plays the system beep sound.

**Examples:**

```typescript
// Alert user when task completes
await longRunningTask();
beep();

// Beep on error
try {
  await riskyOperation();
} catch (err) {
  beep();
  console.error("Operation failed");
}
```

#### `getFileIcon(path: string, size?: number): Promise<FileIcon>`

Retrieves the system icon for a file type.

**Note:** Requires platform-specific native bindings and may throw "not supported" errors.

**Parameters:**
- `path` - File path or extension
- `size` - Icon size in pixels (default: 32)

**Returns:** Object with `data` (base64 PNG), `width`, and `height`

**Throws:**
- Error [8205] if operation fails
- Error [8209] if not supported on platform

**Examples:**

```typescript
try {
  const icon = await getFileIcon(".pdf", 64);
  const img = document.createElement("img");
  img.src = `data:image/png;base64,${icon.data}`;
  document.body.appendChild(img);
} catch (err) {
  console.log("Icon retrieval not supported");
}
```

#### `getDefaultApp(pathOrExtension: string): Promise<DefaultAppInfo>`

Queries the default application for a file type.

**Parameters:**
- `pathOrExtension` - File path or extension (e.g., ".txt")

**Returns:** Object with `name`, `path`, and `identifier` (may be null)

**Platform Fields:**
- **macOS**: Returns app path/name and bundle identifier
- **Windows**: Returns ProgID from registry
- **Linux**: Returns .desktop file name via xdg-mime

**Examples:**

```typescript
// Query default text editor
const app = await getDefaultApp(".txt");
console.log(`Text files open with: ${app.name}`);

// Check if default app exists
const app = await getDefaultApp("./document.pdf");
if (app.name) {
  console.log(`Will open with: ${app.name}`);
} else {
  console.log("No default app configured");
}
```

### Shell Execution Functions

#### `execute(command: string, options?: ExecuteOptions): Promise<ExecuteOutput>`

Executes a shell command and waits for completion.

**Parameters:**
- `command` - Shell command string
- `options` - Optional execution options:
  - `cwd?: string` - Working directory
  - `env?: Record<string, string>` - Environment variables
  - `timeout?: number` - Timeout in milliseconds
  - `stdin?: string` - Input to send to stdin

**Returns:** Object with `code`, `stdout`, and `stderr`

**Throws:**
- Error [8210] if syntax invalid
- Error [8211] if execution fails
- Error [8212] if timeout occurs

**Examples:**

```typescript
// Simple command
const result = await execute("echo hello");
console.log(result.stdout); // "hello\n"

// With pipes
const result = await execute("ls | grep .ts | wc -l");
console.log(`TypeScript files: ${result.stdout.trim()}`);

// With options
const result = await execute("npm test", {
  cwd: "./my-project",
  timeout: 30000,
  env: {
    NODE_ENV: "test",
    CI: "true"
  }
});

// Handle exit codes
if (result.code !== 0) {
  console.error("Command failed:", result.stderr);
}

// With stdin
const result = await execute("grep error", {
  stdin: "line 1\nerror here\nline 3"
});
```

#### `kill(handle: SpawnHandle, signal?: string): Promise<void>`

Terminates a spawned background process.

**Parameters:**
- `handle` - Process handle from spawn()
- `signal` - Signal to send (default: "SIGTERM")

**Available Signals:**
- `SIGTERM` - Graceful termination
- `SIGKILL` or `9` - Forceful termination
- `SIGINT` or `2` - Interrupt (Ctrl+C)
- `SIGQUIT` or `3` - Quit with core dump

**Examples:**

```typescript
// Graceful termination
const handle = await spawn("server");
await kill(handle);

// Force kill
await kill(handle, "SIGKILL");
```

#### `cwd(): string`

Gets the current working directory.

**Returns:** Absolute path of current working directory

**Examples:**

```typescript
const current = cwd();
console.log(`Working directory: ${current}`);

// Save and restore
const original = cwd();
chdir("/tmp");
// ... work ...
chdir(original);
```

#### `chdir(path: string): void`

Changes the current working directory.

**Parameters:**
- `path` - Directory path (relative or absolute)

**Throws:**
- Error [8211] if directory doesn't exist

**Examples:**

```typescript
chdir("/path/to/project");
await execute("npm install");

// Relative paths
chdir("../other-project");
```

#### `getEnv(name: string): string | null`

Gets an environment variable value.

**Parameters:**
- `name` - Variable name (case-sensitive)

**Returns:** Variable value or null if not set

**Examples:**

```typescript
const home = getEnv("HOME");
const path = getEnv("PATH");

// With fallback
const nodeEnv = getEnv("NODE_ENV") ?? "development";

// Check if set
if (getEnv("DEBUG")) {
  console.log("Debug mode enabled");
}
```

#### `setEnv(name: string, value: string): void`

Sets an environment variable.

**Parameters:**
- `name` - Variable name
- `value` - Variable value

**Examples:**

```typescript
setEnv("RUST_LOG", "debug");
setEnv("API_KEY", "secret-123");

// For child processes
setEnv("NODE_ENV", "production");
await execute("npm run build");
```

#### `unsetEnv(name: string): void`

Removes an environment variable.

**Parameters:**
- `name` - Variable name

**Examples:**

```typescript
// Remove sensitive data
setEnv("SECRET_KEY", "temp");
// ... use it ...
unsetEnv("SECRET_KEY");

// Clear debug flag
unsetEnv("DEBUG");
```

#### `getAllEnv(): Record<string, string>`

Gets all environment variables.

**Returns:** Object with all environment variables

**Examples:**

```typescript
const env = getAllEnv();
console.log(`PATH: ${env.PATH}`);
console.log(`Total variables: ${Object.keys(env).length}`);

// Pass modified environment
const result = await execute("command", {
  env: { ...getAllEnv(), CUSTOM_VAR: "value" }
});

// List all variables
for (const [key, value] of Object.entries(getAllEnv())) {
  console.log(`${key}=${value}`);
}
```

#### `which(command: string): string | null`

Finds the full path to an executable in PATH.

**Parameters:**
- `command` - Command name to find

**Returns:** Full path or null if not found

**Examples:**

```typescript
const nodePath = which("node");
console.log(`Node.js is at: ${nodePath}`);

// Check if command exists
if (which("git")) {
  await execute("git --version");
} else {
  console.error("Git not found");
}

// Verify tools
const tools = ["node", "npm", "git"];
const missing = tools.filter(t => !which(t));
if (missing.length > 0) {
  throw new Error(`Missing: ${missing.join(", ")}`);
}
```

## Usage Examples

### Build Automation

```typescript
import { execute, which } from "runtime:shell";

async function buildProject() {
  // Verify required tools
  const tools = ["npm", "cargo"];
  const missing = tools.filter(t => !which(t));
  if (missing.length > 0) {
    throw new Error(`Missing tools: ${missing.join(", ")}`);
  }

  // Build frontend
  await execute("npm install", { cwd: "./frontend" });
  await execute("npm run build", {
    cwd: "./frontend",
    env: { NODE_ENV: "production" }
  });

  // Build backend
  await execute("cargo build --release", {
    cwd: "./backend"
  });

  console.log("✅ Build complete!");
}
```

### File Management

```typescript
import { moveToTrash, showItemInFolder, openPath } from "runtime:shell";

async function cleanupOldFiles(dir: string, days: number) {
  const result = await execute(
    `find "${dir}" -type f -mtime +${days} -print`
  );

  const oldFiles = result.stdout
    .trim()
    .split("\n")
    .filter(Boolean);

  console.log(`Found ${oldFiles.length} old files`);

  for (const file of oldFiles) {
    await moveToTrash(file);
  }

  // Show cleanup location
  if (oldFiles.length > 0) {
    await showItemInFolder(oldFiles[0]);
  }
}
```

### Development Workflow

```typescript
import { execute, setEnv, beep } from "runtime:shell";

async function runTests() {
  setEnv("NODE_ENV", "test");
  setEnv("CI", "true");

  try {
    const result = await execute("npm test", {
      timeout: 60000
    });

    if (result.code === 0) {
      console.log("✅ Tests passed!");
      beep();
    } else {
      console.error("❌ Tests failed!");
      console.error(result.stderr);
    }
  } catch (err) {
    if (err.code === 8212) {
      console.error("⏱️ Tests timed out");
    }
    throw err;
  }
}
```

### Cross-Platform Commands

```typescript
import { execute, getAllEnv } from "runtime:shell";

function detectPlatform(): string {
  const env = getAllEnv();
  if (env.OS?.includes("Windows")) return "windows";
  if (env.HOME?.startsWith("/Users")) return "macos";
  return "linux";
}

async function listFiles() {
  const platform = detectPlatform();

  switch (platform) {
    case "windows":
      await execute("dir /b");
      break;
    default:
      await execute("ls -1");
  }
}
```

### Interactive Downloads

```typescript
import { openExternal, showItemInFolder, beep } from "runtime:shell";

async function handleDownload(url: string, savePath: string) {
  await execute(`curl -o "${savePath}" "${url}"`);

  beep();

  if (confirm("Download complete! Show in folder?")) {
    await showItemInFolder(savePath);
  }

  if (confirm("Open file?")) {
    await openPath(savePath);
  }
}
```

## Best Practices

### ✅ Do: Use Execute Options Instead of chdir()

Prefer passing `cwd` to `execute()` rather than changing global directory:

```typescript
// ✅ Good - isolated to command
const result = await execute("npm install", {
  cwd: "/path/to/project"
});

// ❌ Bad - affects all subsequent operations
chdir("/path/to/project");
await execute("npm install");
```

### ✅ Do: Check Tool Availability

Always verify tools exist before use:

```typescript
// ✅ Good
if (!which("git")) {
  throw new Error("Git is required but not installed");
}
await execute("git clone ...");

// ❌ Bad - will fail cryptically
await execute("git clone ...");
```

### ✅ Do: Handle Non-Zero Exit Codes

Check exit codes even when commands don't throw:

```typescript
// ✅ Good
const result = await execute("test -f file.txt");
if (result.code !== 0) {
  console.log("File doesn't exist");
}

// ❌ Bad - ignores failure
await execute("test -f file.txt");
```

### ✅ Do: Use Timeouts for Long Operations

Prevent hanging on long-running commands:

```typescript
// ✅ Good
const result = await execute("npm install", {
  timeout: 300000  // 5 minutes
});

// ❌ Bad - could hang forever
await execute("npm install");
```

### ✅ Do: Sanitize User Input in Commands

Never pass unsanitized user input to shell:

```typescript
// ✅ Good
const filename = userInput.replace(/[^a-zA-Z0-9._-]/g, "");
await execute(`cat "${filename}"`);

// ❌ Bad - command injection risk
await execute(`cat "${userInput}"`);
```

## Common Pitfalls

### ❌ Command Injection Vulnerabilities

User input in commands can lead to injection attacks:

```typescript
// ❌ Wrong - injection risk
const userFile = req.body.filename;
await execute(`rm "${userFile}"`);
// If userFile is: "; rm -rf /"

// ✅ Correct - validate input
const safeFile = userFile.replace(/[^a-zA-Z0-9._-]/g, "");
if (safeFile !== userFile) {
  throw new Error("Invalid filename");
}
await execute(`rm "${safeFile}"`);
```

### ❌ Ignoring Exit Codes

Not checking exit codes masks failures:

```typescript
// ❌ Wrong - doesn't check if succeeded
await execute("git push");
console.log("Pushed successfully!");

// ✅ Correct - verify success
const result = await execute("git push");
if (result.code !== 0) {
  console.error("Push failed:", result.stderr);
  throw new Error("Git push failed");
}
```

### ❌ Platform-Specific Commands

Using platform-specific commands breaks cross-platform support:

```typescript
// ❌ Wrong - breaks on Windows
await execute("ls -la");

// ✅ Correct - use built-ins or detect platform
const platform = detectPlatform();
if (platform === "windows") {
  await execute("dir");
} else {
  await execute("ls -la");
}

// ✅ Better - use built-in ls
await execute("ls");
```

### ❌ Global State Changes

Changing global state affects all operations:

```typescript
// ❌ Wrong - affects everything
setEnv("DEBUG", "*");
chdir("/tmp");
// ... now all commands run in /tmp with DEBUG=*

// ✅ Correct - scope to command
const result = await execute("my-command", {
  cwd: "/tmp",
  env: { ...getAllEnv(), DEBUG: "*" }
});
```

### ❌ Blocking on Long Operations

Not using timeouts can hang indefinitely:

```typescript
// ❌ Wrong - could hang forever
await execute("npm install");

// ✅ Correct - set reasonable timeout
try {
  await execute("npm install", { timeout: 300000 });
} catch (err) {
  if (err.code === 8212) {
    console.error("npm install timed out after 5 minutes");
  }
  throw err;
}
```

## Error Handling

### Error Codes

| Code | Error | When It Occurs |
|------|-------|----------------|
| 8200 | OpenExternalFailed | Browser launch fails |
| 8201 | OpenPathFailed | Default app launch fails |
| 8202 | ShowItemFailed | File reveal fails |
| 8203 | TrashFailed | Move to trash fails |
| 8204 | BeepFailed | System beep fails |
| 8205 | IconFailed | Icon retrieval fails |
| 8206 | DefaultAppFailed | Default app query fails |
| 8207 | InvalidPath | Path doesn't exist or is invalid |
| 8208 | PermissionDenied | Operation not permitted |
| 8209 | NotSupported | Platform doesn't support operation |
| 8210 | ParseError | Command syntax error |
| 8211 | ExecutionFailed | Command execution fails |
| 8212 | Timeout | Command exceeds timeout |
| 8213 | ProcessKilled | Process was terminated |
| 8214 | InvalidHandle | Process handle is invalid |

### Handling Specific Errors

```typescript
import { execute, openExternal } from "runtime:shell";

try {
  await openExternal("https://example.com");
} catch (err) {
  switch (err.code) {
    case 8200:
      console.error("Failed to open URL");
      break;
    case 8208:
      console.error("Permission denied - check manifest.app.toml");
      break;
    default:
      console.error("Unknown error:", err);
  }
}

// Command execution errors
try {
  const result = await execute("risky-command", {
    timeout: 5000
  });
  if (result.code !== 0) {
    console.error("Command failed:", result.stderr);
  }
} catch (err) {
  if (err.code === 8212) {
    console.error("Command timed out");
  } else if (err.code === 8210) {
    console.error("Syntax error:", err.message);
  }
}
```

## Platform Support

### System Integration Support

| Operation | macOS | Windows | Linux | Notes |
|-----------|-------|---------|-------|-------|
| openExternal | ✅ | ✅ | ✅ | Uses system browser |
| openPath | ✅ | ✅ | ✅ | Uses default app |
| showItemInFolder | ✅ | ✅ | ⚠️ | Linux uses dbus or fallback |
| moveToTrash | ✅ | ✅ | ✅ | Trash/Recycle Bin/freedesktop |
| beep | ✅ | ✅ | ⚠️ | Linux tries paplay, falls back |
| getFileIcon | ❌ | ❌ | ❌ | Requires native bindings |
| getDefaultApp | ✅ | ✅ | ✅ | Platform-specific queries |

✅ Full support | ⚠️ Partial/fallback | ❌ Not implemented

### Shell Execution Support

All shell execution operations work consistently across platforms:
- **macOS/Linux**: sh-compatible shell
- **Windows**: cmd.exe compatible
- **Built-ins**: Cross-platform consistency

### Platform Detection

```typescript
import { getAllEnv } from "runtime:shell";

function detectPlatform(): "windows" | "macos" | "linux" {
  const env = getAllEnv();
  if (env.OS?.includes("Windows")) return "windows";
  if (env.HOME?.startsWith("/Users")) return "macos";
  return "linux";
}
```

## Permissions

Configure shell permissions in `manifest.app.toml`:

```toml
[permissions.shell]
execute = true          # Allow shell command execution
open_external = true    # Allow opening URLs/files
trash = true            # Allow moving to trash
```

**Development Mode** (`forge dev`):
- All permissions automatically granted
- No configuration required for testing

**Production Mode** (`forge build`):
- Strict permission enforcement
- Operations fail with [8208] if not permitted

## See Also

- [ext_process](/docs/crates/ext-process) - Process spawning with I/O control
- [ext_fs](/docs/crates/ext-fs) - Filesystem operations
- [ext_path](/docs/crates/ext-path) - Path manipulation
- [Getting Started](/docs/getting-started) - Forge introduction
- [Architecture](/docs/architecture) - System architecture
