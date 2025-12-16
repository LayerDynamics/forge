// runtime:net module - TypeScript wrapper for Deno core ops

declare const Deno: {
  core: {
    ops: {
      op_net_fetch(url: string, opts: FetchOptions): Promise<RawFetchResponse>;
      op_net_fetch_bytes(url: string, opts: FetchOptions): Promise<RawFetchBytesResponse>;
      // WebSocket operations
      op_net_ws_connect(url: string, opts: WebSocketConnectOptions): Promise<WebSocketConnectionResult>;
      op_net_ws_send(id: bigint, message: WebSocketMessageData): Promise<void>;
      op_net_ws_recv(id: bigint): Promise<WebSocketMessageData | null>;
      op_net_ws_close(id: bigint): Promise<void>;
      // Streaming fetch
      op_net_fetch_stream(url: string, opts: FetchOptions): Promise<StreamResponseResult>;
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

// WebSocket types
interface WebSocketConnectOptions {
  headers?: Record<string, string>;
  protocols?: string[];
}

interface WebSocketConnectionResult {
  id: bigint;
  url: string;
  protocol: string | null;
}

interface WebSocketMessageData {
  type: string;
  data?: string;
  binary?: number[];
}

/**
 * WebSocket connection information
 */
export interface WebSocketConnection {
  id: bigint;
  url: string;
  protocol: string | null;
}

/**
 * WebSocket message
 */
export interface WebSocketMessage {
  type: "text" | "binary" | "ping" | "pong" | "close";
  data?: string;
  binary?: Uint8Array;
}

// Streaming types
interface StreamResponseResult {
  id: bigint;
  status: number;
  status_text: string;
  headers: Record<string, string>;
  url: string;
  ok: boolean;
}

/**
 * Streaming fetch response
 */
export interface StreamResponse {
  id: bigint;
  status: number;
  statusText: string;
  headers: Record<string, string>;
  url: string;
  ok: boolean;
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

// ============================================================================
// WebSocket API
// ============================================================================

/**
 * WebSocket client for real-time communication
 */
export const ws = {
  /**
   * Connect to a WebSocket server.
   * @param url - The WebSocket URL to connect to (ws:// or wss://)
   * @param opts - Connection options
   * @returns Connection information including the connection ID
   */
  async connect(url: string, opts: WebSocketConnectOptions = {}): Promise<WebSocketConnection> {
    const result = await core.ops.op_net_ws_connect(url, opts);
    return {
      id: result.id,
      url: result.url,
      protocol: result.protocol,
    };
  },

  /**
   * Send a message over a WebSocket connection.
   * @param id - The connection ID from connect()
   * @param message - The message to send
   */
  async send(id: bigint, message: WebSocketMessage): Promise<void> {
    const msgData: WebSocketMessageData = {
      type: message.type,
      data: message.data,
      binary: message.binary ? Array.from(message.binary) : undefined,
    };
    return await core.ops.op_net_ws_send(id, msgData);
  },

  /**
   * Send a text message over a WebSocket connection.
   * @param id - The connection ID from connect()
   * @param text - The text message to send
   */
  async sendText(id: bigint, text: string): Promise<void> {
    return await this.send(id, { type: "text", data: text });
  },

  /**
   * Send binary data over a WebSocket connection.
   * @param id - The connection ID from connect()
   * @param data - The binary data to send
   */
  async sendBinary(id: bigint, data: Uint8Array): Promise<void> {
    return await this.send(id, { type: "binary", binary: data });
  },

  /**
   * Receive a message from a WebSocket connection.
   * @param id - The connection ID from connect()
   * @returns The received message, or null if the connection was closed
   */
  async recv(id: bigint): Promise<WebSocketMessage | null> {
    const result = await core.ops.op_net_ws_recv(id);
    if (!result) return null;
    return {
      type: result.type as WebSocketMessage["type"],
      data: result.data,
      binary: result.binary ? new Uint8Array(result.binary) : undefined,
    };
  },

  /**
   * Close a WebSocket connection.
   * @param id - The connection ID from connect()
   */
  async close(id: bigint): Promise<void> {
    return await core.ops.op_net_ws_close(id);
  },

  /**
   * Create an async iterator for receiving WebSocket messages.
   * @param id - The connection ID from connect()
   */
  async *messages(id: bigint): AsyncGenerator<WebSocketMessage, void, unknown> {
    while (true) {
      const msg = await this.recv(id);
      if (msg === null || msg.type === "close") {
        break;
      }
      yield msg;
    }
  },
};

// ============================================================================
// Streaming Fetch API
// ============================================================================

/**
 * Start a streaming fetch request.
 * @param url - The URL to fetch
 * @param opts - Fetch options
 * @returns Stream response with ID for reading chunks
 */
export async function fetchStream(url: string, opts: FetchOptions = {}): Promise<StreamResponse> {
  const result = await core.ops.op_net_fetch_stream(url, opts);
  return {
    id: result.id,
    status: result.status,
    statusText: result.status_text,
    headers: result.headers,
    url: result.url,
    ok: result.ok,
  };
}

// Legacy aliases
export const connectWebSocket = ws.connect;
export const sendWebSocket = ws.send;
export const recvWebSocket = ws.recv;
export const closeWebSocket = ws.close;
