---
title: "host:process"
description: Child process spawning and management capabilities.
---

The `host:process` module provides child process spawning and management capabilities.

## Capabilities

Process spawning requires capability declarations:

```toml
[capabilities.process]
spawn = ["git", "npm", "node"]  # Allowed binaries
```

---

## Spawning Processes

### spawn(binary, options?)

Spawn a child process:

```typescript
import { spawn } from "host:process";

const proc = await spawn("ls", {
  args: ["-la", "/tmp"],
  cwd: "/home/user",
  env: { "MY_VAR": "value" }
});

console.log(`Started process with PID: ${proc.pid}`);
```

**Options:**

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `args` | `string[]` | `[]` | Command line arguments |
| `env` | `Record<string, string>` | - | Environment variables |
| `cwd` | `string` | - | Working directory |
| `stdout` | `"piped" \| "inherit" \| "null"` | `"piped"` | Stdout handling |
| `stderr` | `"piped" \| "inherit" \| "null"` | `"piped"` | Stderr handling |
| `stdin` | `"piped" \| "inherit" \| "null"` | `"null"` | Stdin handling |

**Returns:** `Promise<ChildProcess>`

---

## ChildProcess Interface

```typescript
interface ChildProcess {
  readonly id: string;    // Internal handle ID
  readonly pid: number;   // OS process ID

  // Process control
  kill(signal?: string): Promise<void>;
  wait(): Promise<number>;
  status(): Promise<ProcessStatus>;

  // I/O (when piped)
  writeStdin(data: string): Promise<void>;
  readStdout(): Promise<ProcessOutput>;
  readStderr(): Promise<ProcessOutput>;

  // Async iterators for stdout/stderr
  stdout: AsyncIterable<string>;
  stderr: AsyncIterable<string>;
}
```

---

## Reading Output

### Using Async Iterators

```typescript
const proc = await spawn("npm", { args: ["install"] });

// Read stdout line by line
for await (const line of proc.stdout) {
  console.log("stdout:", line);
}

// Read stderr line by line
for await (const line of proc.stderr) {
  console.error("stderr:", line);
}
```

### Using readStdout/readStderr

```typescript
const proc = await spawn("echo", { args: ["hello"] });

const output = await proc.readStdout();
if (!output.eof) {
  console.log(output.data);
}
```

**ProcessOutput:**

```typescript
interface ProcessOutput {
  data: string | null;  // Line of output, null if EOF
  eof: boolean;         // True if stream ended
}
```

---

## Writing Input

```typescript
const proc = await spawn("cat", {
  stdin: "piped"
});

await proc.writeStdin("Hello, World!\n");
await proc.writeStdin("Goodbye!\n");

// Close stdin to signal EOF (process may wait for this)
// Note: No explicit close needed - process will receive EOF when handle is dropped
```

---

## Process Control

### Waiting for Exit

```typescript
const proc = await spawn("sleep", { args: ["5"] });

const exitCode = await proc.wait();
console.log(`Process exited with code: ${exitCode}`);
```

### Checking Status

```typescript
const proc = await spawn("long-running-task");

const status = await proc.status();
if (status.running) {
  console.log("Still running...");
} else {
  console.log(`Exited with code: ${status.exitCode}`);
}
```

**ProcessStatus:**

```typescript
interface ProcessStatus {
  running: boolean;
  exitCode?: number;
  signal?: string;  // Unix signal that killed the process
}
```

### Killing a Process

```typescript
const proc = await spawn("long-running-task");

// Default signal (SIGTERM)
await proc.kill();

// Specific signal (Unix only)
await proc.kill("SIGKILL");
```

---

## Common Patterns

### Run Command and Get Output

```typescript
async function runCommand(cmd: string, args: string[]): Promise<string> {
  const proc = await spawn(cmd, { args });

  let output = "";
  for await (const line of proc.stdout) {
    output += line + "\n";
  }

  const exitCode = await proc.wait();
  if (exitCode !== 0) {
    throw new Error(`Command failed with exit code ${exitCode}`);
  }

  return output.trim();
}

// Usage
const gitStatus = await runCommand("git", ["status", "--short"]);
```

### Run with Timeout

```typescript
async function runWithTimeout(
  cmd: string,
  args: string[],
  timeoutMs: number
): Promise<string> {
  const proc = await spawn(cmd, { args });

  const timeoutPromise = new Promise<never>((_, reject) => {
    setTimeout(() => {
      proc.kill();
      reject(new Error("Process timed out"));
    }, timeoutMs);
  });

  let output = "";
  const outputPromise = (async () => {
    for await (const line of proc.stdout) {
      output += line + "\n";
    }
    await proc.wait();
    return output.trim();
  })();

  return Promise.race([outputPromise, timeoutPromise]);
}
```

### Interactive Process

```typescript
const proc = await spawn("python3", {
  args: ["-i"],
  stdin: "piped",
  stdout: "piped",
  stderr: "piped"
});

// Send Python code
await proc.writeStdin("print('Hello from Python')\n");
await proc.writeStdin("x = 42\n");
await proc.writeStdin("print(x * 2)\n");

// Read output
for await (const line of proc.stdout) {
  console.log("Python:", line);
}
```

---

## Error Handling

```typescript
import { spawn } from "host:process";

try {
  const proc = await spawn("nonexistent-command");
  await proc.wait();
} catch (error) {
  if (error.message.includes("permission")) {
    console.error("Binary not allowed - check capabilities");
  } else if (error.message.includes("not found")) {
    console.error("Binary not found in PATH");
  } else {
    console.error("Process error:", error);
  }
}
```

---

## Complete Example

```typescript
import { spawn } from "host:process";
import { notify } from "host:sys";

// Git wrapper
class Git {
  private cwd: string;

  constructor(repoPath: string) {
    this.cwd = repoPath;
  }

  private async run(args: string[]): Promise<string> {
    const proc = await spawn("git", { args, cwd: this.cwd });

    let output = "";
    let errors = "";

    // Collect output in parallel
    const stdoutPromise = (async () => {
      for await (const line of proc.stdout) {
        output += line + "\n";
      }
    })();

    const stderrPromise = (async () => {
      for await (const line of proc.stderr) {
        errors += line + "\n";
      }
    })();

    await Promise.all([stdoutPromise, stderrPromise]);
    const exitCode = await proc.wait();

    if (exitCode !== 0) {
      throw new Error(`Git error: ${errors || output}`);
    }

    return output.trim();
  }

  async status(): Promise<string[]> {
    const output = await this.run(["status", "--short"]);
    return output ? output.split("\n") : [];
  }

  async commit(message: string): Promise<void> {
    await this.run(["commit", "-m", message]);
    await notify("Git", "Changes committed successfully");
  }

  async push(): Promise<void> {
    await this.run(["push"]);
    await notify("Git", "Changes pushed to remote");
  }
}
```
