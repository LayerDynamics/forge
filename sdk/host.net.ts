// host:net module - Deno API for network operations
// This is the single source of truth for the host:net SDK

// Type definitions
export interface FetchOptions {
  method?: string;
  headers?: Record<string, string>;
  body?: string;
  timeout_ms?: number;
}

export interface FetchResponse {
  status: number;
  statusText: string;
  headers: Record<string, string>;
  body: string;
  url: string;
  ok: boolean;
}

export interface FetchBytesResponse {
  status: number;
  statusText: string;
  headers: Record<string, string>;
  body: Uint8Array;
  url: string;
  ok: boolean;
}

// Deno.core.ops type declaration
declare const Deno: {
  core: {
    ops: {
      op_net_fetch(url: string, opts?: FetchOptions): Promise<FetchResponse>;
      op_net_fetch_bytes(url: string, opts?: FetchOptions): Promise<FetchBytesResponse>;
    };
  };
};

/**
 * Fetch a URL and return the response as text.
 * Subject to manifest permissions (net.allow / net.deny).
 *
 * @param url - The URL to fetch
 * @param opts - Optional fetch options (method, headers, body, timeout_ms)
 * @returns Promise resolving to the fetch response
 *
 * @example
 * ```ts
 * import { fetch } from "host:net";
 *
 * const response = await fetch("https://api.example.com/data");
 * if (response.ok) {
 *   console.log(response.body);
 * }
 * ```
 */
export async function fetch(url: string, opts: FetchOptions = {}): Promise<FetchResponse> {
  const response = await Deno.core.ops.op_net_fetch(url, opts);
  // Normalize statusText field name (Rust uses status_text)
  return {
    ...response,
    statusText: (response as any).status_text || response.statusText || "",
  };
}

/**
 * Fetch a URL and return the response as raw bytes.
 * Useful for binary data like images, files, etc.
 *
 * @param url - The URL to fetch
 * @param opts - Optional fetch options (method, headers, body, timeout_ms)
 * @returns Promise resolving to the fetch response with body as Uint8Array
 *
 * @example
 * ```ts
 * import { fetchBytes } from "host:net";
 *
 * const response = await fetchBytes("https://example.com/image.png");
 * if (response.ok) {
 *   // response.body is Uint8Array
 *   await Deno.writeFile("./image.png", response.body);
 * }
 * ```
 */
export async function fetchBytes(url: string, opts: FetchOptions = {}): Promise<FetchBytesResponse> {
  const response = await Deno.core.ops.op_net_fetch_bytes(url, opts);
  return {
    ...response,
    statusText: (response as any).status_text || response.statusText || "",
    body: new Uint8Array(response.body),
  };
}

/**
 * Convenience method to fetch JSON data.
 *
 * @param url - The URL to fetch
 * @param opts - Optional fetch options
 * @returns Promise resolving to the parsed JSON data
 *
 * @example
 * ```ts
 * import { fetchJson } from "host:net";
 *
 * const data = await fetchJson<{ name: string }>("https://api.example.com/user");
 * console.log(data.name);
 * ```
 */
export async function fetchJson<T = unknown>(url: string, opts: FetchOptions = {}): Promise<T> {
  const response = await fetch(url, {
    ...opts,
    headers: {
      "Accept": "application/json",
      ...opts.headers,
    },
  });

  if (!response.ok) {
    throw new Error(`HTTP error ${response.status}: ${response.statusText}`);
  }

  return JSON.parse(response.body) as T;
}

/**
 * POST JSON data to a URL.
 *
 * @param url - The URL to POST to
 * @param data - The data to send as JSON
 * @param opts - Additional fetch options
 * @returns Promise resolving to the fetch response
 *
 * @example
 * ```ts
 * import { postJson } from "host:net";
 *
 * const response = await postJson("https://api.example.com/users", {
 *   name: "Alice",
 *   email: "alice@example.com"
 * });
 * ```
 */
export async function postJson(
  url: string,
  data: unknown,
  opts: Omit<FetchOptions, "method" | "body"> = {}
): Promise<FetchResponse> {
  return fetch(url, {
    ...opts,
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      ...opts.headers,
    },
    body: JSON.stringify(data),
  });
}
