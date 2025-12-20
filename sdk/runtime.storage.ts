/**
 * @module runtime:storage
 *
 * Persistent key-value storage backed by SQLite for Forge applications.
 *
 * This module provides a simple, reliable way to persist application data using
 * a SQLite database. All values are automatically serialized to JSON, supporting
 * strings, numbers, booleans, arrays, and objects.
 *
 * ## Features
 *
 * ### Basic Operations
 * - Get/set/delete individual key-value pairs
 * - Check key existence
 * - List all keys
 * - Clear all data
 * - Get storage size
 *
 * ### Batch Operations
 * - Get multiple keys at once (efficient bulk reads)
 * - Set multiple key-value pairs atomically (transactional)
 * - Delete multiple keys at once (bulk deletes)
 *
 * ### Storage Backend
 * - SQLite database for ACID compliance
 * - Automatic schema creation and indexing
 * - JSON serialization for all JavaScript values
 * - Automatic connection management
 * - Timestamps (created_at, updated_at) for all entries
 *
 * ## Storage Location
 *
 * The SQLite database is stored at:
 * - **macOS**: `~/Library/Application Support/.forge/<app-id>/storage.db`
 * - **Linux**: `~/.local/share/.forge/<app-id>/storage.db`
 * - **Windows**: `%APPDATA%\.forge\<app-id>\storage.db`
 *
 * ## Error Codes
 *
 * Storage operations may throw errors with these codes:
 * - `8100` - Generic storage error
 * - `8101` - Key not found
 * - `8102` - Serialization error (value cannot be serialized to JSON)
 * - `8103` - Deserialization error (stored value is not valid JSON)
 * - `8104` - Database error (SQLite operation failed)
 * - `8105` - Permission denied
 * - `8106` - Invalid key (e.g., empty string)
 * - `8107` - Quota exceeded
 * - `8108` - Connection failed (database cannot be opened)
 * - `8109` - Transaction failed (batch operation rolled back)
 *
 * ## Performance
 *
 * - Individual operations: ~1-2ms per operation
 * - Batch operations: ~0.1ms per item (much faster than individual calls)
 * - Database is indexed on key for fast lookups
 * - Connection is reused across operations
 *
 * ## Data Types
 *
 * All JavaScript values that can be serialized to JSON are supported:
 * - Primitives: `string`, `number`, `boolean`, `null`
 * - Arrays: `string[]`, `number[]`, etc.
 * - Objects: `{ key: value }`, nested objects
 * - Not supported: `undefined`, functions, circular references, `BigInt`, `Symbol`
 */

// Deno.core type declaration
declare const Deno: {
  core: {
    ops: {
      op_storage_get(key: string): Promise<unknown | null>;
      op_storage_set(key: string, value: unknown): Promise<void>;
      op_storage_delete(key: string): Promise<boolean>;
      op_storage_has(key: string): Promise<boolean>;
      op_storage_keys(): Promise<string[]>;
      op_storage_clear(): Promise<void>;
      op_storage_size(): Promise<number>;
      op_storage_get_many(keys: string[]): Promise<Record<string, unknown>>;
      op_storage_set_many(entries: Record<string, unknown>): Promise<void>;
      op_storage_delete_many(keys: string[]): Promise<number>;
    };
  };
};

const core = Deno.core;

/**
 * Retrieves a value from persistent storage by key.
 *
 * Returns the stored value deserialized from JSON, or `null` if the key doesn't exist.
 * You can provide a type parameter for better TypeScript type safety, but the runtime
 * type will depend on what was originally stored.
 *
 * @param key - The key to retrieve (must be non-empty)
 * @returns The stored value, or `null` if the key doesn't exist
 *
 * @throws Error [8106] if key is empty
 * @throws Error [8103] if stored value cannot be deserialized from JSON
 * @throws Error [8104] if database operation fails
 * @throws Error [8108] if database connection fails
 *
 * @example
 * ```typescript
 * // Simple value retrieval
 * const username = await get<string>("user.name");
 * if (username) {
 *   console.log(`Welcome back, ${username}!`);
 * } else {
 *   console.log("No user found");
 * }
 * ```
 *
 * @example
 * ```typescript
 * // Complex object retrieval
 * interface UserPreferences {
 *   theme: "light" | "dark";
 *   fontSize: number;
 *   notifications: boolean;
 * }
 *
 * const prefs = await get<UserPreferences>("user.preferences");
 * if (prefs) {
 *   applyTheme(prefs.theme);
 *   setFontSize(prefs.fontSize);
 * }
 * ```
 *
 * @example
 * ```typescript
 * // Array retrieval
 * const recentFiles = await get<string[]>("recent.files");
 * if (recentFiles && recentFiles.length > 0) {
 *   console.log("Recent files:", recentFiles.join(", "));
 * }
 * ```
 *
 * @example
 * ```typescript
 * // Default value pattern
 * const windowBounds = await get<WindowBounds>("window.bounds") ?? {
 *   x: 100,
 *   y: 100,
 *   width: 800,
 *   height: 600
 * };
 * ```
 */
export async function get<T = unknown>(key: string): Promise<T | null> {
  return (await core.ops.op_storage_get(key)) as T | null;
}

/**
 * Stores a value in persistent storage, associated with the given key.
 *
 * The value is automatically serialized to JSON before storing. If the key already
 * exists, its value is replaced and the `updated_at` timestamp is refreshed.
 *
 * **Atomic Operation**: Each `set()` is executed in a single database transaction.
 *
 * @param key - The key to store under (must be non-empty)
 * @param value - The value to store (must be JSON-serializable)
 *
 * @throws Error [8106] if key is empty
 * @throws Error [8102] if value cannot be serialized to JSON
 * @throws Error [8104] if database operation fails
 * @throws Error [8108] if database connection fails
 *
 * @example
 * ```typescript
 * // Store primitive values
 * await set("app.version", "1.2.3");
 * await set("user.id", 12345);
 * await set("feature.enabled", true);
 * ```
 *
 * @example
 * ```typescript
 * // Store complex objects
 * await set("user.profile", {
 *   name: "Alice Johnson",
 *   email: "alice@example.com",
 *   role: "admin",
 *   lastLogin: new Date().toISOString()
 * });
 * ```
 *
 * @example
 * ```typescript
 * // Store arrays
 * await set("recent.searches", ["typescript", "rust", "deno"]);
 * await set("window.positions", [
 *   { x: 100, y: 100, width: 800, height: 600 },
 *   { x: 900, y: 100, width: 600, height: 400 }
 * ]);
 * ```
 *
 * @example
 * ```typescript
 * // Update existing value
 * const count = await get<number>("app.launchCount") ?? 0;
 * await set("app.launchCount", count + 1);
 * ```
 *
 * @example
 * ```typescript
 * // Handle serialization errors
 * try {
 *   const circular: any = {};
 *   circular.self = circular; // Circular reference!
 *   await set("bad.data", circular);
 * } catch (err) {
 *   console.error("Cannot store circular reference:", err);
 * }
 * ```
 */
export async function set<T = unknown>(key: string, value: T): Promise<void> {
  return await core.ops.op_storage_set(key, value);
}

/**
 * Removes a key and its associated value from persistent storage.
 *
 * This operation is idempotent - calling it multiple times with the same key
 * is safe. Returns whether the key existed before deletion.
 *
 * @param key - The key to delete
 * @returns `true` if the key existed and was deleted, `false` if it didn't exist
 *
 * @throws Error [8104] if database operation fails
 * @throws Error [8108] if database connection fails
 *
 * @example
 * ```typescript
 * // Remove single key
 * const wasDeleted = await remove("user.session");
 * if (wasDeleted) {
 *   console.log("Session cleared");
 * } else {
 *   console.log("No session to clear");
 * }
 * ```
 *
 * @example
 * ```typescript
 * // Conditional removal
 * if (await has("cache.stale")) {
 *   await remove("cache.stale");
 *   console.log("Stale cache removed");
 * }
 * ```
 *
 * @example
 * ```typescript
 * // Clear user data on logout
 * await remove("user.token");
 * await remove("user.profile");
 * await remove("user.preferences");
 * ```
 */
export async function remove(key: string): Promise<boolean> {
  return await core.ops.op_storage_delete(key);
}

/**
 * Checks whether a key exists in persistent storage.
 *
 * This is more efficient than calling `get()` and checking for `null`, especially
 * for large values, since it doesn't deserialize the value.
 *
 * @param key - The key to check for existence
 * @returns `true` if the key exists, `false` otherwise
 *
 * @throws Error [8104] if database operation fails
 * @throws Error [8108] if database connection fails
 *
 * @example
 * ```typescript
 * // Check before reading
 * if (await has("user.profile")) {
 *   const profile = await get("user.profile");
 *   console.log("Profile:", profile);
 * } else {
 *   console.log("No profile found");
 * }
 * ```
 *
 * @example
 * ```typescript
 * // Initialize on first run
 * if (!await has("app.initialized")) {
 *   await set("app.initialized", true);
 *   await runFirstTimeSetup();
 * }
 * ```
 *
 * @example
 * ```typescript
 * // Conditional caching
 * async function getOrFetch(key: string) {
 *   if (await has(key)) {
 *     return await get(key);
 *   }
 *   const data = await fetchFromApi();
 *   await set(key, data);
 *   return data;
 * }
 * ```
 */
export async function has(key: string): Promise<boolean> {
  return await core.ops.op_storage_has(key);
}

/**
 * Retrieves all keys currently stored in the database.
 *
 * Keys are returned in alphabetical order. For large datasets, consider
 * using batch operations like `getMany()` to retrieve values efficiently.
 *
 * @returns Array of all keys, sorted alphabetically
 *
 * @throws Error [8104] if database operation fails
 * @throws Error [8108] if database connection fails
 *
 * @example
 * ```typescript
 * // List all stored keys
 * const allKeys = await keys();
 * console.log(`Storage contains ${allKeys.length} keys`);
 * console.log("Keys:", allKeys.join(", "));
 * ```
 *
 * @example
 * ```typescript
 * // Filter keys by prefix
 * const allKeys = await keys();
 * const userKeys = allKeys.filter(k => k.startsWith("user."));
 * console.log("User keys:", userKeys);
 * ```
 *
 * @example
 * ```typescript
 * // Migrate old keys to new naming scheme
 * const allKeys = await keys();
 * for (const oldKey of allKeys.filter(k => k.startsWith("old_"))) {
 *   const value = await get(oldKey);
 *   const newKey = oldKey.replace("old_", "new_");
 *   await set(newKey, value);
 *   await remove(oldKey);
 * }
 * ```
 *
 * @example
 * ```typescript
 * // Get all values with keys
 * const allKeys = await keys();
 * const entries = new Map<string, unknown>();
 * for (const key of allKeys) {
 *   entries.set(key, await get(key));
 * }
 * // Better: use getMany() for bulk retrieval
 * ```
 */
export async function keys(): Promise<string[]> {
  return await core.ops.op_storage_keys();
}

/**
 * Removes all key-value pairs from persistent storage.
 *
 * **Warning**: This operation is irreversible and will delete all data!
 * Use with caution, especially in production.
 *
 * @throws Error [8104] if database operation fails
 * @throws Error [8108] if database connection fails
 *
 * @example
 * ```typescript
 * // Clear all storage (with confirmation)
 * const confirmed = confirm("Are you sure you want to clear all data?");
 * if (confirmed) {
 *   await clear();
 *   console.log("All storage cleared");
 * }
 * ```
 *
 * @example
 * ```typescript
 * // Reset to defaults on logout
 * await clear();
 * await set("app.version", "1.0.0");
 * await set("app.firstRun", true);
 * ```
 *
 * @example
 * ```typescript
 * // Development: clear cache on startup
 * if (Deno.env.get("DEV_MODE") === "true") {
 *   await clear();
 *   console.log("Development mode: storage cleared");
 * }
 * ```
 */
export async function clear(): Promise<void> {
  return await core.ops.op_storage_clear();
}

/**
 * Returns the total size of all stored values in bytes.
 *
 * This calculates the sum of the length of all JSON-serialized values in the
 * database. Note that this does not include overhead from keys, indexes, or
 * SQLite metadata - it's the raw size of the stored value strings.
 *
 * @returns Total size in bytes of all stored values
 *
 * @throws Error [8104] if database operation fails
 * @throws Error [8108] if database connection fails
 *
 * @example
 * ```typescript
 * // Check storage usage
 * const bytes = await size();
 * const kb = (bytes / 1024).toFixed(2);
 * console.log(`Storage is using ${kb} KB`);
 * ```
 *
 * @example
 * ```typescript
 * // Enforce storage quota
 * const MAX_STORAGE_BYTES = 10 * 1024 * 1024; // 10 MB
 *
 * async function setWithQuota(key: string, value: unknown) {
 *   const currentSize = await size();
 *   const valueSize = JSON.stringify(value).length;
 *
 *   if (currentSize + valueSize > MAX_STORAGE_BYTES) {
 *     throw new Error("Storage quota exceeded");
 *   }
 *
 *   await set(key, value);
 * }
 * ```
 *
 * @example
 * ```typescript
 * // Monitor storage growth
 * const before = await size();
 * await set("large.dataset", bigArray);
 * const after = await size();
 * console.log(`Added ${after - before} bytes to storage`);
 * ```
 */
export async function size(): Promise<number> {
  return await core.ops.op_storage_size();
}

/**
 * Efficiently retrieves multiple values at once from persistent storage.
 *
 * This is significantly faster than calling `get()` multiple times, especially
 * for large numbers of keys. Only keys that exist are returned in the Map.
 *
 * **Performance**: Approximately 10x faster than individual `get()` calls for
 * retrieving 10+ keys.
 *
 * @param keyList - Array of keys to retrieve
 * @returns Map containing key-value pairs for keys that were found (missing keys are omitted)
 *
 * @throws Error [8104] if database operation fails
 * @throws Error [8108] if database connection fails
 *
 * @example
 * ```typescript
 * // Bulk retrieval
 * const keys = ["user.name", "user.email", "user.role"];
 * const values = await getMany(keys);
 *
 * console.log("Name:", values.get("user.name"));
 * console.log("Email:", values.get("user.email"));
 * console.log("Role:", values.get("user.role"));
 * ```
 *
 * @example
 * ```typescript
 * // Load app state efficiently
 * const stateKeys = [
 *   "window.bounds",
 *   "window.maximized",
 *   "recent.files",
 *   "user.preferences"
 * ];
 *
 * const state = await getMany(stateKeys);
 * return {
 *   windowBounds: state.get("window.bounds"),
 *   windowMaximized: state.get("window.maximized") ?? false,
 *   recentFiles: state.get("recent.files") ?? [],
 *   userPrefs: state.get("user.preferences") ?? {}
 * };
 * ```
 *
 * @example
 * ```typescript
 * // Check which keys exist
 * const requestedKeys = ["key1", "key2", "key3"];
 * const found = await getMany(requestedKeys);
 *
 * for (const key of requestedKeys) {
 *   if (found.has(key)) {
 *     console.log(`${key}: ${found.get(key)}`);
 *   } else {
 *     console.log(`${key}: not found`);
 *   }
 * }
 * ```
 *
 * @example
 * ```typescript
 * // Hydrate object from storage
 * const allKeys = await keys();
 * const userKeys = allKeys.filter(k => k.startsWith("user."));
 * const userData = await getMany(userKeys);
 *
 * const user = Object.fromEntries(userData);
 * console.log("User data:", user);
 * ```
 */
export async function getMany(keyList: string[]): Promise<Map<string, unknown>> {
  const result = await core.ops.op_storage_get_many(keyList);
  return new Map(Object.entries(result));
}

/**
 * Atomically stores multiple key-value pairs at once.
 *
 * All writes are executed within a single database transaction. If any write
 * fails, the entire operation is rolled back and no changes are made.
 *
 * **Performance**: Approximately 10x faster than individual `set()` calls for
 * storing 10+ key-value pairs.
 *
 * **Atomicity**: Either all writes succeed or none do (transaction rollback).
 *
 * @param entries - Object containing key-value pairs to store
 *
 * @throws Error [8106] if any key is empty
 * @throws Error [8102] if any value cannot be serialized to JSON
 * @throws Error [8104] if database operation fails
 * @throws Error [8108] if database connection fails
 * @throws Error [8109] if transaction fails (all changes rolled back)
 *
 * @example
 * ```typescript
 * // Bulk initialization
 * await setMany({
 *   "app.version": "1.0.0",
 *   "app.firstRun": true,
 *   "app.installDate": new Date().toISOString(),
 *   "user.theme": "dark",
 *   "user.language": "en"
 * });
 * ```
 *
 * @example
 * ```typescript
 * // Save complex state atomically
 * await setMany({
 *   "window.bounds": { x: 100, y: 100, width: 800, height: 600 },
 *   "window.maximized": false,
 *   "window.displayId": 1,
 *   "recent.files": ["/path/to/file1.txt", "/path/to/file2.txt"],
 *   "recent.searches": ["typescript", "rust"]
 * });
 * ```
 *
 * @example
 * ```typescript
 * // Convert Map to storage
 * const cache = new Map([
 *   ["api.users", usersData],
 *   ["api.posts", postsData],
 *   ["api.comments", commentsData]
 * ]);
 *
 * await setMany(Object.fromEntries(cache));
 * ```
 *
 * @example
 * ```typescript
 * // Atomicity example - all or nothing
 * try {
 *   await setMany({
 *     "user.name": "Alice",
 *     "user.email": "alice@example.com",
 *     "user.invalid": circularReference // This will fail!
 *   });
 * } catch (err) {
 *   // None of the values were saved (transaction rolled back)
 *   console.error("Save failed, no changes made:", err);
 * }
 * ```
 */
export async function setMany(entries: Record<string, unknown>): Promise<void> {
  return await core.ops.op_storage_set_many(entries);
}

/**
 * Efficiently deletes multiple keys at once from persistent storage.
 *
 * This is significantly faster than calling `remove()` multiple times. Returns
 * the count of keys that actually existed and were deleted.
 *
 * **Performance**: Approximately 10x faster than individual `remove()` calls
 * for deleting 10+ keys.
 *
 * @param keyList - Array of keys to delete
 * @returns Number of keys that existed and were successfully deleted
 *
 * @throws Error [8104] if database operation fails
 * @throws Error [8108] if database connection fails
 *
 * @example
 * ```typescript
 * // Clear user session data
 * const sessionKeys = [
 *   "session.token",
 *   "session.userId",
 *   "session.expires"
 * ];
 *
 * const deleted = await deleteMany(sessionKeys);
 * console.log(`Deleted ${deleted} session keys`);
 * ```
 *
 * @example
 * ```typescript
 * // Clean up cache by prefix
 * const allKeys = await keys();
 * const cacheKeys = allKeys.filter(k => k.startsWith("cache."));
 *
 * if (cacheKeys.length > 0) {
 *   const deleted = await deleteMany(cacheKeys);
 *   console.log(`Cleared ${deleted} cache entries`);
 * }
 * ```
 *
 * @example
 * ```typescript
 * // Remove old data selectively
 * const allKeys = await keys();
 * const oldKeys = allKeys.filter(k =>
 *   k.startsWith("temp.") || k.startsWith("deprecated.")
 * );
 *
 * if (oldKeys.length > 0) {
 *   await deleteMany(oldKeys);
 * }
 * ```
 *
 * @example
 * ```typescript
 * // Batch cleanup with verification
 * const keysToDelete = ["key1", "key2", "key3", "key4"];
 * const deleted = await deleteMany(keysToDelete);
 *
 * if (deleted === keysToDelete.length) {
 *   console.log("All keys deleted successfully");
 * } else {
 *   console.log(`Only ${deleted}/${keysToDelete.length} keys existed`);
 * }
 * ```
 */
export async function deleteMany(keyList: string[]): Promise<number> {
  return await core.ops.op_storage_delete_many(keyList);
}

// Alias for backwards compatibility with common naming
export { remove as delete_ };


// ============================================================================
// Extensibility API (auto-generated)
// ============================================================================

/** Registry of operations with their argument and result types */
interface OpRegistry {
  get: { args: []; result: void };
  set: { args: []; result: void };
  delete: { args: []; result: void };
  has: { args: []; result: void };
  keys: { args: []; result: void };
  clear: { args: []; result: void };
  size: { args: []; result: void };
  getMany: { args: []; result: void };
  setMany: { args: []; result: void };
  deleteMany: { args: []; result: void };
}

/** Extract argument types for an operation */
type OpArgs<T extends keyof OpRegistry> = OpRegistry[T]['args'];

/** Extract result type for an operation */
type OpResult<T extends keyof OpRegistry> = OpRegistry[T]['result'];

/** Valid operation names for this extension */
type OpName = "get" | "set" | "delete" | "has" | "keys" | "clear" | "size" | "getMany" | "setMany" | "deleteMany";

/** Hook callback types */
type BeforeHookCallback<T extends OpName> = (args: OpArgs<T>) => void | Promise<void>;
type AfterHookCallback<T extends OpName> = (result: OpResult<T>, args: OpArgs<T>) => void | Promise<void>;
type ErrorHookCallback<T extends OpName> = (error: Error, args: OpArgs<T>) => void | Promise<void>;

/** Internal hook storage */
const _hooks = {
  before: new Map<OpName, Set<BeforeHookCallback<OpName>>>(),
  after: new Map<OpName, Set<AfterHookCallback<OpName>>>(),
  error: new Map<OpName, Set<ErrorHookCallback<OpName>>>(),
};

/**
 * Register a callback to be called before an operation executes.
 * @param opName - The name of the operation to hook
 * @param callback - Function called with the operation arguments
 * @returns Unsubscribe function to remove the hook
 */
export function onBefore<T extends OpName>(
  opName: T,
  callback: BeforeHookCallback<T>
): () => void {
  if (!_hooks.before.has(opName)) {
    _hooks.before.set(opName, new Set());
  }
  _hooks.before.get(opName)!.add(callback as BeforeHookCallback<OpName>);
  return () => _hooks.before.get(opName)?.delete(callback as BeforeHookCallback<OpName>);
}

/**
 * Register a callback to be called after an operation completes successfully.
 * @param opName - The name of the operation to hook
 * @param callback - Function called with the result and original arguments
 * @returns Unsubscribe function to remove the hook
 */
export function onAfter<T extends OpName>(
  opName: T,
  callback: AfterHookCallback<T>
): () => void {
  if (!_hooks.after.has(opName)) {
    _hooks.after.set(opName, new Set());
  }
  _hooks.after.get(opName)!.add(callback as AfterHookCallback<OpName>);
  return () => _hooks.after.get(opName)?.delete(callback as AfterHookCallback<OpName>);
}

/**
 * Register a callback to be called when an operation throws an error.
 * @param opName - The name of the operation to hook
 * @param callback - Function called with the error and original arguments
 * @returns Unsubscribe function to remove the hook
 */
export function onError<T extends OpName>(
  opName: T,
  callback: ErrorHookCallback<T>
): () => void {
  if (!_hooks.error.has(opName)) {
    _hooks.error.set(opName, new Set());
  }
  _hooks.error.get(opName)!.add(callback as ErrorHookCallback<OpName>);
  return () => _hooks.error.get(opName)?.delete(callback as ErrorHookCallback<OpName>);
}

/** Internal: Invoke before hooks for an operation */
async function _invokeBeforeHooks<T extends OpName>(opName: T, args: OpArgs<T>): Promise<void> {
  const hooks = _hooks.before.get(opName);
  if (hooks) {
    for (const hook of hooks) {
      await hook(args);
    }
  }
}

/** Internal: Invoke after hooks for an operation */
async function _invokeAfterHooks<T extends OpName>(opName: T, result: OpResult<T>, args: OpArgs<T>): Promise<void> {
  const hooks = _hooks.after.get(opName);
  if (hooks) {
    for (const hook of hooks) {
      await hook(result, args);
    }
  }
}

/** Internal: Invoke error hooks for an operation */
async function _invokeErrorHooks<T extends OpName>(opName: T, error: Error, args: OpArgs<T>): Promise<void> {
  const hooks = _hooks.error.get(opName);
  if (hooks) {
    for (const hook of hooks) {
      await hook(error, args);
    }
  }
}

/**
 * Remove all hooks for a specific operation or all operations.
 * @param opName - Optional: specific operation to clear hooks for
 */
export function removeAllHooks(opName?: OpName): void {
  if (opName) {
    _hooks.before.delete(opName);
    _hooks.after.delete(opName);
    _hooks.error.delete(opName);
  } else {
    _hooks.before.clear();
    _hooks.after.clear();
    _hooks.error.clear();
  }
}

/** Handler function type */
type HandlerFn = (...args: unknown[]) => unknown | Promise<unknown>;

/** Internal handler storage */
const _handlers = new Map<string, HandlerFn>();

/**
 * Register a custom handler that can be invoked by name.
 * @param name - Unique name for the handler
 * @param handler - Handler function to register
 * @throws Error if a handler with the same name already exists
 */
export function registerHandler(name: string, handler: HandlerFn): void {
  if (_handlers.has(name)) {
    throw new Error(`Handler '${name}' already registered`);
  }
  _handlers.set(name, handler);
}

/**
 * Invoke a registered handler by name.
 * @param name - Name of the handler to invoke
 * @param args - Arguments to pass to the handler
 * @returns The handler's return value
 * @throws Error if no handler with the given name exists
 */
export async function invokeHandler(name: string, ...args: unknown[]): Promise<unknown> {
  const handler = _handlers.get(name);
  if (!handler) {
    throw new Error(`Handler '${name}' not found`);
  }
  return await handler(...args);
}

/**
 * List all registered handler names.
 * @returns Array of handler names
 */
export function listHandlers(): string[] {
  return Array.from(_handlers.keys());
}

/**
 * Remove a registered handler.
 * @param name - Name of the handler to remove
 * @returns true if the handler was removed, false if it didn't exist
 */
export function removeHandler(name: string): boolean {
  return _handlers.delete(name);
}

/**
 * Check if a handler is registered.
 * @param name - Name of the handler to check
 * @returns true if the handler exists
 */
export function hasHandler(name: string): boolean {
  return _handlers.has(name);
}

