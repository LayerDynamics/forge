---
title: "ext_storage"
description: SQLite key-value storage extension providing the runtime:storage module.
slug: crates/ext-storage
---

The `ext_storage` crate provides persistent key-value storage backed by SQLite for Forge applications through the `runtime:storage` module.

## Overview

ext_storage provides a simple, reliable way to persist application data using a SQLite database. All values are automatically serialized to JSON, supporting strings, numbers, booleans, arrays, and objects.

**Key Features:**
- **SQLite Backend** - ACID-compliant persistent storage
- **Automatic Serialization** - JSON encoding/decoding for all JavaScript values
- **Batch Operations** - Efficient bulk reads, writes, and deletes (~10x faster)
- **Indexed Queries** - Fast key lookups with SQLite indexing
- **Transactional Writes** - Atomic batch operations with rollback support
- **Timestamps** - Automatic `created_at` and `updated_at` tracking

## Quick Start

```typescript
import { get, set, remove, has, keys } from "runtime:storage";

// Store values
await set("user.name", "Alice");
await set("user.preferences", { theme: "dark", fontSize: 14 });

// Retrieve values
const name = await get<string>("user.name");
const prefs = await get<UserPrefs>("user.preferences");

// Check existence
if (await has("user.session")) {
  await remove("user.session");
}

// List all keys
const allKeys = await keys();
```

## Module Import

```typescript
import {
  // Basic operations
  get,
  set,
  remove,
  has,
  keys,
  clear,
  size,

  // Batch operations
  getMany,
  setMany,
  deleteMany
} from "runtime:storage";
```

## Core Concepts

### Storage Location

The SQLite database is created at:
- **macOS**: `~/Library/Application Support/.forge/<app-id>/storage.db`
- **Linux**: `~/.local/share/.forge/<app-id>/storage.db`
- **Windows**: `%APPDATA%\.forge\<app-id>\storage.db`

### JSON Serialization

All values are automatically serialized to JSON:

**Supported Types**:
- Primitives: `string`, `number`, `boolean`, `null`
- Arrays: `string[]`, `number[]`, etc.
- Objects: `{ key: value }`, nested objects

**Not Supported**:
- `undefined`, functions, circular references, `BigInt`, `Symbol`

### Performance

- **Individual Operations**: ~1-2ms per operation
- **Batch Operations**: ~0.1ms per item (~10x faster for 10+ items)
- **Database**: Indexed for fast key lookups, connection is reused

## API Reference

### get()

Retrieves a value from storage by key.

```typescript
function get<T = unknown>(key: string): Promise<T | null>
```

**Parameters:**
- `key` - The key to retrieve (must be non-empty)

**Returns:**
- The stored value, or `null` if the key doesn't exist

**Throws:**
- `[8106]` if key is empty
- `[8103]` if stored value cannot be deserialized
- `[8104]` if database operation fails

**Examples:**

```typescript
// Simple retrieval
const username = await get<string>("user.name");
if (username) {
  console.log(`Welcome ${username}`);
}

// With default value
const theme = await get<string>("prefs.theme") ?? "light";

// Complex objects
interface WindowBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

const bounds = await get<WindowBounds>("window.bounds");
```

### set()

Stores a value in persistent storage.

```typescript
function set<T = unknown>(key: string, value: T): Promise<void>
```

**Parameters:**
- `key` - The key to store under (must be non-empty)
- `value` - The value to store (must be JSON-serializable)

**Throws:**
- `[8106]` if key is empty
- `[8102]` if value cannot be serialized to JSON
- `[8104]` if database operation fails

**Examples:**

```typescript
// Store primitive values
await set("app.version", "1.0.0");
await set("user.id", 12345);
await set("feature.enabled", true);

// Store complex objects
await set("user.profile", {
  name: "Alice",
  email: "alice@example.com",
  role: "admin"
});

// Store arrays
await set("recent.files", [
  "/path/to/file1.txt",
  "/path/to/file2.txt"
]);

// Update existing value
const count = await get<number>("app.launchCount") ?? 0;
await set("app.launchCount", count + 1);
```

### remove()

Removes a key and its value from storage.

```typescript
function remove(key: string): Promise<boolean>
```

**Parameters:**
- `key` - The key to delete

**Returns:**
- `true` if the key existed and was deleted, `false` otherwise

**Throws:**
- `[8104]` if database operation fails

**Examples:**

```typescript
// Remove single key
const wasDeleted = await remove("user.session");
if (wasDeleted) {
  console.log("Session cleared");
}

// Clear user data on logout
await remove("user.token");
await remove("user.profile");
await remove("user.preferences");
```

### has()

Checks whether a key exists in storage.

```typescript
function has(key: string): Promise<boolean>
```

**Parameters:**
- `key` - The key to check

**Returns:**
- `true` if the key exists, `false` otherwise

**Throws:**
- `[8104]` if database operation fails

**Examples:**

```typescript
// Check before reading
if (await has("user.profile")) {
  const profile = await get("user.profile");
}

// Initialize on first run
if (!await has("app.initialized")) {
  await set("app.initialized", true);
  await runFirstTimeSetup();
}
```

### keys()

Retrieves all keys currently stored in the database.

```typescript
function keys(): Promise<string[]>
```

**Returns:**
- Array of all keys, sorted alphabetically

**Throws:**
- `[8104]` if database operation fails

**Examples:**

```typescript
// List all keys
const allKeys = await keys();
console.log(`Storage contains ${allKeys.length} keys`);

// Filter keys by prefix
const userKeys = allKeys.filter(k => k.startsWith("user."));

// Migrate old keys
for (const oldKey of allKeys.filter(k => k.startsWith("old_"))) {
  const value = await get(oldKey);
  const newKey = oldKey.replace("old_", "new_");
  await set(newKey, value);
  await remove(oldKey);
}
```

### clear()

Removes all key-value pairs from storage.

```typescript
function clear(): Promise<void>
```

**Throws:**
- `[8104]` if database operation fails

**Warning**: This operation is irreversible!

**Examples:**

```typescript
// Reset to defaults
await clear();
await set("app.version", "1.0.0");
await set("app.firstRun", true);

// Development mode reset
if (Deno.env.get("DEV_MODE") === "true") {
  await clear();
}
```

### size()

Returns the total size of all stored values in bytes.

```typescript
function size(): Promise<number>
```

**Returns:**
- Total size in bytes of all stored values

**Throws:**
- `[8104]` if database operation fails

**Examples:**

```typescript
// Check storage usage
const bytes = await size();
console.log(`Using ${(bytes / 1024).toFixed(2)} KB`);

// Enforce quota
const MAX_SIZE = 10 * 1024 * 1024; // 10 MB
if (await size() > MAX_SIZE) {
  throw new Error("Storage quota exceeded");
}

// Monitor growth
const before = await size();
await set("large.dataset", bigArray);
const after = await size();
console.log(`Added ${after - before} bytes`);
```

### getMany()

Efficiently retrieves multiple values at once.

```typescript
function getMany(keys: string[]): Promise<Map<string, unknown>>
```

**Parameters:**
- `keys` - Array of keys to retrieve

**Returns:**
- Map containing key-value pairs for keys that were found (missing keys are omitted)

**Throws:**
- `[8104]` if database operation fails

**Performance**: Approximately **10x faster** than individual `get()` calls for 10+ keys.

**Examples:**

```typescript
// Bulk retrieval
const values = await getMany(["user.name", "user.email", "user.role"]);
console.log("Name:", values.get("user.name"));
console.log("Email:", values.get("user.email"));

// Load app state
const state = await getMany([
  "window.bounds",
  "window.maximized",
  "recent.files"
]);

// Check which keys exist
const found = await getMany(["key1", "key2", "key3"]);
for (const key of ["key1", "key2", "key3"]) {
  if (found.has(key)) {
    console.log(`${key}: ${found.get(key)}`);
  }
}
```

### setMany()

Atomically stores multiple key-value pairs at once.

```typescript
function setMany(entries: Record<string, unknown>): Promise<void>
```

**Parameters:**
- `entries` - Object containing key-value pairs to store

**Throws:**
- `[8106]` if any key is empty
- `[8102]` if any value cannot be serialized
- `[8104]` if database operation fails
- `[8109]` if transaction fails (all changes rolled back)

**Performance**: Approximately **10x faster** than individual `set()` calls for 10+ pairs.

**Atomicity**: Either all writes succeed or none do (transaction rollback).

**Examples:**

```typescript
// Bulk initialization
await setMany({
  "app.version": "1.0.0",
  "app.firstRun": true,
  "user.theme": "dark"
});

// Save app state atomically
await setMany({
  "window.bounds": { x: 100, y: 100, width: 800, height: 600 },
  "window.maximized": false,
  "recent.files": ["/path/to/file1.txt"],
  "recent.searches": ["typescript"]
});

// All-or-nothing behavior
try {
  await setMany({
    "user.name": "Alice",
    "user.invalid": circularRef // This fails!
  });
} catch (err) {
  // Neither value was saved (transaction rolled back)
  console.error("Save failed:", err);
}
```

### deleteMany()

Efficiently deletes multiple keys at once.

```typescript
function deleteMany(keys: string[]): Promise<number>
```

**Parameters:**
- `keys` - Array of keys to delete

**Returns:**
- Number of keys that existed and were successfully deleted

**Throws:**
- `[8104]` if database operation fails

**Performance**: Approximately **10x faster** than individual `remove()` calls for 10+ keys.

**Examples:**

```typescript
// Clear session data
const deleted = await deleteMany([
  "session.token",
  "session.userId",
  "session.expires"
]);
console.log(`Deleted ${deleted} session keys`);

// Clean up cache
const allKeys = await keys();
const cacheKeys = allKeys.filter(k => k.startsWith("cache."));
if (cacheKeys.length > 0) {
  await deleteMany(cacheKeys);
}

// Batch cleanup with verification
const keysToDelete = ["key1", "key2", "key3"];
const deleted = await deleteMany(keysToDelete);
if (deleted === keysToDelete.length) {
  console.log("All keys deleted");
} else {
  console.log(`Only ${deleted}/${keysToDelete.length} existed`);
}
```

## Usage Examples

### 1. Application State Persistence

```typescript
import { getMany, setMany } from "runtime:storage";

// Save app state on exit
async function saveAppState() {
  await setMany({
    "window.bounds": getCurrentWindowBounds(),
    "window.maximized": isWindowMaximized(),
    "recent.files": getRecentFiles(),
    "editor.openFiles": getOpenFiles(),
    "editor.activeFile": getActiveFile()
  });
}

// Restore app state on launch
async function restoreAppState() {
  const state = await getMany([
    "window.bounds",
    "window.maximized",
    "recent.files",
    "editor.openFiles",
    "editor.activeFile"
  ]);

  if (state.has("window.bounds")) {
    restoreWindowBounds(state.get("window.bounds"));
  }
  if (state.get("window.maximized")) {
    maximizeWindow();
  }
  if (state.has("recent.files")) {
    setRecentFiles(state.get("recent.files") as string[]);
  }
}
```

### 2. User Preferences with Defaults

```typescript
import { get, set } from "runtime:storage";

async function getPreference<T>(key: string, defaultValue: T): Promise<T> {
  const value = await get<T>(`prefs.${key}`);
  return value ?? defaultValue;
}

async function setPreference<T>(key: string, value: T): Promise<void> {
  await set(`prefs.${key}`, value);
}

// Usage
const theme = await getPreference("theme", "light");
const fontSize = await getPreference("fontSize", 14);
const autoSave = await getPreference("autoSave", true);

await setPreference("theme", "dark");
```

### 3. Caching with Expiration

```typescript
import { get, set, remove } from "runtime:storage";

interface CacheEntry<T> {
  value: T;
  expiresAt: number;
}

async function cacheSet<T>(key: string, value: T, ttlMs: number): Promise<void> {
  await set(`cache.${key}`, {
    value,
    expiresAt: Date.now() + ttlMs
  } as CacheEntry<T>);
}

async function cacheGet<T>(key: string): Promise<T | null> {
  const entry = await get<CacheEntry<T>>(`cache.${key}`);
  if (!entry) return null;

  if (Date.now() > entry.expiresAt) {
    await remove(`cache.${key}`);
    return null;
  }

  return entry.value;
}

// Usage
await cacheSet("api.users", usersData, 60 * 1000); // 1 minute
const users = await cacheGet("api.users");
```

### 4. Storage Quota Management

```typescript
import { size, keys, deleteMany, set } from "runtime:storage";

const MAX_STORAGE = 10 * 1024 * 1024; // 10 MB

async function setWithQuota(key: string, value: unknown): Promise<void> {
  const currentSize = await size();
  const valueSize = JSON.stringify(value).length;

  if (currentSize + valueSize > MAX_STORAGE) {
    // Clean up cache
    const allKeys = await keys();
    const cacheKeys = allKeys.filter(k => k.startsWith("cache."));

    if (cacheKeys.length > 0) {
      await deleteMany(cacheKeys);
      console.log("Cleared cache to free space");
    } else {
      throw new Error("Storage quota exceeded");
    }
  }

  await set(key, value);
}
```

### 5. Data Migration

```typescript
import { get, set, remove, keys } from "runtime:storage";

const STORAGE_VERSION = 2;

async function migrateStorageIfNeeded(): Promise<void> {
  const version = await get<number>("storage.version") ?? 1;

  if (version < 2) {
    console.log("Migrating storage to version 2...");

    const allKeys = await keys();
    const oldKeys = allKeys.filter(k => k.startsWith("old_"));

    for (const oldKey of oldKeys) {
      const value = await get(oldKey);
      const newKey = oldKey.replace("old_", "new_");
      await set(newKey, value);
      await remove(oldKey);
    }

    await set("storage.version", 2);
    console.log("Migration complete");
  }
}
```

## Best Practices

### ✅ Do

1. **Use Batch Operations for Multiple Items**
   ```typescript
   // Good - 10x faster
   const values = await getMany(["key1", "key2", "key3"]);

   // Bad - 3 separate queries
   const val1 = await get("key1");
   const val2 = await get("key2");
   const val3 = await get("key3");
   ```

2. **Use Namespaced Keys**
   ```typescript
   // Good - organized and clear
   await set("user.profile.name", "Alice");
   await set("cache.api.users", usersData);

   // Bad - hard to manage
   await set("name", "Alice");
   await set("users", usersData);
   ```

3. **Provide Default Values**
   ```typescript
   // Good - safe fallback
   const theme = await get<string>("prefs.theme") ?? "light";

   // Bad - might be null
   const theme = await get<string>("prefs.theme");
   ```

4. **Use TypeScript Generics**
   ```typescript
   // Good - type-safe
   const count = await get<number>("app.launchCount") ?? 0;

   // Bad - requires casting
   const count = (await get("app.launchCount") || 0) as number;
   ```

5. **Handle Serialization Errors**
   ```typescript
   // Good - graceful error handling
   try {
     await set("user.data", complexObject);
   } catch (err) {
     if (err.message.includes("[8102]")) {
       console.error("Circular reference detected");
     }
   }
   ```

## Common Pitfalls

### ❌ Don't

1. **Don't Use `get()` in Loops**
   ```typescript
   // Bad - very slow
   const values = [];
   for (const key of manyKeys) {
     values.push(await get(key));
   }

   // Good - 10x faster
   const values = await getMany(manyKeys);
   ```

2. **Don't Store Circular References**
   ```typescript
   // Bad - will throw [8102]
   const circular: any = { name: "Alice" };
   circular.self = circular;
   await set("user", circular); // Error!

   // Good - break the cycle
   const { self, ...clean } = circular;
   await set("user", clean);
   ```

3. **Don't Assume Values Exist**
   ```typescript
   // Bad - might throw if missing
   const profile = await get<UserProfile>("user.profile");
   console.log(profile.name); // Error if null!

   // Good - check first
   const profile = await get<UserProfile>("user.profile");
   if (profile) {
     console.log(profile.name);
   }
   ```

4. **Don't Store Very Large Values**
   ```typescript
   // Bad - use ext_database instead
   await set("huge.dataset", tenMBArray);

   // Good - use appropriate storage
   import { query } from "runtime:database";
   await query("INSERT INTO datasets VALUES (?)", [data]);
   ```

5. **Don't Use Empty Keys**
   ```typescript
   // Bad - will throw [8106]
   await set("", "value"); // Error!

   // Good - use meaningful keys
   await set("app.defaultValue", "value");
   ```

## Error Handling

All storage operations may throw errors with structured codes:

| Code   | Error                  | Description                              |
|--------|------------------------|------------------------------------------|
| `8100` | Generic                | Unspecified storage error                |
| `8101` | NotFound               | Key does not exist                       |
| `8102` | SerializationError     | Value cannot be serialized to JSON       |
| `8103` | DeserializationError   | Stored value is not valid JSON           |
| `8104` | DatabaseError          | SQLite operation failed                  |
| `8105` | PermissionDenied       | Storage operation not permitted          |
| `8106` | InvalidKey             | Key is invalid (empty)                   |
| `8107` | QuotaExceeded          | Storage quota limit reached              |
| `8108` | ConnectionFailed       | Database connection failed               |
| `8109` | TransactionFailed      | Batch operation rolled back              |

### Error Handling Pattern

```typescript
try {
  await set("user.data", value);
} catch (err) {
  const message = err.message;

  if (message.includes("[8102]")) {
    console.error("Cannot serialize value (circular reference?)");
  } else if (message.includes("[8106]")) {
    console.error("Key cannot be empty");
  } else if (message.includes("[8104]")) {
    console.error("Database error:", message);
  } else {
    console.error("Storage error:", message);
  }
}
```

## Platform Support

| Platform | Supported | Storage Location                                  |
|----------|-----------|---------------------------------------------------|
| macOS    | ✅        | `~/Library/Application Support/.forge/<app-id>`   |
| Linux    | ✅        | `~/.local/share/.forge/<app-id>`                  |
| Windows  | ✅        | `%APPDATA%\.forge\<app-id>`                       |

Database file: `storage.db`

## Permissions

Storage operations do not currently require permissions in `manifest.app.toml`, but this may change in future versions.

## Related Extensions

- [`ext_database`](/docs/crates/ext-database) - Full SQL database access for complex queries
- [`ext_app`](/docs/crates/ext-app) - Application paths and metadata
- [`ext_crypto`](/docs/crates/ext-crypto) - Encryption for sensitive data
