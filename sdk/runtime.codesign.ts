// runtime:codesign module - TypeScript wrapper for code signing operations

// Deno.core type declaration
declare const Deno: {
  core: {
    ops: {
      op_codesign_sign(options: SignOptionsInternal): Promise<void>;
      op_codesign_sign_adhoc(path: string): Promise<void>;
      op_codesign_verify(path: string): Promise<VerifyResultInternal>;
      op_codesign_get_entitlements(path: string): Promise<string>;
      op_codesign_list_identities(): Promise<SigningIdentityInternal[]>;
      op_codesign_get_identity_info(
        identity: string
      ): Promise<SigningIdentityInternal>;
      op_codesign_check_capabilities(): CodesignCapabilitiesInternal;
    };
  };
};

// Internal types matching Rust structs
export interface SignOptionsInternal {
  path: string;
  identity: string;
  entitlements?: string;
  hardened_runtime?: boolean;
  deep?: boolean;
  timestamp_url?: string;
}

export interface VerifyResultInternal {
  valid: boolean;
  signer: string | null;
  timestamp: string | null;
  message: string;
}

export interface SigningIdentityInternal {
  id: string;
  name: string;
  expires: string | null;
  valid: boolean;
  identity_type: string;
}

export interface CodesignCapabilitiesInternal {
  codesign: boolean;
  security: boolean;
  signtool: boolean;
  certutil: boolean;
  platform: string;
}

const core = Deno.core;

// ============================================================================
// Public Types
// ============================================================================

/**
 * Options for code signing operations
 */
export interface SignOptions {
  /** Path to the file or application bundle to sign */
  path: string;
  /** Signing identity (certificate name or SHA-1 thumbprint) */
  identity: string;
  /** Path to entitlements file (macOS only) */
  entitlements?: string;
  /** Enable hardened runtime (macOS, default: true) */
  hardenedRuntime?: boolean;
  /** Deep sign embedded code (macOS) */
  deep?: boolean;
  /** Timestamp server URL (Windows, default: DigiCert) */
  timestampUrl?: string;
}

/**
 * Result of signature verification
 */
export interface VerifyResult {
  /** Whether the signature is valid */
  valid: boolean;
  /** Identity of the signer */
  signer: string | null;
  /** Timestamp of signature (if timestamped) */
  timestamp: string | null;
  /** Detailed status message */
  message: string;
}

/**
 * Information about a signing identity/certificate
 */
export interface SigningIdentity {
  /** Certificate ID (SHA-1 thumbprint) */
  id: string;
  /** Human-readable name */
  name: string;
  /** Expiration date (ISO 8601 format) */
  expires: string | null;
  /** Whether the certificate is currently valid */
  valid: boolean;
  /** Identity type (developer_id_application, distribution, development, etc.) */
  type: string;
}

/**
 * Available signing capabilities on current platform
 */
export interface CodesignCapabilities {
  /** macOS codesign tool available */
  codesign: boolean;
  /** macOS security tool available */
  security: boolean;
  /** Windows SignTool available */
  signtool: boolean;
  /** Windows certutil available */
  certutil: boolean;
  /** Current platform */
  platform: "macos" | "windows" | "linux";
}

// ============================================================================
// Functions
// ============================================================================

/**
 * Sign a file or application bundle with a code signing identity.
 *
 * @param options - Signing options
 * @throws Error if signing fails or platform doesn't support signing
 *
 * @example
 * ```ts
 * // macOS signing with Developer ID
 * await sign({
 *   path: "/path/to/MyApp.app",
 *   identity: "Developer ID Application: My Company (TEAMID)",
 *   entitlements: "./entitlements.plist",
 *   hardenedRuntime: true,
 *   deep: true
 * });
 *
 * // Windows signing with certificate thumbprint
 * await sign({
 *   path: "C:\\path\\to\\app.exe",
 *   identity: "ABC123DEF456...", // SHA-1 thumbprint
 *   timestampUrl: "http://timestamp.digicert.com"
 * });
 * ```
 */
export async function sign(options: SignOptions): Promise<void> {
  return await core.ops.op_codesign_sign({
    path: options.path,
    identity: options.identity,
    entitlements: options.entitlements,
    hardened_runtime: options.hardenedRuntime,
    deep: options.deep,
    timestamp_url: options.timestampUrl,
  });
}

/**
 * Sign with an ad-hoc signature (macOS only).
 *
 * Ad-hoc signatures don't require a certificate but won't pass Gatekeeper.
 * Useful for local development and testing.
 *
 * @param path - Path to the file to sign
 * @throws Error on non-macOS platforms or if signing fails
 *
 * @example
 * ```ts
 * await signAdhoc("/path/to/MyApp.app");
 * ```
 */
export async function signAdhoc(path: string): Promise<void> {
  return await core.ops.op_codesign_sign_adhoc(path);
}

/**
 * Verify a code signature.
 *
 * @param path - Path to the signed file
 * @returns Verification result with validity and signer info
 *
 * @example
 * ```ts
 * const result = await verify("/path/to/MyApp.app");
 * if (result.valid) {
 *   console.log(`Signed by: ${result.signer}`);
 * } else {
 *   console.error(`Invalid signature: ${result.message}`);
 * }
 * ```
 */
export async function verify(path: string): Promise<VerifyResult> {
  const result = await core.ops.op_codesign_verify(path);
  return {
    valid: result.valid,
    signer: result.signer,
    timestamp: result.timestamp,
    message: result.message,
  };
}

/**
 * Get entitlements from a signed binary (macOS only).
 *
 * @param path - Path to the signed binary
 * @returns Entitlements as XML plist string, or empty string if none
 * @throws Error on non-macOS platforms
 *
 * @example
 * ```ts
 * const entitlements = await getEntitlements("/path/to/MyApp.app/Contents/MacOS/MyApp");
 * console.log(entitlements);
 * ```
 */
export async function getEntitlements(path: string): Promise<string> {
  return await core.ops.op_codesign_get_entitlements(path);
}

/**
 * List available signing identities/certificates.
 *
 * @returns Array of available signing identities
 *
 * @example
 * ```ts
 * const identities = await listIdentities();
 * for (const identity of identities) {
 *   console.log(`${identity.name} (${identity.id})`);
 *   console.log(`  Valid: ${identity.valid}, Expires: ${identity.expires}`);
 * }
 * ```
 */
export async function listIdentities(): Promise<SigningIdentity[]> {
  const identities = await core.ops.op_codesign_list_identities();
  return identities.map((id) => ({
    id: id.id,
    name: id.name,
    expires: id.expires,
    valid: id.valid,
    type: id.identity_type,
  }));
}

/**
 * Get detailed information about a specific signing identity.
 *
 * @param identity - Identity name or SHA-1 thumbprint
 * @returns Detailed identity information
 * @throws Error if identity not found
 *
 * @example
 * ```ts
 * const info = await getIdentityInfo("Developer ID Application");
 * console.log(`Found: ${info.name}`);
 * ```
 */
export async function getIdentityInfo(
  identity: string
): Promise<SigningIdentity> {
  const info = await core.ops.op_codesign_get_identity_info(identity);
  return {
    id: info.id,
    name: info.name,
    expires: info.expires,
    valid: info.valid,
    type: info.identity_type,
  };
}

/**
 * Check what signing capabilities are available on the current platform.
 *
 * @returns Available capabilities
 *
 * @example
 * ```ts
 * const caps = checkCapabilities();
 * if (caps.platform === "macos" && caps.codesign) {
 *   console.log("macOS code signing available");
 * } else if (caps.platform === "windows" && caps.signtool) {
 *   console.log("Windows code signing available");
 * } else {
 *   console.log("Code signing not available on this platform");
 * }
 * ```
 */
export function checkCapabilities(): CodesignCapabilities {
  const caps = core.ops.op_codesign_check_capabilities();
  return {
    codesign: caps.codesign,
    security: caps.security,
    signtool: caps.signtool,
    certutil: caps.certutil,
    platform: caps.platform as "macos" | "windows" | "linux",
  };
}

// Convenience aliases
export { sign as codesign };
export { verify as verifySignature };


// ============================================================================
// Extensibility API (auto-generated)
// ============================================================================

/** Registry of operations with their argument and result types */
interface OpRegistry {
  sign: { args: []; result: void };
  signAdhoc: { args: []; result: void };
  verify: { args: []; result: void };
  getEntitlements: { args: []; result: void };
  listIdentities: { args: []; result: void };
  getIdentityInfo: { args: []; result: void };
  checkCapabilities: { args: []; result: void };
}

/** Extract argument types for an operation */
type OpArgs<T extends keyof OpRegistry> = OpRegistry[T]['args'];

/** Extract result type for an operation */
type OpResult<T extends keyof OpRegistry> = OpRegistry[T]['result'];

/** Valid operation names for this extension */
type OpName = "sign" | "signAdhoc" | "verify" | "getEntitlements" | "listIdentities" | "getIdentityInfo" | "checkCapabilities";

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

