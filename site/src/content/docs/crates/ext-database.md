---
title: "ext_database"
description: Database access extension providing the runtime:database module.
slug: crates/ext-database
---

The `ext_database` crate provides SQLite database access for Forge applications through the `runtime:database` module.

## Overview

ext_database handles:

- **Database connections** - Open/close SQLite databases
- **SQL queries** - Execute queries with parameters
- **Transactions** - ACID transaction support
- **Migrations** - Schema version management
- **Prepared statements** - Efficient repeated queries

## Module: `runtime:database`

```typescript
import {
  open,
  close,
  execute,
  query,
  queryOne,
  transaction
} from "runtime:database";
```

## Key Types

### Error Types

```rust
enum DatabaseErrorCode {
    Generic = 9500,
    OpenFailed = 9501,
    CloseFailed = 9502,
    QueryFailed = 9503,
    ExecuteFailed = 9504,
    TransactionFailed = 9505,
    InvalidPath = 9506,
    ConnectionClosed = 9507,
}

struct DatabaseError {
    code: DatabaseErrorCode,
    message: String,
}
```

### Database Types

```rust
struct DatabaseHandle {
    id: u32,
}

struct DatabaseConfig {
    path: String,
    create_if_not_exists: Option<bool>,
    read_only: Option<bool>,
}

struct QueryResult {
    columns: Vec<String>,
    rows: Vec<Vec<Value>>,
    changes: u64,
}

struct DatabaseState {
    connections: HashMap<u32, Connection>,
    next_id: u32,
}
```

## Operations

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_database_open` | `open(path, opts?)` | Open database connection |
| `op_database_close` | `close(handle)` | Close connection |
| `op_database_execute` | `execute(handle, sql, params?)` | Execute SQL statement |
| `op_database_query` | `query(handle, sql, params?)` | Query returning rows |
| `op_database_query_one` | `queryOne(handle, sql, params?)` | Query single row |
| `op_database_transaction` | `transaction(handle, fn)` | Run in transaction |

## Usage Examples

### Opening a Database

```typescript
import { open, close } from "runtime:database";

const db = await open("./data.db", {
  createIfNotExists: true
});

// Use database...

await close(db);
```

### Executing Queries

```typescript
import { open, execute, query } from "runtime:database";

const db = await open("./data.db");

// Create table
await execute(db, `
  CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    email TEXT UNIQUE
  )
`);

// Insert with parameters
await execute(db,
  "INSERT INTO users (name, email) VALUES (?, ?)",
  ["Alice", "alice@example.com"]
);

// Query data
const users = await query(db, "SELECT * FROM users WHERE name LIKE ?", ["%Ali%"]);
for (const row of users.rows) {
  console.log(row);
}
```

### Transactions

```typescript
import { open, transaction, execute } from "runtime:database";

const db = await open("./data.db");

await transaction(db, async (tx) => {
  await execute(tx, "INSERT INTO orders (user_id, total) VALUES (?, ?)", [1, 100]);
  await execute(tx, "UPDATE users SET balance = balance - ? WHERE id = ?", [100, 1]);
  // Transaction commits automatically on success
  // Rolls back on error
});
```

### Query Single Row

```typescript
import { open, queryOne } from "runtime:database";

const db = await open("./data.db");

const user = await queryOne(db,
  "SELECT * FROM users WHERE id = ?",
  [123]
);

if (user) {
  console.log(`Found: ${user.name}`);
} else {
  console.log("User not found");
}
```

## File Structure

```text
crates/ext_database/
├── src/
│   └── lib.rs        # Extension implementation
├── ts/
│   └── init.ts       # TypeScript module shim
├── build.rs          # forge-weld build configuration
└── Cargo.toml
```

## Rust Implementation

Operations are annotated with forge-weld macros for automatic TypeScript binding generation:

```rust
// src/lib.rs
use deno_core::{op2, Extension, OpState};
use forge_weld_macro::{weld_op, weld_struct};
use serde::{Deserialize, Serialize};

#[weld_struct]
#[derive(Debug, Serialize)]
pub struct DatabaseHandle {
    pub id: u32,
}

#[weld_struct]
#[derive(Debug, Serialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub changes: u64,
}

#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_database_open(
    state: Rc<RefCell<OpState>>,
    #[string] path: String,
    #[serde] config: Option<DatabaseConfig>,
) -> Result<DatabaseHandle, DatabaseError> {
    // implementation
}
```

## Build Configuration

```rust
// build.rs
use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_database", "runtime:database")
        .ts_path("ts/init.ts")
        .ops(&["op_database_open", "op_database_close", "op_database_query", /* ... */])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build runtime_database extension");
}
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `deno_core` | Op definitions |
| `rusqlite` | SQLite bindings |
| `serde` | Serialization |
| `serde_json` | JSON values |
| `tokio` | Async runtime |
| `forge-weld` | Build-time code generation |
| `forge-weld-macro` | `#[weld_op]`, `#[weld_struct]` macros |
| `linkme` | Compile-time symbol collection |

## Related

- [ext_storage](/docs/crates/ext-storage) - Key-value storage
- [Architecture](/docs/architecture) - Full system architecture
