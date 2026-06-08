---
title: "runtime:lock"
description: "Named resource locking for coordinating concurrent access in Forge applications"
slug: docs/api/runtime-lock
---

Named resource locking for Forge applications. Coordinate access to shared resources across async operations using token-based lock acquisition.

> **Implementation**: TypeScript types are auto-generated from Rust via [forge-weld](/docs/crates/forge-weld). See [ext_lock](/docs/crates/ext-lock) for implementation details.

## Features

- Named lock acquisition with optional timeout
- Non-blocking try-acquire pattern
- Token-based lock release for safe ownership tracking
- List all active locks
- Cross-async operation coordination

## Import

```typescript
import {
  // Functions
  acquire,
  tryAcquire,
  release,
  list,
  // Types
  type LockInfo,
  // Hooks
  onBefore,
  onAfter,
  onError,
} from "runtime:lock";
```

## API Reference

<!-- forge:api -->
<!-- generated from sdk/runtime.lock.ts — edit signatures in the SDK, run `make docs-api` to refresh -->
```typescript
acquire(name: string, timeoutMs?: number): Promise<bigint>
tryAcquire(name: string): Promise<bigint | null>
release(name: string, token: bigint): boolean
list(): LockInfo[]
```
<!-- /forge:api -->

### acquire(name, timeoutMs?)

Acquire a named lock, waiting if necessary.

Blocks until the lock becomes available or the timeout expires.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `name` | `string` | Unique lock name |
| `timeoutMs` | `number` | Optional timeout in milliseconds |

**Returns:** `Promise<bigint>` - Lock token for releasing

**Throws:** Error if timeout expires without acquiring lock

**Example:**

```typescript
import { acquire, release } from "runtime:lock";

// Acquire lock with 5 second timeout
const token = await acquire("database-write", 5000);
try {
  // Exclusive access to resource
  await performDatabaseWrite();
} finally {
  release("database-write", token);
}
```

---

### tryAcquire(name)

Try to acquire a lock without waiting.

Returns immediately with the lock token if available, or `null` if the lock is held by another caller.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `name` | `string` | Unique lock name |

**Returns:** `Promise<bigint | null>` - Lock token if acquired, `null` if unavailable

**Example:**

```typescript
import { tryAcquire, release } from "runtime:lock";

const token = await tryAcquire("cache-update");
if (token !== null) {
  try {
    await updateCache();
  } finally {
    release("cache-update", token);
  }
} else {
  console.log("Cache update already in progress, skipping");
}
```

---

### release(name, token)

Release a previously acquired lock.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `name` | `string` | Lock name |
| `token` | `bigint` | Token from `acquire()` or `tryAcquire()` |

**Returns:** `boolean` - `true` if lock was released, `false` if token was invalid

**Example:**

```typescript
import { acquire, release } from "runtime:lock";

const token = await acquire("resource");
// ... do work ...
const released = release("resource", token);
console.log("Lock released:", released);
```

---

### list()

List all currently held locks.

**Returns:** `LockInfo[]` - Array of lock information

**Example:**

```typescript
import { list } from "runtime:lock";

const locks = list();
for (const lock of locks) {
  console.log(`Lock "${lock.name}": ${lock.locked ? "held" : "free"}`);
}
```

## Type Definitions

### LockInfo

```typescript
interface LockInfo {
  /** Lock name */
  name: string;
  /** Whether the lock is currently held */
  locked: boolean;
}
```

## Lifecycle Hooks

### onBefore(opName, callback)

```typescript
import { onBefore } from "runtime:lock";

const unsubscribe = onBefore("acquire", () => {
  console.log("Acquiring lock...");
});
```

### onAfter(opName, callback)

```typescript
import { onAfter } from "runtime:lock";

onAfter("release", () => {
  console.log("Lock released");
});
```

### onError(opName, callback)

```typescript
import { onError } from "runtime:lock";

onError("acquire", (error) => {
  console.error("Lock acquisition failed:", error.message);
});
```

**Available operation names:** `"acquire"`, `"try"`, `"release"`, `"list"`

## Complete Examples

### Safe Resource Access Pattern

```typescript
import { acquire, release } from "runtime:lock";

async function withLock<T>(
  name: string,
  fn: () => Promise<T>,
  timeoutMs?: number
): Promise<T> {
  const token = await acquire(name, timeoutMs);
  try {
    return await fn();
  } finally {
    release(name, token);
  }
}

// Usage
const result = await withLock("config-file", async () => {
  const config = await readConfigFile();
  config.updated = Date.now();
  await writeConfigFile(config);
  return config;
});
```

### Database Connection Pool Lock

```typescript
import { acquire, release, tryAcquire } from "runtime:lock";

class ConnectionPool {
  private maxConnections = 10;

  async getConnection(): Promise<Connection> {
    for (let i = 0; i < this.maxConnections; i++) {
      const lockName = `db-conn-${i}`;
      const token = await tryAcquire(lockName);

      if (token !== null) {
        return new Connection(i, lockName, token);
      }
    }

    // All connections busy, wait for first available
    const token = await acquire("db-conn-0", 30000);
    return new Connection(0, "db-conn-0", token);
  }
}

class Connection {
  constructor(
    public id: number,
    private lockName: string,
    private token: bigint
  ) {}

  release(): void {
    release(this.lockName, this.token);
  }
}

// Usage
const pool = new ConnectionPool();
const conn = await pool.getConnection();
try {
  await conn.query("SELECT * FROM users");
} finally {
  conn.release();
}
```

### Singleton Operation Pattern

```typescript
import { tryAcquire, release } from "runtime:lock";

class SingletonTask {
  private lockName: string;

  constructor(name: string) {
    this.lockName = `singleton:${name}`;
  }

  async runIfNotRunning<T>(fn: () => Promise<T>): Promise<T | null> {
    const token = await tryAcquire(this.lockName);

    if (token === null) {
      console.log("Task already running, skipping");
      return null;
    }

    try {
      return await fn();
    } finally {
      release(this.lockName, token);
    }
  }
}

// Usage - only one sync runs at a time
const syncTask = new SingletonTask("data-sync");

// Multiple calls, but only one executes
await Promise.all([
  syncTask.runIfNotRunning(syncData),
  syncTask.runIfNotRunning(syncData),
  syncTask.runIfNotRunning(syncData),
]);
```

### Lock Monitor

```typescript
import { list, onAfter } from "runtime:lock";

function monitorLocks(): void {
  console.log("Active locks:");
  for (const lock of list()) {
    if (lock.locked) {
      console.log(`  - ${lock.name}`);
    }
  }
}

// Log lock activity
onAfter("acquire", () => monitorLocks());
onAfter("release", () => monitorLocks());
```

### Mutex Class

```typescript
import { acquire, release } from "runtime:lock";

class Mutex {
  private name: string;
  private token: bigint | null = null;

  constructor(name: string) {
    this.name = `mutex:${name}`;
  }

  async lock(timeoutMs?: number): Promise<void> {
    if (this.token !== null) {
      throw new Error("Mutex already locked by this instance");
    }
    this.token = await acquire(this.name, timeoutMs);
  }

  unlock(): void {
    if (this.token === null) {
      throw new Error("Mutex not locked");
    }
    release(this.name, this.token);
    this.token = null;
  }

  async withLock<T>(fn: () => Promise<T>): Promise<T> {
    await this.lock();
    try {
      return await fn();
    } finally {
      this.unlock();
    }
  }
}

// Usage
const mutex = new Mutex("shared-state");

await mutex.withLock(async () => {
  // Exclusive access
  state.counter++;
  await saveState(state);
});
```

## Best Practices

### Always Release in Finally Block

```typescript
// Good - lock always released
const token = await acquire("resource");
try {
  await doWork();
} finally {
  release("resource", token);
}

// Bad - lock may leak on error
const token = await acquire("resource");
await doWork();
release("resource", token);
```

### Use Descriptive Lock Names

```typescript
// Good - clear purpose
await acquire("user:123:profile-update");
await acquire("cache:products:rebuild");

// Avoid - too generic
await acquire("lock1");
await acquire("data");
```

### Prefer Try-Acquire for Optional Operations

```typescript
// Good - skip if busy
const token = await tryAcquire("maintenance");
if (token) {
  try {
    await runMaintenance();
  } finally {
    release("maintenance", token);
  }
}

// Avoid unnecessary blocking for optional work
```

### Set Reasonable Timeouts

```typescript
// Good - fail fast with timeout
try {
  const token = await acquire("resource", 5000);
} catch (e) {
  console.error("Could not acquire lock within 5 seconds");
}

// Avoid - infinite wait can cause hangs
const token = await acquire("resource"); // No timeout
```
