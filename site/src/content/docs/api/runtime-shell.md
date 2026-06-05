---
title: "runtime:shell"
description: Shell integration, command execution, and desktop environment interaction for Forge applications.
slug: api/runtime-shell
---

The `runtime:shell` module provides comprehensive shell integration and command execution capabilities, allowing Forge applications to interact with the operating system shell and desktop environment.

> **Implementation**: TypeScript types are auto-generated from Rust via [forge-weld](/docs/crates/forge-weld). See [ext_shell](/docs/crates/ext-shell) for implementation details.

## Features

**System Integration** (7 operations):

- Open URLs in default browser
- Launch files with default applications
- Reveal files in Finder/Explorer
- Move files to trash/recycle bin
- Play system sounds
- Get file icons (platform-dependent)
- Query default applications

**Shell Execution** (8 operations):

- Execute shell commands with full syntax support
- Process management with signals
- Working directory management
- Environment variable management
- Executable path resolution (which)

## Capabilities

Shell operations require permissions in `manifest.app.toml`:

```toml
[permissions.shell]
execute = true          # Allow shell command execution
open_external = true    # Allow opening URLs/files
trash = true            # Allow moving to trash
```

In development mode (`forge dev`), all permissions are granted.

---

## System Integration

### openExternal(url)

Open a URL in the default browser.

Works with `http://`, `https://`, `mailto:`, and other protocol handlers registered on the system.

```typescript
import { openExternal } from "runtime:shell";

// Open website
await openExternal("https://github.com/myproject");

// Open mailto link
await openExternal("mailto:support@example.com?subject=Help");

// Open custom protocol (if registered)
await openExternal("vscode://file/path/to/file.ts");
```

**Use in help menu:**

```typescript
import { openExternal } from "runtime:shell";
import { menu } from "runtime:window";

menu.setAppMenu([
  {
    label: "Help",
    submenu: [
      {
        label: "Documentation",
        click: () => openExternal("https://docs.example.com")
      },
      {
        label: "Report Issue",
        click: () => openExternal("https://github.com/user/repo/issues/new")
      }
    ]
  }
]);
```

**Parameters:**

- `url` - URL or protocol handler to open

**Throws:**

- Error [8200] if opening fails
- Error [8208] if permission denied
- Error [8209] if not supported on platform

### openPath(path)

Open a file or folder with the default application.

The system determines the appropriate application based on the file type (e.g., `.pdf` → PDF viewer, `.txt` → text editor).

```typescript
import { openPath } from "runtime:shell";

// Open a PDF file
await openPath("./document.pdf");

// Open a folder
await openPath("./my-folder");

// Open a text file
await openPath(`${getPath("documents")}/notes.txt`);
```

**Use for "Open in External Editor":**

```typescript
import { openPath } from "runtime:shell";

async function openInExternalEditor(filePath: string) {
  try {
    await openPath(filePath);
    console.log(`Opened ${filePath} in external editor`);
  } catch (error) {
    console.error("Failed to open file:", error);
  }
}
```

**Use for "Show in Folder" action:**

```typescript
import { openPath } from "runtime:shell";

// Open the containing folder
const folderPath = dirname(filePath);
await openPath(folderPath);
```

**Parameters:**

- `path` - File or folder path to open

**Throws:**

- Error [8201] if opening fails
- Error [8207] if path is invalid
- Error [8208] if permission denied

### showItemInFolder(path)

Reveal a file in the file manager (Finder/Explorer).

Opens the file manager and selects the specified file, making it easy for users to locate files in their filesystem.

```typescript
import { showItemInFolder } from "runtime:shell";

// Reveal a downloaded file
await showItemInFolder("~/Downloads/report.pdf");

// Reveal an exported file
await showItemInFolder(`${getPath("documents")}/export.csv`);
```

**Use after file save:**

```typescript
import { showItemInFolder } from "runtime:shell";
import { writeTextFile } from "runtime:fs";

async function saveAndReveal(path: string, content: string) {
  await writeTextFile(path, content);

  const shouldReveal = await confirm("File saved. Show in folder?");
  if (shouldReveal) {
    await showItemInFolder(path);
  }
}
```

**Platform behavior:**

- **macOS**: Opens Finder with file selected (`open -R`)
- **Windows**: Opens Explorer with file selected (`explorer /select`)
- **Linux**: Fallback to opening containing folder (varies by desktop environment)

**Parameters:**

- `path` - File path to reveal

**Throws:**

- Error [8202] if operation fails
- Error [8207] if path is invalid
- Error [8208] if permission denied
- Error [8209] if not fully supported on platform

### moveToTrash(path)

Move a file or folder to the trash/recycle bin.

Provides safe deletion with recovery option. Files can be restored from the trash/recycle bin using the desktop environment's tools.

```typescript
import { moveToTrash } from "runtime:shell";

// Move single file to trash
await moveToTrash("./old-file.txt");

// Clean up temporary files
const tempFiles = ["cache.tmp", "old-data.db", "temp.log"];
for (const file of tempFiles) {
  await moveToTrash(file);
}
```

**Use for "Delete" action:**

```typescript
import { moveToTrash } from "runtime:shell";

async function deleteFile(filePath: string) {
  const confirmed = await confirm(`Move "${filePath}" to trash?`);
  if (confirmed) {
    try {
      await moveToTrash(filePath);
      console.log("File moved to trash");
    } catch (error) {
      console.error("Failed to move to trash:", error);
    }
  }
}
```

**Platform behavior:**

- **macOS**: Moves to Trash
- **Windows**: Moves to Recycle Bin
- **Linux**: Uses freedesktop Trash specification

**Parameters:**

- `path` - File or folder path to move to trash

**Throws:**

- Error [8203] if operation fails
- Error [8207] if path is invalid
- Error [8208] if permission denied

### beep()

Play the system beep/alert sound.

Provides audio feedback using the operating system's default sound.

```typescript
import { beep } from "runtime:shell";

// Play system beep
await beep();
```

**Use for alerts:**

```typescript
import { beep } from "runtime:shell";

async function showAlert(message: string) {
  await beep();
  alert(message);
}

// Alert on process completion
async function longRunningTask() {
  await processData();
  await beep(); // Notify user task is done
  console.log("Task completed!");
}
```

**Platform behavior:**

- **macOS**: Uses AppleScript beep
- **Windows**: Uses PowerShell beep
- **Linux**: Attempts paplay/beep command

**Throws:**

- Error [8204] if beep fails
- Error [8209] if not supported on platform

### getFileIcon(path, size?)

Get the system icon for a file type.

**Note**: This operation is currently platform-dependent and may not be implemented on all platforms.

```typescript
import { getFileIcon } from "runtime:shell";

// Get icon for file type
const icon = await getFileIcon("./document.pdf", 32);
```

**Parameters:**

- `path` - File path or extension
- `size` - Optional icon size in pixels

**Returns:** Icon data (format varies by platform)

**Throws:**

- Error [8205] if operation fails
- Error [8209] if not supported on platform

### getDefaultApp(pathOrExtension)

Query the default application for a file type.

Returns information about which application will open the specified file or extension.

```typescript
import { getDefaultApp } from "runtime:shell";

// Get default app for file
const pdfApp = await getDefaultApp("document.pdf");
console.log(`PDF files open with: ${pdfApp}`);

// Get default app for extension
const textApp = await getDefaultApp(".txt");
console.log(`Text files open with: ${textApp}`);
```

**Use to check tool availability:**

```typescript
import { getDefaultApp } from "runtime:shell";

async function checkMarkdownSupport() {
  try {
    const mdApp = await getDefaultApp(".md");
    console.log(`Markdown files will open with: ${mdApp}`);
    return true;
  } catch {
    console.log("No default app for Markdown files");
    return false;
  }
}
```

**Platform behavior:**

- **macOS**: Uses `osascript` to query launch services
- **Windows**: Uses `assoc` command
- **Linux**: Uses `xdg-mime` command

**Parameters:**

- `pathOrExtension` - File path or extension (e.g., ".pdf", "document.txt")

**Returns:** String describing the default application

**Throws:**

- Error [8206] if operation fails
- Error [8209] if not supported on platform

---

## Shell Execution

### execute(command, options?)

Execute a shell command with full syntax support.

Provides comprehensive shell functionality including pipes, redirections, variables, and more. Returns stdout, stderr, and exit code.

```typescript
import { execute } from "runtime:shell";

// Simple command
const result = await execute("ls -la");
console.log(result.stdout);

// Command with pipes
const result = await execute("cat file.txt | grep error | wc -l");
console.log(`Error count: ${result.stdout.trim()}`);

// Command with options
const result = await execute("npm test", {
  cwd: "./my-project",
  timeout: 30000,
  env: { NODE_ENV: "test" }
});

if (result.code === 0) {
  console.log("Tests passed!");
} else {
  console.error("Tests failed:", result.stderr);
}
```

**ExecuteOptions:**

```typescript
interface ExecuteOptions {
  cwd?: string;                    // Working directory
  env?: Record<string, string>;    // Environment variables
  timeout?: number;                // Timeout in milliseconds
  shell?: string;                  // Shell to use (default: system shell)
}
```

**ExecuteResult:**

```typescript
interface ExecuteResult {
  stdout: string;    // Standard output
  stderr: string;    // Standard error
  code: number;      // Exit code (0 = success)
  signal?: string;   // Signal if process was killed
}
```

**Advanced examples:**

```typescript
// Execute with timeout
try {
  const result = await execute("long-running-command", {
    timeout: 5000 // 5 seconds
  });
} catch (error) {
  console.error("Command timed out");
}

// Execute with custom environment
const result = await execute("echo $MY_VAR", {
  env: { MY_VAR: "Hello" }
});
console.log(result.stdout); // => "Hello"

// Execute in specific directory
const result = await execute("git status", {
  cwd: "./my-repo"
});

// Check exit code
const result = await execute("test -f ./file.txt");
if (result.code === 0) {
  console.log("File exists");
} else {
  console.log("File does not exist");
}
```

**Supported shell syntax:**

- **Pipes**: `cmd1 | cmd2 | cmd3`
- **Logical Operators**: `cmd1 && cmd2`, `cmd1 || cmd2`
- **Sequences**: `cmd1; cmd2; cmd3`
- **Redirections**: `cmd > file`, `cmd 2>&1`, `cmd < input`
- **Variables**: `$VAR`, `${VAR}`
- **Quoting**: `'literal'`, `"expansion $VAR"`
- **Globs**: `*.ts`, `**/*.js`, `file[0-9].txt`
- **Background**: `cmd &`

**Built-in commands** (cross-platform):

- File: `cat`, `cp`, `mv`, `rm`, `mkdir`, `ls`
- Navigation: `cd`, `pwd`
- Output: `echo`
- Environment: `export`, `unset`
- Utilities: `sleep`, `which`, `exit`, `head`, `xargs`

**Parameters:**

- `command` - Shell command to execute
- `options` - Optional execution options

**Returns:** `ExecuteResult` with stdout, stderr, exit code

**Throws:**

- Error [8210] if command syntax is invalid
- Error [8211] if execution fails
- Error [8212] if command times out
- Error [8208] if permission denied

### cwd()

Get the current working directory.

Returns the absolute path of the current working directory for shell commands.

```typescript
import { cwd } from "runtime:shell";

const currentDir = cwd();
console.log(`Working directory: ${currentDir}`);
```

**Use for path resolution:**

```typescript
import { cwd } from "runtime:shell";

function resolveRelativePath(relativePath: string): string {
  return `${cwd()}/${relativePath}`;
}
```

**Returns:** String containing the absolute current working directory path

**Throws:**

- Error [8211] if operation fails

### setCwd(path)

Set the current working directory for shell commands.

Changes the working directory for subsequent `execute()` calls.

```typescript
import { setCwd, execute } from "runtime:shell";

// Change to project directory
setCwd("./my-project");

// Commands now execute in new directory
const result = await execute("npm install");
```

**Use for scoped operations:**

```typescript
import { cwd, setCwd, execute } from "runtime:shell";

async function runInDirectory(dir: string, callback: () => Promise<void>) {
  const originalCwd = cwd();
  try {
    setCwd(dir);
    await callback();
  } finally {
    setCwd(originalCwd); // Restore original directory
  }
}

// Usage
await runInDirectory("./subproject", async () => {
  await execute("npm test");
  await execute("npm build");
});
```

**Parameters:**

- `path` - New working directory path

**Throws:**

- Error [8207] if path is invalid or doesn't exist
- Error [8211] if operation fails

### getEnv(name)

Get an environment variable value.

Returns the value of the specified environment variable, or `null` if not set.

```typescript
import { getEnv } from "runtime:shell";

const path = getEnv("PATH");
console.log(`PATH: ${path}`);

const home = getEnv("HOME");
console.log(`Home directory: ${home}`);

// Check if variable exists
const apiKey = getEnv("API_KEY");
if (!apiKey) {
  console.error("API_KEY not set");
}
```

**Parameters:**

- `name` - Environment variable name

**Returns:** String value or `null` if not set

**Throws:**

- Error [8211] if operation fails

### setEnv(name, value)

Set an environment variable.

Sets an environment variable for the current process and child processes spawned by `execute()`.

```typescript
import { setEnv, execute } from "runtime:shell";

// Set environment variable
setEnv("RUST_LOG", "debug");
setEnv("API_KEY", "secret-123");
setEnv("NODE_ENV", "production");

// Variable is available in executed commands
const result = await execute("echo $API_KEY");
console.log(result.stdout); // => "secret-123"
```

**Use for configuration:**

```typescript
import { setEnv, execute } from "runtime:shell";

function configureEnvironment(config: AppConfig) {
  setEnv("APP_ENV", config.environment);
  setEnv("LOG_LEVEL", config.logLevel);
  setEnv("API_URL", config.apiUrl);
}

configureEnvironment({
  environment: "production",
  logLevel: "info",
  apiUrl: "https://api.example.com"
});

// Now execute commands with configured environment
await execute("node server.js");
```

**Parameters:**

- `name` - Environment variable name
- `value` - Value to set

**Throws:**

- Error [8211] if operation fails

### unsetEnv(name)

Remove an environment variable.

Deletes the specified environment variable.

```typescript
import { unsetEnv, getEnv } from "runtime:shell";

// Remove environment variable
unsetEnv("TEMP_VAR");

// Verify it's gone
const value = getEnv("TEMP_VAR");
console.log(value); // => null
```

**Use for cleanup:**

```typescript
import { setEnv, unsetEnv, execute } from "runtime:shell";

async function runWithTempEnv(vars: Record<string, string>, command: string) {
  // Set temporary variables
  for (const [name, value] of Object.entries(vars)) {
    setEnv(name, value);
  }

  try {
    return await execute(command);
  } finally {
    // Clean up
    for (const name of Object.keys(vars)) {
      unsetEnv(name);
    }
  }
}
```

**Parameters:**

- `name` - Environment variable name to remove

**Throws:**

- Error [8211] if operation fails

### getAllEnv()

Get all environment variables.

Returns an object containing all environment variables and their values.

```typescript
import { getAllEnv } from "runtime:shell";

const env = getAllEnv();
console.log(`Total variables: ${Object.keys(env).length}`);

// List all variables
for (const [name, value] of Object.entries(env)) {
  console.log(`${name}=${value}`);
}

// Find specific variables
const pathVars = Object.keys(env).filter(k => k.includes("PATH"));
console.log("PATH-related variables:", pathVars);
```

**Use for debugging:**

```typescript
import { getAllEnv } from "runtime:shell";

function debugEnvironment() {
  const env = getAllEnv();
  const relevant = {
    NODE_ENV: env.NODE_ENV,
    PATH: env.PATH,
    HOME: env.HOME,
    USER: env.USER
  };
  console.log("Environment:", relevant);
}
```

**Returns:** Object with environment variable names as keys and their values as string values

**Throws:**

- Error [8211] if operation fails

### which(command)

Find the path to an executable.

Searches the system PATH for the specified command and returns its full path, or `null` if not found.

```typescript
import { which } from "runtime:shell";

// Check if git is installed
const gitPath = which("git");
if (gitPath) {
  console.log(`Git found at: ${gitPath}`);
} else {
  console.log("Git not found");
}

// Check multiple tools
const tools = ["node", "npm", "cargo", "python"];
const available = tools.filter(tool => which(tool) !== null);
console.log(`Available tools: ${available.join(", ")}`);
```

**Use for dependency checking:**

```typescript
import { which, execute } from "runtime:shell";

async function runIfAvailable(command: string, args: string = "") {
  const cmdPath = which(command);

  if (!cmdPath) {
    throw new Error(`${command} is not installed`);
  }

  console.log(`Using ${command} from: ${cmdPath}`);
  return await execute(`${command} ${args}`);
}

// Usage
try {
  await runIfAvailable("git", "status");
  await runIfAvailable("npm", "test");
} catch (error) {
  console.error(error.message);
}
```

**Parameters:**

- `command` - Command name to search for

**Returns:** String containing full path to executable, or `null` if not found

**Throws:**

- Error [8211] if operation fails

---

## Error Handling

All operations throw on error:

```typescript
import { execute } from "runtime:shell";

try {
  const result = await execute("invalid-command");
} catch (error) {
  if (error.message.includes("8211")) {
    console.log("Command execution failed");
  } else if (error.message.includes("8212")) {
    console.log("Command timed out");
  } else if (error.message.includes("8208")) {
    console.log("Permission denied");
  }
}
```

---

## Error Codes

| Code | Error | Description |
|------|-------|-------------|
| `8200` | OpenExternalFailed | Failed to open external URL |
| `8201` | OpenPathFailed | Failed to open path with default app |
| `8202` | ShowItemFailed | Failed to show item in folder |
| `8203` | TrashFailed | Failed to move to trash |
| `8204` | BeepFailed | Failed to play system beep |
| `8205` | IconFailed | Failed to get file icon |
| `8206` | DefaultAppFailed | Failed to get default app |
| `8207` | InvalidPath | Invalid path provided |
| `8208` | PermissionDenied | Shell operation not permitted |
| `8209` | NotSupported | Operation not supported on platform |
| `8210` | ParseError | Shell command syntax error |
| `8211` | ExecutionFailed | Command execution failed |
| `8212` | Timeout | Command timed out |
| `8213` | ProcessKilled | Process was terminated by signal |
| `8214` | InvalidHandle | Invalid process handle |

---

## Platform Support

### System Integration

| Operation | macOS | Windows | Linux |
|-----------|-------|---------|-------|
| openExternal | ✅ | ✅ | ✅ |
| openPath | ✅ | ✅ | ✅ |
| showItemInFolder | ✅ `open -R` | ✅ `explorer /select` | ⚠️ fallback |
| moveToTrash | ✅ Trash | ✅ Recycle Bin | ✅ freedesktop |
| beep | ✅ AppleScript | ✅ PowerShell | ⚠️ paplay |
| getFileIcon | ❌ | ❌ | ❌ |
| getDefaultApp | ✅ osascript | ✅ assoc | ✅ xdg-mime |

### Shell Execution

All shell execution operations work consistently across platforms:

- **macOS/Linux**: Uses sh-compatible shell
- **Windows**: Uses cmd.exe compatible commands
- **Built-ins**: Cross-platform implementations

---

## Complete Example

```typescript
import {
  openExternal,
  openPath,
  showItemInFolder,
  moveToTrash,
  execute,
  which,
  cwd,
  setCwd,
  getEnv,
  setEnv,
  getAllEnv
} from "runtime:shell";

// Example 1: Project setup and build

async function setupProject(projectPath: string) {
  console.log(`Setting up project at ${projectPath}`);

  // Check dependencies
  const requiredTools = ["node", "npm", "git"];
  const missing = requiredTools.filter(tool => !which(tool));

  if (missing.length > 0) {
    throw new Error(`Missing required tools: ${missing.join(", ")}`);
  }

  // Change to project directory
  const originalCwd = cwd();
  setCwd(projectPath);

  try {
    // Install dependencies
    console.log("Installing dependencies...");
    const installResult = await execute("npm install", {
      timeout: 120000 // 2 minutes
    });

    if (installResult.code !== 0) {
      throw new Error(`npm install failed: ${installResult.stderr}`);
    }

    // Run tests
    console.log("Running tests...");
    const testResult = await execute("npm test", {
      env: { NODE_ENV: "test" }
    });

    if (testResult.code !== 0) {
      console.error("Tests failed:", testResult.stderr);
      return false;
    }

    // Build project
    console.log("Building project...");
    const buildResult = await execute("npm run build");

    if (buildResult.code !== 0) {
      throw new Error(`Build failed: ${buildResult.stderr}`);
    }

    console.log("Project setup complete!");
    return true;
  } finally {
    // Restore original directory
    setCwd(originalCwd);
  }
}

// Example 2: File operations with user interaction

async function exportAndReveal(data: any, filename: string) {
  // Export data to file
  const exportPath = `${getPath("documents")}/${filename}`;
  await writeTextFile(exportPath, JSON.stringify(data, null, 2));

  // Ask user what to do
  const action = await confirm(
    "File exported successfully. What would you like to do?\n\n" +
    "OK = Show in folder\n" +
    "Cancel = Open in default app"
  );

  if (action) {
    await showItemInFolder(exportPath);
  } else {
    await openPath(exportPath);
  }
}

// Example 3: Development environment detection

function checkDevelopmentEnvironment() {
  const env = getAllEnv();

  const devTools = {
    node: which("node"),
    npm: which("npm"),
    git: which("git"),
    cargo: which("cargo"),
    python: which("python")
  };

  const installed = Object.entries(devTools)
    .filter(([_, path]) => path !== null)
    .map(([name, path]) => `${name} (${path})`);

  const missing = Object.entries(devTools)
    .filter(([_, path]) => path === null)
    .map(([name]) => name);

  console.log("Installed development tools:");
  installed.forEach(tool => console.log(`  ✓ ${tool}`));

  if (missing.length > 0) {
    console.log("\nMissing tools:");
    missing.forEach(tool => console.log(`  ✗ ${tool}`));
  }

  return { installed, missing };
}

// Example 4: Clean up temporary files

async function cleanupTempFiles() {
  const tempDir = getPath("temp");
  setCwd(tempDir);

  // Find temp files
  const result = await execute("ls *.tmp *.cache 2>/dev/null || true");

  if (result.stdout.trim()) {
    const files = result.stdout.trim().split("\n");
    console.log(`Found ${files.length} temporary files`);

    const confirmed = await confirm(
      `Delete ${files.length} temporary files?`
    );

    if (confirmed) {
      for (const file of files) {
        await moveToTrash(`${tempDir}/${file}`);
      }
      console.log("Temporary files moved to trash");
    }
  } else {
    console.log("No temporary files found");
  }
}

// Example 5: Git integration

async function gitStatus(repoPath: string) {
  if (!which("git")) {
    throw new Error("Git is not installed");
  }

  const result = await execute("git status --porcelain", {
    cwd: repoPath
  });

  if (result.code !== 0) {
    throw new Error("Not a git repository or git command failed");
  }

  const changes = result.stdout.trim().split("\n").filter(Boolean);
  return {
    hasChanges: changes.length > 0,
    changedFiles: changes.map(line => line.slice(3))
  };
}
```
