# ext_database

**Full-featured SQLite database access for Forge applications**

[![Crate](https://img.shields.io/badge/crates.io-ext__database-orange)](https://crates.io/crates/ext_database)
[![Docs](https://img.shields.io/badge/docs-runtime%3Adatabase-blue)](https://docs.rs/ext_database)

## Overview

The `ext_database` crate provides comprehensive SQLite database capabilities for Forge applications through the `runtime:database` module. Each app can create and manage multiple named databases with full SQL support, transactions, prepared statements, result streaming, and versioned schema migrations.

### Key Features

- **Multiple Databases** - Each app can have multiple named databases
- **Full SQL Support** - Complete SQLite SQL syntax with parameterized queries
- **Transactions** - BEGIN/COMMIT/ROLLBACK with savepoints for nested behavior
- **Prepared Statements** - Compile SQL once, execute multiple times for performance
- **Result Streaming** - Process large result sets in batches to avoid memory issues
- **Schema Migrations** - Versioned up/down migrations with automatic tracking
- **WAL Mode** - Write-Ahead Logging enabled by default for better concurrency
- **Foreign Keys** - Foreign key constraints enabled by default for referential integrity
- **Type Conversion** - Automatic conversion between SQLite and JavaScript types
- **Connection Management** - Automatic database handle management

## TypeScript Usage

### Basic Operations

```typescript
import { open } from "runtime:database";

// Open database (creates if doesn't exist)
const db = await open("myapp");

// Create table
await db.execute(`
  CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    email TEXT UNIQUE,
    active INTEGER DEFAULT 1
  )
`);

// Insert with parameters (prevents SQL injection)
const result = await db.execute(
  "INSERT INTO users (name, email) VALUES (?, ?)",
  ["Alice", "alice@example.com"]
);
console.log("New user ID:", result.lastInsertRowid);

// Query with type safety
interface User {
  id: number;
  name: string;
  email: string;
  active: number;
}

const users = await db.query<User>("SELECT * FROM users WHERE active = ?", [1]);
for (const user of users.rows) {
  console.log(user.name, user.email);
}

// Update
await db.execute(
  "UPDATE users SET active = ? WHERE id = ?",
  [0, userId]
);

// Delete
await db.execute("DELETE FROM users WHERE active = ?", [0]);

await db.close();
```

### Transactions

```typescript
// Transactions are ~1000x faster for bulk operations
await db.transaction(async () => {
  for (const user of users) {
    await db.execute(
      "INSERT INTO users (name, email) VALUES (?, ?)",
      [user.name, user.email]
    );
  }
}); // Automatically commits on success, rolls back on error

// Manual transaction control
await db.begin();
try {
  await db.execute("INSERT INTO accounts (name, balance) VALUES (?, ?)", ["Alice", 1000]);
  await db.execute("INSERT INTO accounts (name, balance) VALUES (?, ?)", ["Bob", 500]);
  await db.commit();
} catch (e) {
  await db.rollback();
  throw e;
}
```

### Prepared Statements

```typescript
// Compile SQL once, execute many times
const stmt = await db.prepare(
  "INSERT INTO logs (level, message, timestamp) VALUES (?, ?, ?)"
);

try {
  for (const log of logEntries) {
    await stmt.execute([log.level, log.message, Date.now()]);
  }
} finally {
  await stmt.finalize(); // Always finalize to free resources
}
```

### Streaming Large Results

```typescript
// Process large datasets without loading all into memory
for await (const batch of db.stream("SELECT * FROM events", [], 100)) {
  console.log(`Processing ${batch.length} events...`);
  for (const event of batch) {
    await processEvent(event);
  }
}
```

## Database Location

Databases are stored at:

| Platform | Path |
|----------|------|
| macOS    | `~/Library/Application Support/.forge/<app-id>/databases/<name>.db` |
| Linux    | `~/.local/share/.forge/<app-id>/databases/<name>.db` |
| Windows  | `%APPDATA%\.forge\<app-id>\databases\<name>.db` |

The database directory and files are created automatically on first use.

## Error Codes

All database operations may throw errors with structured codes:

| Code   | Error                  | Description                                      |
|--------|------------------------|--------------------------------------------------|
| `8400` | Generic                | Unspecified database error                       |
| `8401` | NotFound               | Database/table/row not found                     |
| `8402` | AlreadyExists          | Database/table already exists                    |
| `8403` | SqlSyntax              | Invalid SQL syntax                               |
| `8404` | ConstraintViolation    | UNIQUE, FOREIGN KEY, CHECK, or NOT NULL violated |
| `8405` | TypeMismatch           | Parameter type doesn't match expected type       |
| `8406` | InvalidHandle          | Database handle is invalid or closed             |
| `8407` | TransactionError       | Already in transaction or not in transaction     |
| `8408` | PermissionDenied       | Insufficient permissions for operation           |
| `8409` | TooManyConnections     | Connection limit reached                         |
| `8410` | PreparedStatementError | Statement cannot be prepared or is invalid       |
| `8411` | DatabaseBusy           | Database is locked by another connection         |
| `8412` | IoError                | File I/O error (disk full, permission denied)    |
| `8413` | MigrationError         | Migration failed or invalid version              |
| `8414` | InvalidParameter       | Wrong parameter count or invalid value           |
| `8415` | StreamError            | Stream is closed or invalid                      |

### Error Handling

```typescript
try {
  await db.execute(
    "INSERT INTO users (name, email) VALUES (?, ?)",
    ["Alice", "alice@example.com"]
  );
} catch (err) {
  if (err.message.includes("[8404]")) {
    console.error("Constraint violation (duplicate email?)");
  } else if (err.message.includes("[8403]")) {
    console.error("SQL syntax error");
  } else {
    console.error("Database error:", err);
  }
}
```

## Common Patterns

### 1. Application Data Persistence

```typescript
// Create application database with schema
const db = await open("appdata");

await db.execute(`
  CREATE TABLE IF NOT EXISTS app_state (
    key TEXT PRIMARY KEY,
    value TEXT,
    updated_at INTEGER
  )
`);

// Save state
async function saveState(key: string, value: unknown): Promise<void> {
  await db.execute(
    "INSERT OR REPLACE INTO app_state (key, value, updated_at) VALUES (?, ?, ?)",
    [key, JSON.stringify(value), Date.now()]
  );
}

// Load state
async function loadState<T>(key: string): Promise<T | null> {
  const row = await db.queryRow<{ value: string }>(
    "SELECT value FROM app_state WHERE key = ?",
    [key]
  );
  return row ? JSON.parse(row.value) : null;
}
```

### 2. User Management System

```typescript
// Create users database with indexes
await db.execute(`
  CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    last_login INTEGER
  )
`);

await db.execute("CREATE INDEX IF NOT EXISTS idx_users_username ON users(username)");
await db.execute("CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)");

// Register user
async function registerUser(username: string, email: string, passwordHash: string) {
  const result = await db.execute(
    "INSERT INTO users (username, email, password_hash, created_at) VALUES (?, ?, ?, ?)",
    [username, email, passwordHash, Date.now()]
  );
  return result.lastInsertRowid;
}

// Authenticate user
async function authenticateUser(username: string): Promise<User | null> {
  return await db.queryRow<User>(
    "SELECT * FROM users WHERE username = ?",
    [username]
  );
}

// Update last login
async function updateLastLogin(userId: number): Promise<void> {
  await db.execute(
    "UPDATE users SET last_login = ? WHERE id = ?",
    [Date.now(), userId]
  );
}
```

### 3. Analytics and Logging

```typescript
// Create analytics database
const analyticsDb = await open("analytics");

await analyticsDb.execute(`
  CREATE TABLE IF NOT EXISTS events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type TEXT NOT NULL,
    user_id INTEGER,
    data TEXT,
    timestamp INTEGER NOT NULL
  )
`);

await analyticsDb.execute("CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type)");
await analyticsDb.execute("CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp)");

// Log event (use transactions for bulk logging)
async function logEvents(events: Event[]): Promise<void> {
  await analyticsDb.transaction(async () => {
    for (const event of events) {
      await analyticsDb.execute(
        "INSERT INTO events (event_type, user_id, data, timestamp) VALUES (?, ?, ?, ?)",
        [event.type, event.userId, JSON.stringify(event.data), event.timestamp]
      );
    }
  });
}

// Query analytics
async function getEventsByType(eventType: string, startTime: number): Promise<Event[]> {
  const result = await analyticsDb.query<Event>(
    "SELECT * FROM events WHERE event_type = ? AND timestamp >= ? ORDER BY timestamp DESC LIMIT 1000",
    [eventType, startTime]
  );
  return result.rows;
}
```

### 4. Schema Migrations

```typescript
// Define migrations
const migrations = [
  {
    version: 1,
    name: "create_users_table",
    upSql: `
      CREATE TABLE users (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        email TEXT UNIQUE
      )
    `,
    downSql: "DROP TABLE users"
  },
  {
    version: 2,
    name: "add_users_active_column",
    upSql: "ALTER TABLE users ADD COLUMN active INTEGER DEFAULT 1",
    downSql: "ALTER TABLE users DROP COLUMN active"
  },
  {
    version: 3,
    name: "create_sessions_table",
    upSql: `
      CREATE TABLE sessions (
        id TEXT PRIMARY KEY,
        user_id INTEGER NOT NULL,
        expires_at INTEGER NOT NULL,
        FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
      )
    `,
    downSql: "DROP TABLE sessions"
  }
];

// Apply migrations
const status = await db.migrate(migrations);
console.log(`Database at version ${status.currentVersion}`);

// Check migration status
const currentStatus = await db.migrationStatus();
console.log("Applied migrations:", currentStatus.applied.map(m => m.name));
console.log("Pending migrations:", currentStatus.pending);

// Rollback to version 1
if (needsRollback) {
  await db.migrateDown(1);
}
```

### 5. Caching with Expiration

```typescript
// Create cache database
const cacheDb = await open("cache");

await cacheDb.execute(`
  CREATE TABLE IF NOT EXISTS cache (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    expires_at INTEGER NOT NULL
  )
`);

await cacheDb.execute("CREATE INDEX IF NOT EXISTS idx_cache_expires ON cache(expires_at)");

// Set cache with TTL
async function cacheSet(key: string, value: unknown, ttlMs: number): Promise<void> {
  await cacheDb.execute(
    "INSERT OR REPLACE INTO cache (key, value, expires_at) VALUES (?, ?, ?)",
    [key, JSON.stringify(value), Date.now() + ttlMs]
  );
}

// Get from cache
async function cacheGet<T>(key: string): Promise<T | null> {
  const row = await cacheDb.queryRow<{ value: string; expires_at: number }>(
    "SELECT value, expires_at FROM cache WHERE key = ?",
    [key]
  );

  if (!row) return null;

  if (Date.now() > row.expires_at) {
    await cacheDb.execute("DELETE FROM cache WHERE key = ?", [key]);
    return null;
  }

  return JSON.parse(row.value);
}

// Clean up expired entries
async function cleanExpiredCache(): Promise<void> {
  const result = await cacheDb.execute(
    "DELETE FROM cache WHERE expires_at < ?",
    [Date.now()]
  );
  console.log(`Cleaned up ${result.rowsAffected} expired cache entries`);
}
```

### 6. Batch Data Processing

```typescript
// Export large dataset to CSV
async function exportToCSV(outputPath: string): Promise<void> {
  const file = await Deno.open(outputPath, { write: true, create: true });

  try {
    let count = 0;
    for await (const batch of db.stream<User>("SELECT * FROM users", [], 1000)) {
      for (const user of batch) {
        const line = `${user.id},${user.name},${user.email}\n`;
        await file.write(new TextEncoder().encode(line));
        count++;
      }
    }
    console.log(`Exported ${count} users`);
  } finally {
    file.close();
  }
}

// Import data from array using batch operations
async function importUsers(users: User[]): Promise<void> {
  const batchSize = 100;

  for (let i = 0; i < users.length; i += batchSize) {
    const batch = users.slice(i, i + batchSize);

    await db.transaction(async () => {
      const stmt = await db.prepare(
        "INSERT INTO users (name, email) VALUES (?, ?)"
      );
      try {
        for (const user of batch) {
          await stmt.execute([user.name, user.email]);
        }
      } finally {
        await stmt.finalize();
      }
    });
  }
}
```

## Performance Considerations

### Transactions

- **Without transaction**: ~0.01 inserts/sec (very slow due to disk sync)
- **With transaction**: ~10,000 inserts/sec (~1000x faster)
- Use `db.transaction()` for bulk inserts/updates
- Batch operations use transactions by default

### Prepared Statements

- **Regular query**: Parse SQL on every call
- **Prepared statement**: Parse once, execute many times
- Use for repeated queries with different parameters
- 2-5x faster for repeated operations

### Result Streaming

- **Regular query**: Loads all rows into memory
- **Streaming**: Processes rows in batches
- Use for result sets larger than 1000 rows
- Default batch size: 100 rows (configurable)

### Indexing

- Create indexes on columns used in WHERE clauses
- Dramatically speeds up queries on large tables
- Use `EXPLAIN QUERY PLAN SELECT ...` to check if index is used

```sql
-- Create index
CREATE INDEX idx_users_email ON users(email);

-- Verify index is used
EXPLAIN QUERY PLAN SELECT * FROM users WHERE email = 'alice@example.com';
-- Should show "USING INDEX idx_users_email"
```

## Database Features

### WAL Mode (Write-Ahead Logging)

Enabled by default:
- Better concurrency (readers don't block writers)
- Better performance (fewer disk syncs)
- Atomic commits (crash-safe transactions)

Disable with `walMode: false` if needed.

### Foreign Key Constraints

Enabled by default:
- Enforces referential integrity
- Prevents orphaned records
- Cascade deletes and updates

```sql
CREATE TABLE orders (
  id INTEGER PRIMARY KEY,
  user_id INTEGER NOT NULL,
  FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
);
```

### Busy Timeout

Default: 5000ms (5 seconds)
- Automatically retries when database is locked
- Configurable with `busyTimeoutMs` option

```typescript
const db = await open("shared", { busyTimeoutMs: 10000 });
```

## Testing

Run the test suite:

```bash
# Run all tests
cargo test -p ext_database

# Run with output
cargo test -p ext_database -- --nocapture

# Run specific test
cargo test -p ext_database test_transactions
```

## Build Configuration

The extension uses `forge-weld` for TypeScript binding generation:

```rust
// build.rs
use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_database", "runtime:database")
        .ts_path("ts/init.ts")
        .ops(&[
            // Connection Management (6 ops)
            "op_database_open",
            "op_database_close",
            "op_database_list",
            "op_database_delete",
            "op_database_exists",
            "op_database_path",
            "op_database_vacuum",
            // Query Execution (5 ops)
            "op_database_query",
            "op_database_execute",
            "op_database_execute_batch",
            "op_database_query_row",
            "op_database_query_value",
            // Prepared Statements (4 ops)
            "op_database_prepare",
            "op_database_stmt_query",
            "op_database_stmt_execute",
            "op_database_stmt_finalize",
            // Transactions (6 ops)
            "op_database_begin",
            "op_database_commit",
            "op_database_rollback",
            "op_database_savepoint",
            "op_database_release",
            "op_database_rollback_to",
            // Schema Operations (3 ops)
            "op_database_tables",
            "op_database_table_info",
            "op_database_table_exists",
            // Streaming (3 ops)
            "op_database_stream_open",
            "op_database_stream_next",
            "op_database_stream_close",
            // Migrations (3 ops)
            "op_database_migrate",
            "op_database_migration_status",
            "op_database_migrate_down",
        ])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build runtime_database extension");
}
```

## Implementation Details

### Connection Management

- Databases are opened lazily on first use
- Each database has a unique handle ID (UUID)
- Connections are stored in `OpState` with `Arc<Mutex<Connection>>`
- Thread-safe access via async mutex locks

### Type Conversion

SQLite to JavaScript type mapping:

| SQLite Type | JavaScript Type |
|-------------|-----------------|
| INTEGER     | number          |
| REAL        | number          |
| TEXT        | string          |
| BLOB        | Uint8Array      |
| NULL        | null            |

### Transactions

- `begin()` starts a transaction (default: deferred mode)
- `commit()` makes changes permanent
- `rollback()` discards all changes
- `transaction()` helper handles commit/rollback automatically
- Savepoints allow nested transaction-like behavior

### Prepared Statements

- Compiled and cached in `OpState`
- Each statement has a unique ID
- Must be finalized to free resources
- Fallback to regular execution if not cached

### Result Streaming

- Opens a cursor for the query
- Fetches results in configurable batches
- Cursor tracked in `OpState` by stream ID
- Automatically closes on completion or error

### Migrations

- Tracked in `_migrations` table (automatically created)
- Each migration has version, name, applied timestamp
- Versions must be sequential (1, 2, 3, ...)
- `upSql` applied in order, `downSql` in reverse
- Failed migrations are rolled back

## Extension Registration

This extension is registered as **Tier 1 (SimpleState)** in the Forge runtime:

```rust
// forge-runtime/src/ext_registry.rs
ExtensionDescriptor {
    name: "runtime_database",
    tier: ExtensionTier::SimpleState,
    init_fn: init_database_state,
    required: false,
}
```

State initialization:

```rust
pub fn init_database_state(op_state: &mut OpState, app_identifier: String) {
    op_state.put(DatabaseAppInfo { app_identifier });
}
```

## Dependencies

| Dependency           | Purpose                              |
|----------------------|--------------------------------------|
| `deno_core`          | Op definitions and extension system  |
| `rusqlite`           | SQLite database bindings             |
| `serde_json`         | JSON serialization for type conversion|
| `dirs`               | Platform-specific directory paths    |
| `tokio`              | Async runtime                        |
| `uuid`               | Database handle and stream IDs       |
| `forge-weld`         | Build-time code generation           |
| `forge-weld-macro`   | `#[weld_op]` proc macros             |
| `linkme`             | Compile-time symbol collection       |
| `thiserror`          | Error type derivation                |
| `deno_error`         | JavaScript error conversion          |

## Security Considerations

### SQL Injection Prevention

- **Always use parameterized queries** (never string concatenation)
- Parameters are passed as array, preventing SQL injection

```typescript
// ✅ SAFE
await db.execute(
  "SELECT * FROM users WHERE name = ?",
  [userName]
);

// ❌ UNSAFE - Never do this!
await db.execute(
  `SELECT * FROM users WHERE name = '${userName}'`
);
```

### Foreign Key Enforcement

- Enabled by default to prevent orphaned records
- Use `ON DELETE CASCADE` to auto-delete related records
- Use `ON DELETE SET NULL` to nullify foreign keys

### File Access

- Database files are stored in app-specific directory
- Apps cannot access other apps' databases
- File permissions controlled by operating system

### Sensitive Data

- Consider encrypting sensitive fields (use `ext_crypto`)
- Don't store passwords in plain text (hash with salt)
- Use transactions to ensure data consistency

## Related Extensions

- [`ext_storage`](../ext_storage) - Simple key-value storage (simpler alternative)
- [`ext_crypto`](../ext_crypto) - Encryption for sensitive database fields
- [`ext_fs`](../ext_fs) - File operations for database backup/restore

## License

Part of the Forge project. See the root LICENSE file for details.
