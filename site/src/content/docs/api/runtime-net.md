---
title: "runtime:net"
description: HTTP networking, WebSocket connections, and streaming capabilities with capability-based access control.
slug: docs/api/runtime-net
---

The `runtime:net` module provides HTTP networking, WebSocket connections, and streaming capabilities with capability-based access control.

> **Implementation**: TypeScript types are auto-generated from Rust via [forge-weld](/docs/crates/forge-weld). See [ext_net](/docs/crates/ext-net) for implementation details.

## Capabilities

Network access must be declared in `manifest.app.toml`:

```toml
[capabilities.net]
fetch = ["https://api.example.com/*", "https://cdn.example.com/*"]
```

Glob patterns for URL matching:
- `*` - matches any characters except `/`
- `**` - matches any characters including `/`

---

## HTTP Fetch

### fetch(url, options?)

Fetch a URL and return response as text:

```typescript
import { fetch } from "runtime:net";

const response = await fetch("https://api.example.com/data");

console.log(response.status);      // 200
console.log(response.ok);          // true
console.log(response.body);        // Response body as string
console.log(response.headers);     // { "content-type": "application/json" }
```

**Options:**

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `method` | `string` | `"GET"` | HTTP method |
| `headers` | `Record<string, string>` | `{}` | Request headers |
| `body` | `string` | - | Request body |
| `timeout_ms` | `number` | `30000` | Timeout in milliseconds |

**Returns:**

```typescript
interface FetchResponse {
  status: number;
  statusText: string;
  headers: Record<string, string>;
  body: string;
  url: string;
  ok: boolean;  // true if status 200-299
}
```

### fetchBytes(url, options?)

Fetch a URL and return response as raw bytes:

```typescript
import { fetchBytes } from "runtime:net";

const response = await fetchBytes("https://example.com/image.png");
const imageData = response.body;  // Uint8Array
```

**Returns:**

```typescript
interface FetchBytesResponse {
  status: number;
  statusText: string;
  headers: Record<string, string>;
  body: Uint8Array;
  url: string;
  ok: boolean;
}
```

### fetchJson<T>(url, options?)

Fetch a URL and parse response as JSON:

```typescript
import { fetchJson } from "runtime:net";

interface User {
  id: number;
  name: string;
  email: string;
}

const user = await fetchJson<User>("https://api.example.com/users/1");
console.log(user.name);
```

### postJson(url, data, options?)

POST JSON data to a URL:

```typescript
import { postJson } from "runtime:net";

const response = await postJson("https://api.example.com/users", {
  name: "John Doe",
  email: "john@example.com"
});

console.log(response.status);  // 201
```

---

## WebSocket

WebSocket provides bidirectional, real-time communication over a single TCP connection. Perfect for chat applications, live updates, gaming, and collaborative tools.

### ws.connect(url, options?)

Connect to a WebSocket server:

```typescript
import { ws } from "runtime:net";

// Basic connection
const conn = await ws.connect("wss://echo.websocket.org");
console.log("Connected:", conn.id);

// With options
const authConn = await ws.connect("wss://api.example.com/ws", {
  headers: {
    "Authorization": "Bearer token"
  },
  protocols: ["chat", "v1"]
});
```

**Options:**

```typescript
interface WebSocketConnectOptions {
  headers?: Record<string, string>;  // Custom headers
  protocols?: string[];               // Subprotocols to negotiate
}
```

**Returns:**

```typescript
interface WebSocketConnection {
  id: bigint;           // Connection ID for subsequent operations
  url: string;          // The connected URL
  protocol: string | null;  // Negotiated subprotocol
}
```

### ws.sendText(id, text)

Send a text message:

```typescript
import { ws } from "runtime:net";

const conn = await ws.connect("wss://chat.example.com");
await ws.sendText(conn.id, "Hello, World!");
await ws.sendText(conn.id, JSON.stringify({ type: "ping" }));
```

### ws.sendBinary(id, data)

Send binary data:

```typescript
import { ws } from "runtime:net";

const conn = await ws.connect("wss://api.example.com/upload");
const imageData = new Uint8Array([0x89, 0x50, 0x4E, 0x47, ...]);
await ws.sendBinary(conn.id, imageData);
```

### ws.recv(id)

Receive the next message (blocking):

```typescript
import { ws } from "runtime:net";

const conn = await ws.connect("wss://echo.websocket.org");
await ws.sendText(conn.id, "Hello");

const msg = await ws.recv(conn.id);
if (msg && msg.type === "text") {
  console.log("Received:", msg.data);
}
```

**Returns:**

```typescript
interface WebSocketMessage {
  type: "text" | "binary" | "ping" | "pong" | "close";
  data?: string;        // For text messages
  binary?: Uint8Array;  // For binary messages
}
```

Returns `null` when the connection is closed.

### ws.messages(id)

Async generator for receiving messages:

```typescript
import { ws } from "runtime:net";

const conn = await ws.connect("wss://chat.example.com");

// Process messages as they arrive
for await (const msg of ws.messages(conn.id)) {
  if (msg.type === "text") {
    console.log("Chat message:", msg.data);
    const parsed = JSON.parse(msg.data!);
    // Handle message...
  } else if (msg.type === "binary") {
    console.log("Binary data:", msg.binary!.length, "bytes");
  }
}

console.log("Connection closed");
```

### ws.close(id)

Close a WebSocket connection:

```typescript
import { ws } from "runtime:net";

const conn = await ws.connect("wss://api.example.com");
// Use the connection...
await ws.close(conn.id);
```

---

## WebSocket Examples

### Chat Client

```typescript
import { ws } from "runtime:net";
import { sendToWindow } from "runtime:ipc";

async function connectToChat(username: string) {
  const conn = await ws.connect("wss://chat.example.com/ws");

  // Send authentication
  await ws.sendText(conn.id, JSON.stringify({
    type: "auth",
    username
  }));

  // Listen for messages
  for await (const msg of ws.messages(conn.id)) {
    if (msg.type === "text") {
      const data = JSON.parse(msg.data!);

      // Forward to UI
      await sendToWindow("main", "chat-message", data);
    }
  }
}
```

### Real-Time Data Stream

```typescript
import { ws } from "runtime:net";

interface StockTick {
  symbol: string;
  price: number;
  timestamp: number;
}

async function streamStockPrices(symbols: string[]) {
  const conn = await ws.connect("wss://api.stocks.com/stream");

  // Subscribe to symbols
  await ws.sendText(conn.id, JSON.stringify({
    action: "subscribe",
    symbols
  }));

  // Process real-time updates
  for await (const msg of ws.messages(conn.id)) {
    if (msg.type === "text") {
      const tick: StockTick = JSON.parse(msg.data!);
      console.log(`${tick.symbol}: $${tick.price}`);

      // Update local cache, notify UI, etc.
    }
  }
}
```

### Binary Protocol

```typescript
import { ws } from "runtime:net";

async function binaryProtocolExample() {
  const conn = await ws.connect("wss://game.example.com");

  // Send binary command
  const command = new Uint8Array([0x01, 0x02, 0x03]);
  await ws.sendBinary(conn.id, command);

  // Receive binary response
  const msg = await ws.recv(conn.id);
  if (msg && msg.type === "binary") {
    const response = msg.binary!;
    console.log("Received", response.length, "bytes");
    // Parse binary protocol...
  }

  await ws.close(conn.id);
}
```

### Ping-Pong Keepalive

```typescript
import { ws } from "runtime:net";

async function keepAlive(id: bigint) {
  const interval = setInterval(async () => {
    try {
      await ws.send(id, { type: "ping" });
    } catch (error) {
      console.error("Keepalive failed:", error);
      clearInterval(interval);
    }
  }, 30000); // Every 30 seconds

  return () => clearInterval(interval);
}

const conn = await ws.connect("wss://api.example.com");
const stopKeepalive = await keepAlive(conn.id);

// Later...
stopKeepalive();
await ws.close(conn.id);
```

---

## Streaming Fetch

For downloading large files or processing responses incrementally:

### fetchStream(url, options?)

Start a streaming HTTP request:

```typescript
import { fetchStream } from "runtime:net";

const stream = await fetchStream("https://cdn.example.com/large-file.bin");

console.log("Status:", stream.status);
console.log("Content-Type:", stream.headers["content-type"]);

// Stream ID can be used to read chunks
// (Note: Chunk reading API depends on implementation)
```

**Returns:**

```typescript
interface StreamResponse {
  id: bigint;
  status: number;
  statusText: string;
  headers: Record<string, string>;
  url: string;
  ok: boolean;
}
```

---

## Request Examples

### GET Request

```typescript
const response = await fetch("https://api.example.com/items");
const items = JSON.parse(response.body);
```

### POST Request

```typescript
const response = await fetch("https://api.example.com/items", {
  method: "POST",
  headers: {
    "Content-Type": "application/json"
  },
  body: JSON.stringify({ name: "New Item" })
});
```

### With Authentication

```typescript
const response = await fetch("https://api.example.com/protected", {
  headers: {
    "Authorization": "Bearer your-token-here"
  }
});
```

### With Timeout

```typescript
const response = await fetch("https://slow-api.example.com/data", {
  timeout_ms: 60000  // 60 seconds
});
```

---

## Error Handling

```typescript
import { fetch } from "runtime:net";

try {
  const response = await fetch("https://api.example.com/data");

  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
  }

  const data = JSON.parse(response.body);
} catch (error) {
  if (error.message.includes("permission")) {
    console.error("URL not allowed - check capabilities");
  } else if (error.message.includes("timeout")) {
    console.error("Request timed out");
  } else {
    console.error("Network error:", error);
  }
}
```

---

## Complete Example

```typescript
import { fetchJson, postJson } from "runtime:net";
import { notify } from "runtime:sys";

interface WeatherData {
  temperature: number;
  description: string;
  humidity: number;
}

async function getWeather(city: string): Promise<WeatherData> {
  try {
    const data = await fetchJson<WeatherData>(
      `https://api.weather.com/v1/current?city=${encodeURIComponent(city)}`
    );
    return data;
  } catch (error) {
    await notify("Weather Error", `Failed to fetch weather: ${error.message}`);
    throw error;
  }
}

async function reportAnalytics(event: string, data: unknown): Promise<void> {
  try {
    await postJson("https://analytics.example.com/events", {
      event,
      data,
      timestamp: Date.now()
    });
  } catch (error) {
    // Silent fail for analytics
    console.warn("Analytics failed:", error);
  }
}
```

---

## Error Codes

Network operations use structured error codes for precise error handling:

| Code | Name | Description |
|------|------|-------------|
| 1000 | IoError | Generic I/O error |
| 1001 | PermissionDenied | URL not allowed by capabilities |
| 1002 | InvalidUrl | Malformed URL |
| 1003 | Timeout | Request exceeded timeout limit |
| 1004 | ConnectionFailed | Failed to connect to server |
| 1005 | HttpError | HTTP error response (4xx, 5xx) |
| 1006 | RequestBuildError | Failed to construct request |
| 1007 | WebSocketConnect | WebSocket connection failed |
| 1008 | WebSocketSend | Failed to send WebSocket message |
| 1009 | WebSocketRecv | Failed to receive WebSocket message |
| 1010 | WebSocketClose | Failed to close WebSocket |
| 1011 | WebSocketNotFound | WebSocket connection ID not found |
| 1012 | StreamError | Streaming operation failed |

### Error Handling Examples

```typescript
import { fetch, ws } from "runtime:net";

// HTTP error handling
try {
  const response = await fetch("https://api.example.com/data");
} catch (error) {
  // Error messages include [code] prefix
  if (error.message.includes("[1001]")) {
    console.error("Permission denied - check manifest.app.toml capabilities");
  } else if (error.message.includes("[1003]")) {
    console.error("Request timed out - increase timeout_ms");
  } else if (error.message.includes("[1002]")) {
    console.error("Invalid URL format");
  }
}

// WebSocket error handling
try {
  const conn = await ws.connect("wss://invalid-url");
} catch (error) {
  if (error.message.includes("[1007]")) {
    console.error("WebSocket connection failed");
  }
}
```

---

## Complete Example: Multi-Protocol Client

```typescript
import { fetch, fetchJson, ws } from "runtime:net";
import { sendToWindow } from "runtime:ipc";
import { notify } from "runtime:sys";
import { set, get } from "runtime:storage";

interface WeatherData {
  temperature: number;
  description: string;
  humidity: number;
}

interface ChatMessage {
  id: string;
  username: string;
  text: string;
  timestamp: number;
}

/**
 * Fetches weather data via HTTP
 */
async function getWeather(city: string): Promise<WeatherData> {
  try {
    const data = await fetchJson<WeatherData>(
      `https://api.weather.com/v1/current?city=${encodeURIComponent(city)}`,
      { timeout_ms: 10000 }
    );

    // Cache the result
    await set(`weather:${city}`, data);

    return data;
  } catch (error) {
    await notify("Weather Error", `Failed to fetch weather: ${error.message}`);

    // Try to return cached data
    const cached = await get(`weather:${city}`);
    if (cached) return cached as WeatherData;

    throw error;
  }
}

/**
 * Establishes WebSocket connection for real-time chat
 */
async function startChatClient(username: string, roomId: string) {
  let conn: { id: bigint } | null = null;

  try {
    // Connect with authentication
    conn = await ws.connect("wss://chat.example.com/ws", {
      headers: {
        "Authorization": `Bearer ${await get("auth_token")}`
      }
    });

    // Join room
    await ws.sendText(conn.id, JSON.stringify({
      type: "join",
      username,
      roomId
    }));

    await notify("Chat", `Connected to ${roomId}`);

    // Process messages
    for await (const msg of ws.messages(conn.id)) {
      if (msg.type === "text") {
        const chatMsg: ChatMessage = JSON.parse(msg.data!);

        // Forward to UI
        await sendToWindow("main", "chat-message", chatMsg);

        // Handle special messages
        if (chatMsg.text.startsWith("/weather ")) {
          const city = chatMsg.text.slice(9);
          const weather = await getWeather(city);

          await ws.sendText(conn.id, JSON.stringify({
            type: "message",
            text: `Weather in ${city}: ${weather.temperature}°C, ${weather.description}`
          }));
        }
      }
    }

    console.log("Chat connection closed");

  } catch (error) {
    console.error("Chat error:", error);
    await notify("Chat Error", error.message);
  } finally {
    if (conn) {
      try {
        await ws.close(conn.id);
      } catch (e) {
        console.error("Failed to close WebSocket:", e);
      }
    }
  }
}

/**
 * Reports analytics events (fire-and-forget)
 */
async function reportAnalytics(event: string, data: unknown): Promise<void> {
  try {
    await fetch("https://analytics.example.com/events", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "X-API-Key": await get("analytics_key") as string
      },
      body: JSON.stringify({
        event,
        data,
        timestamp: Date.now(),
        session: await get("session_id")
      }),
      timeout_ms: 5000
    });
  } catch (error) {
    // Silent fail for analytics
    console.warn("Analytics failed:", error);
  }
}

// Usage
await reportAnalytics("app_start", { version: "1.0.0" });
const weather = await getWeather("San Francisco");
await startChatClient("alice", "general");
```
