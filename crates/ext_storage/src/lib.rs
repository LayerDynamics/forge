//! runtime:storage extension - Persistent key-value storage for Forge apps
//!
//! Provides SQLite-backed storage at ~/.forge/<app-identifier>/storage.db

use deno_core::{op2, Extension, OpState};
use forge_weld_macro::weld_op;
use rusqlite::Connection;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

// ============================================================================
// Error Types with Structured Codes
// ============================================================================

/// Error codes for storage operations (8100-8109)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum StorageErrorCode {
    /// Generic storage error
    Generic = 8100,
    /// Key not found
    NotFound = 8101,
    /// Serialization error
    SerializationError = 8102,
    /// Deserialization error
    DeserializationError = 8103,
    /// Database error
    DatabaseError = 8104,
    /// Permission denied
    PermissionDenied = 8105,
    /// Invalid key
    InvalidKey = 8106,
    /// Storage quota exceeded
    QuotaExceeded = 8107,
    /// Connection failed
    ConnectionFailed = 8108,
    /// Transaction failed
    TransactionFailed = 8109,
}

/// Custom error type for storage operations
#[derive(Debug, thiserror::Error, deno_error::JsError)]
pub enum StorageError {
    #[error("[{code}] Storage error: {message}")]
    #[class(generic)]
    Generic { code: u32, message: String },

    #[error("[{code}] Key not found: {message}")]
    #[class(generic)]
    NotFound { code: u32, message: String },

    #[error("[{code}] Serialization error: {message}")]
    #[class(generic)]
    SerializationError { code: u32, message: String },

    #[error("[{code}] Deserialization error: {message}")]
    #[class(generic)]
    DeserializationError { code: u32, message: String },

    #[error("[{code}] Database error: {message}")]
    #[class(generic)]
    DatabaseError { code: u32, message: String },

    #[error("[{code}] Permission denied: {message}")]
    #[class(generic)]
    PermissionDenied { code: u32, message: String },

    #[error("[{code}] Invalid key: {message}")]
    #[class(generic)]
    InvalidKey { code: u32, message: String },

    #[error("[{code}] Quota exceeded: {message}")]
    #[class(generic)]
    QuotaExceeded { code: u32, message: String },

    #[error("[{code}] Connection failed: {message}")]
    #[class(generic)]
    ConnectionFailed { code: u32, message: String },

    #[error("[{code}] Transaction failed: {message}")]
    #[class(generic)]
    TransactionFailed { code: u32, message: String },
}

impl StorageError {
    pub fn generic(message: impl Into<String>) -> Self {
        Self::Generic {
            code: StorageErrorCode::Generic as u32,
            message: message.into(),
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound {
            code: StorageErrorCode::NotFound as u32,
            message: message.into(),
        }
    }

    pub fn serialization_error(message: impl Into<String>) -> Self {
        Self::SerializationError {
            code: StorageErrorCode::SerializationError as u32,
            message: message.into(),
        }
    }

    pub fn deserialization_error(message: impl Into<String>) -> Self {
        Self::DeserializationError {
            code: StorageErrorCode::DeserializationError as u32,
            message: message.into(),
        }
    }

    pub fn database_error(message: impl Into<String>) -> Self {
        Self::DatabaseError {
            code: StorageErrorCode::DatabaseError as u32,
            message: message.into(),
        }
    }

    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self::PermissionDenied {
            code: StorageErrorCode::PermissionDenied as u32,
            message: message.into(),
        }
    }

    pub fn invalid_key(message: impl Into<String>) -> Self {
        Self::InvalidKey {
            code: StorageErrorCode::InvalidKey as u32,
            message: message.into(),
        }
    }

    pub fn connection_failed(message: impl Into<String>) -> Self {
        Self::ConnectionFailed {
            code: StorageErrorCode::ConnectionFailed as u32,
            message: message.into(),
        }
    }
}

impl From<rusqlite::Error> for StorageError {
    fn from(e: rusqlite::Error) -> Self {
        Self::database_error(e.to_string())
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(e: serde_json::Error) -> Self {
        Self::serialization_error(e.to_string())
    }
}

impl From<std::io::Error> for StorageError {
    fn from(e: std::io::Error) -> Self {
        Self::generic(e.to_string())
    }
}

// ============================================================================
// State Types
// ============================================================================

/// App identifier for storage location
pub struct StorageAppInfo {
    pub app_identifier: String,
}

/// Storage connection state
pub struct StorageConnection {
    pub db_path: PathBuf,
    pub connection: Arc<Mutex<Connection>>,
}

// ============================================================================
// Capability Checker
// ============================================================================

/// Capability checker trait for storage operations
pub trait StorageCapabilityChecker: Send + Sync {
    fn check_storage(&self) -> Result<(), String>;
}

/// Default permissive checker
pub struct PermissiveChecker;

impl StorageCapabilityChecker for PermissiveChecker {
    fn check_storage(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Wrapper to store capability checker in OpState
pub struct StorageCapabilities {
    pub checker: Arc<dyn StorageCapabilityChecker>,
}

impl Default for StorageCapabilities {
    fn default() -> Self {
        Self {
            checker: Arc::new(PermissiveChecker),
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Get or create the storage database connection
async fn get_connection(state: &Rc<RefCell<OpState>>) -> Result<Arc<Mutex<Connection>>, StorageError> {
    // Check if already connected
    {
        let s = state.borrow();
        if let Some(conn) = s.try_borrow::<StorageConnection>() {
            return Ok(conn.connection.clone());
        }
    }

    // Get app identifier
    let app_identifier = {
        let s = state.borrow();
        s.try_borrow::<StorageAppInfo>()
            .map(|info| info.app_identifier.clone())
            .unwrap_or_else(|| "forge-app".to_string())
    };

    // Create storage directory
    let storage_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".forge")
        .join(&app_identifier);

    tokio::fs::create_dir_all(&storage_dir).await?;

    let db_path = storage_dir.join("storage.db");
    let db_path_clone = db_path.clone();

    // Open database connection in blocking task
    let connection = tokio::task::spawn_blocking(move || -> Result<Connection, StorageError> {
        let conn = Connection::open(&db_path_clone)?;

        // Create table if not exists
        conn.execute(
            "CREATE TABLE IF NOT EXISTS kv_store (
                key TEXT PRIMARY KEY NOT NULL,
                value TEXT NOT NULL,
                created_at INTEGER DEFAULT (strftime('%s', 'now')),
                updated_at INTEGER DEFAULT (strftime('%s', 'now'))
            )",
            [],
        )?;

        // Create index for faster lookups
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_kv_key ON kv_store(key)",
            [],
        )?;

        Ok(conn)
    })
    .await
    .map_err(|e| StorageError::connection_failed(e.to_string()))??;

    let connection = Arc::new(Mutex::new(connection));

    // Store connection in state
    {
        let mut s = state.borrow_mut();
        s.put(StorageConnection {
            db_path,
            connection: connection.clone(),
        });
    }

    Ok(connection)
}

// ============================================================================
// Operations
// ============================================================================

/// Get a value by key
#[weld_op(async)]
#[op2(async)]
#[serde]
async fn op_storage_get(
    state: Rc<RefCell<OpState>>,
    #[string] key: String,
) -> Result<Option<serde_json::Value>, StorageError> {
    debug!(key = %key, "storage.get");

    let conn = get_connection(&state).await?;
    let conn = conn.lock().await;

    let result: Result<String, rusqlite::Error> = conn.query_row(
        "SELECT value FROM kv_store WHERE key = ?",
        [&key],
        |row| row.get(0),
    );

    match result {
        Ok(value_str) => {
            let value: serde_json::Value = serde_json::from_str(&value_str)?;
            Ok(Some(value))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(StorageError::from(e)),
    }
}

/// Set a value by key
#[weld_op(async)]
#[op2(async)]
async fn op_storage_set(
    state: Rc<RefCell<OpState>>,
    #[string] key: String,
    #[serde] value: serde_json::Value,
) -> Result<(), StorageError> {
    debug!(key = %key, "storage.set");

    if key.is_empty() {
        return Err(StorageError::invalid_key("Key cannot be empty"));
    }

    let value_str = serde_json::to_string(&value)?;

    let conn = get_connection(&state).await?;
    let conn = conn.lock().await;

    conn.execute(
        "INSERT INTO kv_store (key, value, updated_at) VALUES (?, ?, strftime('%s', 'now'))
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = strftime('%s', 'now')",
        rusqlite::params![key, value_str],
    )?;

    Ok(())
}

/// Delete a key
#[weld_op(async)]
#[op2(async)]
async fn op_storage_delete(
    state: Rc<RefCell<OpState>>,
    #[string] key: String,
) -> Result<bool, StorageError> {
    debug!(key = %key, "storage.delete");

    let conn = get_connection(&state).await?;
    let conn = conn.lock().await;

    let rows_affected = conn.execute("DELETE FROM kv_store WHERE key = ?", [&key])?;

    Ok(rows_affected > 0)
}

/// Check if a key exists
#[weld_op(async)]
#[op2(async)]
async fn op_storage_has(
    state: Rc<RefCell<OpState>>,
    #[string] key: String,
) -> Result<bool, StorageError> {
    debug!(key = %key, "storage.has");

    let conn = get_connection(&state).await?;
    let conn = conn.lock().await;

    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM kv_store WHERE key = ?)",
        [&key],
        |row| row.get(0),
    )?;

    Ok(exists)
}

/// List all keys
#[weld_op(async)]
#[op2(async)]
#[serde]
async fn op_storage_keys(
    state: Rc<RefCell<OpState>>,
) -> Result<Vec<String>, StorageError> {
    debug!("storage.keys");

    let conn = get_connection(&state).await?;
    let conn = conn.lock().await;

    let mut stmt = conn.prepare("SELECT key FROM kv_store ORDER BY key")?;
    let keys: Vec<String> = stmt
        .query_map([], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(keys)
}

/// Clear all data
#[weld_op(async)]
#[op2(async)]
async fn op_storage_clear(
    state: Rc<RefCell<OpState>>,
) -> Result<(), StorageError> {
    debug!("storage.clear");

    let conn = get_connection(&state).await?;
    let conn = conn.lock().await;

    conn.execute("DELETE FROM kv_store", [])?;

    Ok(())
}

/// Get storage size in bytes
#[weld_op(async)]
#[op2(async)]
#[bigint]
async fn op_storage_size(
    state: Rc<RefCell<OpState>>,
) -> Result<u64, StorageError> {
    debug!("storage.size");

    let conn = get_connection(&state).await?;
    let conn = conn.lock().await;

    // Get total size of all values
    let size: i64 = conn.query_row(
        "SELECT COALESCE(SUM(LENGTH(value)), 0) FROM kv_store",
        [],
        |row| row.get(0),
    )?;

    Ok(size as u64)
}

/// Batch get multiple keys
#[weld_op(async)]
#[op2(async)]
#[serde]
async fn op_storage_get_many(
    state: Rc<RefCell<OpState>>,
    #[serde] keys: Vec<String>,
) -> Result<HashMap<String, serde_json::Value>, StorageError> {
    debug!(count = keys.len(), "storage.get_many");

    if keys.is_empty() {
        return Ok(HashMap::new());
    }

    let conn = get_connection(&state).await?;
    let conn = conn.lock().await;

    let placeholders: Vec<&str> = keys.iter().map(|_| "?").collect();
    let sql = format!(
        "SELECT key, value FROM kv_store WHERE key IN ({})",
        placeholders.join(",")
    );

    let mut stmt = conn.prepare(&sql)?;
    let params: Vec<&dyn rusqlite::ToSql> = keys.iter().map(|k| k as &dyn rusqlite::ToSql).collect();

    let mut result = HashMap::new();
    let rows = stmt.query_map(params.as_slice(), |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;

    for row in rows {
        let (key, value_str) = row?;
        if let Ok(value) = serde_json::from_str(&value_str) {
            result.insert(key, value);
        }
    }

    Ok(result)
}

/// Batch set multiple key-value pairs
#[weld_op(async)]
#[op2(async)]
async fn op_storage_set_many(
    state: Rc<RefCell<OpState>>,
    #[serde] entries: HashMap<String, serde_json::Value>,
) -> Result<(), StorageError> {
    debug!(count = entries.len(), "storage.set_many");

    if entries.is_empty() {
        return Ok(());
    }

    // Validate keys
    for key in entries.keys() {
        if key.is_empty() {
            return Err(StorageError::invalid_key("Key cannot be empty"));
        }
    }

    let conn = get_connection(&state).await?;
    let mut conn = conn.lock().await;

    // Use transaction for atomicity
    let tx = conn.transaction()?;

    for (key, value) in entries {
        let value_str = serde_json::to_string(&value)?;
        tx.execute(
            "INSERT INTO kv_store (key, value, updated_at) VALUES (?, ?, strftime('%s', 'now'))
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = strftime('%s', 'now')",
            rusqlite::params![key, value_str],
        )?;
    }

    tx.commit()?;
    Ok(())
}

/// Batch delete multiple keys
#[weld_op(async)]
#[op2(async)]
async fn op_storage_delete_many(
    state: Rc<RefCell<OpState>>,
    #[serde] keys: Vec<String>,
) -> Result<u32, StorageError> {
    debug!(count = keys.len(), "storage.delete_many");

    if keys.is_empty() {
        return Ok(0);
    }

    let conn = get_connection(&state).await?;
    let conn = conn.lock().await;

    let placeholders: Vec<&str> = keys.iter().map(|_| "?").collect();
    let sql = format!(
        "DELETE FROM kv_store WHERE key IN ({})",
        placeholders.join(",")
    );

    let params: Vec<&dyn rusqlite::ToSql> = keys.iter().map(|k| k as &dyn rusqlite::ToSql).collect();
    let rows_deleted = conn.execute(&sql, params.as_slice())?;

    Ok(rows_deleted as u32)
}

// ============================================================================
// State Initialization
// ============================================================================

/// Initialize storage state in OpState
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

// ============================================================================
// Extension Registration
// ============================================================================

// Include generated extension! macro from build.rs
include!(concat!(env!("OUT_DIR"), "/extension.rs"));

pub fn storage_extension() -> Extension {
    runtime_storage::ext()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        let err = StorageError::not_found("test");
        match err {
            StorageError::NotFound { code, .. } => {
                assert_eq!(code, StorageErrorCode::NotFound as u32);
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_invalid_key_error() {
        let err = StorageError::invalid_key("empty");
        match err {
            StorageError::InvalidKey { code, message } => {
                assert_eq!(code, StorageErrorCode::InvalidKey as u32);
                assert!(message.contains("empty"));
            }
            _ => panic!("Wrong error type"),
        }
    }
}
