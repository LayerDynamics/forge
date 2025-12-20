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

export interface FetchOptions {
  method?: string;
  headers?: Record<string, string>;
  body?: string;
}

export interface RawFetchResponse {
  ok: boolean;
  status: number;
  status_text?: string;
  statusText?: string;
  headers: Record<string, string>;
  body: string;
}

export interface RawFetchBytesResponse {
  ok: boolean;
  status: number;
  status_text?: string;
  statusText?: string;
  headers: Record<string, string>;
  body: number[];
}

export interface FetchResponse {
  ok: boolean;
  status: number;
  statusText: string;
  headers: Record<string, string>;
  body: string;
}

export interface FetchBytesResponse {
  ok: boolean;
  status: number;
  statusText: string;
  headers: Record<string, string>;
  body: Uint8Array;
}

// WebSocket types
export interface WebSocketConnectOptions {
  headers?: Record<string, string>;
  protocols?: string[];
}

export interface WebSocketConnectionResult {
  id: bigint;
  url: string;
  protocol: string | null;
}

export interface WebSocketMessageData {
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
export interface StreamResponseResult {
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


// ============================================================================
// Extensibility API (auto-generated)
// ============================================================================

/** Registry of operations with their argument and result types */
interface OpRegistry {
  fetch: { args: []; result: void };
  fetchBytes: { args: []; result: void };
  wsConnect: { args: []; result: void };
  wsSend: { args: []; result: void };
  wsRecv: { args: []; result: void };
  wsClose: { args: []; result: void };
  fetchStream: { args: []; result: void };
}

/** Extract argument types for an operation */
type OpArgs<T extends keyof OpRegistry> = OpRegistry[T]['args'];

/** Extract result type for an operation */
type OpResult<T extends keyof OpRegistry> = OpRegistry[T]['result'];

/** Valid operation names for this extension */
type OpName = "fetch" | "fetchBytes" | "wsConnect" | "wsSend" | "wsRecv" | "wsClose" | "fetchStream";

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

