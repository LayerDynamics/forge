// runtime:protocol extension bindings
// Custom URL protocol handler for deep linking (myapp://)

// ============================================================================
// Type Definitions
// ============================================================================

/** Extension metadata */
export interface ExtensionInfo {
  name: string;
  version: string;
  status: string;
}

/** Options for registering a protocol handler */
export interface RegistrationOptions {
  /** Human-readable description of the protocol */
  description?: string;
  /** Path to icon file (platform-specific format) */
  icon_path?: string;
  /** Whether to set this app as the default handler (default: true) */
  set_as_default?: boolean;
}

/** Result of protocol registration */
export interface RegistrationResult {
  /** Whether registration succeeded */
  success: boolean;
  /** The scheme that was registered */
  scheme: string;
  /** Whether the scheme was already registered before this call */
  was_already_registered: boolean;
  /** Previous handler app identifier, if any */
  previous_handler: string | null;
}

/** Status of a protocol registration */
export interface RegistrationStatus {
  /** Whether any handler is registered for this scheme */
  is_registered: boolean;
  /** Whether this app is the default handler */
  is_default: boolean;
  /** App identifier of current handler, if known */
  registered_by: string | null;
}

/** Information about a registered protocol */
export interface ProtocolInfo {
  /** The URL scheme (e.g., "myapp") */
  scheme: string;
  /** Human-readable description */
  description: string | null;
  /** Path to icon file */
  icon_path: string | null;
  /** Whether this app is the default handler */
  is_default: boolean;
  /** App identifier that registered this protocol */
  registered_by: string | null;
}

/** A protocol URL invocation event */
export interface ProtocolInvocation {
  /** Unique invocation ID */
  id: string;
  /** Full URL that was invoked */
  url: string;
  /** URL scheme (e.g., "myapp") */
  scheme: string;
  /** Path portion of URL */
  path: string;
  /** Query parameters as key-value pairs */
  query: Record<string, string>;
  /** URL fragment (after #), if any */
  fragment: string | null;
  /** Unix timestamp (milliseconds) of invocation */
  timestamp: number;
  /** Whether this invocation launched the app */
  is_launch: boolean;
}

/** Parsed URL components */
export interface ParsedProtocolUrl {
  /** URL scheme */
  scheme: string;
  /** Path portion */
  path: string;
  /** Query parameters */
  query: Record<string, string>;
  /** Fragment, if any */
  fragment: string | null;
  /** Whether the URL is valid */
  is_valid: boolean;
}

/** Platform capabilities for protocol handling */
export interface ProtocolCapabilities {
  /** Whether protocol registration is supported */
  can_register: boolean;
  /** Whether protocol querying is supported */
  can_query: boolean;
  /** Whether deep linking is supported */
  can_deep_link: boolean;
  /** Current platform identifier */
  platform: string;
  /** Additional notes about capabilities */
  notes: string | null;
}

// ============================================================================
// Deno Core Bindings
// ============================================================================

declare const Deno: {
  core: {
    ops: {
      op_protocol_info(): ExtensionInfo;
      op_protocol_register(
        scheme: string,
        app_identifier: string,
        app_name: string,
        exe_path: string,
        options: RegistrationOptions
      ): Promise<RegistrationResult>;
      op_protocol_unregister(scheme: string): Promise<boolean>;
      op_protocol_is_registered(scheme: string): Promise<RegistrationStatus>;
      op_protocol_list_registered(): ProtocolInfo[];
      op_protocol_set_as_default(scheme: string): Promise<boolean>;
      op_protocol_get_launch_url(): string | null;
      op_protocol_receive_invocation(): Promise<ProtocolInvocation>;
      op_protocol_parse_url(url: string): ParsedProtocolUrl;
      op_protocol_build_url(
        scheme: string,
        path: string,
        query: Record<string, string> | null
      ): string;
      op_protocol_check_capabilities(): ProtocolCapabilities;
    };
  };
};

const { core } = Deno;
const ops = core.ops;

// ============================================================================
// Public API
// ============================================================================

/**
 * Get extension information
 */
export function info(): ExtensionInfo {
  return ops.op_protocol_info();
}

/**
 * Register a custom URL protocol handler
 *
 * @param scheme - The URL scheme to register (e.g., "myapp" for myapp://)
 * @param options - Optional registration options
 * @returns Registration result with success status and previous handler info
 *
 * @example
 * ```typescript
 * import { register } from "runtime:protocol";
 *
 * const result = await register("myapp", {
 *   description: "My Application Protocol",
 *   set_as_default: true
 * });
 *
 * if (result.success) {
 *   console.log("Protocol registered successfully");
 *   if (result.was_already_registered) {
 *     console.log(`Previously handled by: ${result.previous_handler}`);
 *   }
 * }
 * ```
 */
export async function register(
  scheme: string,
  options: RegistrationOptions = {}
): Promise<RegistrationResult> {
  // Get app info from environment or defaults
  const appIdentifier =
    (globalThis as any).FORGE_APP_IDENTIFIER || "com.forge.app";
  const appName = (globalThis as any).FORGE_APP_NAME || "Forge App";
  const exePath = (globalThis as any).FORGE_EXE_PATH || "";

  return await ops.op_protocol_register(
    scheme,
    appIdentifier,
    appName,
    exePath,
    options
  );
}

/**
 * Unregister a custom URL protocol handler
 *
 * @param scheme - The URL scheme to unregister
 * @returns true if unregistered, false if wasn't registered
 *
 * @example
 * ```typescript
 * import { unregister } from "runtime:protocol";
 *
 * const wasRegistered = await unregister("myapp");
 * ```
 */
export async function unregister(scheme: string): Promise<boolean> {
  return await ops.op_protocol_unregister(scheme);
}

/**
 * Check if a URL scheme is registered
 *
 * @param scheme - The URL scheme to check
 * @returns Registration status including default handler info
 *
 * @example
 * ```typescript
 * import { isRegistered } from "runtime:protocol";
 *
 * const status = await isRegistered("myapp");
 * if (status.is_registered && status.is_default) {
 *   console.log("This app is the default handler");
 * }
 * ```
 */
export async function isRegistered(scheme: string): Promise<RegistrationStatus> {
  return await ops.op_protocol_is_registered(scheme);
}

/**
 * List all protocols registered by this app
 *
 * @returns Array of protocol information
 *
 * @example
 * ```typescript
 * import { listRegistered } from "runtime:protocol";
 *
 * const protocols = listRegistered();
 * for (const proto of protocols) {
 *   console.log(`${proto.scheme}:// - ${proto.description}`);
 * }
 * ```
 */
export function listRegistered(): ProtocolInfo[] {
  return ops.op_protocol_list_registered();
}

/**
 * Set this app as the default handler for a scheme
 *
 * @param scheme - The URL scheme to become default handler for
 * @returns true if successful
 *
 * @example
 * ```typescript
 * import { setAsDefault } from "runtime:protocol";
 *
 * await setAsDefault("myapp");
 * ```
 */
export async function setAsDefault(scheme: string): Promise<boolean> {
  return await ops.op_protocol_set_as_default(scheme);
}

/**
 * Get the URL that launched this app, if any
 *
 * @returns The launch URL or null if app wasn't launched via protocol
 *
 * @example
 * ```typescript
 * import { getLaunchUrl } from "runtime:protocol";
 *
 * const launchUrl = getLaunchUrl();
 * if (launchUrl) {
 *   console.log(`App launched with: ${launchUrl}`);
 *   // Handle the deep link
 * }
 * ```
 */
export function getLaunchUrl(): string | null {
  return ops.op_protocol_get_launch_url();
}

/**
 * Receive protocol invocation events
 *
 * This is a low-level API. For most use cases, prefer `onInvoke()`.
 *
 * @returns Promise that resolves with the next invocation
 */
export async function receiveInvocation(): Promise<ProtocolInvocation> {
  return await ops.op_protocol_receive_invocation();
}

/**
 * Listen for protocol URL invocations
 *
 * @param callback - Function called when a protocol URL is invoked
 * @returns Cleanup function to stop listening
 *
 * @example
 * ```typescript
 * import { onInvoke } from "runtime:protocol";
 *
 * const cleanup = onInvoke((invocation) => {
 *   console.log(`Received: ${invocation.url}`);
 *   console.log(`Path: ${invocation.path}`);
 *   console.log(`Query: ${JSON.stringify(invocation.query)}`);
 *
 *   // Route based on path
 *   if (invocation.path === "/auth/callback") {
 *     handleAuthCallback(invocation.query);
 *   }
 * });
 *
 * // Later, stop listening
 * cleanup();
 * ```
 */
export function onInvoke(
  callback: (invocation: ProtocolInvocation) => void
): () => void {
  let active = true;

  // Async loop to receive invocations
  (async () => {
    while (active) {
      try {
        const invocation = await receiveInvocation();
        if (active) {
          callback(invocation);
        }
      } catch (err) {
        // Channel closed or error - exit loop
        if (active) {
          console.error("Protocol invocation listener error:", err);
        }
        break;
      }
    }
  })();

  // Return cleanup function
  return () => {
    active = false;
  };
}

/**
 * Parse a protocol URL into components
 *
 * @param url - The URL to parse
 * @returns Parsed URL components
 *
 * @example
 * ```typescript
 * import { parseUrl } from "runtime:protocol";
 *
 * const parsed = parseUrl("myapp://settings/theme?dark=true#section1");
 * // {
 * //   scheme: "myapp",
 * //   path: "settings/theme",
 * //   query: { dark: "true" },
 * //   fragment: "section1",
 * //   is_valid: true
 * // }
 * ```
 */
export function parseUrl(url: string): ParsedProtocolUrl {
  return ops.op_protocol_parse_url(url);
}

/**
 * Build a protocol URL from components
 *
 * @param scheme - The URL scheme
 * @param path - The path portion
 * @param query - Optional query parameters
 * @returns Constructed URL string
 *
 * @example
 * ```typescript
 * import { buildUrl } from "runtime:protocol";
 *
 * const url = buildUrl("myapp", "auth/callback", {
 *   token: "abc123",
 *   redirect: "/dashboard"
 * });
 * // "myapp://auth/callback?token=abc123&redirect=%2Fdashboard"
 * ```
 */
export function buildUrl(
  scheme: string,
  path: string,
  query?: Record<string, string>
): string {
  return ops.op_protocol_build_url(scheme, path, query || null);
}

/**
 * Check platform capabilities for protocol handling
 *
 * @returns Platform capability information
 *
 * @example
 * ```typescript
 * import { checkCapabilities } from "runtime:protocol";
 *
 * const caps = checkCapabilities();
 * if (caps.can_register) {
 *   console.log(`Protocol registration supported on ${caps.platform}`);
 * } else {
 *   console.log(`Note: ${caps.notes}`);
 * }
 * ```
 */
export function checkCapabilities(): ProtocolCapabilities {
  return ops.op_protocol_check_capabilities();
}

// ============================================================================
// Convenience Exports
// ============================================================================

export default {
  info,
  register,
  unregister,
  isRegistered,
  listRegistered,
  setAsDefault,
  getLaunchUrl,
  receiveInvocation,
  onInvoke,
  parseUrl,
  buildUrl,
  checkCapabilities,
};
