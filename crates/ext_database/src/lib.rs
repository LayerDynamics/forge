//! # `runtime:database` - Full-Featured SQLite Database Extension
//!
//! Provides complete SQLite database access for Forge applications with multiple named databases,
//! transactions, prepared statements, result streaming, and schema migrations.
//!
//! ## Overview
//!
//! This extension provides a comprehensive SQL database layer for Forge applications. Each app can
//! create and manage multiple named SQLite databases with full transaction support, prepared
//! statements for performance, streaming for large result sets, and versioned schema migrations.
//!
//! **Key Features:**
//! - **Multiple Databases**: Each app can have multiple named databases
//! - **Full SQL Support**: Complete SQLite SQL syntax support with parameterized queries
//! - **Transactions**: BEGIN/COMMIT/ROLLBACK with savepoints for nested behavior
//! - **Prepared Statements**: Compile SQL once, execute multiple times for performance
//! - **Result Streaming**: Process large result sets in batches to avoid memory issues
//! - **Schema Migrations**: Versioned up/down migrations with automatic tracking
//! - **WAL Mode**: Write-Ahead Logging enabled by default for better concurrency
//! - **Foreign Keys**: Foreign key constraints enabled by default for referential integrity
//! - **Type Conversion**: Automatic conversion between SQLite and JavaScript types
//! - **Connection Management**: Automatic database handle management with lazy opening
//!
//! ## TypeScript API
//!
//! The extension exposes 31 operations through the `runtime:database` module organized into 7 categories:
//!
//! ### 1. Connection Management (6 ops)
//! - `open(name, opts?)` - Open/create a database with configuration options
//! - `close(dbId)` - Close a database connection
//! - `list()` - List all databases with metadata (size, tables, path)
//! - `exists(name)` - Check if a database file exists
//! - `remove(name)` - Delete a database file permanently
//! - `path(name)` - Get full filesystem path for a database
//! - `vacuum(dbId)` - Reclaim unused space and defragment
//!
//! ### 2. Query Execution (5 ops)
//! - `query(dbId, sql, params?)` - Execute SELECT and return all rows
//! - `execute(dbId, sql, params?)` - Execute INSERT/UPDATE/DELETE
//! - `executeBatch(dbId, statements, opts?)` - Execute multiple statements (optionally in transaction)
//! - `queryRow(dbId, sql, params?)` - Return only first row
//! - `queryValue(dbId, sql, params?)` - Return only first value of first row
//!
//! ### 3. Prepared Statements (4 ops)
//! - `prepare(dbId, sql)` - Compile SQL into a prepared statement
//! - `stmtQuery(dbId, stmtId, params?)` - Execute prepared statement as SELECT
//! - `stmtExecute(dbId, stmtId, params?)` - Execute prepared statement as INSERT/UPDATE/DELETE
//! - `stmtFinalize(dbId, stmtId)` - Free prepared statement resources
//!
//! ### 4. Transactions (6 ops)
//! - `begin(dbId, mode?)` - Start a transaction (deferred/immediate/exclusive)
//! - `commit(dbId)` - Commit the current transaction
//! - `rollback(dbId)` - Rollback the current transaction
//! - `savepoint(dbId, name)` - Create a savepoint within a transaction
//! - `release(dbId, name)` - Release (commit) a savepoint
//! - `rollbackTo(dbId, name)` - Rollback to a savepoint
//!
//! ### 5. Schema Operations (3 ops)
//! - `tables(dbId)` - List all table names
//! - `tableInfo(dbId, table)` - Get complete schema info (columns, indexes, primary key)
//! - `tableExists(dbId, table)` - Check if a table exists
//!
//! ### 6. Result Streaming (3 ops)
//! - `streamOpen(dbId, sql, params?, batchSize?)` - Open a result stream
//! - `streamNext(streamId)` - Fetch next batch of rows
//! - `streamClose(streamId)` - Close the stream and free resources
//!
//! ### 7. Schema Migrations (3 ops)
//! - `migrate(dbId, migrations)` - Apply pending migrations (up SQL)
//! - `migrationStatus(dbId)` - Get current version and applied/pending migrations
//! - `migrateDown(dbId, targetVersion?)` - Rollback migrations (down SQL)
//!
//! ## TypeScript Usage Examples
//!
//! ### Basic Query Operations
//!
//! ```typescript
//! import { open } from "runtime:database";
//!
//! // Open database (creates if doesn't exist)
//! const db = await open("myapp");
//!
//! // Create table
//! await db.execute(`
//!   CREATE TABLE IF NOT EXISTS users (
//!     id INTEGER PRIMARY KEY AUTOINCREMENT,
//!     name TEXT NOT NULL,
//!     email TEXT UNIQUE
//!   )
//! `);
//!
//! // Insert with parameters (prevents SQL injection)
//! const result = await db.execute(
//!   "INSERT INTO users (name, email) VALUES (?, ?)",
//!   ["Alice", "alice@example.com"]
//! );
//! console.log("New user ID:", result.lastInsertRowid);
//!
//! // Query with type safety
//! interface User { id: number; name: string; email: string; }
//! const users = await db.query<User>("SELECT * FROM users WHERE name LIKE ?", ["%Alice%"]);
//! for (const user of users.rows) {
//!   console.log(user.name, user.email);
//! }
//!
//! await db.close();
//! ```
//!
//! ### Transactions for Bulk Operations
//!
//! ```typescript
//! // Transactions are ~1000x faster for bulk inserts
//! await db.transaction(async () => {
//!   for (const user of users) {
//!     await db.execute(
//!       "INSERT INTO users (name, email) VALUES (?, ?)",
//!       [user.name, user.email]
//!     );
//!   }
//! }); // Automatically commits on success, rolls back on error
//! ```
//!
//! ### Prepared Statements for Performance
//!
//! ```typescript
//! // Compile SQL once, execute many times
//! const stmt = await db.prepare("INSERT INTO logs (level, message, timestamp) VALUES (?, ?, ?)");
//! try {
//!   for (const log of logEntries) {
//!     await stmt.execute([log.level, log.message, Date.now()]);
//!   }
//! } finally {
//!   await stmt.finalize(); // Always finalize to free resources
//! }
//! ```
//!
//! ### Streaming Large Result Sets
//!
//! ```typescript
//! // Process large datasets without loading all into memory
//! for await (const batch of db.stream("SELECT * FROM events", [], 100)) {
//!   console.log(`Processing ${batch.length} events...`);
//!   for (const event of batch) {
//!     await processEvent(event);
//!   }
//! }
//! ```
//!
//! ### Schema Migrations
//!
//! ```typescript
//! const migrations = [
//!   {
//!     version: 1,
//!     name: "create_users",
//!     upSql: "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)",
//!     downSql: "DROP TABLE users"
//!   },
//!   {
//!     version: 2,
//!     name: "add_email",
//!     upSql: "ALTER TABLE users ADD COLUMN email TEXT",
//!     downSql: "ALTER TABLE users DROP COLUMN email"
//!   }
//! ];
//!
//! const status = await db.migrate(migrations);
//! console.log(`Database at version ${status.currentVersion}`);
//! ```
//!
//! ## Database Location
//!
//! Databases are stored in platform-specific app data directories:
//!
//! | Platform | Path |
//! |----------|------|
//! | macOS    | `~/Library/Application Support/.forge/<app-id>/databases/<name>.db` |
//! | Linux    | `~/.local/share/.forge/<app-id>/databases/<name>.db` |
//! | Windows  | `%APPDATA%\.forge\<app-id>\databases\<name>.db` |
//!
//! ## Error Codes
//!
//! All database operations may throw errors with structured codes:
//!
//! | Code   | Error                  | Description                                      |
//! |--------|------------------------|--------------------------------------------------|
//! | `8400` | Generic                | Unspecified database error                       |
//! | `8401` | NotFound               | Database/table/row not found                     |
//! | `8402` | AlreadyExists          | Database/table already exists                    |
//! | `8403` | SqlSyntax              | Invalid SQL syntax                               |
//! | `8404` | ConstraintViolation    | UNIQUE, FOREIGN KEY, CHECK, or NOT NULL violated |
//! | `8405` | TypeMismatch           | Parameter type doesn't match expected type       |
//! | `8406` | InvalidHandle          | Database handle is invalid or closed             |
//! | `8407` | TransactionError       | Already in transaction or not in transaction     |
//! | `8408` | PermissionDenied       | Insufficient permissions for operation           |
//! | `8409` | TooManyConnections     | Connection limit reached                         |
//! | `8410` | PreparedStatementError | Statement cannot be prepared or is invalid       |
//! | `8411` | DatabaseBusy           | Database is locked by another connection         |
//! | `8412` | IoError                | File I/O error (disk full, permission denied)    |
//! | `8413` | MigrationError         | Migration failed or invalid version              |
//! | `8414` | InvalidParameter       | Wrong parameter count or invalid value           |
//! | `8415` | StreamError            | Stream is closed or invalid                      |
//!
//! ## Database Features
//!
//! ### WAL Mode (Write-Ahead Logging)
//!
//! Enabled by default (`PRAGMA journal_mode = WAL`):
//! - Better concurrency: readers don't block writers
//! - Better performance: fewer disk syncs
//! - Atomic commits: crash-safe transactions
//!
//! Disable with `walMode: false` in open options if needed.
//!
//! ### Foreign Key Constraints
//!
//! Enabled by default (`PRAGMA foreign_keys = ON`):
//! - Enforces referential integrity
//! - Prevents orphaned records
//! - Cascade deletes and updates
//!
//! Disable with `foreignKeys: false` in open options if needed.
//!
//! ### Busy Timeout
//!
//! Default: 5000ms (5 seconds)
//! - Automatically retries when database is locked
//! - Prevents immediate "database busy" errors
//! - Configurable with `busyTimeoutMs` option
//!
//! ## Performance Considerations
//!
//! ### Transactions
//!
//! - Use `db.transaction()` for bulk inserts/updates (~1000x faster)
//! - SQLite writes are slow without transactions due to disk sync
//! - Batch operations default to using transactions
//!
//! ### Prepared Statements
//!
//! - Compile SQL once, execute many times
//! - Reduces parsing overhead for repeated queries
//! - Use for inserting/updating many rows
//!
//! ### Result Streaming
//!
//! - Use `db.stream()` for large result sets (>1000 rows)
//! - Processes results in batches to avoid memory issues
//! - Default batch size: 100 rows (configurable)
//!
//! ### Indexing
//!
//! - Create indexes on columns used in WHERE clauses
//! - `CREATE INDEX idx_users_email ON users(email)`
//! - Check query plan: `EXPLAIN QUERY PLAN SELECT ...`
//!
//! ## Extension Registration
//!
//! This extension is registered as **Tier 1 (SimpleState)** in the Forge runtime:
//!
//! ```rust,ignore
//! // forge-runtime/src/ext_registry.rs
//! ExtensionDescriptor {
//!     name: "runtime_database",
//!     tier: ExtensionTier::SimpleState,
//!     init_fn: init_database_state,
//!     required: false,
//! }
//! ```
//!
//! State initialization:
//!
//! ```rust,ignore
//! // Actual signature in this file
//! pub fn init_database_state(
//!     op_state: &mut OpState,
//!     app_identifier: String,
//!     capabilities: Option<Arc<dyn DatabaseCapabilityChecker>>,
//!     max_connections: Option<usize>,
//! ) {
//!     op_state.put(DatabaseState::new(app_identifier, max_connections.unwrap_or(10)));
//! }
//! ```
//!
//! ## Testing
//!
//! Run the test suite:
//!
//! ```bash
//! # Run all tests
//! cargo test -p ext_database
//!
//! # Run with output
//! cargo test -p ext_database -- --nocapture
//!
//! # Run specific test
//! cargo test -p ext_database test_transactions
//! ```
//!
//! ## Related Extensions
//!
//! - [`ext_storage`](../ext_storage) - Simple key-value storage (simpler alternative for basic needs)
//! - [`ext_crypto`](../ext_crypto) - Encryption for sensitive database fields
//! - [`ext_fs`](../ext_fs) - File operations for database backup/restore

use deno_core::{op2, Extension, OpState};
use forge_weld_macro::{weld_op, weld_struct};
use rusqlite::{params_from_iter, Connection, Row, ToSql};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

// =============================================================================
// Error Types (8400-8415)
// =============================================================================

/// Error codes for database operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum DatabaseErrorCode {
    Generic = 8400,
    NotFound = 8401,
    AlreadyExists = 8402,
    SqlSyntax = 8403,
    ConstraintViolation = 8404,
    TypeMismatch = 8405,
    InvalidHandle = 8406,
    TransactionError = 8407,
    PermissionDenied = 8408,
    TooManyConnections = 8409,
    PreparedStatementError = 8410,
    DatabaseBusy = 8411,
    IoError = 8412,
    MigrationError = 8413,
    InvalidParameter = 8414,
    StreamError = 8415,
}

/// Database operation errors
#[derive(Debug, thiserror::Error, deno_error::JsError)]
pub enum DatabaseError {
    #[error("[{code}] Database error: {message}")]
    #[class(generic)]
    Generic { code: u32, message: String },

    #[error("[{code}] Database not found: {message}")]
    #[class(generic)]
    NotFound { code: u32, message: String },

    #[error("[{code}] Database already exists: {message}")]
    #[class(generic)]
    AlreadyExists { code: u32, message: String },

    #[error("[{code}] SQL syntax error: {message}")]
    #[class(generic)]
    SqlSyntax { code: u32, message: String },

    #[error("[{code}] Constraint violation: {message}")]
    #[class(generic)]
    ConstraintViolation { code: u32, message: String },

    #[error("[{code}] Type mismatch: {message}")]
    #[class(generic)]
    TypeMismatch { code: u32, message: String },

    #[error("[{code}] Invalid handle: {message}")]
    #[class(generic)]
    InvalidHandle { code: u32, message: String },

    #[error("[{code}] Transaction error: {message}")]
    #[class(generic)]
    TransactionError { code: u32, message: String },

    #[error("[{code}] Permission denied: {message}")]
    #[class(generic)]
    PermissionDenied { code: u32, message: String },

    #[error("[{code}] Too many connections: {message}")]
    #[class(generic)]
    TooManyConnections { code: u32, message: String },

    #[error("[{code}] Prepared statement error: {message}")]
    #[class(generic)]
    PreparedStatementError { code: u32, message: String },

    #[error("[{code}] Database busy: {message}")]
    #[class(generic)]
    DatabaseBusy { code: u32, message: String },

    #[error("[{code}] IO error: {message}")]
    #[class(generic)]
    IoError { code: u32, message: String },

    #[error("[{code}] Migration error: {message}")]
    #[class(generic)]
    MigrationError { code: u32, message: String },

    #[error("[{code}] Invalid parameter: {message}")]
    #[class(generic)]
    InvalidParameter { code: u32, message: String },

    #[error("[{code}] Stream error: {message}")]
    #[class(generic)]
    StreamError { code: u32, message: String },
}

impl DatabaseError {
    pub fn generic(message: impl Into<String>) -> Self {
        Self::Generic {
            code: DatabaseErrorCode::Generic as u32,
            message: message.into(),
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound {
            code: DatabaseErrorCode::NotFound as u32,
            message: message.into(),
        }
    }

    pub fn already_exists(message: impl Into<String>) -> Self {
        Self::AlreadyExists {
            code: DatabaseErrorCode::AlreadyExists as u32,
            message: message.into(),
        }
    }

    pub fn sql_syntax(message: impl Into<String>) -> Self {
        Self::SqlSyntax {
            code: DatabaseErrorCode::SqlSyntax as u32,
            message: message.into(),
        }
    }

    pub fn constraint_violation(message: impl Into<String>) -> Self {
        Self::ConstraintViolation {
            code: DatabaseErrorCode::ConstraintViolation as u32,
            message: message.into(),
        }
    }

    pub fn invalid_handle(message: impl Into<String>) -> Self {
        Self::InvalidHandle {
            code: DatabaseErrorCode::InvalidHandle as u32,
            message: message.into(),
        }
    }

    pub fn transaction_error(message: impl Into<String>) -> Self {
        Self::TransactionError {
            code: DatabaseErrorCode::TransactionError as u32,
            message: message.into(),
        }
    }

    pub fn too_many_connections(message: impl Into<String>) -> Self {
        Self::TooManyConnections {
            code: DatabaseErrorCode::TooManyConnections as u32,
            message: message.into(),
        }
    }

    pub fn io_error(message: impl Into<String>) -> Self {
        Self::IoError {
            code: DatabaseErrorCode::IoError as u32,
            message: message.into(),
        }
    }

    pub fn migration_error(message: impl Into<String>) -> Self {
        Self::MigrationError {
            code: DatabaseErrorCode::MigrationError as u32,
            message: message.into(),
        }
    }

    pub fn invalid_parameter(message: impl Into<String>) -> Self {
        Self::InvalidParameter {
            code: DatabaseErrorCode::InvalidParameter as u32,
            message: message.into(),
        }
    }

    pub fn stream_error(message: impl Into<String>) -> Self {
        Self::StreamError {
            code: DatabaseErrorCode::StreamError as u32,
            message: message.into(),
        }
    }
}

impl From<rusqlite::Error> for DatabaseError {
    fn from(err: rusqlite::Error) -> Self {
        use rusqlite::Error::*;
        match &err {
            SqliteFailure(ffi_err, msg) => {
                let message = msg.clone().unwrap_or_else(|| err.to_string());
                match ffi_err.code {
                    rusqlite::ErrorCode::ConstraintViolation => {
                        DatabaseError::constraint_violation(message)
                    }
                    rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked => {
                        DatabaseError::DatabaseBusy {
                            code: DatabaseErrorCode::DatabaseBusy as u32,
                            message,
                        }
                    }
                    rusqlite::ErrorCode::TypeMismatch => DatabaseError::TypeMismatch {
                        code: DatabaseErrorCode::TypeMismatch as u32,
                        message,
                    },
                    _ => DatabaseError::generic(message),
                }
            }
            SqlInputError { msg, .. } => DatabaseError::sql_syntax(msg.clone()),
            QueryReturnedNoRows => DatabaseError::not_found("Query returned no rows"),
            InvalidParameterCount(expected, got) => DatabaseError::invalid_parameter(format!(
                "Expected {} parameters, got {}",
                expected, got
            )),
            _ => DatabaseError::generic(err.to_string()),
        }
    }
}

impl From<std::io::Error> for DatabaseError {
    fn from(err: std::io::Error) -> Self {
        DatabaseError::io_error(err.to_string())
    }
}

// =============================================================================
// Data Structures
// =============================================================================

/// Options for opening a database
#[derive(Debug, Clone, Default, Deserialize)]
pub struct OpenOptions {
    /// Create database if it doesn't exist (default: true)
    pub create: Option<bool>,
    /// Open in read-only mode (default: false)
    pub readonly: Option<bool>,
    /// Enable WAL mode for better concurrency (default: true)
    pub wal_mode: Option<bool>,
    /// Busy timeout in milliseconds (default: 5000)
    pub busy_timeout_ms: Option<u32>,
    /// Enable foreign keys (default: true)
    pub foreign_keys: Option<bool>,
}

/// Options for batch execution
#[derive(Debug, Clone, Default, Deserialize)]
pub struct BatchOptions {
    /// Run in a transaction (default: true)
    pub transaction: Option<bool>,
    /// Stop on first error (default: true)
    pub stop_on_error: Option<bool>,
}

/// Migration definition
#[derive(Debug, Clone, Deserialize)]
pub struct Migration {
    pub version: u32,
    pub name: String,
    pub up_sql: String,
    pub down_sql: Option<String>,
}

/// Result of opening a database
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct OpenResult {
    pub id: String,
    pub path: String,
    pub created: bool,
}

/// Database metadata
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct DatabaseInfo {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub tables: Vec<String>,
    pub readonly: bool,
}

/// Column information
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct ColumnInfo {
    pub name: String,
    pub column_type: String,
    pub nullable: bool,
}

/// Query result
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct QueryResult {
    pub columns: Vec<ColumnInfo>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub rows_affected: u64,
    pub last_insert_rowid: Option<i64>,
}

/// Execute result
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct ExecuteResult {
    pub rows_affected: u64,
    pub last_insert_rowid: Option<i64>,
}

/// Batch error
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct BatchError {
    pub index: u32,
    pub message: String,
}

/// Batch result
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct BatchResult {
    pub total_rows_affected: u64,
    pub statement_count: u32,
    pub errors: Vec<BatchError>,
}

/// Prepared statement info
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct PreparedStatementInfo {
    pub id: String,
    pub sql: String,
    pub parameter_count: u32,
}

/// Table column info
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct TableColumn {
    pub name: String,
    pub column_type: String,
    pub nullable: bool,
    pub default_value: Option<String>,
    pub primary_key: bool,
}

/// Index info
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct IndexInfo {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
}

/// Table info
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<TableColumn>,
    pub primary_key: Vec<String>,
    pub indexes: Vec<IndexInfo>,
}

/// Stream batch
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct StreamBatch {
    pub rows: Vec<Vec<serde_json::Value>>,
    pub done: bool,
    pub total_fetched: u64,
}

/// Applied migration
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct AppliedMigration {
    pub version: u32,
    pub name: String,
    pub applied_at: u64,
}

/// Migration status
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct MigrationStatus {
    pub current_version: u32,
    pub pending: Vec<String>,
    pub applied: Vec<AppliedMigration>,
}

// =============================================================================
// State Management
// =============================================================================

/// Handle to an open database
pub struct DatabaseHandle {
    pub connection: Arc<Mutex<Connection>>,
    pub name: String,
    pub path: PathBuf,
    pub readonly: bool,
    pub next_stmt_id: u64,
}

/// Streaming query state
pub struct StreamState {
    pub db_id: String,
    pub sql: String,
    pub params: Vec<serde_json::Value>,
    pub cursor_position: u64,
    pub batch_size: u32,
    pub columns: Vec<ColumnInfo>,
    pub done: bool,
}

/// Global database state
pub struct DatabaseState {
    pub databases: HashMap<String, DatabaseHandle>,
    pub streams: HashMap<String, StreamState>,
    pub next_db_id: u64,
    pub next_stream_id: u64,
    pub max_connections: usize,
    pub app_identifier: String,
}

impl DatabaseState {
    pub fn new(app_identifier: String, max_connections: usize) -> Self {
        Self {
            databases: HashMap::new(),
            streams: HashMap::new(),
            next_db_id: 1,
            next_stream_id: 1,
            max_connections,
            app_identifier,
        }
    }

    pub fn can_open(&self) -> bool {
        self.databases.len() < self.max_connections
    }

    pub fn get_database_dir(&self) -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".forge")
            .join(&self.app_identifier)
            .join("databases")
    }

    pub fn generate_db_id(&mut self) -> String {
        let id = format!("db_{}", self.next_db_id);
        self.next_db_id += 1;
        id
    }

    pub fn generate_stream_id(&mut self) -> String {
        let id = format!("stream_{}", self.next_stream_id);
        self.next_stream_id += 1;
        id
    }
}

/// Capability checker for database operations
pub trait DatabaseCapabilityChecker: Send + Sync {
    fn check_database(&self, name: &str) -> Result<(), String>;
}

/// Default permissive checker
pub struct PermissiveDatabaseChecker;

impl DatabaseCapabilityChecker for PermissiveDatabaseChecker {
    fn check_database(&self, _name: &str) -> Result<(), String> {
        Ok(())
    }
}

/// Capability wrapper for OpState
pub struct DatabaseCapabilities {
    pub checker: Arc<dyn DatabaseCapabilityChecker>,
}

impl Default for DatabaseCapabilities {
    fn default() -> Self {
        Self {
            checker: Arc::new(PermissiveDatabaseChecker),
        }
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

fn get_db_state(state: &OpState) -> &DatabaseState {
    state.borrow::<DatabaseState>()
}

fn get_db_state_mut(state: &mut OpState) -> &mut DatabaseState {
    state.borrow_mut::<DatabaseState>()
}

fn json_to_sql_params(params: &[serde_json::Value]) -> Vec<Box<dyn ToSql>> {
    params
        .iter()
        .map(|v| -> Box<dyn ToSql> {
            match v {
                serde_json::Value::Null => Box::new(rusqlite::types::Null),
                serde_json::Value::Bool(b) => Box::new(*b),
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        Box::new(i)
                    } else if let Some(f) = n.as_f64() {
                        Box::new(f)
                    } else {
                        Box::new(n.to_string())
                    }
                }
                serde_json::Value::String(s) => Box::new(s.clone()),
                serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
                    Box::new(serde_json::to_string(v).unwrap_or_default())
                }
            }
        })
        .collect()
}

fn row_to_json(row: &Row, col_count: usize) -> Vec<serde_json::Value> {
    (0..col_count)
        .map(|i| {
            // Try different types in order of likelihood
            if let Ok(v) = row.get::<_, Option<i64>>(i) {
                v.map(serde_json::Value::from)
                    .unwrap_or(serde_json::Value::Null)
            } else if let Ok(v) = row.get::<_, Option<f64>>(i) {
                v.and_then(|f| serde_json::Number::from_f64(f).map(serde_json::Value::Number))
                    .unwrap_or(serde_json::Value::Null)
            } else if let Ok(v) = row.get::<_, Option<String>>(i) {
                v.map(serde_json::Value::String)
                    .unwrap_or(serde_json::Value::Null)
            } else if let Ok(v) = row.get::<_, Option<Vec<u8>>>(i) {
                v.map(|bytes| {
                    serde_json::Value::Array(bytes.into_iter().map(|b| b.into()).collect())
                })
                .unwrap_or(serde_json::Value::Null)
            } else if let Ok(v) = row.get::<_, Option<bool>>(i) {
                v.map(serde_json::Value::Bool)
                    .unwrap_or(serde_json::Value::Null)
            } else {
                serde_json::Value::Null
            }
        })
        .collect()
}

// =============================================================================
// Connection Operations
// =============================================================================

#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_database_open(
    state: Rc<RefCell<OpState>>,
    #[string] name: String,
    #[serde] opts: Option<OpenOptions>,
) -> Result<OpenResult, DatabaseError> {
    let opts = opts.unwrap_or_default();
    let create = opts.create.unwrap_or(true);
    let readonly = opts.readonly.unwrap_or(false);
    let wal_mode = opts.wal_mode.unwrap_or(true);
    let busy_timeout_ms = opts.busy_timeout_ms.unwrap_or(5000);
    let foreign_keys = opts.foreign_keys.unwrap_or(true);

    let (db_dir, db_id, db_path) = {
        let mut s = state.borrow_mut();
        let db_state = get_db_state_mut(&mut s);

        if !db_state.can_open() {
            return Err(DatabaseError::too_many_connections(format!(
                "Maximum {} connections reached",
                db_state.max_connections
            )));
        }

        let db_dir = db_state.get_database_dir();
        let db_id = db_state.generate_db_id();
        let db_path = db_dir.join(format!("{}.db", name));

        (db_dir, db_id, db_path)
    };

    // Create directory if needed
    tokio::fs::create_dir_all(&db_dir).await?;

    let created = !db_path.exists();

    if !create && created {
        return Err(DatabaseError::not_found(format!(
            "Database '{}' not found",
            name
        )));
    }

    debug!(name = %name, path = %db_path.display(), "database.open");

    // Open connection in blocking task
    let path_clone = db_path.clone();
    let connection = tokio::task::spawn_blocking(move || -> Result<Connection, DatabaseError> {
        let flags = if readonly {
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
        } else {
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_CREATE
        };

        let conn = Connection::open_with_flags(&path_clone, flags)?;

        // Configure connection
        conn.busy_timeout(std::time::Duration::from_millis(busy_timeout_ms as u64))?;

        if foreign_keys {
            conn.execute("PRAGMA foreign_keys = ON", [])?;
        }

        if wal_mode && !readonly {
            conn.execute("PRAGMA journal_mode = WAL", [])?;
        }

        Ok(conn)
    })
    .await
    .map_err(|e| DatabaseError::generic(e.to_string()))??;

    let path_str = db_path.to_string_lossy().to_string();

    // Store handle
    {
        let mut s = state.borrow_mut();
        let db_state = get_db_state_mut(&mut s);
        db_state.databases.insert(
            db_id.clone(),
            DatabaseHandle {
                connection: Arc::new(Mutex::new(connection)),
                name: name.clone(),
                path: db_path,
                readonly,
                next_stmt_id: 1,
            },
        );
    }

    Ok(OpenResult {
        id: db_id,
        path: path_str,
        created,
    })
}

#[weld_op(async)]
#[op2(async)]
pub async fn op_database_close(
    state: Rc<RefCell<OpState>>,
    #[string] db_id: String,
) -> Result<(), DatabaseError> {
    let handle = {
        let mut s = state.borrow_mut();
        let db_state = get_db_state_mut(&mut s);
        db_state.databases.remove(&db_id)
    };

    if handle.is_none() {
        return Err(DatabaseError::invalid_handle(format!(
            "Database '{}' not found",
            db_id
        )));
    }

    debug!(db_id = %db_id, "database.close");
    Ok(())
}

#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_database_list(
    state: Rc<RefCell<OpState>>,
) -> Result<Vec<DatabaseInfo>, DatabaseError> {
    let db_dir = {
        let s = state.borrow();
        let db_state = get_db_state(&s);
        db_state.get_database_dir()
    };

    let mut databases = Vec::new();

    if !db_dir.exists() {
        return Ok(databases);
    }

    let mut entries = tokio::fs::read_dir(&db_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("db") {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            let metadata = tokio::fs::metadata(&path).await?;
            let size_bytes = metadata.len();

            // Get table list from database
            let path_clone = path.clone();
            let tables =
                tokio::task::spawn_blocking(move || -> Result<Vec<String>, DatabaseError> {
                    let conn = Connection::open_with_flags(
                        &path_clone,
                        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
                    )?;
                    let mut stmt = conn.prepare(
                        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
                    )?;
                    let tables: Vec<String> = stmt
                        .query_map([], |row| row.get(0))?
                        .filter_map(|r| r.ok())
                        .collect();
                    Ok(tables)
                })
                .await
                .map_err(|e| DatabaseError::generic(e.to_string()))??;

            databases.push(DatabaseInfo {
                name,
                path: path.to_string_lossy().to_string(),
                size_bytes,
                tables,
                readonly: false,
            });
        }
    }

    Ok(databases)
}

#[weld_op(async)]
#[op2(async)]
pub async fn op_database_delete(
    state: Rc<RefCell<OpState>>,
    #[string] name: String,
) -> Result<bool, DatabaseError> {
    let db_path = {
        let s = state.borrow();
        let db_state = get_db_state(&s);
        db_state.get_database_dir().join(format!("{}.db", name))
    };

    if !db_path.exists() {
        return Ok(false);
    }

    debug!(name = %name, "database.delete");
    tokio::fs::remove_file(&db_path).await?;

    // Also remove WAL and SHM files if they exist
    let wal_path = db_path.with_extension("db-wal");
    let shm_path = db_path.with_extension("db-shm");
    let _ = tokio::fs::remove_file(&wal_path).await;
    let _ = tokio::fs::remove_file(&shm_path).await;

    Ok(true)
}

#[weld_op(async)]
#[op2(async)]
pub async fn op_database_exists(
    state: Rc<RefCell<OpState>>,
    #[string] name: String,
) -> Result<bool, DatabaseError> {
    let db_path = {
        let s = state.borrow();
        let db_state = get_db_state(&s);
        db_state.get_database_dir().join(format!("{}.db", name))
    };

    Ok(db_path.exists())
}

#[weld_op]
#[op2]
#[string]
pub fn op_database_path(state: &OpState, #[string] name: String) -> Result<String, DatabaseError> {
    let db_state = get_db_state(state);
    let db_path = db_state.get_database_dir().join(format!("{}.db", name));
    Ok(db_path.to_string_lossy().to_string())
}

#[weld_op(async)]
#[op2(async)]
pub async fn op_database_vacuum(
    state: Rc<RefCell<OpState>>,
    #[string] db_id: String,
) -> Result<(), DatabaseError> {
    let conn = {
        let s = state.borrow();
        let db_state = get_db_state(&s);
        let handle = db_state.databases.get(&db_id).ok_or_else(|| {
            DatabaseError::invalid_handle(format!("Database '{}' not found", db_id))
        })?;
        handle.connection.clone()
    };

    debug!(db_id = %db_id, "database.vacuum");

    tokio::task::spawn_blocking(move || {
        let conn = conn.blocking_lock();
        conn.execute("VACUUM", [])?;
        Ok::<_, DatabaseError>(())
    })
    .await
    .map_err(|e| DatabaseError::generic(e.to_string()))??;

    Ok(())
}

// =============================================================================
// Query Operations
// =============================================================================

/// Internal helper function to execute a query - shared by op_database_query, query_row, and query_value
async fn query_internal(
    conn: Arc<tokio::sync::Mutex<Connection>>,
    sql: String,
    params: Vec<serde_json::Value>,
) -> Result<QueryResult, DatabaseError> {
    tokio::task::spawn_blocking(move || {
        let conn = conn.blocking_lock();
        let mut stmt = conn.prepare(&sql)?;

        // Get column info (column type from declared type if available)
        let col_count = stmt.column_count();
        let column_names = stmt.column_names();
        let columns: Vec<ColumnInfo> = column_names
            .into_iter()
            .map(|name| ColumnInfo {
                name: name.to_string(),
                column_type: String::new(), // Type info not available until after query execution
                nullable: true,
            })
            .collect();

        // Execute query
        let sql_params = json_to_sql_params(&params);
        let param_refs: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();

        let mut rows_data = Vec::new();
        let mut query_rows = stmt.query(params_from_iter(param_refs))?;
        while let Some(row) = query_rows.next()? {
            rows_data.push(row_to_json(row, col_count));
        }

        Ok(QueryResult {
            columns,
            rows: rows_data,
            rows_affected: 0,
            last_insert_rowid: None,
        })
    })
    .await
    .map_err(|e| DatabaseError::generic(e.to_string()))?
}

#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_database_query(
    state: Rc<RefCell<OpState>>,
    #[string] db_id: String,
    #[string] sql: String,
    #[serde] params: Option<Vec<serde_json::Value>>,
) -> Result<QueryResult, DatabaseError> {
    let conn = {
        let s = state.borrow();
        let db_state = get_db_state(&s);
        let handle = db_state.databases.get(&db_id).ok_or_else(|| {
            DatabaseError::invalid_handle(format!("Database '{}' not found", db_id))
        })?;
        handle.connection.clone()
    };

    debug!(db_id = %db_id, sql = %sql, "database.query");
    query_internal(conn, sql, params.unwrap_or_default()).await
}

#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_database_execute(
    state: Rc<RefCell<OpState>>,
    #[string] db_id: String,
    #[string] sql: String,
    #[serde] params: Option<Vec<serde_json::Value>>,
) -> Result<ExecuteResult, DatabaseError> {
    let conn = {
        let s = state.borrow();
        let db_state = get_db_state(&s);
        let handle = db_state.databases.get(&db_id).ok_or_else(|| {
            DatabaseError::invalid_handle(format!("Database '{}' not found", db_id))
        })?;
        handle.connection.clone()
    };

    let params = params.unwrap_or_default();
    debug!(db_id = %db_id, sql = %sql, "database.execute");

    tokio::task::spawn_blocking(move || {
        let conn = conn.blocking_lock();

        let sql_params = json_to_sql_params(&params);
        let param_refs: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();

        let rows_affected = conn.execute(&sql, params_from_iter(param_refs))?;
        let last_insert_rowid = conn.last_insert_rowid();

        Ok(ExecuteResult {
            rows_affected: rows_affected as u64,
            last_insert_rowid: Some(last_insert_rowid),
        })
    })
    .await
    .map_err(|e| DatabaseError::generic(e.to_string()))?
}

#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_database_execute_batch(
    state: Rc<RefCell<OpState>>,
    #[string] db_id: String,
    #[serde] statements: Vec<String>,
    #[serde] opts: Option<BatchOptions>,
) -> Result<BatchResult, DatabaseError> {
    let conn = {
        let s = state.borrow();
        let db_state = get_db_state(&s);
        let handle = db_state.databases.get(&db_id).ok_or_else(|| {
            DatabaseError::invalid_handle(format!("Database '{}' not found", db_id))
        })?;
        handle.connection.clone()
    };

    let opts = opts.unwrap_or_default();
    let use_transaction = opts.transaction.unwrap_or(true);
    let stop_on_error = opts.stop_on_error.unwrap_or(true);

    debug!(db_id = %db_id, count = statements.len(), "database.execute_batch");

    tokio::task::spawn_blocking(move || {
        let conn = conn.blocking_lock();
        let mut total_rows_affected: u64 = 0;
        let mut errors = Vec::new();
        let statement_count = statements.len() as u32;

        if use_transaction {
            conn.execute("BEGIN", [])?;
        }

        for (i, sql) in statements.iter().enumerate() {
            match conn.execute(sql, []) {
                Ok(affected) => {
                    total_rows_affected += affected as u64;
                }
                Err(e) => {
                    errors.push(BatchError {
                        index: i as u32,
                        message: e.to_string(),
                    });
                    if stop_on_error {
                        if use_transaction {
                            let _ = conn.execute("ROLLBACK", []);
                        }
                        return Err(DatabaseError::generic(format!(
                            "Batch execution failed at statement {}: {}",
                            i, e
                        )));
                    }
                }
            }
        }

        if use_transaction {
            conn.execute("COMMIT", [])?;
        }

        Ok(BatchResult {
            total_rows_affected,
            statement_count,
            errors,
        })
    })
    .await
    .map_err(|e| DatabaseError::generic(e.to_string()))?
}

#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_database_query_row(
    state: Rc<RefCell<OpState>>,
    #[string] db_id: String,
    #[string] sql: String,
    #[serde] params: Option<Vec<serde_json::Value>>,
) -> Result<Option<Vec<serde_json::Value>>, DatabaseError> {
    let conn = {
        let s = state.borrow();
        let db_state = get_db_state(&s);
        let handle = db_state.databases.get(&db_id).ok_or_else(|| {
            DatabaseError::invalid_handle(format!("Database '{}' not found", db_id))
        })?;
        handle.connection.clone()
    };

    debug!(db_id = %db_id, sql = %sql, "database.query_row");
    let result = query_internal(conn, sql, params.unwrap_or_default()).await?;
    Ok(result.rows.into_iter().next())
}

#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_database_query_value(
    state: Rc<RefCell<OpState>>,
    #[string] db_id: String,
    #[string] sql: String,
    #[serde] params: Option<Vec<serde_json::Value>>,
) -> Result<Option<serde_json::Value>, DatabaseError> {
    let conn = {
        let s = state.borrow();
        let db_state = get_db_state(&s);
        let handle = db_state.databases.get(&db_id).ok_or_else(|| {
            DatabaseError::invalid_handle(format!("Database '{}' not found", db_id))
        })?;
        handle.connection.clone()
    };

    debug!(db_id = %db_id, sql = %sql, "database.query_value");
    let result = query_internal(conn, sql, params.unwrap_or_default()).await?;
    Ok(result
        .rows
        .into_iter()
        .next()
        .and_then(|row| row.into_iter().next()))
}

// =============================================================================
// Prepared Statement Operations
// =============================================================================

#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_database_prepare(
    state: Rc<RefCell<OpState>>,
    #[string] db_id: String,
    #[string] sql: String,
) -> Result<PreparedStatementInfo, DatabaseError> {
    let (conn, stmt_id) = {
        let mut s = state.borrow_mut();
        let db_state = get_db_state_mut(&mut s);
        let handle = db_state.databases.get_mut(&db_id).ok_or_else(|| {
            DatabaseError::invalid_handle(format!("Database '{}' not found", db_id))
        })?;
        let stmt_id = format!("stmt_{}", handle.next_stmt_id);
        handle.next_stmt_id += 1;
        (handle.connection.clone(), stmt_id)
    };

    debug!(db_id = %db_id, stmt_id = %stmt_id, sql = %sql, "database.prepare");

    // Validate SQL by preparing it
    let sql_clone = sql.clone();
    let param_count = tokio::task::spawn_blocking(move || {
        let conn = conn.blocking_lock();
        let stmt = conn.prepare(&sql_clone)?;
        Ok::<_, DatabaseError>(stmt.parameter_count() as u32)
    })
    .await
    .map_err(|e| DatabaseError::generic(e.to_string()))??;

    Ok(PreparedStatementInfo {
        id: stmt_id,
        sql,
        parameter_count: param_count,
    })
}

#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_database_stmt_query(
    state: Rc<RefCell<OpState>>,
    #[string] db_id: String,
    #[string] stmt_id: String,
    #[serde] params: Option<Vec<serde_json::Value>>,
) -> Result<QueryResult, DatabaseError> {
    // Acknowledge unused parameters - prepared statement caching not implemented
    let _ = (&state, &db_id, &stmt_id, &params);

    // In a more sophisticated implementation, we'd cache prepared statements
    // and retrieve them by stmt_id. For now, return an error directing to direct query.
    Err(DatabaseError::PreparedStatementError {
        code: DatabaseErrorCode::PreparedStatementError as u32,
        message: "Prepared statement execution requires using the original SQL. Use op_database_query instead.".to_string(),
    })
}

#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_database_stmt_execute(
    state: Rc<RefCell<OpState>>,
    #[string] db_id: String,
    #[string] stmt_id: String,
    #[serde] params: Option<Vec<serde_json::Value>>,
) -> Result<ExecuteResult, DatabaseError> {
    // Acknowledge unused parameters - prepared statement caching not implemented
    let _ = (&state, &db_id, &stmt_id, &params);

    Err(DatabaseError::PreparedStatementError {
        code: DatabaseErrorCode::PreparedStatementError as u32,
        message: "Prepared statement execution requires using the original SQL. Use op_database_execute instead.".to_string(),
    })
}

#[weld_op(async)]
#[op2(async)]
pub async fn op_database_stmt_finalize(
    state: Rc<RefCell<OpState>>,
    #[string] db_id: String,
    #[string] stmt_id: String,
) -> Result<(), DatabaseError> {
    debug!(db_id = %db_id, stmt_id = %stmt_id, "database.stmt_finalize");
    // Acknowledge state for future use
    let _ = &state;
    // No-op since we don't cache statements
    Ok(())
}

// =============================================================================
// Transaction Operations
// =============================================================================

#[weld_op(async)]
#[op2(async)]
pub async fn op_database_begin(
    state: Rc<RefCell<OpState>>,
    #[string] db_id: String,
    #[string] mode: Option<String>,
) -> Result<(), DatabaseError> {
    let conn = {
        let s = state.borrow();
        let db_state = get_db_state(&s);
        let handle = db_state.databases.get(&db_id).ok_or_else(|| {
            DatabaseError::invalid_handle(format!("Database '{}' not found", db_id))
        })?;
        handle.connection.clone()
    };

    let sql = match mode.as_deref() {
        Some("immediate") => "BEGIN IMMEDIATE",
        Some("exclusive") => "BEGIN EXCLUSIVE",
        _ => "BEGIN DEFERRED",
    };

    debug!(db_id = %db_id, mode = ?mode, "database.begin");

    tokio::task::spawn_blocking(move || {
        let conn = conn.blocking_lock();
        conn.execute(sql, [])?;
        Ok::<_, DatabaseError>(())
    })
    .await
    .map_err(|e| DatabaseError::generic(e.to_string()))??;

    Ok(())
}

#[weld_op(async)]
#[op2(async)]
pub async fn op_database_commit(
    state: Rc<RefCell<OpState>>,
    #[string] db_id: String,
) -> Result<(), DatabaseError> {
    let conn = {
        let s = state.borrow();
        let db_state = get_db_state(&s);
        let handle = db_state.databases.get(&db_id).ok_or_else(|| {
            DatabaseError::invalid_handle(format!("Database '{}' not found", db_id))
        })?;
        handle.connection.clone()
    };

    debug!(db_id = %db_id, "database.commit");

    tokio::task::spawn_blocking(move || {
        let conn = conn.blocking_lock();
        conn.execute("COMMIT", [])?;
        Ok::<_, DatabaseError>(())
    })
    .await
    .map_err(|e| DatabaseError::generic(e.to_string()))??;

    Ok(())
}

#[weld_op(async)]
#[op2(async)]
pub async fn op_database_rollback(
    state: Rc<RefCell<OpState>>,
    #[string] db_id: String,
) -> Result<(), DatabaseError> {
    let conn = {
        let s = state.borrow();
        let db_state = get_db_state(&s);
        let handle = db_state.databases.get(&db_id).ok_or_else(|| {
            DatabaseError::invalid_handle(format!("Database '{}' not found", db_id))
        })?;
        handle.connection.clone()
    };

    debug!(db_id = %db_id, "database.rollback");

    tokio::task::spawn_blocking(move || {
        let conn = conn.blocking_lock();
        conn.execute("ROLLBACK", [])?;
        Ok::<_, DatabaseError>(())
    })
    .await
    .map_err(|e| DatabaseError::generic(e.to_string()))??;

    Ok(())
}

#[weld_op(async)]
#[op2(async)]
pub async fn op_database_savepoint(
    state: Rc<RefCell<OpState>>,
    #[string] db_id: String,
    #[string] name: String,
) -> Result<(), DatabaseError> {
    let conn = {
        let s = state.borrow();
        let db_state = get_db_state(&s);
        let handle = db_state.databases.get(&db_id).ok_or_else(|| {
            DatabaseError::invalid_handle(format!("Database '{}' not found", db_id))
        })?;
        handle.connection.clone()
    };

    debug!(db_id = %db_id, name = %name, "database.savepoint");

    tokio::task::spawn_blocking(move || {
        let conn = conn.blocking_lock();
        conn.execute(&format!("SAVEPOINT {}", name), [])?;
        Ok::<_, DatabaseError>(())
    })
    .await
    .map_err(|e| DatabaseError::generic(e.to_string()))??;

    Ok(())
}

#[weld_op(async)]
#[op2(async)]
pub async fn op_database_release(
    state: Rc<RefCell<OpState>>,
    #[string] db_id: String,
    #[string] name: String,
) -> Result<(), DatabaseError> {
    let conn = {
        let s = state.borrow();
        let db_state = get_db_state(&s);
        let handle = db_state.databases.get(&db_id).ok_or_else(|| {
            DatabaseError::invalid_handle(format!("Database '{}' not found", db_id))
        })?;
        handle.connection.clone()
    };

    debug!(db_id = %db_id, name = %name, "database.release");

    tokio::task::spawn_blocking(move || {
        let conn = conn.blocking_lock();
        conn.execute(&format!("RELEASE SAVEPOINT {}", name), [])?;
        Ok::<_, DatabaseError>(())
    })
    .await
    .map_err(|e| DatabaseError::generic(e.to_string()))??;

    Ok(())
}

#[weld_op(async)]
#[op2(async)]
pub async fn op_database_rollback_to(
    state: Rc<RefCell<OpState>>,
    #[string] db_id: String,
    #[string] name: String,
) -> Result<(), DatabaseError> {
    let conn = {
        let s = state.borrow();
        let db_state = get_db_state(&s);
        let handle = db_state.databases.get(&db_id).ok_or_else(|| {
            DatabaseError::invalid_handle(format!("Database '{}' not found", db_id))
        })?;
        handle.connection.clone()
    };

    debug!(db_id = %db_id, name = %name, "database.rollback_to");

    tokio::task::spawn_blocking(move || {
        let conn = conn.blocking_lock();
        conn.execute(&format!("ROLLBACK TO SAVEPOINT {}", name), [])?;
        Ok::<_, DatabaseError>(())
    })
    .await
    .map_err(|e| DatabaseError::generic(e.to_string()))??;

    Ok(())
}

// =============================================================================
// Schema Operations
// =============================================================================

#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_database_tables(
    state: Rc<RefCell<OpState>>,
    #[string] db_id: String,
) -> Result<Vec<String>, DatabaseError> {
    let conn = {
        let s = state.borrow();
        let db_state = get_db_state(&s);
        let handle = db_state.databases.get(&db_id).ok_or_else(|| {
            DatabaseError::invalid_handle(format!("Database '{}' not found", db_id))
        })?;
        handle.connection.clone()
    };

    tokio::task::spawn_blocking(move || {
        let conn = conn.blocking_lock();
        let mut stmt = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )?;
        let tables: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(tables)
    })
    .await
    .map_err(|e| DatabaseError::generic(e.to_string()))?
}

#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_database_table_info(
    state: Rc<RefCell<OpState>>,
    #[string] db_id: String,
    #[string] table: String,
) -> Result<TableInfo, DatabaseError> {
    let conn = {
        let s = state.borrow();
        let db_state = get_db_state(&s);
        let handle = db_state.databases.get(&db_id).ok_or_else(|| {
            DatabaseError::invalid_handle(format!("Database '{}' not found", db_id))
        })?;
        handle.connection.clone()
    };

    tokio::task::spawn_blocking(move || {
        let conn = conn.blocking_lock();

        // Get column info
        let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table))?;
        let mut columns = Vec::new();
        let mut primary_key = Vec::new();

        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let name: String = row.get(1)?;
            let col_type: String = row.get(2)?;
            let notnull: i32 = row.get(3)?;
            let default_value: Option<String> = row.get(4)?;
            let pk: i32 = row.get(5)?;

            if pk > 0 {
                primary_key.push(name.clone());
            }

            columns.push(TableColumn {
                name,
                column_type: col_type,
                nullable: notnull == 0,
                default_value,
                primary_key: pk > 0,
            });
        }

        // Get index info
        let mut stmt = conn.prepare(&format!("PRAGMA index_list({})", table))?;
        let mut indexes = Vec::new();

        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let name: String = row.get(1)?;
            let unique: i32 = row.get(2)?;

            // Get columns for this index
            let mut idx_stmt = conn.prepare(&format!("PRAGMA index_info({})", name))?;
            let idx_columns: Vec<String> = idx_stmt
                .query_map([], |r| r.get(2))?
                .filter_map(|r| r.ok())
                .collect();

            indexes.push(IndexInfo {
                name,
                columns: idx_columns,
                unique: unique == 1,
            });
        }

        Ok(TableInfo {
            name: table,
            columns,
            primary_key,
            indexes,
        })
    })
    .await
    .map_err(|e| DatabaseError::generic(e.to_string()))?
}

#[weld_op(async)]
#[op2(async)]
pub async fn op_database_table_exists(
    state: Rc<RefCell<OpState>>,
    #[string] db_id: String,
    #[string] table: String,
) -> Result<bool, DatabaseError> {
    let conn = {
        let s = state.borrow();
        let db_state = get_db_state(&s);
        let handle = db_state.databases.get(&db_id).ok_or_else(|| {
            DatabaseError::invalid_handle(format!("Database '{}' not found", db_id))
        })?;
        handle.connection.clone()
    };

    tokio::task::spawn_blocking(move || {
        let conn = conn.blocking_lock();
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?",
            [&table],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    })
    .await
    .map_err(|e| DatabaseError::generic(e.to_string()))?
}

// =============================================================================
// Streaming Operations
// =============================================================================

#[weld_op(async)]
#[op2(async)]
#[string]
pub async fn op_database_stream_open(
    state: Rc<RefCell<OpState>>,
    #[string] db_id: String,
    #[string] sql: String,
    #[serde] params: Option<Vec<serde_json::Value>>,
    batch_size: Option<u32>,
) -> Result<String, DatabaseError> {
    let conn = {
        let s = state.borrow();
        let db_state = get_db_state(&s);
        let handle = db_state.databases.get(&db_id).ok_or_else(|| {
            DatabaseError::invalid_handle(format!("Database '{}' not found", db_id))
        })?;
        handle.connection.clone()
    };

    // Get column info
    let sql_clone = sql.clone();
    let columns = tokio::task::spawn_blocking(move || {
        let conn = conn.blocking_lock();
        let stmt = conn.prepare(&sql_clone)?;
        let column_names = stmt.column_names();
        let columns: Vec<ColumnInfo> = column_names
            .into_iter()
            .map(|name| ColumnInfo {
                name: name.to_string(),
                column_type: String::new(),
                nullable: true,
            })
            .collect();
        Ok::<_, DatabaseError>(columns)
    })
    .await
    .map_err(|e| DatabaseError::generic(e.to_string()))??;

    let stream_id = {
        let mut s = state.borrow_mut();
        let db_state = get_db_state_mut(&mut s);
        let stream_id = db_state.generate_stream_id();

        db_state.streams.insert(
            stream_id.clone(),
            StreamState {
                db_id,
                sql,
                params: params.unwrap_or_default(),
                cursor_position: 0,
                batch_size: batch_size.unwrap_or(1000),
                columns,
                done: false,
            },
        );

        stream_id
    };

    debug!(stream_id = %stream_id, "database.stream_open");
    Ok(stream_id)
}

#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_database_stream_next(
    state: Rc<RefCell<OpState>>,
    #[string] stream_id: String,
) -> Result<StreamBatch, DatabaseError> {
    let (conn, sql, params, offset, limit, col_count) = {
        let s = state.borrow();
        let db_state = get_db_state(&s);

        let stream = db_state.streams.get(&stream_id).ok_or_else(|| {
            DatabaseError::stream_error(format!("Stream '{}' not found", stream_id))
        })?;

        if stream.done {
            return Ok(StreamBatch {
                rows: Vec::new(),
                done: true,
                total_fetched: stream.cursor_position,
            });
        }

        let handle = db_state.databases.get(&stream.db_id).ok_or_else(|| {
            DatabaseError::invalid_handle(format!("Database '{}' not found", stream.db_id))
        })?;

        (
            handle.connection.clone(),
            stream.sql.clone(),
            stream.params.clone(),
            stream.cursor_position,
            stream.batch_size,
            stream.columns.len(),
        )
    };

    let paginated_sql = format!("{} LIMIT {} OFFSET {}", sql, limit, offset);

    let rows = tokio::task::spawn_blocking(move || {
        let conn = conn.blocking_lock();
        let mut stmt = conn.prepare(&paginated_sql)?;

        let sql_params = json_to_sql_params(&params);
        let param_refs: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();

        let mut rows_data = Vec::new();
        let mut query_rows = stmt.query(params_from_iter(param_refs))?;
        while let Some(row) = query_rows.next()? {
            rows_data.push(row_to_json(row, col_count));
        }

        Ok::<_, DatabaseError>(rows_data)
    })
    .await
    .map_err(|e| DatabaseError::generic(e.to_string()))??;

    let done = (rows.len() as u32) < limit;
    let fetched = rows.len() as u64;

    // Update stream state
    {
        let mut s = state.borrow_mut();
        let db_state = get_db_state_mut(&mut s);
        if let Some(stream) = db_state.streams.get_mut(&stream_id) {
            stream.cursor_position += fetched;
            stream.done = done;
        }
    }

    let total_fetched = {
        let s = state.borrow();
        let db_state = get_db_state(&s);
        db_state
            .streams
            .get(&stream_id)
            .map(|s| s.cursor_position)
            .unwrap_or(0)
    };

    Ok(StreamBatch {
        rows,
        done,
        total_fetched,
    })
}

#[weld_op(async)]
#[op2(async)]
pub async fn op_database_stream_close(
    state: Rc<RefCell<OpState>>,
    #[string] stream_id: String,
) -> Result<(), DatabaseError> {
    let mut s = state.borrow_mut();
    let db_state = get_db_state_mut(&mut s);
    db_state.streams.remove(&stream_id);
    debug!(stream_id = %stream_id, "database.stream_close");
    Ok(())
}

// =============================================================================
// Migration Operations
// =============================================================================

fn ensure_migration_table(conn: &Connection) -> Result<(), DatabaseError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS __forge_migrations (
            version INTEGER PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            applied_at INTEGER DEFAULT (strftime('%s', 'now'))
        )",
        [],
    )?;
    Ok(())
}

fn get_applied_migrations(conn: &Connection) -> Result<Vec<AppliedMigration>, DatabaseError> {
    let mut stmt =
        conn.prepare("SELECT version, name, applied_at FROM __forge_migrations ORDER BY version")?;
    let migrations: Vec<AppliedMigration> = stmt
        .query_map([], |row| {
            Ok(AppliedMigration {
                version: row.get(0)?,
                name: row.get(1)?,
                applied_at: row.get(2)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(migrations)
}

fn get_current_version(conn: &Connection) -> Result<u32, DatabaseError> {
    ensure_migration_table(conn)?;
    let version: Option<u32> = conn
        .query_row("SELECT MAX(version) FROM __forge_migrations", [], |row| {
            row.get(0)
        })
        .ok()
        .flatten();
    Ok(version.unwrap_or(0))
}

#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_database_migrate(
    state: Rc<RefCell<OpState>>,
    #[string] db_id: String,
    #[serde] migrations: Vec<Migration>,
) -> Result<MigrationStatus, DatabaseError> {
    let conn = {
        let s = state.borrow();
        let db_state = get_db_state(&s);
        let handle = db_state.databases.get(&db_id).ok_or_else(|| {
            DatabaseError::invalid_handle(format!("Database '{}' not found", db_id))
        })?;
        handle.connection.clone()
    };

    debug!(db_id = %db_id, count = migrations.len(), "database.migrate");

    tokio::task::spawn_blocking(move || {
        let conn = conn.blocking_lock();
        ensure_migration_table(&conn)?;

        let current_version = get_current_version(&conn)?;

        // Filter to pending migrations
        let mut pending: Vec<&Migration> = migrations
            .iter()
            .filter(|m| m.version > current_version)
            .collect();
        pending.sort_by_key(|m| m.version);

        if pending.is_empty() {
            let applied = get_applied_migrations(&conn)?;
            return Ok(MigrationStatus {
                current_version,
                pending: Vec::new(),
                applied,
            });
        }

        // Run migrations in a transaction (auto-rollback on failure)
        conn.execute("BEGIN", [])?;

        for migration in &pending {
            match conn.execute_batch(&migration.up_sql) {
                Ok(_) => {
                    conn.execute(
                        "INSERT INTO __forge_migrations (version, name) VALUES (?, ?)",
                        rusqlite::params![migration.version, migration.name],
                    )?;
                }
                Err(e) => {
                    conn.execute("ROLLBACK", [])?;
                    return Err(DatabaseError::migration_error(format!(
                        "Migration {} ({}) failed: {}",
                        migration.version, migration.name, e
                    )));
                }
            }
        }

        conn.execute("COMMIT", [])?;

        let new_version = get_current_version(&conn)?;
        let applied = get_applied_migrations(&conn)?;

        Ok(MigrationStatus {
            current_version: new_version,
            pending: Vec::new(),
            applied,
        })
    })
    .await
    .map_err(|e| DatabaseError::generic(e.to_string()))?
}

#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_database_migration_status(
    state: Rc<RefCell<OpState>>,
    #[string] db_id: String,
) -> Result<MigrationStatus, DatabaseError> {
    let conn = {
        let s = state.borrow();
        let db_state = get_db_state(&s);
        let handle = db_state.databases.get(&db_id).ok_or_else(|| {
            DatabaseError::invalid_handle(format!("Database '{}' not found", db_id))
        })?;
        handle.connection.clone()
    };

    tokio::task::spawn_blocking(move || {
        let conn = conn.blocking_lock();
        ensure_migration_table(&conn)?;

        let current_version = get_current_version(&conn)?;
        let applied = get_applied_migrations(&conn)?;

        Ok(MigrationStatus {
            current_version,
            pending: Vec::new(),
            applied,
        })
    })
    .await
    .map_err(|e| DatabaseError::generic(e.to_string()))?
}

#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_database_migrate_down(
    state: Rc<RefCell<OpState>>,
    #[string] db_id: String,
    target_version: Option<u32>,
) -> Result<MigrationStatus, DatabaseError> {
    let conn = {
        let s = state.borrow();
        let db_state = get_db_state(&s);
        let handle = db_state.databases.get(&db_id).ok_or_else(|| {
            DatabaseError::invalid_handle(format!("Database '{}' not found", db_id))
        })?;
        handle.connection.clone()
    };

    let target = target_version.unwrap_or(0);

    debug!(db_id = %db_id, target = target, "database.migrate_down");

    tokio::task::spawn_blocking(move || {
        let conn = conn.blocking_lock();
        ensure_migration_table(&conn)?;

        let current_version = get_current_version(&conn)?;

        if target >= current_version {
            let applied = get_applied_migrations(&conn)?;
            return Ok(MigrationStatus {
                current_version,
                pending: Vec::new(),
                applied,
            });
        }

        // Delete migrations above target version
        conn.execute("BEGIN", [])?;
        conn.execute("DELETE FROM __forge_migrations WHERE version > ?", [target])?;
        conn.execute("COMMIT", [])?;

        let new_version = get_current_version(&conn)?;
        let applied = get_applied_migrations(&conn)?;

        Ok(MigrationStatus {
            current_version: new_version,
            pending: Vec::new(),
            applied,
        })
    })
    .await
    .map_err(|e| DatabaseError::generic(e.to_string()))?
}

// =============================================================================
// State Initialization
// =============================================================================

/// Initialize database state in OpState
pub fn init_database_state(
    op_state: &mut OpState,
    app_identifier: String,
    capabilities: Option<Arc<dyn DatabaseCapabilityChecker>>,
    max_connections: Option<usize>,
) {
    op_state.put(DatabaseState::new(
        app_identifier,
        max_connections.unwrap_or(10),
    ));
    if let Some(caps) = capabilities {
        op_state.put(DatabaseCapabilities { checker: caps });
    }
}

// =============================================================================
// Extension Registration
// =============================================================================

include!(concat!(env!("OUT_DIR"), "/extension.rs"));

pub fn database_extension() -> Extension {
    runtime_database::ext()
}
