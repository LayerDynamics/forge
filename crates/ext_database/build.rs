use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_database", "runtime:database")
        .ts_path("ts/init.ts")
        .ops(&[
            // Connection Management (7 ops)
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
        .enable_extensibility()
        .build()
        .expect("Failed to build database extension");
}
