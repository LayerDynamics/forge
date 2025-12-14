// host:net module - TypeScript wrapper for Deno core ops

declare const Deno: {
  core: {
    ops: {
      op_net_fetch(url: string, opts: FetchOptions): Promise<RawFetchResponse>;
      op_net_fetch_bytes(url: string, opts: FetchOptions): Promise<RawFetchBytesResponse>;
    };
  };
};

interface FetchOptions {
  method?: string;
  headers?: Record<string, string>;
  body?: string;
}

interface RawFetchResponse {
  ok: boolean;
  status: number;
  status_text?: string;
  statusText?: string;
  headers: Record<string, string>;
  body: string;
}

interface RawFetchBytesResponse {
  ok: boolean;
  status: number;
  status_text?: string;
  statusText?: string;
  headers: Record<string, string>;
  body: number[];
}

interface FetchResponse {
  ok: boolean;
  status: number;
  statusText: string;
  headers: Record<string, string>;
  body: string;
}

interface FetchBytesResponse {
  ok: boolean;
  status: number;
  statusText: string;
  headers: Record<string, string>;
  body: Uint8Array;
}

const core = Deno.core;

export async function fetch(url: string, opts: FetchOptions = {}): Promise<FetchResponse> {
  const response = await core.ops.op_net_fetch(url, opts);
  return {
    ok: response.ok,
    status: response.status,
    statusText: response.status_text || response.statusText || "",
    headers: response.headers,
    body: response.body,
  };
}

export async function fetchBytes(url: string, opts: FetchOptions = {}): Promise<FetchBytesResponse> {
  const response = await core.ops.op_net_fetch_bytes(url, opts);
  return {
    ok: response.ok,
    status: response.status,
    statusText: response.status_text || response.statusText || "",
    headers: response.headers,
    body: new Uint8Array(response.body),
  };
}

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

export async function postJson<T = unknown>(url: string, data: unknown, opts: FetchOptions = {}): Promise<FetchResponse> {
  return await fetch(url, {
    ...opts,
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      ...opts.headers,
    },
    body: JSON.stringify(data),
  });
}
