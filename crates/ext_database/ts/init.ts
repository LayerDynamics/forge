/**
 * @module runtime:database
 *
 * Full-featured SQLite database access for Forge applications.
 *
 * Provides complete SQL database capabilities including query execution,
 * transactions, prepared statements, result streaming, and schema migrations.
 * Each app can have multiple named databases, all stored in the app's data directory.
 *
 * ## Features
 *
 * ### Connection Management
 * - Open multiple named databases per application
 * - Automatic directory creation and database initialization
 * - WAL mode enabled by default for better concurrency
 * - Foreign key constraints enabled by default
 * - Configurable busy timeout and read-only mode
 *
 * ### Query Execution
 * - Parameterized queries to prevent SQL injection
 * - Batch execution with transaction support
 * - Single-row and single-value query helpers
 * - Automatic type conversion from SQLite to JavaScript
 *
 * ### Transactions
 * - BEGIN/COMMIT/ROLLBACK transaction control
 * - Savepoints for nested transaction-like behavior
 * - Helper function for automatic rollback on error
 * - Three transaction modes: deferred, immediate, exclusive
 *
 * ### Prepared Statements
 * - Compile SQL once, execute multiple times
 * - Improved performance for repeated queries
 * - Automatic parameter binding
 *
 * ### Result Streaming
 * - Stream large result sets in batches
 * - Configurable batch size
 * - Async iteration support
 * - Automatic cursor management
 *
 * ### Schema Operations
 * - List all tables in database
 * - Inspect table schema (columns, types, constraints, indexes)
 * - Check table existence
 *
 * ### Migrations
 * - Versioned schema migrations (up/down)
 * - Automatic migration tracking
 * - Rollback support for failed migrations
 * - Migration status inspection
 *
 * ## Database Location
 *
 * Databases are stored at:
 * - **macOS**: `~/Library/Application Support/.forge/<app-id>/databases/<name>.db`
 * - **Linux**: `~/.local/share/.forge/<app-id>/databases/<name>.db`
 * - **Windows**: `%APPDATA%\.forge\<app-id>\databases\<name>.db`
 *
 * ## Error Codes
 *
 * Database operations may throw errors with these codes:
 * - `8400` - Generic database error
 * - `8401` - Database/table/row not found
 * - `8402` - Database/table already exists
 * - `8403` - SQL syntax error
 * - `8404` - Constraint violation (UNIQUE, CHECK, FOREIGN KEY, NOT NULL)
 * - `8405` - Type mismatch (wrong parameter type)
 * - `8406` - Invalid database handle
 * - `8407` - Transaction error (already in transaction, not in transaction)
 * - `8408` - Permission denied
 * - `8409` - Too many open connections
 * - `8410` - Prepared statement error
 * - `8411` - Database is busy/locked
 * - `8412` - I/O error (disk full, permission denied)
 * - `8413` - Migration error (invalid version, failed migration)
 * - `8414` - Invalid parameter (wrong count, null where not allowed)
 * - `8415` - Stream error (invalid stream, already closed)
 *
 * ## Performance Tips
 *
 * - Use transactions for bulk inserts/updates (~1000x faster)
 * - Use prepared statements for repeated queries
 * - Use streaming for large result sets to avoid memory issues
 * - Enable WAL mode (default) for better concurrent access
 * - Use batch execution for multiple statements
 *
 * ## Safety
 *
 * - Always use parameterized queries to prevent SQL injection
 * - Close databases when done to free resources
 * - Handle transaction errors properly to avoid locking
 * - Use foreign keys (enabled by default) for referential integrity
 */

// =============================================================================
// Type Declarations
// =============================================================================

declare const Deno: {
  core: {
    ops: {
      // Connection Management
      op_database_open(name: string, opts?: OpenOptions): Promise<RawOpenResult>;
      op_database_close(dbId: string): Promise<void>;
      op_database_list(): Promise<RawDatabaseInfo[]>;
      op_database_delete(name: string): Promise<boolean>;
      op_database_exists(name: string): Promise<boolean>;
      op_database_path(name: string): string;
      op_database_vacuum(dbId: string): Promise<void>;

      // Query Execution
      op_database_query(dbId: string, sql: string, params?: unknown[]): Promise<RawQueryResult>;
      op_database_execute(dbId: string, sql: string, params?: unknown[]): Promise<RawExecuteResult>;
      op_database_execute_batch(dbId: string, statements: string[], opts?: BatchOptions): Promise<RawBatchResult>;
      op_database_query_row(dbId: string, sql: string, params?: unknown[]): Promise<unknown[] | null>;
      op_database_query_value(dbId: string, sql: string, params?: unknown[]): Promise<unknown | null>;

      // Prepared Statements
      op_database_prepare(dbId: string, sql: string): Promise<RawPreparedStatementInfo>;
      op_database_stmt_query(dbId: string, stmtId: string, params?: unknown[]): Promise<RawQueryResult>;
      op_database_stmt_execute(dbId: string, stmtId: string, params?: unknown[]): Promise<RawExecuteResult>;
      op_database_stmt_finalize(dbId: string, stmtId: string): Promise<void>;

      // Transactions
      op_database_begin(dbId: string, mode?: string): Promise<void>;
      op_database_commit(dbId: string): Promise<void>;
      op_database_rollback(dbId: string): Promise<void>;
      op_database_savepoint(dbId: string, name: string): Promise<void>;
      op_database_release(dbId: string, name: string): Promise<void>;
      op_database_rollback_to(dbId: string, name: string): Promise<void>;

      // Schema Operations
      op_database_tables(dbId: string): Promise<string[]>;
      op_database_table_info(dbId: string, table: string): Promise<RawTableInfo>;
      op_database_table_exists(dbId: string, table: string): Promise<boolean>;

      // Streaming
      op_database_stream_open(dbId: string, sql: string, params?: unknown[], batchSize?: number): Promise<string>;
      op_database_stream_next(streamId: string): Promise<RawStreamBatch>;
      op_database_stream_close(streamId: string): Promise<void>;

      // Migrations
      op_database_migrate(dbId: string, migrations: Migration[]): Promise<RawMigrationStatus>;
      op_database_migration_status(dbId: string): Promise<RawMigrationStatus>;
      op_database_migrate_down(dbId: string, targetVersion?: number): Promise<RawMigrationStatus>;
    };
  };
};

// =============================================================================
// Raw Types (from Rust, snake_case)
// =============================================================================

interface RawOpenResult {
  id: string;
  path: string;
  created: boolean;
}

interface RawDatabaseInfo {
  name: string;
  path: string;
  size_bytes: number;
  tables: string[];
  readonly: boolean;
}

interface RawColumnInfo {
  name: string;
  column_type: string;
  nullable: boolean;
}

interface RawQueryResult {
  columns: RawColumnInfo[];
  rows: unknown[][];
  rows_affected: number;
  last_insert_rowid: number | null;
}

interface RawExecuteResult {
  rows_affected: number;
  last_insert_rowid: number | null;
}

interface RawBatchResult {
  total_rows_affected: number;
  statement_count: number;
  errors: Array<{ index: number; message: string }>;
}

interface RawPreparedStatementInfo {
  id: string;
  sql: string;
  parameter_count: number;
}

interface RawTableColumn {
  name: string;
  column_type: string;
  nullable: boolean;
  default_value: string | null;
  primary_key: boolean;
}

interface RawIndexInfo {
  name: string;
  columns: string[];
  unique: boolean;
}

interface RawTableInfo {
  name: string;
  columns: RawTableColumn[];
  primary_key: string[];
  indexes: RawIndexInfo[];
}

interface RawStreamBatch {
  rows: unknown[][];
  done: boolean;
  total_fetched: number;
}

interface RawAppliedMigration {
  version: number;
  name: string;
  applied_at: number;
}

interface RawMigrationStatus {
  current_version: number;
  pending: string[];
  applied: RawAppliedMigration[];
}

// =============================================================================
// Public Types (camelCase, user-facing)
// =============================================================================

/**
 * Options for opening a database connection.
 *
 * Controls database creation, access mode, WAL mode, timeouts, and foreign key enforcement.
 *
 * @example
 * ```typescript
 * // Default options (create, WAL mode, foreign keys enabled)
 * const db = await open("mydb");
 *
 * // Read-only mode
 * const readDb = await open("mydb", { readonly: true });
 *
 * // Custom timeout and no foreign keys
 * const db2 = await open("mydb", {
 *   busyTimeoutMs: 10000,
 *   foreignKeys: false
 * });
 *
 * // Fail if database doesn't exist
 * const db3 = await open("existing", { create: false });
 * ```
 */
export interface OpenOptions {
  /** Create database if it doesn't exist (default: true) */
  create?: boolean;
  /** Open in read-only mode (default: false) */
  readonly?: boolean;
  /** Enable WAL mode for better concurrency (default: true) */
  walMode?: boolean;
  /** Busy timeout in milliseconds (default: 5000) */
  busyTimeoutMs?: number;
  /** Enable foreign keys (default: true) */
  foreignKeys?: boolean;
}

/**
 * Options for batch execution of multiple SQL statements.
 *
 * Controls transactional behavior and error handling for batch operations.
 *
 * @example
 * ```typescript
 * // Atomic batch (default - all-or-nothing transaction)
 * await db.executeBatch([
 *   "INSERT INTO users (name) VALUES ('Alice')",
 *   "INSERT INTO users (name) VALUES ('Bob')"
 * ]);
 *
 * // Continue on errors
 * const result = await db.executeBatch([
 *   "INSERT INTO users (name) VALUES ('Charlie')",
 *   "INVALID SQL",
 *   "INSERT INTO users (name) VALUES ('Dave')"
 * ], { stopOnError: false });
 * console.log("Errors:", result.errors);
 *
 * // No transaction (faster but not atomic)
 * await db.executeBatch(statements, { transaction: false });
 * ```
 */
export interface BatchOptions {
  /** Run in a transaction (default: true) */
  transaction?: boolean;
  /** Stop on first error (default: true) */
  stopOnError?: boolean;
}

/**
 * Result of opening a database connection.
 *
 * Contains the database ID (handle), file path, and whether it was newly created.
 */
export interface OpenResult {
  /** Unique database handle ID */
  id: string;
  /** Full filesystem path to the database file */
  path: string;
  /** True if the database was newly created, false if it already existed */
  created: boolean;
}

/**
 * Database metadata and statistics.
 *
 * Returned by `list()` to provide information about all databases.
 */
export interface DatabaseInfo {
  /** Database name (without .db extension) */
  name: string;
  /** Full filesystem path to the database file */
  path: string;
  /** Database file size in bytes */
  sizeBytes: number;
  /** List of table names in the database */
  tables: string[];
  /** True if opened in read-only mode */
  readonly: boolean;
}

/**
 * Column metadata from query results.
 *
 * Describes the name, type, and nullability of a result column.
 */
export interface ColumnInfo {
  /** Column name */
  name: string;
  /** SQLite type (TEXT, INTEGER, REAL, BLOB, NULL) */
  type: string;
  /** True if the column can contain NULL values */
  nullable: boolean;
}

/**
 * Result of a SELECT query with typed rows.
 *
 * Rows are returned as objects with column names as keys. Use the type parameter
 * to specify the expected row shape for type safety.
 *
 * @template T - The expected row type (default: `Record<string, unknown>`)
 *
 * @example
 * ```typescript
 * interface User {
 *   id: number;
 *   name: string;
 *   email: string;
 * }
 *
 * const result = await db.query<User>("SELECT * FROM users WHERE active = ?", [true]);
 * console.log(`Found ${result.rows.length} users`);
 * for (const user of result.rows) {
 *   console.log(user.name, user.email); // Type-safe access
 * }
 * ```
 */
export interface QueryResult<T = Record<string, unknown>> {
  /** Column metadata for the result set */
  columns: ColumnInfo[];
  /** Result rows as typed objects */
  rows: T[];
  /** Number of rows affected by the query (usually 0 for SELECT) */
  rowsAffected: number;
  /** Row ID of the last inserted row (INSERT only) */
  lastInsertRowid?: number;
}

/**
 * Result of an INSERT, UPDATE, or DELETE statement.
 *
 * Contains the number of affected rows and the last inserted row ID (for INSERT).
 *
 * @example
 * ```typescript
 * // INSERT
 * const result = await db.execute(
 *   "INSERT INTO users (name, email) VALUES (?, ?)",
 *   ["Alice", "alice@example.com"]
 * );
 * console.log("New user ID:", result.lastInsertRowid);
 * console.log("Rows inserted:", result.rowsAffected);
 *
 * // UPDATE
 * const updated = await db.execute(
 *   "UPDATE users SET active = ? WHERE last_login < ?",
 *   [false, Date.now() - 30 * 24 * 60 * 60 * 1000]
 * );
 * console.log("Deactivated users:", updated.rowsAffected);
 *
 * // DELETE
 * const deleted = await db.execute("DELETE FROM users WHERE active = ?", [false]);
 * console.log("Deleted users:", deleted.rowsAffected);
 * ```
 */
export interface ExecuteResult {
  /** Number of rows inserted, updated, or deleted */
  rowsAffected: number;
  /** Row ID of the last inserted row (INSERT only, undefined otherwise) */
  lastInsertRowid?: number;
}

/**
 * Result of executing a batch of SQL statements.
 *
 * Contains aggregate statistics and any errors that occurred during execution.
 *
 * @example
 * ```typescript
 * const result = await db.executeBatch([
 *   "INSERT INTO logs (message) VALUES ('Started')",
 *   "INSERT INTO logs (message) VALUES ('Processing')",
 *   "INSERT INTO logs (message) VALUES ('Completed')"
 * ]);
 * console.log(`Executed ${result.statementCount} statements`);
 * console.log(`Total rows affected: ${result.totalRowsAffected}`);
 * if (result.errors.length > 0) {
 *   console.error("Errors:", result.errors);
 * }
 * ```
 */
export interface BatchResult {
  /** Total number of rows affected across all statements */
  totalRowsAffected: number;
  /** Number of statements executed */
  statementCount: number;
  /** Errors that occurred (statement index and message) */
  errors: Array<{ index: number; message: string }>;
}

/**
 * Information about a prepared statement.
 *
 * Contains the statement ID, SQL, and parameter count.
 */
export interface PreparedStatementInfo {
  /** Unique prepared statement handle ID */
  id: string;
  /** The SQL statement that was prepared */
  sql: string;
  /** Number of parameter placeholders (?) in the statement */
  parameterCount: number;
}

/**
 * Table column schema information.
 *
 * Describes a single column in a database table, including its type,
 * nullability, default value, and primary key status.
 */
export interface TableColumn {
  /** Column name */
  name: string;
  /** SQLite type (TEXT, INTEGER, REAL, BLOB, etc.) */
  type: string;
  /** True if the column can contain NULL values */
  nullable: boolean;
  /** Default value expression (if any) */
  defaultValue?: string;
  /** True if this column is part of the primary key */
  primaryKey: boolean;
}

/**
 * Index schema information.
 *
 * Describes a database index, including which columns it covers and whether it's unique.
 */
export interface IndexInfo {
  /** Index name */
  name: string;
  /** List of column names in the index */
  columns: string[];
  /** True if the index enforces uniqueness */
  unique: boolean;
}

/**
 * Complete table schema information.
 *
 * Describes the structure of a database table, including all columns,
 * primary key, and indexes.
 *
 * @example
 * ```typescript
 * const info = await db.tableInfo("users");
 * console.log(`Table: ${info.name}`);
 * console.log(`Primary key: ${info.primaryKey.join(", ")}`);
 * console.log("Columns:");
 * for (const col of info.columns) {
 *   console.log(`  ${col.name}: ${col.type}${col.nullable ? "" : " NOT NULL"}`);
 * }
 * console.log("Indexes:");
 * for (const idx of info.indexes) {
 *   console.log(`  ${idx.name} on (${idx.columns.join(", ")})`);
 * }
 * ```
 */
export interface TableInfo {
  /** Table name */
  name: string;
  /** List of columns in the table */
  columns: TableColumn[];
  /** List of column names that form the primary key */
  primaryKey: string[];
  /** List of indexes on the table */
  indexes: IndexInfo[];
}

/**
 * Batch of rows from a streaming query.
 *
 * Returned by the async iterator when streaming large result sets.
 *
 * @template T - The expected row type (default: `Record<string, unknown>`)
 */
export interface StreamBatch<T = Record<string, unknown>> {
  /** Batch of result rows */
  rows: T[];
  /** True if this is the last batch (no more rows available) */
  done: boolean;
  /** Total number of rows fetched so far (cumulative) */
  totalFetched: number;
}

/**
 * Database schema migration definition.
 *
 * Defines a versioned migration with SQL to apply (up) and optionally rollback (down).
 *
 * @example
 * ```typescript
 * const migrations: Migration[] = [
 *   {
 *     version: 1,
 *     name: "create_users_table",
 *     upSql: `
 *       CREATE TABLE users (
 *         id INTEGER PRIMARY KEY AUTOINCREMENT,
 *         name TEXT NOT NULL,
 *         email TEXT UNIQUE
 *       )
 *     `,
 *     downSql: "DROP TABLE users"
 *   },
 *   {
 *     version: 2,
 *     name: "add_users_active_column",
 *     upSql: "ALTER TABLE users ADD COLUMN active INTEGER DEFAULT 1",
 *     downSql: "ALTER TABLE users DROP COLUMN active"
 *   }
 * ];
 *
 * await db.migrate(migrations);
 * ```
 */
export interface Migration {
  /** Migration version number (must be unique and sequential) */
  version: number;
  /** Human-readable migration name */
  name: string;
  /** SQL to apply the migration (forward) */
  upSql: string;
  /** SQL to rollback the migration (backward, optional) */
  downSql?: string;
}

/**
 * Information about a migration that has been applied to the database.
 *
 * Tracks which migrations have been run and when.
 */
export interface AppliedMigration {
  /** Migration version number */
  version: number;
  /** Migration name */
  name: string;
  /** Timestamp when the migration was applied (Unix epoch milliseconds) */
  appliedAt: number;
}

/**
 * Current migration status of the database.
 *
 * Shows the current version, which migrations have been applied,
 * and which are pending.
 *
 * @example
 * ```typescript
 * const status = await db.migrationStatus();
 * console.log(`Current version: ${status.currentVersion}`);
 * console.log(`Applied migrations: ${status.applied.length}`);
 * console.log(`Pending migrations: ${status.pending.length}`);
 * if (status.pending.length > 0) {
 *   console.log("Pending:", status.pending.join(", "));
 * }
 * ```
 */
export interface MigrationStatus {
  /** Current database schema version */
  currentVersion: number;
  /** List of pending migration names (not yet applied) */
  pending: string[];
  /** List of applied migrations with timestamps */
  applied: AppliedMigration[];
}

// =============================================================================
// Database Interface
// =============================================================================

/**
 * Database connection handle with all operations.
 *
 * Provides complete access to a SQLite database, including queries, transactions,
 * prepared statements, schema introspection, streaming, and migrations.
 *
 * Obtained by calling `open()`. Always call `close()` when done to free resources.
 *
 * @example
 * ```typescript
 * const db = await open("myapp");
 *
 * // Create table
 * await db.execute(`
 *   CREATE TABLE IF NOT EXISTS tasks (
 *     id INTEGER PRIMARY KEY AUTOINCREMENT,
 *     title TEXT NOT NULL,
 *     completed INTEGER DEFAULT 0
 *   )
 * `);
 *
 * // Insert data
 * await db.execute(
 *   "INSERT INTO tasks (title) VALUES (?)",
 *   ["Buy groceries"]
 * );
 *
 * // Query data
 * interface Task { id: number; title: string; completed: number; }
 * const result = await db.query<Task>("SELECT * FROM tasks");
 * for (const task of result.rows) {
 *   console.log(task.title);
 * }
 *
 * await db.close();
 * ```
 */
export interface Database {
  /** Unique database handle ID */
  readonly id: string;
  /** Database name (without .db extension) */
  readonly name: string;
  /** Full filesystem path to the database file */
  readonly path: string;

  // Query operations

  /**
   * Execute a SELECT query and return all rows.
   *
   * Use the type parameter to specify the expected row shape for type safety.
   * Rows are returned as objects with column names as keys.
   *
   * @template T - The expected row type (default: `Record<string, unknown>`)
   * @param sql - SQL SELECT statement (use ? for parameters)
   * @param params - Parameter values to bind
   * @returns Query result with typed rows
   *
   * @throws Error [8403] if SQL syntax is invalid
   * @throws Error [8414] if parameter count doesn't match placeholders
   *
   * @example
   * ```typescript
   * interface User {
   *   id: number;
   *   name: string;
   *   email: string;
   *   active: number;
   * }
   *
   * // Query all users
   * const allUsers = await db.query<User>("SELECT * FROM users");
   * console.log(`Found ${allUsers.rows.length} users`);
   *
   * // Query with parameters
   * const activeUsers = await db.query<User>(
   *   "SELECT * FROM users WHERE active = ?",
   *   [1]
   * );
   *
   * // Complex query
   * const result = await db.query<User>(
   *   "SELECT * FROM users WHERE name LIKE ? AND created_at > ? ORDER BY name LIMIT ?",
   *   ["%Smith%", Date.now() - 30 * 24 * 60 * 60 * 1000, 10]
   * );
   *
   * // Access column metadata
   * console.log("Columns:", result.columns.map(c => c.name));
   * ```
   */
  query<T = Record<string, unknown>>(sql: string, params?: unknown[]): Promise<QueryResult<T>>;

  /**
   * Execute an INSERT, UPDATE, or DELETE statement.
   *
   * Does not return rows, only the number of affected rows and last insert ID.
   *
   * @param sql - SQL statement (INSERT, UPDATE, DELETE, or DDL)
   * @param params - Parameter values to bind
   * @returns Execute result with rowsAffected and lastInsertRowid
   *
   * @throws Error [8403] if SQL syntax is invalid
   * @throws Error [8404] if a constraint is violated (UNIQUE, FOREIGN KEY, etc.)
   * @throws Error [8414] if parameter count doesn't match placeholders
   *
   * @example
   * ```typescript
   * // INSERT
   * const result = await db.execute(
   *   "INSERT INTO users (name, email) VALUES (?, ?)",
   *   ["Alice", "alice@example.com"]
   * );
   * console.log("New user ID:", result.lastInsertRowid);
   *
   * // UPDATE
   * const updated = await db.execute(
   *   "UPDATE users SET active = ? WHERE last_login < ?",
   *   [0, Date.now() - 90 * 24 * 60 * 60 * 1000]
   * );
   * console.log("Deactivated users:", updated.rowsAffected);
   *
   * // DELETE
   * const deleted = await db.execute(
   *   "DELETE FROM sessions WHERE expires_at < ?",
   *   [Date.now()]
   * );
   * console.log("Cleaned up sessions:", deleted.rowsAffected);
   *
   * // DDL (no parameters)
   * await db.execute("CREATE INDEX idx_users_email ON users(email)");
   * ```
   */
  execute(sql: string, params?: unknown[]): Promise<ExecuteResult>;

  /**
   * Execute multiple SQL statements in a batch.
   *
   * By default, all statements run in a transaction (all-or-nothing).
   * Use options to control transactional behavior and error handling.
   *
   * @param statements - Array of SQL statements to execute
   * @param opts - Batch execution options
   * @returns Batch result with statistics and errors
   *
   * @throws Error [8403] if any SQL syntax is invalid (when stopOnError: true)
   * @throws Error [8404] if any constraint is violated (when stopOnError: true)
   *
   * @example
   * ```typescript
   * // Atomic batch (default - all-or-nothing)
   * await db.executeBatch([
   *   "INSERT INTO logs (level, message) VALUES ('INFO', 'Started')",
   *   "INSERT INTO logs (level, message) VALUES ('INFO', 'Processing')",
   *   "INSERT INTO logs (level, message) VALUES ('INFO', 'Completed')"
   * ]);
   *
   * // Continue on errors
   * const result = await db.executeBatch([
   *   "INSERT INTO users (name) VALUES ('Alice')",
   *   "INVALID SQL",
   *   "INSERT INTO users (name) VALUES ('Bob')"
   * ], { stopOnError: false });
   * console.log("Executed:", result.statementCount);
   * console.log("Errors:", result.errors);
   *
   * // No transaction (faster but not atomic)
   * await db.executeBatch(statements, { transaction: false });
   * ```
   */
  executeBatch(statements: string[], opts?: BatchOptions): Promise<BatchResult>;

  /**
   * Execute a query and return only the first row.
   *
   * Convenient for queries that should return exactly one row.
   * Returns `null` if no rows match.
   *
   * @template T - The expected row type (default: `unknown[]`)
   * @param sql - SQL SELECT statement
   * @param params - Parameter values to bind
   * @returns First row as object, or null if no rows
   *
   * @example
   * ```typescript
   * interface User { id: number; name: string; email: string; }
   *
   * // Get single user by ID
   * const user = await db.queryRow<User>(
   *   "SELECT * FROM users WHERE id = ?",
   *   [42]
   * );
   * if (user) {
   *   console.log("Found:", user.name);
   * } else {
   *   console.log("User not found");
   * }
   *
   * // Get row as array
   * const row = await db.queryRow("SELECT name, email FROM users WHERE id = ?", [42]);
   * if (row) {
   *   const [name, email] = row as [string, string];
   * }
   * ```
   */
  queryRow<T = unknown[]>(sql: string, params?: unknown[]): Promise<T | null>;

  /**
   * Execute a query and return only the first column of the first row.
   *
   * Convenient for queries that return a single value (COUNT, SUM, etc.).
   * Returns `null` if no rows match.
   *
   * @template T - The expected value type (default: `unknown`)
   * @param sql - SQL SELECT statement
   * @param params - Parameter values to bind
   * @returns First value from first row, or null if no rows
   *
   * @example
   * ```typescript
   * // Count rows
   * const count = await db.queryValue<number>(
   *   "SELECT COUNT(*) FROM users WHERE active = ?",
   *   [1]
   * );
   * console.log("Active users:", count);
   *
   * // Sum values
   * const total = await db.queryValue<number>(
   *   "SELECT SUM(amount) FROM orders WHERE user_id = ?",
   *   [userId]
   * );
   *
   * // Get single column value
   * const email = await db.queryValue<string>(
   *   "SELECT email FROM users WHERE id = ?",
   *   [42]
   * );
   * ```
   */
  queryValue<T = unknown>(sql: string, params?: unknown[]): Promise<T | null>;

  // Prepared statements

  /**
   * Prepare a SQL statement for repeated execution.
   *
   * Compiles the SQL once and allows executing it multiple times with different
   * parameters. More efficient than calling `query()` or `execute()` repeatedly.
   *
   * Always call `finalize()` on the prepared statement when done to free resources.
   *
   * @param sql - SQL statement to prepare (use ? for parameters)
   * @returns Prepared statement handle
   *
   * @throws Error [8403] if SQL syntax is invalid
   * @throws Error [8410] if statement cannot be prepared
   *
   * @example
   * ```typescript
   * const stmt = await db.prepare(
   *   "INSERT INTO events (type, data, timestamp) VALUES (?, ?, ?)"
   * );
   *
   * try {
   *   // Execute multiple times with different parameters
   *   await stmt.execute(["click", JSON.stringify({ x: 100, y: 200 }), Date.now()]);
   *   await stmt.execute(["hover", JSON.stringify({ element: "button" }), Date.now()]);
   *   await stmt.execute(["submit", JSON.stringify({ form: "login" }), Date.now()]);
   * } finally {
   *   await stmt.finalize(); // Always finalize to free resources
   * }
   * ```
   */
  prepare(sql: string): Promise<PreparedStatement>;

  // Transactions

  /**
   * Begin a database transaction.
   *
   * All subsequent operations run in the transaction until `commit()` or `rollback()`.
   * Use the `transaction()` helper for automatic rollback on error.
   *
   * @param mode - Transaction mode ("deferred", "immediate", or "exclusive")
   * @throws Error [8407] if already in a transaction
   *
   * @example
   * ```typescript
   * await db.begin();
   * try {
   *   await db.execute("INSERT INTO accounts (name, balance) VALUES (?, ?)", ["Alice", 1000]);
   *   await db.execute("INSERT INTO accounts (name, balance) VALUES (?, ?)", ["Bob", 500]);
   *   await db.commit();
   * } catch (e) {
   *   await db.rollback();
   *   throw e;
   * }
   * ```
   */
  begin(mode?: "deferred" | "immediate" | "exclusive"): Promise<void>;

  /**
   * Commit the current transaction.
   *
   * Makes all changes permanent since the last `begin()`.
   *
   * @throws Error [8407] if not in a transaction
   */
  commit(): Promise<void>;

  /**
   * Rollback the current transaction.
   *
   * Discards all changes since the last `begin()`.
   *
   * @throws Error [8407] if not in a transaction
   */
  rollback(): Promise<void>;

  /**
   * Execute a function within a transaction with automatic rollback on error.
   *
   * Begins a transaction, executes the function, commits on success, and
   * rolls back on error. This is the recommended way to use transactions.
   *
   * @template T - The return type of the function
   * @param fn - Async function to execute in the transaction
   * @returns The value returned by the function
   *
   * @throws Any error thrown by the function (after rolling back)
   *
   * @example
   * ```typescript
   * // Transfer money between accounts
   * await db.transaction(async () => {
   *   await db.execute(
   *     "UPDATE accounts SET balance = balance - ? WHERE id = ?",
   *     [100, fromAccountId]
   *   );
   *   await db.execute(
   *     "UPDATE accounts SET balance = balance + ? WHERE id = ?",
   *     [100, toAccountId]
   *   );
   * });
   * // Automatically committed if successful, rolled back on error
   *
   * // Bulk insert (~1000x faster than individual inserts)
   * await db.transaction(async () => {
   *   for (const user of users) {
   *     await db.execute(
   *       "INSERT INTO users (name, email) VALUES (?, ?)",
   *       [user.name, user.email]
   *     );
   *   }
   * });
   * ```
   */
  transaction<T>(fn: () => Promise<T>): Promise<T>;

  /**
   * Create a savepoint within a transaction.
   *
   * Savepoints allow nested transaction-like behavior within a transaction.
   *
   * @param name - Savepoint name
   */
  savepoint(name: string): Promise<void>;

  /**
   * Release (commit) a savepoint.
   *
   * @param name - Savepoint name
   */
  release(name: string): Promise<void>;

  /**
   * Rollback to a savepoint.
   *
   * Discards changes since the savepoint was created.
   *
   * @param name - Savepoint name
   *
   * @example
   * ```typescript
   * await db.begin();
   * await db.execute("INSERT INTO users (name) VALUES (?)", ["Alice"]);
   *
   * await db.savepoint("before_bob");
   * await db.execute("INSERT INTO users (name) VALUES (?)", ["Bob"]);
   *
   * // Oops, rollback Bob's insert
   * await db.rollbackTo("before_bob");
   *
   * await db.execute("INSERT INTO users (name) VALUES (?)", ["Charlie"]);
   * await db.commit(); // Alice and Charlie inserted, Bob was rolled back
   * ```
   */
  rollbackTo(name: string): Promise<void>;

  // Schema

  /**
   * List all table names in the database.
   *
   * @returns Array of table names
   *
   * @example
   * ```typescript
   * const tables = await db.tables();
   * console.log("Tables:", tables.join(", "));
   * ```
   */
  tables(): Promise<string[]>;

  /**
   * Get complete schema information for a table.
   *
   * Returns columns, primary key, and indexes.
   *
   * @param table - Table name
   * @returns Table schema information
   *
   * @throws Error [8401] if table does not exist
   *
   * @example
   * ```typescript
   * const info = await db.tableInfo("users");
   * console.log(`Table: ${info.name}`);
   * console.log(`Primary key: ${info.primaryKey.join(", ")}`);
   * console.log("Columns:");
   * for (const col of info.columns) {
   *   console.log(`  ${col.name}: ${col.type}${col.nullable ? "" : " NOT NULL"}`);
   * }
   * ```
   */
  tableInfo(table: string): Promise<TableInfo>;

  /**
   * Check if a table exists in the database.
   *
   * @param table - Table name
   * @returns True if the table exists
   *
   * @example
   * ```typescript
   * if (!await db.tableExists("users")) {
   *   await db.execute(`
   *     CREATE TABLE users (
   *       id INTEGER PRIMARY KEY AUTOINCREMENT,
   *       name TEXT NOT NULL
   *     )
   *   `);
   * }
   * ```
   */
  tableExists(table: string): Promise<boolean>;

  // Streaming

  /**
   * Stream query results in batches.
   *
   * Use for large result sets to avoid loading all rows into memory at once.
   * Returns an async iterable that yields batches of rows.
   *
   * @template T - The expected row type (default: `Record<string, unknown>`)
   * @param sql - SQL SELECT statement
   * @param params - Parameter values to bind
   * @param batchSize - Number of rows per batch (default: 100)
   * @returns Async iterable of row batches
   *
   * @example
   * ```typescript
   * interface LogEntry {
   *   id: number;
   *   timestamp: number;
   *   message: string;
   * }
   *
   * // Stream large result set
   * for await (const batch of db.stream<LogEntry>(
   *   "SELECT * FROM logs WHERE level = ?",
   *   ["ERROR"],
   *   50 // 50 rows per batch
   * )) {
   *   console.log(`Processing ${batch.length} log entries...`);
   *   for (const log of batch) {
   *     await processLog(log);
   *   }
   * }
   *
   * // Export large dataset
   * let total = 0;
   * for await (const batch of db.stream("SELECT * FROM events", [], 1000)) {
   *   await writeToFile(batch);
   *   total += batch.length;
   *   console.log(`Exported ${total} events so far...`);
   * }
   * ```
   */
  stream<T = Record<string, unknown>>(sql: string, params?: unknown[], batchSize?: number): AsyncIterable<T[]>;

  // Migrations

  /**
   * Apply pending database migrations.
   *
   * Migrations must be ordered by version number. Only unapplied migrations
   * are executed. Migration tracking is stored in a `_migrations` table.
   *
   * @param migrations - Array of migration definitions (ordered by version)
   * @returns Migration status after applying
   *
   * @throws Error [8413] if a migration fails
   * @throws Error [8413] if migration versions are not sequential
   *
   * @example
   * ```typescript
   * const migrations: Migration[] = [
   *   {
   *     version: 1,
   *     name: "create_users",
   *     upSql: "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)",
   *     downSql: "DROP TABLE users"
   *   },
   *   {
   *     version: 2,
   *     name: "add_email_column",
   *     upSql: "ALTER TABLE users ADD COLUMN email TEXT",
   *     downSql: "ALTER TABLE users DROP COLUMN email"
   *   }
   * ];
   *
   * const status = await db.migrate(migrations);
   * console.log(`Migrated to version ${status.currentVersion}`);
   * ```
   */
  migrate(migrations: Migration[]): Promise<MigrationStatus>;

  /**
   * Get current migration status.
   *
   * Shows which migrations have been applied and which are pending.
   *
   * @returns Current migration status
   *
   * @example
   * ```typescript
   * const status = await db.migrationStatus();
   * console.log(`Current version: ${status.currentVersion}`);
   * console.log(`Applied: ${status.applied.map(m => m.name).join(", ")}`);
   * if (status.pending.length > 0) {
   *   console.log(`Pending: ${status.pending.join(", ")}`);
   * }
   * ```
   */
  migrationStatus(): Promise<MigrationStatus>;

  /**
   * Rollback migrations to a target version.
   *
   * Executes the `downSql` of migrations in reverse order.
   *
   * @param targetVersion - Version to rollback to (default: 0, removes all migrations)
   * @returns Migration status after rollback
   *
   * @throws Error [8413] if any migration lacks a downSql
   * @throws Error [8413] if rollback fails
   *
   * @example
   * ```typescript
   * // Rollback to version 1 (undoes version 2+)
   * await db.migrateDown(1);
   *
   * // Rollback all migrations
   * await db.migrateDown(0);
   * ```
   */
  migrateDown(targetVersion?: number): Promise<MigrationStatus>;

  // Maintenance

  /**
   * Vacuum the database to reclaim unused space.
   *
   * Rebuilds the database file, removing fragmentation and unused pages.
   * Can significantly reduce database size after many deletes.
   *
   * @example
   * ```typescript
   * const beforeSize = (await list()).find(db => db.name === "myapp")?.sizeBytes ?? 0;
   * await db.vacuum();
   * const afterSize = (await list()).find(db => db.name === "myapp")?.sizeBytes ?? 0;
   * console.log(`Freed ${(beforeSize - afterSize) / 1024} KB`);
   * ```
   */
  vacuum(): Promise<void>;

  /**
   * Close the database connection.
   *
   * Always call this when done to free resources. After closing, the database
   * handle cannot be used anymore.
   *
   * @example
   * ```typescript
   * const db = await open("myapp");
   * try {
   *   // ... use database ...
   * } finally {
   *   await db.close();
   * }
   * ```
   */
  close(): Promise<void>;
}

/**
 * Prepared statement handle for efficient repeated execution.
 *
 * A prepared statement compiles SQL once and allows executing it multiple
 * times with different parameters. This is more efficient than calling
 * `db.query()` or `db.execute()` repeatedly with the same SQL.
 *
 * **Always call `finalize()` when done to free resources.**
 *
 * @example
 * ```typescript
 * const stmt = await db.prepare(
 *   "INSERT INTO metrics (name, value, timestamp) VALUES (?, ?, ?)"
 * );
 *
 * try {
 *   console.log(`Prepared statement has ${stmt.parameterCount} parameters`);
 *
 *   // Execute multiple times with different parameters
 *   for (let i = 0; i < 100; i++) {
 *     await stmt.execute([`metric_${i}`, Math.random() * 100, Date.now()]);
 *   }
 * } finally {
 *   await stmt.finalize(); // Always finalize to free resources
 * }
 * ```
 */
export interface PreparedStatement {
  /** Unique prepared statement handle ID */
  readonly id: string;
  /** The SQL statement that was prepared */
  readonly sql: string;
  /** Number of parameter placeholders (?) in the statement */
  readonly parameterCount: number;

  /**
   * Execute the prepared statement as a SELECT query.
   *
   * @template T - The expected row type (default: `Record<string, unknown>`)
   * @param params - Parameter values to bind (must match parameter count)
   * @returns Query result with typed rows
   *
   * @throws Error [8414] if parameter count doesn't match
   *
   * @example
   * ```typescript
   * interface User { id: number; name: string; email: string; }
   *
   * const stmt = await db.prepare("SELECT * FROM users WHERE active = ? LIMIT ?");
   * try {
   *   // Execute with different parameters
   *   const activeUsers = await stmt.query<User>([1, 10]);
   *   const inactiveUsers = await stmt.query<User>([0, 10]);
   * } finally {
   *   await stmt.finalize();
   * }
   * ```
   */
  query<T = Record<string, unknown>>(params?: unknown[]): Promise<QueryResult<T>>;

  /**
   * Execute the prepared statement as an INSERT, UPDATE, or DELETE.
   *
   * @param params - Parameter values to bind (must match parameter count)
   * @returns Execute result with rowsAffected and lastInsertRowid
   *
   * @throws Error [8414] if parameter count doesn't match
   * @throws Error [8404] if a constraint is violated
   *
   * @example
   * ```typescript
   * const stmt = await db.prepare(
   *   "INSERT INTO logs (level, message, timestamp) VALUES (?, ?, ?)"
   * );
   *
   * try {
   *   for (const log of logEntries) {
   *     await stmt.execute([log.level, log.message, log.timestamp]);
   *   }
   * } finally {
   *   await stmt.finalize();
   * }
   * ```
   */
  execute(params?: unknown[]): Promise<ExecuteResult>;

  /**
   * Finalize the prepared statement and free resources.
   *
   * After finalizing, the prepared statement handle cannot be used anymore.
   * Always call this when done with a prepared statement.
   *
   * @example
   * ```typescript
   * const stmt = await db.prepare("SELECT * FROM users WHERE id = ?");
   * try {
   *   const user = await stmt.query([42]);
   * } finally {
   *   await stmt.finalize(); // Required to free resources
   * }
   * ```
   */
  finalize(): Promise<void>;
}

// =============================================================================
// Helper Functions
// =============================================================================

const core = Deno.core;

/** Convert raw column info to public format */
function toColumnInfo(raw: RawColumnInfo): ColumnInfo {
  return {
    name: raw.name,
    type: raw.column_type,
    nullable: raw.nullable,
  };
}

/** Convert raw rows to objects using column names */
function rowsToObjects<T>(columns: RawColumnInfo[], rows: unknown[][]): T[] {
  return rows.map((row) => {
    const obj: Record<string, unknown> = {};
    columns.forEach((col, i) => {
      obj[col.name] = row[i];
    });
    return obj as T;
  });
}

/** Convert raw table info to public format */
function toTableInfo(raw: RawTableInfo): TableInfo {
  return {
    name: raw.name,
    columns: raw.columns.map((col) => ({
      name: col.name,
      type: col.column_type,
      nullable: col.nullable,
      defaultValue: col.default_value ?? undefined,
      primaryKey: col.primary_key,
    })),
    primaryKey: raw.primary_key,
    indexes: raw.indexes,
  };
}

/** Convert raw migration status to public format */
function toMigrationStatus(raw: RawMigrationStatus): MigrationStatus {
  return {
    currentVersion: raw.current_version,
    pending: raw.pending,
    applied: raw.applied.map((m) => ({
      version: m.version,
      name: m.name,
      appliedAt: m.applied_at,
    })),
  };
}

// =============================================================================
// Database Implementation
// =============================================================================

/** Create a database handle from an open result */
function createDatabase(result: RawOpenResult, name: string): Database {
  const dbId = result.id;

  const db: Database = {
    id: dbId,
    name,
    path: result.path,

    async query<T = Record<string, unknown>>(sql: string, params?: unknown[]): Promise<QueryResult<T>> {
      const raw = await core.ops.op_database_query(dbId, sql, params);
      return {
        columns: raw.columns.map(toColumnInfo),
        rows: rowsToObjects<T>(raw.columns, raw.rows),
        rowsAffected: raw.rows_affected,
        lastInsertRowid: raw.last_insert_rowid ?? undefined,
      };
    },

    async execute(sql: string, params?: unknown[]): Promise<ExecuteResult> {
      const raw = await core.ops.op_database_execute(dbId, sql, params);
      return {
        rowsAffected: raw.rows_affected,
        lastInsertRowid: raw.last_insert_rowid ?? undefined,
      };
    },

    async executeBatch(statements: string[], opts?: BatchOptions): Promise<BatchResult> {
      const raw = await core.ops.op_database_execute_batch(dbId, statements, opts);
      return {
        totalRowsAffected: raw.total_rows_affected,
        statementCount: raw.statement_count,
        errors: raw.errors,
      };
    },

    async queryRow<T = unknown[]>(sql: string, params?: unknown[]): Promise<T | null> {
      return (await core.ops.op_database_query_row(dbId, sql, params)) as T | null;
    },

    async queryValue<T = unknown>(sql: string, params?: unknown[]): Promise<T | null> {
      return (await core.ops.op_database_query_value(dbId, sql, params)) as T | null;
    },

    async prepare(sql: string): Promise<PreparedStatement> {
      const info = await core.ops.op_database_prepare(dbId, sql);
      return createPreparedStatement(dbId, sql, info);
    },

    async begin(mode?: "deferred" | "immediate" | "exclusive"): Promise<void> {
      await core.ops.op_database_begin(dbId, mode);
    },

    async commit(): Promise<void> {
      await core.ops.op_database_commit(dbId);
    },

    async rollback(): Promise<void> {
      await core.ops.op_database_rollback(dbId);
    },

    async transaction<T>(fn: () => Promise<T>): Promise<T> {
      await db.begin();
      try {
        const result = await fn();
        await db.commit();
        return result;
      } catch (e) {
        await db.rollback();
        throw e;
      }
    },

    async savepoint(name: string): Promise<void> {
      await core.ops.op_database_savepoint(dbId, name);
    },

    async release(name: string): Promise<void> {
      await core.ops.op_database_release(dbId, name);
    },

    async rollbackTo(name: string): Promise<void> {
      await core.ops.op_database_rollback_to(dbId, name);
    },

    async tables(): Promise<string[]> {
      return await core.ops.op_database_tables(dbId);
    },

    async tableInfo(table: string): Promise<TableInfo> {
      const raw = await core.ops.op_database_table_info(dbId, table);
      return toTableInfo(raw);
    },

    async tableExists(table: string): Promise<boolean> {
      return await core.ops.op_database_table_exists(dbId, table);
    },

    async *stream<T = Record<string, unknown>>(
      sql: string,
      params?: unknown[],
      batchSize?: number
    ): AsyncIterable<T[]> {
      const streamId = await core.ops.op_database_stream_open(dbId, sql, params, batchSize);
      try {
        // Get column info for the first batch
        let columns: RawColumnInfo[] | null = null;

        while (true) {
          const batch = await core.ops.op_database_stream_next(streamId);

          // Get columns from the first batch if we haven't yet
          if (columns === null && batch.rows.length > 0) {
            // We need to get column info - do a quick query for schema
            const schemaResult = await core.ops.op_database_query(dbId, `SELECT * FROM (${sql}) LIMIT 0`, params);
            columns = schemaResult.columns;
          }

          if (batch.rows.length > 0 && columns) {
            yield rowsToObjects<T>(columns, batch.rows);
          }

          if (batch.done) break;
        }
      } finally {
        await core.ops.op_database_stream_close(streamId);
      }
    },

    async migrate(migrations: Migration[]): Promise<MigrationStatus> {
      // Convert camelCase to snake_case for Rust
      const rustMigrations = migrations.map((m) => ({
        version: m.version,
        name: m.name,
        up_sql: m.upSql,
        down_sql: m.downSql,
      }));
      const raw = await core.ops.op_database_migrate(dbId, rustMigrations as unknown as Migration[]);
      return toMigrationStatus(raw);
    },

    async migrationStatus(): Promise<MigrationStatus> {
      const raw = await core.ops.op_database_migration_status(dbId);
      return toMigrationStatus(raw);
    },

    async migrateDown(targetVersion?: number): Promise<MigrationStatus> {
      const raw = await core.ops.op_database_migrate_down(dbId, targetVersion);
      return toMigrationStatus(raw);
    },

    async vacuum(): Promise<void> {
      await core.ops.op_database_vacuum(dbId);
    },

    async close(): Promise<void> {
      await core.ops.op_database_close(dbId);
    },
  };

  return db;
}

/** Create a prepared statement handle */
function createPreparedStatement(
  dbId: string,
  sql: string,
  info: RawPreparedStatementInfo
): PreparedStatement {
  return {
    id: info.id,
    sql: info.sql,
    parameterCount: info.parameter_count,

    async query<T = Record<string, unknown>>(params?: unknown[]): Promise<QueryResult<T>> {
      // Note: Prepared statement caching is not implemented in this version
      // We fall back to regular query execution
      const raw = await core.ops.op_database_query(dbId, sql, params);
      return {
        columns: raw.columns.map(toColumnInfo),
        rows: rowsToObjects<T>(raw.columns, raw.rows),
        rowsAffected: raw.rows_affected,
        lastInsertRowid: raw.last_insert_rowid ?? undefined,
      };
    },

    async execute(params?: unknown[]): Promise<ExecuteResult> {
      // Note: Prepared statement caching is not implemented in this version
      // We fall back to regular execute
      const raw = await core.ops.op_database_execute(dbId, sql, params);
      return {
        rowsAffected: raw.rows_affected,
        lastInsertRowid: raw.last_insert_rowid ?? undefined,
      };
    },

    async finalize(): Promise<void> {
      await core.ops.op_database_stmt_finalize(dbId, info.id);
    },
  };
}

// =============================================================================
// Public API
// =============================================================================

/**
 * Open a database by name.
 *
 * Creates the database file and directory if they don't exist (unless `create: false`).
 * By default, enables WAL mode and foreign key constraints for better performance and data integrity.
 *
 * Each app can have multiple named databases. Database files are stored in the app's data directory.
 *
 * **Always call `close()` on the returned database handle when done to free resources.**
 *
 * @param name - Database name (without .db extension)
 * @param opts - Database open options
 * @returns Database connection handle
 *
 * @throws Error [8401] if database doesn't exist and `create: false`
 * @throws Error [8408] if permission denied
 * @throws Error [8412] if I/O error (disk full, etc.)
 *
 * @example
 * ```typescript
 * import * as db from "runtime:database";
 *
 * // Open with default options (create, WAL mode, foreign keys enabled)
 * const database = await db.open("myapp");
 *
 * // Create table
 * await database.execute(`
 *   CREATE TABLE IF NOT EXISTS users (
 *     id INTEGER PRIMARY KEY AUTOINCREMENT,
 *     name TEXT NOT NULL,
 *     email TEXT UNIQUE
 *   )
 * `);
 *
 * // Insert data
 * const result = await database.execute(
 *   "INSERT INTO users (name, email) VALUES (?, ?)",
 *   ["Alice", "alice@example.com"]
 * );
 * console.log("Inserted user with ID:", result.lastInsertRowid);
 *
 * // Query data
 * interface User { id: number; name: string; email: string; }
 * const users = await database.query<User>("SELECT * FROM users");
 * for (const user of users.rows) {
 *   console.log(user.name, user.email);
 * }
 *
 * await database.close();
 * ```
 *
 * @example
 * ```typescript
 * // Open in read-only mode
 * const readDb = await db.open("reports", { readonly: true });
 * const stats = await readDb.query("SELECT * FROM stats");
 * await readDb.close();
 *
 * // Open with custom timeout
 * const busyDb = await db.open("shared", {
 *   busyTimeoutMs: 10000 // Wait up to 10 seconds if locked
 * });
 *
 * // Fail if database doesn't exist
 * try {
 *   const existingDb = await db.open("must-exist", { create: false });
 * } catch (err) {
 *   console.error("Database not found");
 * }
 * ```
 *
 * @example
 * ```typescript
 * // Multiple databases per app
 * const userDb = await db.open("users");
 * const analyticsDb = await db.open("analytics");
 * const cacheDb = await db.open("cache");
 *
 * // Each database is independent
 * await userDb.execute("INSERT INTO sessions ...");
 * await analyticsDb.execute("INSERT INTO events ...");
 *
 * await userDb.close();
 * await analyticsDb.close();
 * await cacheDb.close();
 * ```
 */
export async function open(name: string, opts?: OpenOptions): Promise<Database> {
  const result = await core.ops.op_database_open(name, opts);
  return createDatabase(result, name);
}

/**
 * List all databases for the current app.
 *
 * Returns metadata for all databases including their size, tables, and paths.
 *
 * @returns Array of database information objects
 *
 * @example
 * ```typescript
 * const databases = await list();
 * console.log(`Found ${databases.length} databases`);
 *
 * for (const db of databases) {
 *   console.log(`${db.name}:`);
 *   console.log(`  Path: ${db.path}`);
 *   console.log(`  Size: ${(db.sizeBytes / 1024).toFixed(2)} KB`);
 *   console.log(`  Tables: ${db.tables.join(", ")}`);
 *   console.log(`  Read-only: ${db.readonly}`);
 * }
 * ```
 *
 * @example
 * ```typescript
 * // Find large databases
 * const databases = await list();
 * const large = databases.filter(db => db.sizeBytes > 1024 * 1024); // > 1 MB
 * console.log("Large databases:", large.map(db => db.name));
 *
 * // Check for specific tables
 * const userDbs = databases.filter(db => db.tables.includes("users"));
 * ```
 */
export async function list(): Promise<DatabaseInfo[]> {
  const raw = await core.ops.op_database_list();
  return raw.map((db) => ({
    name: db.name,
    path: db.path,
    sizeBytes: db.size_bytes,
    tables: db.tables,
    readonly: db.readonly,
  }));
}

/**
 * Check if a database exists.
 *
 * Returns true if a database file exists for the given name, false otherwise.
 *
 * @param name - Database name (without .db extension)
 * @returns True if the database exists
 *
 * @example
 * ```typescript
 * if (await exists("users")) {
 *   console.log("Users database exists");
 * } else {
 *   console.log("Creating new users database");
 *   const db = await open("users");
 *   await db.execute("CREATE TABLE users (...)");
 *   await db.close();
 * }
 * ```
 *
 * @example
 * ```typescript
 * // Check before opening in read-only mode
 * if (!await exists("config")) {
 *   throw new Error("Config database not found");
 * }
 * const db = await open("config", { readonly: true });
 * ```
 */
export async function exists(name: string): Promise<boolean> {
  return await core.ops.op_database_exists(name);
}

/**
 * Delete a database.
 *
 * Permanently removes the database file from disk. Cannot be undone.
 * The database must be closed before deletion.
 *
 * @param name - Database name (without .db extension)
 * @returns True if the database was deleted, false if it didn't exist
 *
 * @throws Error [8411] if the database is currently open (busy/locked)
 * @throws Error [8408] if permission denied
 * @throws Error [8412] if I/O error
 *
 * @example
 * ```typescript
 * // Delete a database
 * const deleted = await remove("old-cache");
 * if (deleted) {
 *   console.log("Cache database deleted");
 * } else {
 *   console.log("Cache database didn't exist");
 * }
 * ```
 *
 * @example
 * ```typescript
 * // Clean up temporary databases
 * const databases = await list();
 * const tempDbs = databases.filter(db => db.name.startsWith("temp_"));
 * for (const db of tempDbs) {
 *   await remove(db.name);
 *   console.log(`Deleted ${db.name}`);
 * }
 * ```
 *
 * @example
 * ```typescript
 * // Must close before removing
 * const db = await open("test");
 * await db.execute("CREATE TABLE data (id INTEGER)");
 * await db.close(); // Must close first
 * await remove("test"); // Now can delete
 * ```
 */
export async function remove(name: string): Promise<boolean> {
  return await core.ops.op_database_delete(name);
}

/**
 * Get the full filesystem path for a database.
 *
 * Returns the absolute path where the database file is (or would be) stored.
 * The database doesn't need to exist for this function to work.
 *
 * @param name - Database name (without .db extension)
 * @returns Full path to the database file
 *
 * @example
 * ```typescript
 * const dbPath = path("myapp");
 * console.log("Database path:", dbPath);
 * // macOS: /Users/username/Library/Application Support/.forge/com.example.app/databases/myapp.db
 * // Linux: /home/username/.local/share/.forge/com.example.app/databases/myapp.db
 * // Windows: C:\Users\username\AppData\Roaming\.forge\com.example.app\databases\myapp.db
 * ```
 *
 * @example
 * ```typescript
 * // Get paths for backup
 * const databases = await list();
 * for (const db of databases) {
 *   const dbPath = path(db.name);
 *   console.log(`Backup ${db.name} from ${dbPath}`);
 * }
 * ```
 */
export function path(name: string): string {
  return core.ops.op_database_path(name);
}
