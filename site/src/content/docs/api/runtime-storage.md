---
title: "runtime:storage"
description: Persistent key-value storage backed by SQLite with automatic JSON serialization.
slug: docs/api/runtime-storage
---

The `runtime:storage` module provides persistent key-value storage for Forge applications, backed by SQLite with automatic JSON serialization and ACID compliance.

> **Implementation**: TypeScript types are auto-generated from Rust via [forge-weld](/docs/crates/forge-weld). See [ext_storage](/docs/crates/ext-storage) for implementation details.

## Features

**Basic Operations**:
- Get/set/delete individual key-value pairs
- Check key existence
- List all keys
- Clear all data
- Get storage size

**Batch Operations**:
- Get multiple keys at once (~10x faster for 10+ keys)
- Set multiple key-value pairs atomically (transactional)
- Delete multiple keys at once (~10x faster for 10+ keys)

**Storage Backend**:
- SQLite database for ACID compliance
- Automatic schema creation and indexing
- JSON serialization for all JavaScript values
- Automatic connection management
- Timestamps (created_at, updated_at) for all entries

## Storage Location

The SQLite database is stored at:
- **macOS**: `~/Library/Application Support/.forge/<app-id>/storage.db`
- **Linux**: `~/.local/share/.forge/<app-id>/storage.db`
- **Windows**: `%APPDATA%\.forge\<app-id>\storage.db`

---

## Basic Operations

### get<T>(key)

Retrieves a value from persistent storage by key.

Returns the stored value deserialized from JSON, or `null` if the key doesn't exist. You can provide a type parameter for better TypeScript type safety, but the runtime type will depend on what was originally stored.

```typescript
import { get } from "runtime:storage";

// Simple value retrieval
const username = await get<string>("user.name");
if (username) {
  console.log(`Welcome back, ${username}!`);
} else {
  console.log("No user found");
}
```

**Complex object retrieval:**

```typescript
interface UserPreferences {
  theme: "light" | "dark";
  fontSize: number;
  notifications: boolean;
}

const prefs = await get<UserPreferences>("user.preferences");
if (prefs) {
  applyTheme(prefs.theme);
  setFontSize(prefs.fontSize);
}
```

**Default value pattern:**

```typescript
const windowBounds = await get<WindowBounds>("window.bounds") ?? {
  x: 100,
  y: 100,
  width: 800,
  height: 600
};
```

**Throws:**
- Error [8106] if key is empty
- Error [8103] if stored value cannot be deserialized from JSON
- Error [8104] if database operation fails
- Error [8108] if database connection fails

### set<T>(key, value)

Stores a value in persistent storage, associated with the given key.

The value is automatically serialized to JSON before storing. If the key already exists, its value is replaced and the `updated_at` timestamp is refreshed.

**Atomic Operation**: Each `set()` is executed in a single database transaction.

```typescript
import { set } from "runtime:storage";

// Store primitive values
await set("app.version", "1.2.3");
await set("user.id", 12345);
await set("feature.enabled", true);
```

**Store complex objects:**

```typescript
await set("user.profile", {
  name: "Alice Johnson",
  email: "alice@example.com",
  role: "admin",
  lastLogin: new Date().toISOString()
});
```

**Store arrays:**

```typescript
await set("recent.searches", ["typescript", "rust", "deno"]);
await set("window.positions", [
  { x: 100, y: 100, width: 800, height: 600 },
  { x: 900, y: 100, width: 600, height: 400 }
]);
```

**Update existing value:**

```typescript
const count = await get<number>("app.launchCount") ?? 0;
await set("app.launchCount", count + 1);
```

**Throws:**
- Error [8106] if key is empty
- Error [8102] if value cannot be serialized to JSON
- Error [8104] if database operation fails
- Error [8108] if database connection fails

### remove(key)

Removes a key and its associated value from persistent storage.

This operation is idempotent - calling it multiple times with the same key is safe. Returns whether the key existed before deletion.

```typescript
import { remove } from "runtime:storage";

// Remove single key
const wasDeleted = await remove("user.session");
if (wasDeleted) {
  console.log("Session cleared");
} else {
  console.log("No session to clear");
}
```

**Conditional removal:**

```typescript
if (await has("cache.stale")) {
  await remove("cache.stale");
  console.log("Stale cache removed");
}
```

**Clear user data on logout:**

```typescript
await remove("user.token");
await remove("user.profile");
await remove("user.preferences");
```

**Returns:** `true` if the key existed and was deleted, `false` if it didn't exist

**Throws:**
- Error [8104] if database operation fails
- Error [8108] if database connection fails

### has(key)

Checks whether a key exists in persistent storage.

This is more efficient than calling `get()` and checking for `null`, especially for large values, since it doesn't deserialize the value.

```typescript
import { has } from "runtime:storage";

// Check before reading
if (await has("user.profile")) {
  const profile = await get("user.profile");
  console.log("Profile:", profile);
} else {
  console.log("No profile found");
}
```

**Initialize on first run:**

```typescript
if (!await has("app.initialized")) {
  await set("app.initialized", true);
  await runFirstTimeSetup();
}
```

**Conditional caching:**

```typescript
async function getOrFetch(key: string) {
  if (await has(key)) {
    return await get(key);
  }
  const data = await fetchFromApi();
  await set(key, data);
  return data;
}
```

**Returns:** `true` if the key exists, `false` otherwise

**Throws:**
- Error [8104] if database operation fails
- Error [8108] if database connection fails

### keys()

Retrieves all keys currently stored in the database.

Keys are returned in alphabetical order. For large datasets, consider using batch operations like `getMany()` to retrieve values efficiently.

```typescript
import { keys } from "runtime:storage";

// List all stored keys
const allKeys = await keys();
console.log(`Storage contains ${allKeys.length} keys`);
console.log("Keys:", allKeys.join(", "));
```

**Filter keys by prefix:**

```typescript
const allKeys = await keys();
const userKeys = allKeys.filter(k => k.startsWith("user."));
console.log("User keys:", userKeys);
```

**Migrate old keys to new naming scheme:**

```typescript
const allKeys = await keys();
for (const oldKey of allKeys.filter(k => k.startsWith("old_"))) {
  const value = await get(oldKey);
  const newKey = oldKey.replace("old_", "new_");
  await set(newKey, value);
  await remove(oldKey);
}
```

**Returns:** Array of all keys, sorted alphabetically

**Throws:**
- Error [8104] if database operation fails
- Error [8108] if database connection fails

### clear()

Removes all key-value pairs from persistent storage.

**Warning**: This operation is irreversible and will delete all data! Use with caution, especially in production.

```typescript
import { clear } from "runtime:storage";

// Clear all storage (with confirmation)
const confirmed = confirm("Are you sure you want to clear all data?");
if (confirmed) {
  await clear();
  console.log("All storage cleared");
}
```

**Reset to defaults on logout:**

```typescript
await clear();
await set("app.version", "1.0.0");
await set("app.firstRun", true);
```

**Development: clear cache on startup:**

```typescript
if (Deno.env.get("DEV_MODE") === "true") {
  await clear();
  console.log("Development mode: storage cleared");
}
```

**Throws:**
- Error [8104] if database operation fails
- Error [8108] if database connection fails

### size()

Returns the total size of all stored values in bytes.

This calculates the sum of the length of all JSON-serialized values in the database. Note that this does not include overhead from keys, indexes, or SQLite metadata - it's the raw size of the stored value strings.

```typescript
import { size } from "runtime:storage";

// Check storage usage
const bytes = await size();
const kb = (bytes / 1024).toFixed(2);
console.log(`Storage is using ${kb} KB`);
```

**Enforce storage quota:**

```typescript
const MAX_STORAGE_BYTES = 10 * 1024 * 1024; // 10 MB

async function setWithQuota(key: string, value: unknown) {
  const currentSize = await size();
  const valueSize = JSON.stringify(value).length;

  if (currentSize + valueSize > MAX_STORAGE_BYTES) {
    throw new Error("Storage quota exceeded");
  }

  await set(key, value);
}
```

**Monitor storage growth:**

```typescript
const before = await size();
await set("large.dataset", bigArray);
const after = await size();
console.log(`Added ${after - before} bytes to storage`);
```

**Returns:** Total size in bytes of all stored values

**Throws:**
- Error [8104] if database operation fails
- Error [8108] if database connection fails

---

## Batch Operations

### getMany(keyList)

Efficiently retrieves multiple values at once from persistent storage.

This is significantly faster than calling `get()` multiple times, especially for large numbers of keys. Only keys that exist are returned in the Map.

**Performance**: Approximately 10x faster than individual `get()` calls for retrieving 10+ keys.

```typescript
import { getMany } from "runtime:storage";

// Bulk retrieval
const keys = ["user.name", "user.email", "user.role"];
const values = await getMany(keys);

console.log("Name:", values.get("user.name"));
console.log("Email:", values.get("user.email"));
console.log("Role:", values.get("user.role"));
```

**Load app state efficiently:**

```typescript
const stateKeys = [
  "window.bounds",
  "window.maximized",
  "recent.files",
  "user.preferences"
];

const state = await getMany(stateKeys);
return {
  windowBounds: state.get("window.bounds"),
  windowMaximized: state.get("window.maximized") ?? false,
  recentFiles: state.get("recent.files") ?? [],
  userPrefs: state.get("user.preferences") ?? {}
};
```

**Check which keys exist:**

```typescript
const requestedKeys = ["key1", "key2", "key3"];
const found = await getMany(requestedKeys);

for (const key of requestedKeys) {
  if (found.has(key)) {
    console.log(`${key}: ${found.get(key)}`);
  } else {
    console.log(`${key}: not found`);
  }
}
```

**Hydrate object from storage:**

```typescript
const allKeys = await keys();
const userKeys = allKeys.filter(k => k.startsWith("user."));
const userData = await getMany(userKeys);

const user = Object.fromEntries(userData);
console.log("User data:", user);
```

**Returns:** Map containing key-value pairs for keys that were found (missing keys are omitted)

**Throws:**
- Error [8104] if database operation fails
- Error [8108] if database connection fails

### setMany(entries)

Atomically stores multiple key-value pairs at once.

All writes are executed within a single database transaction. If any write fails, the entire operation is rolled back and no changes are made.

**Performance**: Approximately 10x faster than individual `set()` calls for storing 10+ key-value pairs.

**Atomicity**: Either all writes succeed or none do (transaction rollback).

```typescript
import { setMany } from "runtime:storage";

// Bulk initialization
await setMany({
  "app.version": "1.0.0",
  "app.firstRun": true,
  "app.installDate": new Date().toISOString(),
  "user.theme": "dark",
  "user.language": "en"
});
```

**Save complex state atomically:**

```typescript
await setMany({
  "window.bounds": { x: 100, y: 100, width: 800, height: 600 },
  "window.maximized": false,
  "window.displayId": 1,
  "recent.files": ["/path/to/file1.txt", "/path/to/file2.txt"],
  "recent.searches": ["typescript", "rust"]
});
```

**Convert Map to storage:**

```typescript
const cache = new Map([
  ["api.users", usersData],
  ["api.posts", postsData],
  ["api.comments", commentsData]
]);

await setMany(Object.fromEntries(cache));
```

**Atomicity example - all or nothing:**

```typescript
try {
  await setMany({
    "user.name": "Alice",
    "user.email": "alice@example.com",
    "user.invalid": circularReference // This will fail!
  });
} catch (err) {
  // None of the values were saved (transaction rolled back)
  console.error("Save failed, no changes made:", err);
}
```

**Throws:**
- Error [8106] if any key is empty
- Error [8102] if any value cannot be serialized to JSON
- Error [8104] if database operation fails
- Error [8108] if database connection fails
- Error [8109] if transaction fails (all changes rolled back)

### deleteMany(keyList)

Efficiently deletes multiple keys at once from persistent storage.

This is significantly faster than calling `remove()` multiple times. Returns the count of keys that actually existed and were deleted.

**Performance**: Approximately 10x faster than individual `remove()` calls for deleting 10+ keys.

```typescript
import { deleteMany } from "runtime:storage";

// Clear user session data
const sessionKeys = [
  "session.token",
  "session.userId",
  "session.expires"
];

const deleted = await deleteMany(sessionKeys);
console.log(`Deleted ${deleted} session keys`);
```

**Clean up cache by prefix:**

```typescript
const allKeys = await keys();
const cacheKeys = allKeys.filter(k => k.startsWith("cache."));

if (cacheKeys.length > 0) {
  const deleted = await deleteMany(cacheKeys);
  console.log(`Cleared ${deleted} cache entries`);
}
```

**Remove old data selectively:**

```typescript
const allKeys = await keys();
const oldKeys = allKeys.filter(k =>
  k.startsWith("temp.") || k.startsWith("deprecated.")
);

if (oldKeys.length > 0) {
  await deleteMany(oldKeys);
}
```

**Batch cleanup with verification:**

```typescript
const keysToDelete = ["key1", "key2", "key3", "key4"];
const deleted = await deleteMany(keysToDelete);

if (deleted === keysToDelete.length) {
  console.log("All keys deleted successfully");
} else {
  console.log(`Only ${deleted}/${keysToDelete.length} keys existed`);
}
```

**Returns:** Number of keys that existed and were successfully deleted

**Throws:**
- Error [8104] if database operation fails
- Error [8108] if database connection fails

---

## Data Types

All JavaScript values that can be serialized to JSON are supported:
- **Primitives**: `string`, `number`, `boolean`, `null`
- **Arrays**: `string[]`, `number[]`, etc.
- **Objects**: `{ key: value }`, nested objects
- **Not supported**: `undefined`, functions, circular references, `BigInt`, `Symbol`

---

## Performance

- Individual operations: ~1-2ms per operation
- Batch operations: ~0.1ms per item (much faster than individual calls)
- Database is indexed on key for fast lookups
- Connection is reused across operations

---

## Lifecycle Hooks

Intercept storage operations with before/after/error hooks:

### onBefore(opName, handler)

Execute before an operation:

```typescript
import { onBefore } from "runtime:storage";

onBefore("set", (args) => {
  console.log("Storing key:", args[0]);
  // Optionally throw to prevent operation
});
```

### onAfter(opName, handler)

Execute after successful operation:

```typescript
import { onAfter } from "runtime:storage";

onAfter("set", (result, args) => {
  console.log("Stored key:", args[0]);
});
```

### onError(opName, handler)

Execute when operation fails:

```typescript
import { onError } from "runtime:storage";

onError("get", (error, args) => {
  console.error("Failed to get:", args[0], error);
});
```

### removeAllHooks(opName?)

Remove all hooks for an operation (or all operations if no name provided):

```typescript
import { removeAllHooks } from "runtime:storage";

// Remove all hooks for specific operation
removeAllHooks("set");

// Remove all hooks for all operations
removeAllHooks();
```

**Supported operations:**
`get`, `set`, `delete`, `has`, `keys`, `clear`, `size`, `getMany`, `setMany`, `deleteMany`

---

## Handler System

Register custom named handlers for storage operations:

### registerHandler(name, handler)

Register a named handler:

```typescript
import { registerHandler } from "runtime:storage";

registerHandler("loadConfig", async (path: string) => {
  const content = await get<string>(path);
  return content ? JSON.parse(content) : null;
});
```

### invokeHandler(name, ...args)

Invoke a handler by name:

```typescript
import { invokeHandler } from "runtime:storage";

const config = await invokeHandler("loadConfig", "app.config");
console.log(config);
```

### listHandlers()

List all registered handlers:

```typescript
import { listHandlers } from "runtime:storage";

const handlers = listHandlers();
console.log("Registered handlers:", handlers); // => ["loadConfig", ...]
```

### hasHandler(name)

Check if a handler exists:

```typescript
import { hasHandler } from "runtime:storage";

if (hasHandler("loadConfig")) {
  const config = await invokeHandler("loadConfig", "app.config");
}
```

### removeHandler(name)

Unregister a handler:

```typescript
import { removeHandler } from "runtime:storage";

removeHandler("loadConfig");
```

---

## Error Handling

All operations throw on error:

```typescript
import { get } from "runtime:storage";

try {
  const value = await get("my.key");
} catch (error) {
  if (error.message.includes("8104")) {
    console.log("Database error");
  } else if (error.message.includes("8108")) {
    console.log("Connection failed");
  }
}
```

---

## Error Codes

| Code | Error | Description |
|------|-------|-------------|
| `8100` | Generic | Unspecified storage error |
| `8101` | NotFound | Key does not exist (rarely thrown, `get()` returns `null`) |
| `8102` | SerializationError | Value cannot be serialized to JSON |
| `8103` | DeserializationError | Stored value is not valid JSON |
| `8104` | DatabaseError | SQLite operation failed |
| `8105` | PermissionDenied | Storage operation not permitted |
| `8106` | InvalidKey | Key is invalid (e.g., empty string) |
| `8107` | QuotaExceeded | Storage quota limit reached |
| `8108` | ConnectionFailed | Database connection cannot be opened |
| `8109` | TransactionFailed | Batch operation rolled back |

---

## Complete Example

```typescript
import {
  get,
  set,
  has,
  keys,
  getMany,
  setMany,
  clear
} from "runtime:storage";

// Initialize app on first run
if (!await has("app.initialized")) {
  await setMany({
    "app.initialized": true,
    "app.installDate": new Date().toISOString(),
    "app.launchCount": 0,
    "user.preferences": {
      theme: "light",
      fontSize: 14,
      notifications: true
    }
  });
}

// Increment launch count
const launchCount = await get<number>("app.launchCount") ?? 0;
await set("app.launchCount", launchCount + 1);

// Load user preferences
const prefs = await get<UserPreferences>("user.preferences");
if (prefs) {
  applyTheme(prefs.theme);
  setFontSize(prefs.fontSize);
}

// Save window state on close
window.addEventListener("beforeunload", async () => {
  await setMany({
    "window.bounds": getCurrentBounds(),
    "window.maximized": isMaximized(),
    "recent.files": getRecentFiles()
  });
});

// Clean up old cache entries
const allKeys = await keys();
const oldCacheKeys = allKeys.filter(k =>
  k.startsWith("cache.") && isCacheExpired(k)
);
if (oldCacheKeys.length > 0) {
  await deleteMany(oldCacheKeys);
  console.log(`Cleaned up ${oldCacheKeys.length} expired cache entries`);
}

// Bulk load app state
const stateKeys = [
  "window.bounds",
  "window.maximized",
  "recent.files",
  "user.preferences"
];
const state = await getMany(stateKeys);
console.log("App state loaded:", Object.fromEntries(state));
```
