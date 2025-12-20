// runtime:svelte module - SvelteKit adapter functionality for Forge/Deno

// Deno.core type declaration
declare const Deno: {
  core: {
    ops: {
      // Detection ops
      op_svelte_detect(dir: string): Promise<SvelteDetectionResult>;
      // Build phase ops
      op_svelte_walk(dir: string): Promise<WalkResult>;
      op_svelte_generate_deploy_config(
        prerenderedPages: PrerenderedPage[],
        staticDir: string,
        basePath: string,
        assets: string[],
        assetDir: string
      ): Promise<DeployConfig>;
      op_svelte_generate_svelte_data(routes: RouteConfig[]): SvelteData;
      // Runtime phase ops
      op_svelte_prepare_server(
        svelteData: SvelteData,
        deployConfig: DeployConfig,
        cwd: string
      ): Promise<string>;
      op_svelte_get_isr_config(
        serverId: string,
        pathname: string,
        method: string,
        headers: [string, string][],
        cookies: string[]
      ): IsrConfigResult;
      op_svelte_to_cache_key(
        pathname: string,
        searchParams: [string, string][],
        allowQuery: string[]
      ): CacheKeyResult;
      op_svelte_is_static(serverId: string, pathname: string): boolean;
      op_svelte_get_redirect(serverId: string, pathname: string): Redirect | null;
      op_svelte_get_headers(serverId: string, pathname: string): Header[];
      op_svelte_close_server(serverId: string): Promise<void>;
    };
  };
};

// ============================================================================
// Types
// ============================================================================

/** Serializable regex pattern (matches JS RegExp serialization) */
export interface RegexPattern {
  /** Regex source pattern */
  source: string;
  /** Regex flags (e.g., "i" for case-insensitive) */
  flags: string;
}

/** Raw ISR config stored in svelte.json */
export interface IsrConfigRaw {
  /** Regex pattern to match URLs */
  pattern: RegexPattern;
  /** Cache expiration in seconds (default: 604800 = 7 days) */
  expiration: number;
  /** Token to bypass cache (optional) */
  bypassToken: string | null;
  /** Query params to include in cache key */
  allowQuery: string[];
}

/** Svelte metadata stored in svelte.json */
export interface SvelteData {
  /** ISR configurations for routes */
  isr: IsrConfigRaw[];
}

/** Static file mapping */
export interface StaticFile {
  /** URL pattern to match */
  source: string;
  /** Destination file path */
  destination: string;
}

/** Redirect rule */
export interface Redirect {
  /** Source URL pattern */
  source: string;
  /** Destination URL */
  destination: string;
  /** Whether this is a permanent (301) redirect */
  permanent: boolean;
}

/** HTTP header */
export interface Header {
  /** Header name */
  key: string;
  /** Header value */
  value: string;
}

/** Header rule (applies headers to matching paths) */
export interface HeaderRule {
  /** URL pattern to match */
  source: string;
  /** Headers to apply */
  headers: Header[];
}

/** URL rewrite rule */
export interface Rewrite {
  /** Source URL pattern */
  source: string;
  /** Destination to rewrite to */
  destination: string;
}

/** Deploy configuration (deploy.json) */
export interface DeployConfig {
  /** Static file mappings */
  staticFiles: StaticFile[];
  /** Redirect rules */
  redirects: Redirect[];
  /** Header rules */
  headers: HeaderRule[];
  /** Rewrite rules */
  rewrites: Rewrite[];
}

/** Prerendered page info */
export interface PrerenderedPage {
  /** URL pathname (e.g., "/about/") */
  pathname: string;
  /** File path relative to output dir */
  file: string;
}

/** Route configuration from SvelteKit builder */
export interface RouteConfig {
  /** Regex pattern source */
  patternSource: string;
  /** Regex pattern flags */
  patternFlags: string;
  /** Whether this route is prerendered */
  prerender: boolean;
  /** ISR configuration for this route */
  isr?: IsrRouteConfig;
}

/** ISR configuration for a single route */
export interface IsrRouteConfig {
  /** Cache expiration in seconds */
  expiration?: number;
  /** Bypass token */
  bypassToken?: string;
  /** Query params to include in cache key */
  allowQuery?: string[];
}

/** Result from walk operation */
export interface WalkResult {
  /** List of file paths found */
  files: string[];
}

/** ISR config result from getIsrConfig */
export interface IsrConfigResult {
  /** Whether an ISR config was found */
  found: boolean;
  /** The ISR config if found */
  config: IsrConfigRaw | null;
}

/** Cache key result */
export interface CacheKeyResult {
  /** The generated cache key */
  key: string;
}

/** SvelteKit project detection result */
export interface SvelteDetectionResult {
  /** Whether this is a SvelteKit project */
  isSveltekit: boolean;
  /** Whether svelte.config.js/ts exists */
  hasSvelteConfig: boolean;
  /** Whether @sveltejs/kit is in package.json dependencies */
  hasKitDependency: boolean;
  /** Whether src/routes directory exists */
  hasRoutesDir: boolean;
  /** Svelte version from package.json (if found) */
  svelteVersion: string | null;
  /** SvelteKit version from package.json (if found) */
  kitVersion: string | null;
  /** Path to svelte.config.js/ts (if found) */
  configPath: string | null;
  /** Detected adapter from svelte.config (if any) */
  adapter: string | null;
  /** Confidence score (0.0 - 1.0) */
  confidence: number;
  /** Detection messages/notes */
  messages: string[];
}

// ============================================================================
// Constants
// ============================================================================

/** Default ISR expiration in seconds (7 days) */
export const DEFAULT_ISR_EXPIRATION = 604800;

/** Output directory for adapter build */
export const OUT_DIR = ".deno-deploy";

// ============================================================================
// Core ops access
// ============================================================================

const core = Deno.core;

// ============================================================================
// Detection Functions
// ============================================================================

/**
 * Detect if a directory contains a SvelteKit project
 * Checks for: svelte.config.js/ts, @sveltejs/kit in package.json, src/routes directory
 * @param dir - Directory path to check
 * @returns Detection result with confidence score and details
 */
export async function detect(dir: string): Promise<SvelteDetectionResult> {
  return await core.ops.op_svelte_detect(dir);
}

// ============================================================================
// Build Phase Functions
// ============================================================================

/**
 * Walk directory recursively and return all file paths
 * @param dir - Directory path to walk
 * @returns Array of file paths
 */
export async function walk(dir: string): Promise<string[]> {
  const result = await core.ops.op_svelte_walk(dir);
  return result.files;
}

/**
 * Generate deploy.json configuration from prerendered pages and assets
 * @param prerenderedPages - Array of prerendered pages
 * @param staticDir - Output directory for static files
 * @param basePath - Base path for the app
 * @param assets - Array of asset file paths
 * @param assetDir - Source asset directory
 * @returns Deploy configuration
 */
export async function generateDeployConfig(
  prerenderedPages: PrerenderedPage[],
  staticDir: string,
  basePath: string,
  assets: string[],
  assetDir: string
): Promise<DeployConfig> {
  return await core.ops.op_svelte_generate_deploy_config(
    prerenderedPages,
    staticDir,
    basePath,
    assets,
    assetDir
  );
}

/**
 * Generate svelte.json with ISR configuration from routes
 * @param routes - Array of route configurations
 * @returns Svelte metadata with ISR configs
 */
export function generateSvelteData(routes: RouteConfig[]): SvelteData {
  return core.ops.op_svelte_generate_svelte_data(routes);
}

// ============================================================================
// Runtime Phase Functions
// ============================================================================

/**
 * Prepare server for handling requests
 * Compiles ISR patterns and returns a server ID for subsequent operations
 * @param svelteData - Svelte metadata from svelte.json
 * @param deployConfig - Deploy configuration from deploy.json
 * @param cwd - Current working directory
 * @returns Server ID for subsequent operations
 */
export async function prepareServer(
  svelteData: SvelteData,
  deployConfig: DeployConfig,
  cwd: string
): Promise<string> {
  return await core.ops.op_svelte_prepare_server(svelteData, deployConfig, cwd);
}

/**
 * Check if a URL matches ISR rules and return configuration
 * Returns null if bypass token matches (cache should be skipped)
 * @param serverId - Server ID from prepareServer
 * @param pathname - URL pathname to check
 * @param method - HTTP method (GET, HEAD, etc.)
 * @param headers - HTTP headers as key-value pairs or Map
 * @param cookies - Array of cookie strings
 * @returns ISR config if found, null otherwise
 */
export function getIsrConfig(
  serverId: string,
  pathname: string,
  method: string,
  headers: Map<string, string> | [string, string][],
  cookies: string[]
): IsrConfigRaw | null {
  const headerArr: [string, string][] =
    headers instanceof Map ? Array.from(headers.entries()) : headers;

  const result = core.ops.op_svelte_get_isr_config(
    serverId,
    pathname,
    method,
    headerArr,
    cookies
  );

  return result.found ? result.config : null;
}

/**
 * Generate cache key from URL and ISR configuration
 * @param url - URL object
 * @param config - ISR configuration with allowQuery list
 * @returns Cache key string
 */
export function toCacheKey(url: URL, config: IsrConfigRaw): string {
  const searchParams: [string, string][] = Array.from(url.searchParams.entries());
  const result = core.ops.op_svelte_to_cache_key(
    url.pathname,
    searchParams,
    config.allowQuery
  );
  return result.key;
}

/**
 * Check if a path should be served as a static file
 * @param serverId - Server ID from prepareServer
 * @param pathname - URL pathname to check
 * @returns True if path matches static file pattern
 */
export function isStatic(serverId: string, pathname: string): boolean {
  return core.ops.op_svelte_is_static(serverId, pathname);
}

/**
 * Get redirect destination if path matches a redirect rule
 * @param serverId - Server ID from prepareServer
 * @param pathname - URL pathname to check
 * @returns Redirect rule if found, null otherwise
 */
export function getRedirect(serverId: string, pathname: string): Redirect | null {
  return core.ops.op_svelte_get_redirect(serverId, pathname);
}

/**
 * Get headers to apply for a given path
 * @param serverId - Server ID from prepareServer
 * @param pathname - URL pathname to check
 * @returns Array of headers to apply
 */
export function getHeaders(serverId: string, pathname: string): Header[] {
  return core.ops.op_svelte_get_headers(serverId, pathname);
}

/**
 * Clean up server resources
 * @param serverId - Server ID to close
 */
export async function closeServer(serverId: string): Promise<void> {
  return await core.ops.op_svelte_close_server(serverId);
}

// ============================================================================
// Helper Functions
// ============================================================================

/**
 * Parse cookies from Cookie header string
 * @param cookieHeader - Cookie header value
 * @returns Array of cookie strings (key=value)
 */
export function parseCookies(cookieHeader: string | null): string[] {
  if (!cookieHeader) return [];
  return cookieHeader.split(";").map((c) => c.trim());
}

/**
 * Get client IP address from Deno serve info
 * @param info - Deno.ServeHandlerInfo
 * @returns Client IP address string
 */
export function getClientAddress(info: { remoteAddr: { hostname?: string } }): string {
  if ("hostname" in info.remoteAddr && info.remoteAddr.hostname) {
    return info.remoteAddr.hostname;
  }
  return "127.0.0.1";
}
