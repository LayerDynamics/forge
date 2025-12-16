// runtime:storage module - TypeScript wrapper for Deno core ops

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
 * Get a value from storage.
 * @param key - The key to retrieve
 * @returns The stored value or null if not found
 */
export async function get<T = unknown>(key: string): Promise<T | null> {
  return (await core.ops.op_storage_get(key)) as T | null;
}

/**
 * Set a value in storage.
 * @param key - The key to set
 * @param value - The value to store (must be JSON serializable)
 */
export async function set<T = unknown>(key: string, value: T): Promise<void> {
  return await core.ops.op_storage_set(key, value);
}

/**
 * Delete a key from storage.
 * @param key - The key to delete
 * @returns true if the key existed and was deleted, false otherwise
 */
export async function remove(key: string): Promise<boolean> {
  return await core.ops.op_storage_delete(key);
}

/**
 * Check if a key exists in storage.
 * @param key - The key to check
 * @returns true if the key exists, false otherwise
 */
export async function has(key: string): Promise<boolean> {
  return await core.ops.op_storage_has(key);
}

/**
 * Get all keys in storage.
 * @returns Array of all keys
 */
export async function keys(): Promise<string[]> {
  return await core.ops.op_storage_keys();
}

/**
 * Clear all data from storage.
 */
export async function clear(): Promise<void> {
  return await core.ops.op_storage_clear();
}

/**
 * Get the total size of stored data in bytes.
 * @returns Size in bytes
 */
export async function size(): Promise<number> {
  return await core.ops.op_storage_size();
}

/**
 * Get multiple values at once.
 * @param keyList - Array of keys to retrieve
 * @returns Map of key-value pairs for found keys
 */
export async function getMany(keyList: string[]): Promise<Map<string, unknown>> {
  const result = await core.ops.op_storage_get_many(keyList);
  return new Map(Object.entries(result));
}

/**
 * Set multiple values at once (atomic operation).
 * @param entries - Object with key-value pairs to set
 */
export async function setMany(entries: Record<string, unknown>): Promise<void> {
  return await core.ops.op_storage_set_many(entries);
}

/**
 * Delete multiple keys at once.
 * @param keyList - Array of keys to delete
 * @returns Number of keys that were deleted
 */
export async function deleteMany(keyList: string[]): Promise<number> {
  return await core.ops.op_storage_delete_many(keyList);
}

// Alias for backwards compatibility with common naming
export { remove as delete_ };