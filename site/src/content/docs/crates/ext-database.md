---
title: "ext_database"
description: Full-featured SQLite database extension providing the runtime:database module.
slug: crates/ext-database
---

The `ext_database` crate provides comprehensive SQLite database capabilities for Forge applications through the `runtime:database` module.

## Overview

ext_database gives you complete SQL database access with:

- **Multiple Databases** - Each app can have multiple named databases
- **Full SQL Support** - Complete SQLite SQL syntax with parameterized queries
- **Transactions** - BEGIN/COMMIT/ROLLBACK with savepoints
- **Prepared Statements** - Compile SQL once, execute many times
- **Result Streaming** - Process large result sets in batches
- **Schema Migrations** - Versioned up/down migrations
- **WAL Mode** - Write-Ahead Logging for better concurrency (default)
- **Foreign Keys** - Constraint enforcement enabled by default

## Quick Start

```typescript
import { open } from "runtime:database";

// Open database (creates if doesn't exist)
const db = await open("myapp");

// Create table
await db.execute(`
  CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    email TEXT UNIQUE
  )
`);

// Insert data
const result = await db.execute(
  "INSERT INTO users (name, email) VALUES (?, ?)",
  ["Alice", "alice@example.com"]
);
console.log("New user ID:", result.lastInsertRowid);

// Query data
interface User { id: number; name: string; email: string; }
const users = await db.query<User>("SELECT * FROM users");
for (const user of users.rows) {
  console.log(user.name, user.email);
}

await db.close();
```

## Core Concepts

### Database Handles

Each database connection returns a `Database` object with all operations:

```typescript
const db = await open("myapp");  // Returns Database object
await db.query("SELECT ...");     // Query on the database
await db.execute("INSERT ...");   // Execute on the database
await db.close();                 // Close when done
```

### Multiple Databases

Apps can have multiple named databases:

```typescript
const userDb = await open("users");
const analyticsDb = await open("analytics");
const cacheDb = await open("cache");
```

### Database Location

Databases are stored in platform-specific directories:

- **macOS**: `~/Library/Application Support/.forge/<app-id>/databases/<name>.db`
- **Linux**: `~/.local/share/.forge/<app-id>/databases/<name>.db`
- **Windows**: `%APPDATA%\.forge\<app-id>\databases\<name>.db`

### Parameterized Queries

Always use `?` placeholders to prevent SQL injection:

```typescript
// ✅ Safe - parameterized
await db.execute(
  "SELECT * FROM users WHERE name = ?",
  [userName]
);

// ❌ Unsafe - never concatenate!
await db.execute(`SELECT * FROM users WHERE name = '${userName}'`);
```

### Type Safety

Use TypeScript interfaces for query results:

```typescript
interface User {
  id: number;
  name: string;
  email: string;
  active: number;
}

const result = await db.query<User>("SELECT * FROM users");
// result.rows is typed as User[]
```

## API Reference

### Module-Level Functions

#### `open(name, opts?): Promise<Database>`

Open or create a database by name.

**Parameters:**
- `name` (string) - Database name (without .db extension)
- `opts` (OpenOptions, optional) - Configuration options
  - `create` (boolean) - Create if doesn't exist (default: true)
  - `readonly` (boolean) - Open in read-only mode (default: false)
  - `walMode` (boolean) - Enable WAL mode (default: true)
  - `busyTimeoutMs` (number) - Busy timeout in milliseconds (default: 5000)
  - `foreignKeys` (boolean) - Enable foreign keys (default: true)

**Returns:** Database connection handle

**Throws:**
- `[8401]` if database doesn't exist and `create: false`
- `[8408]` if permission denied
- `[8412]` if I/O error

**Examples:**

```typescript
// Default options (create, WAL mode, foreign keys)
const db = await open("myapp");

// Read-only mode
const readDb = await open("reports", { readonly: true });

// Custom timeout
const busyDb = await open("shared", { busyTimeoutMs: 10000 });

// Multiple databases
const userDb = await open("users");
const cacheDb = await open("cache");
```

#### `list(): Promise<DatabaseInfo[]>`

List all databases for the current app.

**Returns:** Array of database metadata

**Examples:**

```typescript
const databases = await list();
for (const db of databases) {
  console.log(`${db.name}: ${db.sizeBytes} bytes, ${db.tables.length} tables`);
}

// Find large databases
const large = databases.filter(db => db.sizeBytes > 1024 * 1024);
```

#### `exists(name): Promise<boolean>`

Check if a database file exists.

**Parameters:**
- `name` (string) - Database name

**Returns:** True if database exists

**Examples:**

```typescript
if (await exists("users")) {
  console.log("Users database exists");
} else {
  const db = await open("users");
  await db.execute("CREATE TABLE users (...)");
}
```

#### `remove(name): Promise<boolean>`

Delete a database file permanently.

**Parameters:**
- `name` (string) - Database name

**Returns:** True if deleted, false if didn't exist

**Throws:**
- `[8411]` if database is currently open (busy/locked)

**Examples:**

```typescript
await remove("old-cache");

// Clean up temp databases
const databases = await list();
for (const db of databases.filter(d => d.name.startsWith("temp_"))) {
  await remove(db.name);
}
```

#### `path(name): string`

Get full filesystem path for a database.

**Parameters:**
- `name` (string) - Database name

**Returns:** Absolute path to database file

**Examples:**

```typescript
const dbPath = path("myapp");
console.log("Database at:", dbPath);
```

### Database Object Methods

#### `query<T>(sql, params?): Promise<QueryResult<T>>`

Execute a SELECT query and return all rows.

**Parameters:**
- `sql` (string) - SQL SELECT statement (use ? for parameters)
- `params` (unknown[], optional) - Parameter values to bind

**Returns:** QueryResult with typed rows

**Throws:**
- `[8403]` if SQL syntax is invalid
- `[8414]` if parameter count doesn't match

**Examples:**

```typescript
interface User { id: number; name: string; email: string; }

// Query all
const all = await db.query<User>("SELECT * FROM users");

// Query with parameters
const active = await db.query<User>(
  "SELECT * FROM users WHERE active = ?",
  [1]
);

// Complex query
const result = await db.query<User>(
  "SELECT * FROM users WHERE name LIKE ? AND created_at > ? ORDER BY name LIMIT ?",
  ["%Smith%", Date.now() - 30 * 24 * 60 * 60 * 1000, 10]
);
```

#### `execute(sql, params?): Promise<ExecuteResult>`

Execute an INSERT, UPDATE, or DELETE statement.

**Parameters:**
- `sql` (string) - SQL statement
- `params` (unknown[], optional) - Parameter values to bind

**Returns:** ExecuteResult with rowsAffected and lastInsertRowid

**Throws:**
- `[8403]` if SQL syntax is invalid
- `[8404]` if constraint is violated (UNIQUE, FOREIGN KEY, etc.)
- `[8414]` if parameter count doesn't match

**Examples:**

```typescript
// INSERT
const result = await db.execute(
  "INSERT INTO users (name, email) VALUES (?, ?)",
  ["Alice", "alice@example.com"]
);
console.log("New user ID:", result.lastInsertRowid);

// UPDATE
const updated = await db.execute(
  "UPDATE users SET active = ? WHERE last_login < ?",
  [0, Date.now() - 90 * 24 * 60 * 60 * 1000]
);
console.log("Deactivated:", updated.rowsAffected);

// DELETE
const deleted = await db.execute(
  "DELETE FROM sessions WHERE expires_at < ?",
  [Date.now()]
);
```

#### `executeBatch(statements, opts?): Promise<BatchResult>`

Execute multiple SQL statements in a batch.

**Parameters:**
- `statements` (string[]) - Array of SQL statements
- `opts` (BatchOptions, optional)
  - `transaction` (boolean) - Run in transaction (default: true)
  - `stopOnError` (boolean) - Stop on first error (default: true)

**Returns:** BatchResult with statistics and errors

**Examples:**

```typescript
// Atomic batch (default)
await db.executeBatch([
  "INSERT INTO logs (message) VALUES ('Started')",
  "INSERT INTO logs (message) VALUES ('Processing')",
  "INSERT INTO logs (message) VALUES ('Completed')"
]);

// Continue on errors
const result = await db.executeBatch([
  "INSERT INTO users (name) VALUES ('Alice')",
  "INVALID SQL",
  "INSERT INTO users (name) VALUES ('Bob')"
], { stopOnError: false });
console.log("Errors:", result.errors);
```

#### `queryRow<T>(sql, params?): Promise<T | null>`

Execute a query and return only the first row.

**Parameters:**
- `sql` (string) - SQL SELECT statement
- `params` (unknown[], optional) - Parameter values

**Returns:** First row as object, or null if no rows

**Examples:**

```typescript
interface User { id: number; name: string; email: string; }

const user = await db.queryRow<User>(
  "SELECT * FROM users WHERE id = ?",
  [42]
);

if (user) {
  console.log("Found:", user.name);
} else {
  console.log("User not found");
}
```

#### `queryValue<T>(sql, params?): Promise<T | null>`

Execute a query and return only the first column of the first row.

**Parameters:**
- `sql` (string) - SQL SELECT statement
- `params` (unknown[], optional) - Parameter values

**Returns:** First value from first row, or null

**Examples:**

```typescript
// Count rows
const count = await db.queryValue<number>(
  "SELECT COUNT(*) FROM users WHERE active = ?",
  [1]
);

// Sum values
const total = await db.queryValue<number>(
  "SELECT SUM(amount) FROM orders WHERE user_id = ?",
  [userId]
);
```

#### `prepare(sql): Promise<PreparedStatement>`

Prepare a SQL statement for repeated execution.

**Parameters:**
- `sql` (string) - SQL statement (use ? for parameters)

**Returns:** Prepared statement handle

**Throws:**
- `[8403]` if SQL syntax is invalid
- `[8410]` if statement cannot be prepared

**Examples:**

```typescript
const stmt = await db.prepare(
  "INSERT INTO events (type, data) VALUES (?, ?)"
);

try {
  for (const event of events) {
    await stmt.execute([event.type, JSON.stringify(event.data)]);
  }
} finally {
  await stmt.finalize(); // Always finalize
}
```

#### `transaction<T>(fn): Promise<T>`

Execute a function within a transaction.

**Parameters:**
- `fn` (() => Promise<T>) - Async function to execute

**Returns:** Value returned by the function

**Throws:** Any error thrown by the function (after rolling back)

**Examples:**

```typescript
// Transfer money between accounts
await db.transaction(async () => {
  await db.execute(
    "UPDATE accounts SET balance = balance - ? WHERE id = ?",
    [100, fromAccountId]
  );
  await db.execute(
    "UPDATE accounts SET balance = balance + ? WHERE id = ?",
    [100, toAccountId]
  );
}); // Commits on success, rolls back on error

// Bulk insert (~1000x faster than individual inserts)
await db.transaction(async () => {
  for (const user of users) {
    await db.execute(
      "INSERT INTO users (name, email) VALUES (?, ?)",
      [user.name, user.email]
    );
  }
});
```

#### `begin(mode?): Promise<void>`

Begin a database transaction manually.

**Parameters:**
- `mode` ("deferred" | "immediate" | "exclusive", optional) - Transaction mode

**Throws:** `[8407]` if already in a transaction

#### `commit(): Promise<void>`

Commit the current transaction.

**Throws:** `[8407]` if not in a transaction

#### `rollback(): Promise<void>`

Rollback the current transaction.

**Throws:** `[8407]` if not in a transaction

**Examples:**

```typescript
await db.begin();
try {
  await db.execute("INSERT INTO ...");
  await db.execute("UPDATE ...");
  await db.commit();
} catch (e) {
  await db.rollback();
  throw e;
}
```

#### `savepoint(name): Promise<void>`

Create a savepoint within a transaction.

#### `release(name): Promise<void>`

Release (commit) a savepoint.

#### `rollbackTo(name): Promise<void>`

Rollback to a savepoint.

**Examples:**

```typescript
await db.begin();
await db.execute("INSERT INTO users (name) VALUES (?)", ["Alice"]);

await db.savepoint("before_bob");
await db.execute("INSERT INTO users (name) VALUES (?)", ["Bob"]);

// Rollback Bob's insert
await db.rollbackTo("before_bob");

await db.execute("INSERT INTO users (name) VALUES (?)", ["Charlie"]);
await db.commit(); // Alice and Charlie inserted
```

#### `tables(): Promise<string[]>`

List all table names in the database.

**Examples:**

```typescript
const tables = await db.tables();
console.log("Tables:", tables.join(", "));
```

#### `tableInfo(table): Promise<TableInfo>`

Get complete schema information for a table.

**Parameters:**
- `table` (string) - Table name

**Returns:** Table schema with columns, indexes, primary key

**Throws:** `[8401]` if table doesn't exist

**Examples:**

```typescript
const info = await db.tableInfo("users");
console.log(`Table: ${info.name}`);
console.log(`Primary key: ${info.primaryKey.join(", ")}`);
for (const col of info.columns) {
  console.log(`  ${col.name}: ${col.type}${col.nullable ? "" : " NOT NULL"}`);
}
```

#### `tableExists(table): Promise<boolean>`

Check if a table exists.

**Parameters:**
- `table` (string) - Table name

**Returns:** True if table exists

**Examples:**

```typescript
if (!await db.tableExists("users")) {
  await db.execute(`
    CREATE TABLE users (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      name TEXT NOT NULL
    )
  `);
}
```

#### `stream<T>(sql, params?, batchSize?): AsyncIterable<T[]>`

Stream query results in batches.

**Parameters:**
- `sql` (string) - SQL SELECT statement
- `params` (unknown[], optional) - Parameter values
- `batchSize` (number, optional) - Rows per batch (default: 100)

**Returns:** Async iterable of row batches

**Examples:**

```typescript
interface LogEntry {
  id: number;
  timestamp: number;
  message: string;
}

// Stream large result set
for await (const batch of db.stream<LogEntry>(
  "SELECT * FROM logs WHERE level = ?",
  ["ERROR"],
  50 // 50 rows per batch
)) {
  console.log(`Processing ${batch.length} log entries...`);
  for (const log of batch) {
    await processLog(log);
  }
}

// Export large dataset
let total = 0;
for await (const batch of db.stream("SELECT * FROM events", [], 1000)) {
  await writeToFile(batch);
  total += batch.length;
  console.log(`Exported ${total} events so far...`);
}
```

#### `migrate(migrations): Promise<MigrationStatus>`

Apply pending database migrations.

**Parameters:**
- `migrations` (Migration[]) - Array of migration definitions

**Returns:** Migration status after applying

**Throws:**
- `[8413]` if a migration fails
- `[8413]` if versions are not sequential

**Examples:**

```typescript
const migrations = [
  {
    version: 1,
    name: "create_users",
    upSql: "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)",
    downSql: "DROP TABLE users"
  },
  {
    version: 2,
    name: "add_email",
    upSql: "ALTER TABLE users ADD COLUMN email TEXT",
    downSql: "ALTER TABLE users DROP COLUMN email"
  }
];

const status = await db.migrate(migrations);
console.log(`Migrated to version ${status.currentVersion}`);
```

#### `migrationStatus(): Promise<MigrationStatus>`

Get current migration status.

**Returns:** Current version, applied migrations, pending migrations

**Examples:**

```typescript
const status = await db.migrationStatus();
console.log(`Current version: ${status.currentVersion}`);
console.log(`Applied: ${status.applied.map(m => m.name).join(", ")}`);
console.log(`Pending: ${status.pending.join(", ")}`);
```

#### `migrateDown(targetVersion?): Promise<MigrationStatus>`

Rollback migrations to a target version.

**Parameters:**
- `targetVersion` (number, optional) - Version to rollback to (default: 0)

**Returns:** Migration status after rollback

**Throws:**
- `[8413]` if any migration lacks downSql
- `[8413]` if rollback fails

**Examples:**

```typescript
// Rollback to version 1
await db.migrateDown(1);

// Rollback all migrations
await db.migrateDown(0);
```

#### `vacuum(): Promise<void>`

Vacuum the database to reclaim unused space.

**Examples:**

```typescript
await db.vacuum();
```

#### `close(): Promise<void>`

Close the database connection.

**Examples:**

```typescript
const db = await open("myapp");
try {
  // ... use database ...
} finally {
  await db.close();
}
```

### PreparedStatement Methods

#### `query<T>(params?): Promise<QueryResult<T>>`

Execute prepared statement as a SELECT query.

#### `execute(params?): Promise<ExecuteResult>`

Execute prepared statement as INSERT/UPDATE/DELETE.

#### `finalize(): Promise<void>`

Finalize the prepared statement and free resources.

## Usage Examples

### Application State Persistence

```typescript
const db = await open("appdata");

await db.execute(`
  CREATE TABLE IF NOT EXISTS app_state (
    key TEXT PRIMARY KEY,
    value TEXT,
    updated_at INTEGER
  )
`);

async function saveState(key: string, value: unknown): Promise<void> {
  await db.execute(
    "INSERT OR REPLACE INTO app_state (key, value, updated_at) VALUES (?, ?, ?)",
    [key, JSON.stringify(value), Date.now()]
  );
}

async function loadState<T>(key: string): Promise<T | null> {
  const row = await db.queryRow<{ value: string }>(
    "SELECT value FROM app_state WHERE key = ?",
    [key]
  );
  return row ? JSON.parse(row.value) : null;
}
```

### User Management

```typescript
await db.execute(`
  CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at INTEGER NOT NULL
  )
`);

await db.execute("CREATE INDEX idx_users_username ON users(username)");
await db.execute("CREATE INDEX idx_users_email ON users(email)");

async function registerUser(username: string, email: string, passwordHash: string) {
  const result = await db.execute(
    "INSERT INTO users (username, email, password_hash, created_at) VALUES (?, ?, ?, ?)",
    [username, email, passwordHash, Date.now()]
  );
  return result.lastInsertRowid;
}

async function authenticateUser(username: string) {
  return await db.queryRow<User>(
    "SELECT * FROM users WHERE username = ?",
    [username]
  );
}
```

### Analytics Logging

```typescript
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

await analyticsDb.execute("CREATE INDEX idx_events_type ON events(event_type)");

// Batch logging with transactions
async function logEvents(events: Event[]): Promise<void> {
  await analyticsDb.transaction(async () => {
    for (const event of events) {
      await analyticsDb.execute(
        "INSERT INTO events (event_type, user_id, data, timestamp) VALUES (?, ?, ?, ?)",
        [event.type, event.userId, JSON.stringify(event.data), Date.now()]
      );
    }
  });
}
```

### Caching with Expiration

```typescript
const cacheDb = await open("cache");

await cacheDb.execute(`
  CREATE TABLE IF NOT EXISTS cache (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    expires_at INTEGER NOT NULL
  )
`);

async function cacheSet(key: string, value: unknown, ttlMs: number): Promise<void> {
  await cacheDb.execute(
    "INSERT OR REPLACE INTO cache (key, value, expires_at) VALUES (?, ?, ?)",
    [key, JSON.stringify(value), Date.now() + ttlMs]
  );
}

async function cacheGet<T>(key: string): Promise<T | null> {
  const row = await cacheDb.queryRow<{ value: string; expires_at: number }>(
    "SELECT value, expires_at FROM cache WHERE key = ?",
    [key]
  );

  if (!row || Date.now() > row.expires_at) {
    if (row) await cacheDb.execute("DELETE FROM cache WHERE key = ?", [key]);
    return null;
  }

  return JSON.parse(row.value);
}
```

### Batch Data Export

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
```

## Best Practices

### ✅ Do: Use Transactions for Bulk Operations

Transactions are ~1000x faster for bulk inserts/updates:

```typescript
// Good - use transaction
await db.transaction(async () => {
  for (const user of users) {
    await db.execute("INSERT INTO users ...", [user.name, user.email]);
  }
}); // ~10,000 inserts/sec

// Bad - individual inserts without transaction
for (const user of users) {
  await db.execute("INSERT INTO users ...", [user.name, user.email]);
} // ~10 inserts/sec (1000x slower!)
```

### ✅ Do: Use Prepared Statements for Repeated Queries

```typescript
// Good - prepare once, execute many times
const stmt = await db.prepare("INSERT INTO logs (message) VALUES (?)");
try {
  for (const log of logs) {
    await stmt.execute([log.message]);
  }
} finally {
  await stmt.finalize();
}

// Bad - parse SQL on every call
for (const log of logs) {
  await db.execute("INSERT INTO logs (message) VALUES (?)", [log.message]);
}
```

### ✅ Do: Use Parameterized Queries

```typescript
// Good - parameters prevent SQL injection
await db.query("SELECT * FROM users WHERE name = ?", [userName]);

// Bad - concatenation allows SQL injection
await db.query(`SELECT * FROM users WHERE name = '${userName}'`);
```

### ✅ Do: Create Indexes on Queried Columns

```typescript
// Good - create indexes for WHERE clauses
await db.execute("CREATE INDEX idx_users_email ON users(email)");
await db.query("SELECT * FROM users WHERE email = ?", [email]); // Fast!

// Bad - no index on queried column
await db.query("SELECT * FROM users WHERE email = ?", [email]); // Slow table scan
```

### ✅ Do: Use Streaming for Large Result Sets

```typescript
// Good - stream large datasets
for await (const batch of db.stream("SELECT * FROM events", [], 1000)) {
  await processBatch(batch); // Memory efficient
}

// Bad - load all rows into memory
const result = await db.query("SELECT * FROM events"); // May run out of memory
```

### ✅ Do: Close Databases When Done

```typescript
// Good - always close
const db = await open("myapp");
try {
  await db.execute("...");
} finally {
  await db.close(); // Free resources
}

// Bad - never closes (resource leak)
const db = await open("myapp");
await db.execute("...");
```

### ✅ Do: Use Type Safety

```typescript
// Good - typed results
interface User { id: number; name: string; email: string; }
const users = await db.query<User>("SELECT * FROM users");
for (const user of users.rows) {
  console.log(user.name); // Type-safe
}

// Bad - untyped results
const users = await db.query("SELECT * FROM users");
for (const user of users.rows) {
  console.log(user.name); // No type checking
}
```

## Common Pitfalls

### ❌ Don't: Forget to Use Transactions for Bulk Operations

Without transactions, bulk inserts are extremely slow:

```typescript
// Wrong - 1000x slower
for (const user of users) {
  await db.execute("INSERT INTO users ...", [user.name]);
}

// Correct - wrap in transaction
await db.transaction(async () => {
  for (const user of users) {
    await db.execute("INSERT INTO users ...", [user.name]);
  }
});
```

### ❌ Don't: Concatenate User Input into SQL

SQL injection vulnerability:

```typescript
// Wrong - SQL injection!
const name = "'; DROP TABLE users; --";
await db.query(`SELECT * FROM users WHERE name = '${name}'`);

// Correct - use parameters
await db.query("SELECT * FROM users WHERE name = ?", [name]);
```

### ❌ Don't: Load Large Result Sets Without Streaming

Memory issues with large datasets:

```typescript
// Wrong - loads all 1M rows into memory
const result = await db.query("SELECT * FROM events"); // Out of memory!

// Correct - stream in batches
for await (const batch of db.stream("SELECT * FROM events", [], 100)) {
  await processBatch(batch);
}
```

### ❌ Don't: Forget to Finalize Prepared Statements

Resource leak:

```typescript
// Wrong - never finalized (leak)
const stmt = await db.prepare("INSERT INTO logs ...");
await stmt.execute([message]);
// Missing: await stmt.finalize();

// Correct - always finalize
const stmt = await db.prepare("INSERT INTO logs ...");
try {
  await stmt.execute([message]);
} finally {
  await stmt.finalize();
}
```

### ❌ Don't: Ignore Indexes on Large Tables

Slow queries without indexes:

```typescript
// Wrong - no index (slow table scan)
await db.query("SELECT * FROM users WHERE email = ?", [email]); // Slow!

// Correct - create index first
await db.execute("CREATE INDEX idx_users_email ON users(email)");
await db.query("SELECT * FROM users WHERE email = ?", [email]); // Fast!
```

## Error Handling

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

### Handling Errors

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
  } else if (err.message.includes("[8411]")) {
    console.error("Database is busy/locked");
  } else {
    console.error("Database error:", err);
  }
}
```

## Platform Support

| Platform | Supported | Database Location |
|----------|-----------|-------------------|
| macOS    | ✅ | `~/Library/Application Support/.forge/<app-id>/databases/` |
| Linux    | ✅ | `~/.local/share/.forge/<app-id>/databases/` |
| Windows  | ✅ | `%APPDATA%\.forge\<app-id>\databases\` |

## Permissions

No special permissions required. Database files are created in the app's data directory, which the app has full access to by default.

For security:
- Apps cannot access other apps' databases
- File permissions are controlled by the operating system
- Always use parameterized queries to prevent SQL injection

## See Also

- [ext_storage](./ext-storage) - Simple key-value storage (simpler alternative)
- [ext_crypto](./ext-crypto) - Encryption for sensitive database fields
- [ext_fs](./ext-fs) - File operations for database backup/restore
