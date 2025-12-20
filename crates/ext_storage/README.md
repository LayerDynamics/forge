# ext_storage

**SQLite-backed persistent key-value storage for Forge applications**

[![Crate](https://img.shields.io/badge/crates.io-ext__storage-orange)](https://crates.io/crates/ext_storage)
[![Docs](https://img.shields.io/badge/docs-runtime%3Astorage-blue)](https://docs.rs/ext_storage)

## Overview

The `ext_storage` crate provides a simple, reliable persistent key-value store for Forge applications through the `runtime:storage` module. All data is automatically persisted to a SQLite database with JSON serialization, making it easy to store and retrieve complex data structures.

### Key Features

- **SQLite Backend** - ACID-compliant persistent storage
- **Automatic Serialization** - JSON encoding/decoding for any serializable value
- **Indexed Queries** - Fast key lookups with SQLite indexing
- **Connection Pooling** - Automatic connection reuse and management
- **Batch Operations** - Efficient bulk reads, writes, and deletes (~10x faster)
- **Timestamps** - Automatic `created_at` and `updated_at` tracking
- **Transactional Writes** - Atomic batch operations with rollback support

## TypeScript Usage

### Basic Operations

```typescript
import { get, set, remove, has, keys, clear, size } from "runtime:storage";

// Store values
await set("user.name", "Alice");
await set("user.preferences", { theme: "dark", fontSize: 14 });
await set("recent.files", ["/path/to/file1.txt", "/path/to/file2.txt"]);

// Retrieve values
const name = await get<string>("user.name");
const prefs = await get<UserPrefs>("user.preferences");
const files = await get<string[]>("recent.files");

// Check existence
if (await has("user.session")) {
  console.log("User is logged in");
}

// Remove values
await remove("user.session");

// List all keys
const allKeys = await keys();
console.log(`Storage contains ${allKeys.length} keys`);

// Get storage size
const bytes = await size();
console.log(`Using ${(bytes / 1024).toFixed(2)} KB`);

// Clear all data
await clear();
```

### Batch Operations

Batch operations are **approximately 10x faster** than individual calls for 10+ items:

```typescript
import { getMany, setMany, deleteMany } from "runtime:storage";

// Bulk retrieval
const values = await getMany(["key1", "key2", "key3"]);
console.log("Key1:", values.get("key1"));
console.log("Key2:", values.get("key2"));

// Atomic bulk write (all-or-nothing transaction)
await setMany({
  "app.version": "1.0.0",
  "app.firstRun": true,
  "app.installDate": new Date().toISOString(),
  "window.bounds": { x: 100, y: 100, width: 800, height: 600 },
  "window.maximized": false
});

// Bulk deletion
const cacheKeys = (await keys()).filter(k => k.startsWith("cache."));
const deleted = await deleteMany(cacheKeys);
console.log(`Deleted ${deleted} cache entries`);
```

## Storage Location

The SQLite database is created at:

| Platform | Path |
|----------|------|
| macOS    | `~/Library/Application Support/.forge/<app-id>/storage.db` |
| Linux    | `~/.local/share/.forge/<app-id>/storage.db` |
| Windows  | `%APPDATA%\.forge\<app-id>\storage.db` |

The database directory and file are created automatically on first use.

## Error Codes

All storage operations may throw errors with structured codes:

| Code   | Error                  | Description                                      |
|--------|------------------------|--------------------------------------------------|
| `8100` | Generic                | Unspecified storage error                        |
| `8101` | NotFound               | Key does not exist (rarely thrown)               |
| `8102` | SerializationError     | Value cannot be serialized to JSON               |
| `8103` | DeserializationError   | Stored value is not valid JSON                   |
| `8104` | DatabaseError          | SQLite operation failed                          |
| `8105` | PermissionDenied       | Storage operation not permitted                  |
| `8106` | InvalidKey             | Key is invalid (e.g., empty string)              |
| `8107` | QuotaExceeded          | Storage quota limit reached                      |
| `8108` | ConnectionFailed       | Database connection cannot be opened             |
| `8109` | TransactionFailed      | Batch operation failed and rolled back           |

### Error Handling

```typescript
try {
  await set("user.data", complexObject);
} catch (err) {
  if (err.message.includes("[8102]")) {
    console.error("Cannot serialize circular reference");
  } else if (err.message.includes("[8106]")) {
    console.error("Key cannot be empty");
  } else {
    console.error("Storage error:", err);
  }
}
```

## Common Patterns

### 1. Application State Persistence

```typescript
// Save app state on exit
async function saveAppState() {
  await setMany({
    "window.bounds": getCurrentWindowBounds(),
    "window.maximized": isWindowMaximized(),
    "recent.files": getRecentFiles(),
    "recent.searches": getRecentSearches(),
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
    "recent.searches",
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

### 2. User Preferences

```typescript
// Default preferences with override
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
interface CacheEntry<T> {
  value: T;
  expiresAt: number;
}

async function cacheSet<T>(key: string, value: T, ttlMs: number): Promise<void> {
  const entry: CacheEntry<T> = {
    value,
    expiresAt: Date.now() + ttlMs
  };
  await set(`cache.${key}`, entry);
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
await cacheSet("api.users", usersData, 60 * 1000); // 1 minute TTL
const users = await cacheGet("api.users");
```

### 4. Storage Quota Management

```typescript
const MAX_STORAGE_BYTES = 10 * 1024 * 1024; // 10 MB

async function setWithQuota(key: string, value: unknown): Promise<void> {
  const currentSize = await size();
  const valueSize = JSON.stringify(value).length;

  if (currentSize + valueSize > MAX_STORAGE_BYTES) {
    // Clean up old cache entries
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

### 5. Migration and Versioning

```typescript
const STORAGE_VERSION = 2;

async function migrateStorageIfNeeded(): Promise<void> {
  const version = await get<number>("storage.version") ?? 1;

  if (version < 2) {
    // Migrate from version 1 to 2
    const allKeys = await keys();
    const oldKeys = allKeys.filter(k => k.startsWith("old_"));

    for (const oldKey of oldKeys) {
      const value = await get(oldKey);
      const newKey = oldKey.replace("old_", "new_");
      await set(newKey, value);
      await remove(oldKey);
    }

    await set("storage.version", 2);
    console.log("Migrated storage to version 2");
  }
}
```

## Performance Considerations

### Individual Operations

- **Latency**: ~1-2ms per operation
- **Throughput**: ~500-1000 ops/sec
- **Use case**: Occasional reads/writes, single values

### Batch Operations

- **Latency**: ~0.1ms per item
- **Throughput**: ~5000-10000 items/sec
- **Use case**: Loading/saving app state, bulk cache operations

### Recommendations

- ✅ Use `getMany()`, `setMany()`, `deleteMany()` for 10+ items
- ✅ Use batch operations when saving app state
- ✅ Cache frequently accessed values in memory
- ❌ Don't call `get()` in a loop - use `getMany()` instead
- ❌ Don't store very large values (>1 MB) - use `ext_database` instead

## Database Schema

The storage table is created automatically on first use:

```sql
CREATE TABLE IF NOT EXISTS kv_store (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    created_at INTEGER DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_kv_key ON kv_store(key);
```

Fields:
- `key`: Primary key for fast lookups
- `value`: JSON-serialized value string
- `created_at`: Unix timestamp when key was first created
- `updated_at`: Unix timestamp when key was last modified

## Testing

Run the test suite:

```bash
# Run all tests
cargo test -p ext_storage

# Run with output
cargo test -p ext_storage -- --nocapture

# Run specific test
cargo test -p ext_storage test_error_codes
```

## Build Configuration

The extension uses `forge-weld` for TypeScript binding generation:

```rust
// build.rs
use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_storage", "runtime:storage")
        .ts_path("ts/init.ts")
        .ops(&[
            "op_storage_get",
            "op_storage_set",
            "op_storage_delete",
            "op_storage_has",
            "op_storage_keys",
            "op_storage_clear",
            "op_storage_size",
            "op_storage_get_many",
            "op_storage_set_many",
            "op_storage_delete_many",
        ])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build runtime_storage extension");
}
```

## Implementation Details

### Connection Management

The extension uses lazy connection initialization:

1. On first operation, checks if connection exists in `OpState`
2. If not, creates database directory and opens connection
3. Creates schema (table + index) if not exists
4. Stores connection in `OpState` for reuse
5. All subsequent operations reuse the same connection

The connection is wrapped in `Arc<Mutex<Connection>>` for thread-safe access.

### Serialization

All values are serialized using `serde_json`:

- **Supported**: `string`, `number`, `boolean`, `null`, arrays, objects
- **Not supported**: `undefined`, functions, circular references, `BigInt`, `Symbol`

### Batch Operations

**`setMany()` uses SQLite transactions**:

```rust
let tx = conn.transaction()?;
for (key, value) in entries {
    tx.execute("INSERT ... ON CONFLICT ... UPDATE ...", params)?;
}
tx.commit()?; // All-or-nothing
```

If any write fails, the entire transaction is rolled back.

### Key Validation

- Empty keys are rejected with `InvalidKey [8106]` error
- Other characters are not restricted
- Recommended: Use alphanumeric + dots/underscores (e.g., `user.preferences`)

## Extension Registration

This extension is registered as **Tier 1 (SimpleState)** in the Forge runtime:

```rust
// forge-runtime/src/ext_registry.rs
ExtensionDescriptor {
    name: "runtime_storage",
    tier: ExtensionTier::SimpleState,
    init_fn: init_storage_state,
    required: false,
}
```

State initialization:

```rust
pub fn init_storage_state(
    op_state: &mut OpState,
    app_identifier: String,
    capabilities: Option<Arc<dyn StorageCapabilityChecker>>,
) {
    op_state.put(StorageAppInfo { app_identifier });
    if let Some(caps) = capabilities {
        op_state.put(StorageCapabilities { checker: caps });
    }
}
```

## Dependencies

| Dependency           | Purpose                              |
|----------------------|--------------------------------------|
| `deno_core`          | Op definitions and extension system  |
| `rusqlite`           | SQLite database bindings             |
| `serde_json`         | JSON serialization/deserialization   |
| `dirs`               | Platform-specific directory paths    |
| `tokio`              | Async runtime                        |
| `forge-weld`         | Build-time code generation           |
| `forge-weld-macro`   | `#[weld_op]` proc macros             |
| `linkme`             | Compile-time symbol collection       |
| `thiserror`          | Error type derivation                |
| `deno_error`         | JavaScript error conversion          |

## Security Considerations

### Key Naming

- Use namespaced keys (e.g., `user.`, `app.`, `cache.`) for organization
- Avoid storing sensitive data with obvious key names
- Don't expose internal key names to untrusted code

### Value Validation

- Always validate values after retrieval
- Don't trust that stored values match expected types
- Handle deserialization errors gracefully

### Storage Limits

- Monitor storage size with `size()`
- Implement quota management for user-generated content
- Clean up cache/temp data periodically

## Related Extensions

- [`ext_database`](../ext_database) - Full SQL database access
- [`ext_app`](../ext_app) - Application paths and metadata
- [`ext_crypto`](../ext_crypto) - Encryption for sensitive data

## License

Part of the Forge project. See the root LICENSE file for details.
