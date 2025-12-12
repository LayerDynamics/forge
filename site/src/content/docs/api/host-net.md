---
title: "host:net"
description: HTTP networking capabilities with capability-based access control.
---

The `host:net` module provides HTTP networking capabilities with capability-based access control.

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
import { fetch } from "host:net";

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
import { fetchBytes } from "host:net";

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
import { fetchJson } from "host:net";

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
import { postJson } from "host:net";

const response = await postJson("https://api.example.com/users", {
  name: "John Doe",
  email: "john@example.com"
});

console.log(response.status);  // 201
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
import { fetch } from "host:net";

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
import { fetchJson, postJson } from "host:net";
import { notify } from "host:sys";

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
