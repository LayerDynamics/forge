---
title: "runtime:log"
description: "Structured logging for Forge applications with host and browser output"
slug: api/runtime-log
---

Structured logging for Forge applications with dual output capabilities. Log to the host terminal via the Rust tracing system and/or forward logs to browser DevTools via IPC.

> **Implementation**: TypeScript types are auto-generated from Rust via [forge-weld](/docs/crates/forge-weld). See [ext_log](/docs/crates/ext-log) for implementation details.

## Features

- Structured logging with key-value fields
- Host terminal output via Rust tracing
- Browser DevTools console forwarding
- Dual output mode for development
- Configurable log levels (trace, debug, info, warn, error)

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   Deno Runtime (main.ts)                     │
├─────────────────────────────────────────────────────────────┤
│  emit()           │  browserConsole     │  dualLog           │
│  trace/debug/...  │  .log/.warn/...     │  .info/.error/...  │
├───────────────────┼─────────────────────┼───────────────────┤
│         ↓         │          ↓          │    ↓         ↓    │
│   op_log_emit()   │   op_ipc_send()     │  Both operations   │
├───────────────────┼─────────────────────┼───────────────────┤
│         ↓         │          ↓          │    ↓         ↓    │
│   Rust tracing    │   WebView IPC       │   Both outputs     │
│   (Terminal)      │   (Browser DevTools)│                    │
└───────────────────┴─────────────────────┴───────────────────┘
```

## Import

```typescript
import {
  // Host logging (terminal)
  emit,
  trace,
  debug,
  infoLog,
  warn,
  error,
  // Browser console forwarding
  setDefaultWindow,
  browserConsole,
  // Dual output
  dualLog,
  // Types
  type LogLevel,
} from "runtime:log";
```

## Log Levels

| Level | Description | Environment Variable Filter |
|-------|-------------|----------------------------|
| `trace` | Verbose diagnostic information | `FORGE_LOG=trace` |
| `debug` | Debugging information | `FORGE_LOG=debug` |
| `info` | General operational information | `FORGE_LOG=info` (default) |
| `warn` / `warning` | Warning conditions | `FORGE_LOG=warn` |
| `error` | Error conditions | `FORGE_LOG=error` |

## API Reference

<!-- forge:api -->
<!-- generated from sdk/runtime.log.ts — edit signatures in the SDK, run `make docs-api` to refresh -->
```typescript
info(): ExtensionInfo
emit(level: LogLevel, message: string, fields?: Record<string, unknown>): void
trace(message: string, fields?: Record<string, unknown>): void
debug(message: string, fields?: Record<string, unknown>): void
infoLog(message: string, fields?: Record<string, unknown>): void
warn(message: string, fields?: Record<string, unknown>): void
error(message: string, fields?: Record<string, unknown>): void
setDefaultWindow(windowId: string): void
```
<!-- /forge:api -->

### Host Logging (Terminal)

These functions output to the terminal via the Rust tracing system. Controlled by the `FORGE_LOG` environment variable.

#### emit(level, message, fields?)

Emit a log message at a specified level with optional structured fields.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `level` | `LogLevel` | Log level |
| `message` | `string` | Log message |
| `fields` | `Record<string, unknown>` | Optional structured fields |

**Example:**

```typescript
import { emit } from "runtime:log";

emit("info", "User logged in", { userId: "123", method: "oauth" });
emit("error", "Database connection failed", { host: "db.example.com", retries: 3 });
```

---

#### trace(message, fields?)

Log a trace-level message. Use for verbose diagnostic information.

**Example:**

```typescript
import { trace } from "runtime:log";

trace("Entering function", { fn: "processData", args: { id: 42 } });
trace("Loop iteration", { index: 5, total: 100 });
```

---

#### debug(message, fields?)

Log a debug-level message. Use for debugging information during development.

**Example:**

```typescript
import { debug } from "runtime:log";

debug("Processing request", { path: "/api/users", method: "GET" });
debug("Cache status", { hit: true, key: "user:123" });
```

---

#### infoLog(message, fields?)

Log an info-level message. Use for general operational information.

**Note:** Named `infoLog` to avoid conflict with the `info()` function that returns extension metadata.

**Example:**

```typescript
import { infoLog } from "runtime:log";

infoLog("Server started", { port: 3000, env: "production" });
infoLog("Request completed", { duration_ms: 45, status: 200 });
```

---

#### warn(message, fields?)

Log a warning message. Use for potentially problematic conditions.

**Example:**

```typescript
import { warn } from "runtime:log";

warn("Rate limit approaching", { current: 95, limit: 100 });
warn("Deprecated API used", { endpoint: "/v1/legacy", replacement: "/v2/new" });
```

---

#### error(message, fields?)

Log an error message. Use for error conditions that need attention.

**Example:**

```typescript
import { error } from "runtime:log";

error("Failed to process payment", { orderId: "456", reason: "Card declined" });
error("Unhandled exception", { error: err.message, stack: err.stack });
```

### Browser Console Forwarding

These functions send log messages to the browser DevTools console via IPC. Useful for debugging when you need to see backend logs in the browser.

#### setDefaultWindow(windowId)

Set the default window ID for browser console forwarding. Once set, you can omit the `windowId` parameter in `browserConsole` calls.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `windowId` | `string` | Window ID to send logs to |

**Example:**

```typescript
import { setDefaultWindow } from "runtime:log";

// Set once when window is created
setDefaultWindow(mainWindow.id);
```

---

#### browserConsole

Object with logging methods that send messages to browser DevTools.

**Methods:**

| Method | Description |
|--------|-------------|
| `trace(message, fields?, windowId?)` | Trace-level log |
| `debug(message, fields?, windowId?)` | Debug-level log |
| `log(message, fields?, windowId?)` | Info-level log |
| `info(message, fields?, windowId?)` | Info-level log |
| `warn(message, fields?, windowId?)` | Warning log |
| `error(message, fields?, windowId?)` | Error log |

**Example:**

```typescript
import { setDefaultWindow, browserConsole } from "runtime:log";

// Set default window
setDefaultWindow(win.id);

// Log without specifying window
browserConsole.log("Hello from Deno backend!");
browserConsole.warn("This is a warning");
browserConsole.error("Something went wrong", { code: 500 });

// Or specify window explicitly
browserConsole.log("Message to specific window", undefined, otherWindowId);
```

### Dual Output Logging

These functions output to BOTH the host terminal AND browser DevTools. Ideal for development when you want to see logs in both places.

#### dualLog

Object with logging methods that output to both terminal and browser.

**Methods:**

| Method | Description |
|--------|-------------|
| `trace(message, fields?, windowId?)` | Trace to both outputs |
| `debug(message, fields?, windowId?)` | Debug to both outputs |
| `info(message, fields?, windowId?)` | Info to both outputs |
| `warn(message, fields?, windowId?)` | Warning to both outputs |
| `error(message, fields?, windowId?)` | Error to both outputs |

**Example:**

```typescript
import { setDefaultWindow, dualLog } from "runtime:log";

setDefaultWindow(win.id);

// Appears in terminal AND browser DevTools
dualLog.info("Starting application...");
dualLog.debug("Loading configuration", { path: "./config.json" });
dualLog.error("Failed to connect", { service: "api" });
```

## Type Definitions

```typescript
/**
 * Available log levels
 */
type LogLevel = "trace" | "debug" | "info" | "warn" | "warning" | "error";
```

## Lifecycle Hooks

### onBefore(opName, callback)

```typescript
import { onBefore } from "runtime:log";

onBefore("emit", () => {
  // Called before every log emission
});
```

### onAfter(opName, callback)

```typescript
import { onAfter } from "runtime:log";

onAfter("emit", () => {
  // Called after every log emission
});
```

**Available operation names:** `"info"`, `"emit"`

## Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `FORGE_LOG` | Set log level filter | `FORGE_LOG=debug` |
| `FORGE_LOG` | Module-specific filtering | `FORGE_LOG=ext_fs=trace,info` |

**Examples:**

```bash
# Show all logs
FORGE_LOG=trace forge dev my-app

# Show debug and above
FORGE_LOG=debug forge dev my-app

# Show only errors
FORGE_LOG=error forge dev my-app

# Module-specific filtering
FORGE_LOG=ext_window=debug,ext_fs=trace forge dev my-app
```

## Complete Examples

### Application Logger

```typescript
import {
  infoLog,
  debug,
  warn,
  error,
  setDefaultWindow,
  dualLog,
} from "runtime:log";

class Logger {
  private context: string;
  private windowId: string | null = null;

  constructor(context: string) {
    this.context = context;
  }

  setWindow(windowId: string) {
    this.windowId = windowId;
    setDefaultWindow(windowId);
  }

  private formatMessage(message: string): string {
    return `[${this.context}] ${message}`;
  }

  trace(message: string, fields?: Record<string, unknown>) {
    dualLog.trace(this.formatMessage(message), fields);
  }

  debug(message: string, fields?: Record<string, unknown>) {
    dualLog.debug(this.formatMessage(message), fields);
  }

  info(message: string, fields?: Record<string, unknown>) {
    dualLog.info(this.formatMessage(message), fields);
  }

  warn(message: string, fields?: Record<string, unknown>) {
    dualLog.warn(this.formatMessage(message), fields);
  }

  error(message: string, fields?: Record<string, unknown>) {
    dualLog.error(this.formatMessage(message), fields);
  }

  // Convenience method for error objects
  exception(message: string, err: Error, fields?: Record<string, unknown>) {
    this.error(message, {
      ...fields,
      error: err.message,
      stack: err.stack,
    });
  }
}

// Usage
const log = new Logger("App");
log.setWindow(mainWindow.id);

log.info("Application started", { version: "1.0.0" });
log.debug("Loading user preferences");

try {
  await riskyOperation();
} catch (err) {
  log.exception("Operation failed", err, { operation: "riskyOperation" });
}
```

### Request Logger Middleware

```typescript
import { infoLog, warn, error } from "runtime:log";

interface Request {
  id: string;
  method: string;
  path: string;
  startTime: number;
}

function logRequest(req: Request) {
  infoLog("Request started", {
    requestId: req.id,
    method: req.method,
    path: req.path,
  });
}

function logResponse(req: Request, status: number) {
  const duration = Date.now() - req.startTime;

  const fields = {
    requestId: req.id,
    method: req.method,
    path: req.path,
    status,
    duration_ms: duration,
  };

  if (status >= 500) {
    error("Request failed", fields);
  } else if (status >= 400) {
    warn("Request client error", fields);
  } else {
    infoLog("Request completed", fields);
  }
}

// Usage in request handler
async function handleRequest(method: string, path: string) {
  const req: Request = {
    id: crypto.randomUUID(),
    method,
    path,
    startTime: Date.now(),
  };

  logRequest(req);

  try {
    const result = await processRequest(req);
    logResponse(req, 200);
    return result;
  } catch (err) {
    logResponse(req, err.status || 500);
    throw err;
  }
}
```

### Conditional Logging

```typescript
import { debug, trace } from "runtime:log";

// Only log in development
const isDev = Deno.env.get("FORGE_ENV") !== "production";

function devLog(message: string, fields?: Record<string, unknown>) {
  if (isDev) {
    debug(message, fields);
  }
}

// Performance-sensitive logging
function traceIfEnabled(message: string, fields?: Record<string, unknown>) {
  // Trace logs are filtered by FORGE_LOG env var
  // Safe to call even in production - will be no-op if filtered
  trace(message, fields);
}

// Usage
devLog("Debug info only in dev", { data: sensitiveData });
traceIfEnabled("Performance trace", { step: "database_query", duration: 45 });
```

### Multi-Window Logging

```typescript
import { setDefaultWindow, browserConsole, infoLog } from "runtime:log";

class WindowLogger {
  private windowId: string;
  private name: string;

  constructor(windowId: string, name: string) {
    this.windowId = windowId;
    this.name = name;
  }

  log(message: string, fields?: Record<string, unknown>) {
    // Log to host terminal
    infoLog(`[${this.name}] ${message}`, { ...fields, windowId: this.windowId });

    // Also log to this specific window's DevTools
    browserConsole.log(message, fields, this.windowId);
  }

  error(message: string, fields?: Record<string, unknown>) {
    infoLog(`[${this.name}] ERROR: ${message}`, { ...fields, windowId: this.windowId });
    browserConsole.error(message, fields, this.windowId);
  }
}

// Usage with multiple windows
const mainLogger = new WindowLogger(mainWindow.id, "Main");
const settingsLogger = new WindowLogger(settingsWindow.id, "Settings");

mainLogger.log("Main window initialized");
settingsLogger.log("Settings window opened");
```

### Structured Error Logging

```typescript
import { error, warn } from "runtime:log";

interface AppError {
  code: string;
  message: string;
  context?: Record<string, unknown>;
  cause?: Error;
}

function logError(appError: AppError) {
  const fields: Record<string, unknown> = {
    code: appError.code,
    ...appError.context,
  };

  if (appError.cause) {
    fields.cause = appError.cause.message;
    fields.causeStack = appError.cause.stack;
  }

  error(appError.message, fields);
}

// Usage
try {
  await connectToDatabase();
} catch (err) {
  logError({
    code: "DB_CONNECTION_FAILED",
    message: "Failed to connect to database",
    context: {
      host: "localhost",
      port: 5432,
      database: "myapp",
    },
    cause: err,
  });
}
```

## Best Practices

### Use Structured Fields

```typescript
// Good - structured data is searchable and parseable
infoLog("User action", { userId: "123", action: "login", method: "oauth" });

// Avoid - embedded data is harder to analyze
infoLog("User 123 performed login via oauth");
```

### Include Context

```typescript
// Good - includes relevant context
error("Payment failed", {
  orderId: "ord_123",
  amount: 99.99,
  currency: "USD",
  errorCode: "card_declined",
});

// Avoid - missing context makes debugging harder
error("Payment failed");
```

### Use Appropriate Levels

```typescript
// Trace: Very verbose, for debugging specific issues
trace("Cache lookup", { key: "user:123", hit: false });

// Debug: Development-time debugging info
debug("Processing request", { path: "/api/users" });

// Info: Normal operational events
infoLog("Server started", { port: 3000 });

// Warn: Something unexpected but not critical
warn("API rate limit at 80%", { current: 80, limit: 100 });

// Error: Something went wrong
error("Database query failed", { query: "SELECT...", error: "timeout" });
```

### Set Default Window Early

```typescript
import { setDefaultWindow, browserConsole } from "runtime:log";

// Set as soon as window is created
const mainWindow = await createWindow({ ... });
setDefaultWindow(mainWindow.id);

// Now all browserConsole calls go to the main window
browserConsole.log("Ready!");
```
