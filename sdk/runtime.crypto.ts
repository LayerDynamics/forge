// runtime:crypto module - TypeScript wrapper for Deno core ops

// Deno.core type declaration
declare const Deno: {
  core: {
    ops: {
      op_crypto_random_bytes(size: number): number[];
      op_crypto_random_uuid(): string;
      op_crypto_hash(algorithm: string, data: number[]): number[];
      op_crypto_hash_hex(algorithm: string, data: number[]): string;
      op_crypto_hmac(algorithm: string, key: number[], data: number[]): number[];
      op_crypto_encrypt(
        algorithm: string,
        key: number[],
        data: number[],
        iv?: number[]
      ): EncryptedData;
      op_crypto_decrypt(
        algorithm: string,
        key: number[],
        encrypted: EncryptedData
      ): number[];
      op_crypto_generate_key(algorithm: string, length?: number): number[];
      op_crypto_derive_key(
        password: string,
        salt: number[],
        iterations: number,
        keyLength: number
      ): number[];
      op_crypto_verify(
        algorithm: string,
        key: number[],
        data: number[],
        signature: number[]
      ): boolean;
    };
  };
};

export interface EncryptedData {
  ciphertext: number[];
  iv: number[];
  tag: number[];
}

export type HashAlgorithm = "sha256" | "sha384" | "sha512";
export type EncryptionAlgorithm = "aes-256-gcm" | "aes-128-gcm";

export interface EncryptResult {
  ciphertext: Uint8Array;
  iv: Uint8Array;
  tag: Uint8Array;
}

const core = Deno.core;

/**
 * Generate cryptographically secure random bytes.
 * @param size - Number of bytes to generate
 * @returns Random bytes as Uint8Array
 */
export function randomBytes(size: number): Uint8Array {
  const bytes = core.ops.op_crypto_random_bytes(size);
  return new Uint8Array(bytes);
}

/**
 * Generate a random UUID v4.
 * @returns UUID string in standard format
 */
export function randomUUID(): string {
  return core.ops.op_crypto_random_uuid();
}

/**
 * Hash data using specified algorithm.
 * @param algorithm - Hash algorithm (sha256, sha384, sha512)
 * @param data - Data to hash (string or Uint8Array)
 * @returns Hash as Uint8Array
 */
export function hash(
  algorithm: HashAlgorithm,
  data: Uint8Array | string
): Uint8Array {
  const input =
    typeof data === "string" ? new TextEncoder().encode(data) : data;
  const result = core.ops.op_crypto_hash(algorithm, Array.from(input));
  return new Uint8Array(result);
}

/**
 * Hash data and return hex string.
 * @param algorithm - Hash algorithm (sha256, sha384, sha512)
 * @param data - Data to hash (string or Uint8Array)
 * @returns Hash as hex string
 */
export function hashHex(
  algorithm: HashAlgorithm,
  data: Uint8Array | string
): string {
  const input =
    typeof data === "string" ? new TextEncoder().encode(data) : data;
  return core.ops.op_crypto_hash_hex(algorithm, Array.from(input));
}

/**
 * Compute HMAC signature.
 * @param algorithm - HMAC algorithm (sha256, sha384, sha512)
 * @param key - Secret key
 * @param data - Data to sign
 * @returns HMAC signature as Uint8Array
 */
export function hmac(
  algorithm: HashAlgorithm,
  key: Uint8Array,
  data: Uint8Array
): Uint8Array {
  const result = core.ops.op_crypto_hmac(
    algorithm,
    Array.from(key),
    Array.from(data)
  );
  return new Uint8Array(result);
}

/**
 * Encrypt data using symmetric encryption (AES-GCM).
 * @param algorithm - Encryption algorithm (aes-256-gcm)
 * @param key - 32-byte encryption key
 * @param data - Data to encrypt
 * @param iv - Optional 12-byte IV (generated if not provided)
 * @returns Encrypted data with ciphertext, IV, and authentication tag
 */
export function encrypt(
  algorithm: EncryptionAlgorithm,
  key: Uint8Array,
  data: Uint8Array,
  iv?: Uint8Array
): EncryptResult {
  const result = core.ops.op_crypto_encrypt(
    algorithm,
    Array.from(key),
    Array.from(data),
    iv ? Array.from(iv) : undefined
  );
  return {
    ciphertext: new Uint8Array(result.ciphertext),
    iv: new Uint8Array(result.iv),
    tag: new Uint8Array(result.tag),
  };
}

/**
 * Decrypt data using symmetric decryption (AES-GCM).
 * @param algorithm - Decryption algorithm (aes-256-gcm)
 * @param key - 32-byte decryption key
 * @param encrypted - Encrypted data with ciphertext, IV, and tag
 * @returns Decrypted plaintext
 */
export function decrypt(
  algorithm: EncryptionAlgorithm,
  key: Uint8Array,
  encrypted: EncryptResult
): Uint8Array {
  const result = core.ops.op_crypto_decrypt(algorithm, Array.from(key), {
    ciphertext: Array.from(encrypted.ciphertext),
    iv: Array.from(encrypted.iv),
    tag: Array.from(encrypted.tag),
  });
  return new Uint8Array(result);
}

/**
 * Generate a random encryption key.
 * @param algorithm - Algorithm for key (aes-256-gcm, aes-128-gcm, hmac-sha256)
 * @param length - Optional key length (for HMAC keys)
 * @returns Generated key as Uint8Array
 */
export function generateKey(
  algorithm: EncryptionAlgorithm | "hmac-sha256" | "hmac-sha384" | "hmac-sha512",
  length?: number
): Uint8Array {
  const result = core.ops.op_crypto_generate_key(algorithm, length);
  return new Uint8Array(result);
}

/**
 * Derive a key from a password using PBKDF2.
 * @param password - Password string
 * @param salt - Salt bytes (at least 8 bytes recommended)
 * @param iterations - Number of iterations (10000+ recommended)
 * @param keyLength - Desired key length in bytes
 * @returns Derived key as Uint8Array
 */
export function deriveKey(
  password: string,
  salt: Uint8Array,
  iterations: number,
  keyLength: number
): Uint8Array {
  const result = core.ops.op_crypto_derive_key(
    password,
    Array.from(salt),
    iterations,
    keyLength
  );
  return new Uint8Array(result);
}

/**
 * Verify an HMAC signature.
 * @param algorithm - HMAC algorithm (sha256, sha384, sha512)
 * @param key - Secret key used to create signature
 * @param data - Original data
 * @param signature - Signature to verify
 * @returns true if signature is valid, false otherwise
 */
export function verify(
  algorithm: HashAlgorithm,
  key: Uint8Array,
  data: Uint8Array,
  signature: Uint8Array
): boolean {
  return core.ops.op_crypto_verify(
    algorithm,
    Array.from(key),
    Array.from(data),
    Array.from(signature)
  );
}


// ============================================================================
// Extensibility API (auto-generated)
// ============================================================================

/** Registry of operations with their argument and result types */
interface OpRegistry {
  randomBytes: { args: []; result: void };
  randomUuid: { args: []; result: void };
  hash: { args: []; result: void };
  hashHex: { args: []; result: void };
  hmac: { args: []; result: void };
  encrypt: { args: []; result: void };
  decrypt: { args: []; result: void };
  generateKey: { args: []; result: void };
  deriveKey: { args: []; result: void };
  verify: { args: []; result: void };
}

/** Extract argument types for an operation */
type OpArgs<T extends keyof OpRegistry> = OpRegistry[T]['args'];

/** Extract result type for an operation */
type OpResult<T extends keyof OpRegistry> = OpRegistry[T]['result'];

/** Valid operation names for this extension */
type OpName = "randomBytes" | "randomUuid" | "hash" | "hashHex" | "hmac" | "encrypt" | "decrypt" | "generateKey" | "deriveKey" | "verify";

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

