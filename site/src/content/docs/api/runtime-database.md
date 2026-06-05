---
title: "runtime:database"
description: "Full-featured SQLite database access for Forge applications"
slug: api/runtime-database
---

Full-featured SQLite database access for Forge applications with query execution, transactions, prepared statements, result streaming, and schema migrations.

> **Implementation**: TypeScript types are auto-generated from Rust via [forge-weld](/docs/crates/forge-weld). See [ext_database](/docs/crates/ext-database) for implementation details.

## Features

### Connection Management
- Open multiple named databases per application
- Automatic directory creation and database initialization
- WAL mode enabled by default for better concurrency
- Foreign key constraints enabled by default
- Configurable busy timeout and read-only mode

### Query Execution
- Parameterized queries to prevent SQL injection
- Batch execution with transaction support
- Single-row and single-value query helpers
- Automatic type conversion from SQLite to JavaScript

### Transactions
- BEGIN/COMMIT/ROLLBACK transaction control
- Savepoints for nested transaction-like behavior
- Helper function for automatic rollback on error
- Three transaction modes: deferred, immediate, exclusive

### Prepared Statements
- Compile SQL once, execute multiple times
- Improved performance for repeated queries
- Automatic parameter binding

### Result Streaming
- Stream large result sets in batches
- Configurable batch size
- Async iteration support
- Automatic cursor management

### Schema Operations
- List all tables in database
- Inspect table schema (columns, types, constraints, indexes)
- Check table existence

### Migrations
- Versioned schema migrations (up/down)
- Automatic migration tracking
- Rollback support for failed migrations
- Migration status inspection

## Database Location

Databases are stored in the app's data directory:

| Platform | Location |
|----------|----------|
| macOS | `~/Library/Application Support/.forge/<app-id>/databases/<name>.db` |
| Linux | `~/.local/share/.forge/<app-id>/databases/<name>.db` |
| Windows | `%APPDATA%\.forge\<app-id>\databases\<name>.db` |

## Import

```typescript
import {
  open,
  list,
  exists,
  remove,
  path,
  type Database,
  type Migration,
  type QueryResult,
  type ExecuteResult
} from "runtime:database";
```

## Module-Level Functions

### open(name, opts?)

Open a database by name. Creates the database file and directory if they don't exist.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `name` | `string` | Database name (without .db extension) |
| `opts` | `OpenOptions` | Optional configuration |

**OpenOptions:**

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `create` | `boolean` | `true` | Create database if it doesn't exist |
| `readonly` | `boolean` | `false` | Open in read-only mode |
| `walMode` | `boolean` | `true` | Enable WAL mode for better concurrency |
| `busyTimeoutMs` | `number` | `5000` | Busy timeout in milliseconds |
| `foreignKeys` | `boolean` | `true` | Enable foreign key constraints |

**Returns:** `Promise<Database>` - Database connection handle

**Throws:**
- Error [8401] if database doesn't exist and `create: false`
- Error [8408] if permission denied
- Error [8412] if I/O error (disk full, etc.)

**Example:**

```typescript
import * as db from "runtime:database";

// Open with default options
const database = await db.open("myapp");

// Create table
await database.execute(`
  CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    email TEXT UNIQUE
  )
`);

// Insert and query data
const result = await database.execute(
  "INSERT INTO users (name, email) VALUES (?, ?)",
  ["Alice", "alice@example.com"]
);
console.log("New user ID:", result.lastInsertRowid);

await database.close();
```

```typescript
// Open in read-only mode
const readDb = await db.open("reports", { readonly: true });

// Custom timeout
const busyDb = await db.open("shared", { busyTimeoutMs: 10000 });

// Fail if doesn't exist
const existingDb = await db.open("must-exist", { create: false });
```

---

### list()

List all databases for the current app.

**Returns:** `Promise<DatabaseInfo[]>` - Array of database information

**Example:**

```typescript
const databases = await list();
for (const db of databases) {
  console.log(`${db.name}: ${(db.sizeBytes / 1024).toFixed(2)} KB`);
  console.log(`  Tables: ${db.tables.join(", ")}`);
}
```

---

### exists(name)

Check if a database exists.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `name` | `string` | Database name |

**Returns:** `Promise<boolean>` - True if the database exists

**Example:**

```typescript
if (!await exists("users")) {
  const db = await open("users");
  await db.execute("CREATE TABLE users (...)");
  await db.close();
}
```

---

### remove(name)

Delete a database permanently. The database must be closed first.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `name` | `string` | Database name |

**Returns:** `Promise<boolean>` - True if deleted, false if didn't exist

**Throws:**
- Error [8411] if database is currently open
- Error [8408] if permission denied

**Example:**

```typescript
const db = await open("temp");
// ... use database ...
await db.close(); // Must close first
await remove("temp"); // Now can delete
```

---

### path(name)

Get the full filesystem path for a database.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `name` | `string` | Database name |

**Returns:** `string` - Full path to database file

**Example:**

```typescript
const dbPath = path("myapp");
// macOS: ~/Library/Application Support/.forge/com.example.app/databases/myapp.db
```

## Database Methods

### Query Methods

#### db.query(sql, params?)

Execute a SELECT query and return all rows.

**Type Parameter:** `T` - Expected row type (default: `Record<string, unknown>`)

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `sql` | `string` | SQL SELECT statement |
| `params` | `unknown[]` | Parameter values to bind |

**Returns:** `Promise<QueryResult<T>>` - Query result with typed rows

**Example:**

```typescript
interface User {
  id: number;
  name: string;
  email: string;
}

// Query with type safety
const result = await db.query<User>("SELECT * FROM users WHERE active = ?", [1]);
for (const user of result.rows) {
  console.log(user.name, user.email);
}

// Access metadata
console.log("Columns:", result.columns.map(c => c.name));
console.log("Rows affected:", result.rowsAffected);
```

---

#### db.execute(sql, params?)

Execute an INSERT, UPDATE, DELETE, or DDL statement.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `sql` | `string` | SQL statement |
| `params` | `unknown[]` | Parameter values to bind |

**Returns:** `Promise<ExecuteResult>` - Result with rowsAffected and lastInsertRowid

**Throws:**
- Error [8403] if SQL syntax is invalid
- Error [8404] if constraint violated (UNIQUE, FOREIGN KEY, etc.)
- Error [8414] if parameter count doesn't match placeholders

**Example:**

```typescript
// INSERT
const result = await db.execute(
  "INSERT INTO users (name, email) VALUES (?, ?)",
  ["Alice", "alice@example.com"]
);
console.log("New ID:", result.lastInsertRowid);

// UPDATE
const updated = await db.execute(
  "UPDATE users SET active = ? WHERE last_login < ?",
  [0, Date.now() - 90 * 24 * 60 * 60 * 1000]
);
console.log("Deactivated:", updated.rowsAffected);

// DELETE
const deleted = await db.execute("DELETE FROM sessions WHERE expires_at < ?", [Date.now()]);
console.log("Cleaned:", deleted.rowsAffected);

// DDL
await db.execute("CREATE INDEX idx_users_email ON users(email)");
```

---

#### db.executeBatch(statements, opts?)

Execute multiple SQL statements in a batch.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `statements` | `string[]` | Array of SQL statements |
| `opts` | `BatchOptions` | Optional batch configuration |

**BatchOptions:**

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `transaction` | `boolean` | `true` | Run in a transaction (all-or-nothing) |
| `stopOnError` | `boolean` | `true` | Stop on first error |

**Returns:** `Promise<BatchResult>` - Batch result with statistics

**Example:**

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

// No transaction (faster)
await db.executeBatch(statements, { transaction: false });
```

---

#### db.queryRow(sql, params?)

Execute a query and return only the first row.

**Returns:** `Promise<T | null>` - First row or null if no results

**Example:**

```typescript
interface User { id: number; name: string; email: string; }

const user = await db.queryRow<User>(
  "SELECT * FROM users WHERE id = ?",
  [42]
);
if (user) {
  console.log("Found:", user.name);
}
```

---

#### db.queryValue(sql, params?)

Execute a query and return the first column of the first row.

**Returns:** `Promise<T | null>` - Single value or null if no results

**Example:**

```typescript
// Count
const count = await db.queryValue<number>(
  "SELECT COUNT(*) FROM users WHERE active = ?",
  [1]
);
console.log("Active users:", count);

// Sum
const total = await db.queryValue<number>(
  "SELECT SUM(amount) FROM orders WHERE user_id = ?",
  [userId]
);
```

### Transaction Methods

#### db.begin(mode?)

Begin a database transaction.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `mode` | `"deferred" \| "immediate" \| "exclusive"` | Transaction mode |

**Throws:** Error [8407] if already in a transaction

**Example:**

```typescript
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

---

#### db.commit()

Commit the current transaction. Makes all changes permanent.

**Throws:** Error [8407] if not in a transaction

---

#### db.rollback()

Rollback the current transaction. Discards all changes.

**Throws:** Error [8407] if not in a transaction

---

#### db.transaction(fn)

Execute a function within a transaction with automatic rollback on error. **This is the recommended way to use transactions.**

**Type Parameter:** `T` - Return type of the function

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `fn` | `() => Promise<T>` | Async function to execute |

**Returns:** `Promise<T>` - Value returned by the function

**Example:**

```typescript
// Transfer money
await db.transaction(async () => {
  await db.execute("UPDATE accounts SET balance = balance - ? WHERE id = ?", [100, fromId]);
  await db.execute("UPDATE accounts SET balance = balance + ? WHERE id = ?", [100, toId]);
});
// Automatically committed if successful, rolled back on error

// Bulk insert (~1000x faster than individual inserts)
await db.transaction(async () => {
  for (const user of users) {
    await db.execute("INSERT INTO users (name, email) VALUES (?, ?)", [user.name, user.email]);
  }
});
```

---

#### Savepoints

Savepoints allow nested transaction-like behavior within a transaction.

```typescript
await db.begin();
await db.execute("INSERT INTO users (name) VALUES (?)", ["Alice"]);

await db.savepoint("before_bob");
await db.execute("INSERT INTO users (name) VALUES (?)", ["Bob"]);

// Oops, rollback Bob's insert
await db.rollbackTo("before_bob");

await db.execute("INSERT INTO users (name) VALUES (?)", ["Charlie"]);
await db.commit(); // Alice and Charlie inserted, Bob was rolled back
```

### Prepared Statements

#### db.prepare(sql)

Prepare a SQL statement for repeated execution.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `sql` | `string` | SQL statement with ? placeholders |

**Returns:** `Promise<PreparedStatement>` - Prepared statement handle

**Example:**

```typescript
const stmt = await db.prepare(
  "INSERT INTO events (type, data, timestamp) VALUES (?, ?, ?)"
);

try {
  // Execute multiple times efficiently
  for (let i = 0; i < 100; i++) {
    await stmt.execute([`event_${i}`, JSON.stringify({ value: i }), Date.now()]);
  }
} finally {
  await stmt.finalize(); // Always finalize to free resources
}
```

#### PreparedStatement Interface

```typescript
interface PreparedStatement {
  readonly id: string;
  readonly sql: string;
  readonly parameterCount: number;

  query<T>(params?: unknown[]): Promise<QueryResult<T>>;
  execute(params?: unknown[]): Promise<ExecuteResult>;
  finalize(): Promise<void>;
}
```

### Schema Methods

#### db.tables()

List all table names in the database.

**Returns:** `Promise<string[]>` - Array of table names

**Example:**

```typescript
const tables = await db.tables();
console.log("Tables:", tables.join(", "));
```

---

#### db.tableInfo(table)

Get complete schema information for a table.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `table` | `string` | Table name |

**Returns:** `Promise<TableInfo>` - Table schema information

**Throws:** Error [8401] if table does not exist

**Example:**

```typescript
const info = await db.tableInfo("users");
console.log(`Table: ${info.name}`);
console.log(`Primary key: ${info.primaryKey.join(", ")}`);
console.log("Columns:");
for (const col of info.columns) {
  console.log(`  ${col.name}: ${col.type}${col.nullable ? "" : " NOT NULL"}`);
}
console.log("Indexes:");
for (const idx of info.indexes) {
  console.log(`  ${idx.name} on (${idx.columns.join(", ")})`);
}
```

---

#### db.tableExists(table)

Check if a table exists in the database.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `table` | `string` | Table name |

**Returns:** `Promise<boolean>` - True if table exists

**Example:**

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

### Streaming

#### db.stream(sql, params?, batchSize?)

Stream query results in batches. Use for large result sets to avoid memory issues.

**Type Parameter:** `T` - Expected row type

**Parameters:**

| Name | Type | Default | Description |
|------|------|---------|-------------|
| `sql` | `string` | - | SQL SELECT statement |
| `params` | `unknown[]` | `[]` | Parameter values |
| `batchSize` | `number` | `100` | Rows per batch |

**Returns:** `AsyncIterable<T[]>` - Async iterable of row batches

**Example:**

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

### Migrations

#### db.migrate(migrations)

Apply pending database migrations.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `migrations` | `Migration[]` | Array of migration definitions |

**Returns:** `Promise<MigrationStatus>` - Status after applying migrations

**Throws:**
- Error [8413] if a migration fails
- Error [8413] if versions are not sequential

**Example:**

```typescript
const migrations: Migration[] = [
  {
    version: 1,
    name: "create_users",
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
    name: "add_active_column",
    upSql: "ALTER TABLE users ADD COLUMN active INTEGER DEFAULT 1",
    downSql: "ALTER TABLE users DROP COLUMN active"
  },
  {
    version: 3,
    name: "create_sessions",
    upSql: `
      CREATE TABLE sessions (
        id TEXT PRIMARY KEY,
        user_id INTEGER NOT NULL REFERENCES users(id),
        expires_at INTEGER NOT NULL
      );
      CREATE INDEX idx_sessions_user ON sessions(user_id);
    `,
    downSql: "DROP TABLE sessions"
  }
];

const status = await db.migrate(migrations);
console.log(`Migrated to version ${status.currentVersion}`);
```

---

#### db.migrationStatus()

Get current migration status.

**Returns:** `Promise<MigrationStatus>` - Current status

**Example:**

```typescript
const status = await db.migrationStatus();
console.log(`Current version: ${status.currentVersion}`);
console.log(`Applied: ${status.applied.map(m => m.name).join(", ")}`);
if (status.pending.length > 0) {
  console.log(`Pending: ${status.pending.join(", ")}`);
}
```

---

#### db.migrateDown(targetVersion?)

Rollback migrations to a target version.

**Parameters:**

| Name | Type | Default | Description |
|------|------|---------|-------------|
| `targetVersion` | `number` | `0` | Version to rollback to |

**Returns:** `Promise<MigrationStatus>` - Status after rollback

**Throws:**
- Error [8413] if migration lacks downSql
- Error [8413] if rollback fails

**Example:**

```typescript
// Rollback to version 1 (undoes version 2+)
await db.migrateDown(1);

// Rollback all migrations
await db.migrateDown(0);
```

### Maintenance

#### db.vacuum()

Vacuum the database to reclaim unused space.

**Example:**

```typescript
await db.vacuum();
```

---

#### db.close()

Close the database connection. Always call when done.

**Example:**

```typescript
const db = await open("myapp");
try {
  // ... use database ...
} finally {
  await db.close();
}
```

## Type Definitions

```typescript
interface OpenOptions {
  create?: boolean;      // Default: true
  readonly?: boolean;    // Default: false
  walMode?: boolean;     // Default: true
  busyTimeoutMs?: number; // Default: 5000
  foreignKeys?: boolean; // Default: true
}

interface DatabaseInfo {
  name: string;
  path: string;
  sizeBytes: number;
  tables: string[];
  readonly: boolean;
}

interface ColumnInfo {
  name: string;
  type: string;        // TEXT, INTEGER, REAL, BLOB, NULL
  nullable: boolean;
}

interface QueryResult<T = Record<string, unknown>> {
  columns: ColumnInfo[];
  rows: T[];
  rowsAffected: number;
  lastInsertRowid?: number;
}

interface ExecuteResult {
  rowsAffected: number;
  lastInsertRowid?: number;
}

interface BatchResult {
  totalRowsAffected: number;
  statementCount: number;
  errors: Array<{ index: number; message: string }>;
}

interface TableColumn {
  name: string;
  type: string;
  nullable: boolean;
  defaultValue?: string;
  primaryKey: boolean;
}

interface IndexInfo {
  name: string;
  columns: string[];
  unique: boolean;
}

interface TableInfo {
  name: string;
  columns: TableColumn[];
  primaryKey: string[];
  indexes: IndexInfo[];
}

interface Migration {
  version: number;
  name: string;
  upSql: string;
  downSql?: string;
}

interface AppliedMigration {
  version: number;
  name: string;
  appliedAt: number;
}

interface MigrationStatus {
  currentVersion: number;
  pending: string[];
  applied: AppliedMigration[];
}
```

## Error Codes

| Code | Name | Description |
|------|------|-------------|
| 8400 | Generic | Generic database error |
| 8401 | NotFound | Database/table/row not found |
| 8402 | AlreadyExists | Database/table already exists |
| 8403 | SyntaxError | SQL syntax error |
| 8404 | Constraint | Constraint violation (UNIQUE, CHECK, FOREIGN KEY, NOT NULL) |
| 8405 | TypeMismatch | Wrong parameter type |
| 8406 | InvalidHandle | Invalid database handle |
| 8407 | Transaction | Transaction error (already in/not in transaction) |
| 8408 | PermissionDenied | Permission denied |
| 8409 | TooManyConnections | Too many open connections |
| 8410 | PreparedStatement | Prepared statement error |
| 8411 | Busy | Database is busy/locked |
| 8412 | IoError | I/O error (disk full, permission denied) |
| 8413 | Migration | Migration error (invalid version, failed migration) |
| 8414 | InvalidParameter | Invalid parameter (wrong count, null where not allowed) |
| 8415 | Stream | Stream error (invalid stream, already closed) |

## Error Handling

```typescript
import * as db from "runtime:database";

try {
  const database = await db.open("myapp");

  await database.execute(
    "INSERT INTO users (name, email) VALUES (?, ?)",
    ["Alice", "alice@example.com"]
  );
} catch (error) {
  if (error.code === 8404) {
    console.error("Constraint violated:", error.message);
  } else if (error.code === 8403) {
    console.error("SQL syntax error:", error.message);
  } else if (error.code === 8411) {
    console.error("Database is busy, try again later");
  } else {
    throw error;
  }
}
```

## Lifecycle Hooks

Database operations support the standard extensibility hooks.

### onBefore(opName, callback)

```typescript
import { onBefore } from "runtime:database";

onBefore("execute", (args) => {
  console.log("Executing SQL:", args);
});
```

### onAfter(opName, callback)

```typescript
import { onAfter } from "runtime:database";

onAfter("query", (result, args) => {
  console.log(`Query returned ${result.rows.length} rows`);
});
```

### onError(opName, callback)

```typescript
import { onError } from "runtime:database";

onError("execute", (error, args) => {
  console.error("SQL execution failed:", error.message);
});
```

**Available operation names:** `"open"`, `"close"`, `"list"`, `"delete"`, `"exists"`, `"path"`, `"vacuum"`, `"query"`, `"execute"`, `"executeBatch"`, `"queryRow"`, `"queryValue"`, `"prepare"`, `"stmtQuery"`, `"stmtExecute"`, `"stmtFinalize"`, `"begin"`, `"commit"`, `"rollback"`, `"savepoint"`, `"release"`, `"rollbackTo"`, `"tables"`, `"tableInfo"`, `"tableExists"`, `"streamOpen"`, `"streamNext"`, `"streamClose"`, `"migrate"`, `"migrationStatus"`, `"migrateDown"`

## Performance Tips

### Use Transactions for Bulk Operations

Transactions are ~1000x faster for bulk inserts/updates:

```typescript
// Slow: Individual inserts (1000 inserts = ~10 seconds)
for (const user of users) {
  await db.execute("INSERT INTO users (name) VALUES (?)", [user.name]);
}

// Fast: Transaction (1000 inserts = ~10 milliseconds)
await db.transaction(async () => {
  for (const user of users) {
    await db.execute("INSERT INTO users (name) VALUES (?)", [user.name]);
  }
});
```

### Use Prepared Statements for Repeated Queries

```typescript
const stmt = await db.prepare("INSERT INTO metrics (name, value) VALUES (?, ?)");
try {
  for (const metric of metrics) {
    await stmt.execute([metric.name, metric.value]);
  }
} finally {
  await stmt.finalize();
}
```

### Use Streaming for Large Result Sets

```typescript
// Avoid loading millions of rows into memory
for await (const batch of db.stream("SELECT * FROM large_table", [], 1000)) {
  await processBatch(batch);
}
```

### Enable WAL Mode (Default)

WAL mode is enabled by default and provides better concurrent access. Don't disable it unless you have a specific reason.

## Complete Example

```typescript
import * as db from "runtime:database";

// Migrations
const migrations: db.Migration[] = [
  {
    version: 1,
    name: "initial_schema",
    upSql: `
      CREATE TABLE users (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        email TEXT UNIQUE NOT NULL,
        active INTEGER DEFAULT 1,
        created_at INTEGER NOT NULL
      );
      CREATE INDEX idx_users_email ON users(email);
      CREATE INDEX idx_users_active ON users(active);

      CREATE TABLE tasks (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
        title TEXT NOT NULL,
        completed INTEGER DEFAULT 0,
        due_date INTEGER,
        created_at INTEGER NOT NULL
      );
      CREATE INDEX idx_tasks_user ON tasks(user_id);
      CREATE INDEX idx_tasks_due ON tasks(due_date);
    `,
    downSql: "DROP TABLE tasks; DROP TABLE users;"
  }
];

// User interface
interface User {
  id: number;
  name: string;
  email: string;
  active: number;
  created_at: number;
}

interface Task {
  id: number;
  user_id: number;
  title: string;
  completed: number;
  due_date: number | null;
  created_at: number;
}

async function main() {
  // Open database and run migrations
  const database = await db.open("todo-app");
  await database.migrate(migrations);

  // Create a user
  const userResult = await database.execute(
    "INSERT INTO users (name, email, created_at) VALUES (?, ?, ?)",
    ["Alice", "alice@example.com", Date.now()]
  );
  const userId = userResult.lastInsertRowid!;

  // Add tasks in a transaction
  await database.transaction(async () => {
    const tasks = [
      { title: "Buy groceries", dueDate: Date.now() + 86400000 },
      { title: "Write report", dueDate: Date.now() + 172800000 },
      { title: "Call dentist", dueDate: null },
    ];

    for (const task of tasks) {
      await database.execute(
        "INSERT INTO tasks (user_id, title, due_date, created_at) VALUES (?, ?, ?, ?)",
        [userId, task.title, task.dueDate, Date.now()]
      );
    }
  });

  // Query user's tasks
  const tasks = await database.query<Task>(
    "SELECT * FROM tasks WHERE user_id = ? ORDER BY due_date",
    [userId]
  );

  console.log(`${tasks.rows.length} tasks for user ${userId}:`);
  for (const task of tasks.rows) {
    const status = task.completed ? "[x]" : "[ ]";
    const due = task.due_date ? new Date(task.due_date).toLocaleDateString() : "No due date";
    console.log(`${status} ${task.title} - ${due}`);
  }

  // Get aggregate stats
  const totalTasks = await database.queryValue<number>(
    "SELECT COUNT(*) FROM tasks WHERE user_id = ?",
    [userId]
  );
  const completedTasks = await database.queryValue<number>(
    "SELECT COUNT(*) FROM tasks WHERE user_id = ? AND completed = 1",
    [userId]
  );

  console.log(`\nProgress: ${completedTasks}/${totalTasks} tasks completed`);

  // Clean up
  await database.close();
}

main().catch(console.error);
```

## Best Practices

### Always Close Databases

```typescript
const db = await open("myapp");
try {
  // ... use database ...
} finally {
  await db.close();
}
```

### Use Parameterized Queries

Always use `?` placeholders to prevent SQL injection:

```typescript
// GOOD - Safe from SQL injection
await db.query("SELECT * FROM users WHERE email = ?", [email]);

// BAD - Vulnerable to SQL injection
await db.query(`SELECT * FROM users WHERE email = '${email}'`);
```

### Handle Transaction Errors

Use the `transaction()` helper for automatic rollback:

```typescript
// Recommended
await db.transaction(async () => {
  await db.execute("INSERT ...");
  await db.execute("UPDATE ...");
});
// Automatically rolled back if any statement fails
```

### Close Prepared Statements

Always finalize prepared statements when done:

```typescript
const stmt = await db.prepare("SELECT * FROM users WHERE id = ?");
try {
  const user1 = await stmt.query([1]);
  const user2 = await stmt.query([2]);
} finally {
  await stmt.finalize(); // Required to free resources
}
```
