---
title: "nextjs-app"
description: Next.js-style SSR patterns with server-side data fetching
slug: examples/nextjs-app
---

A demonstration of Next.js-style server-side rendering patterns in Forge.

## Overview

This example shows:
- Route-based data fetching in Deno
- Simulated server-side rendering patterns
- Dynamic page data loading

## Features

- Server-side data fetching simulation
- Route-based content switching
- Dashboard with dynamic stats

## Running

```bash
forge dev examples/nextjs-app
```

## Capabilities

```toml
[capabilities.channels]
allowed = ["*"]
```

## Key Patterns

### Server-Side Data Fetching

In Forge, Deno acts as the "server" that provides data to the WebView:

```typescript
import { windowEvents, sendToWindow } from "runtime:ipc";

for await (const event of windowEvents()) {
  if (event.channel === "fetch-data") {
    const { page } = event.payload as { page: string };
    const data = await fetchPageData(page);
    sendToWindow(win.id, "page-data", { page, data });
  }
}
```

### Route-Based Data

```typescript
async function fetchPageData(page: string): Promise<unknown> {
  switch (page) {
    case "/":
      return { title: "Home", content: "Welcome!" };
    case "/dashboard":
      return {
        title: "Dashboard",
        stats: [
          { label: "Users", value: 1234 },
          { label: "Revenue", value: "$12,345" }
        ]
      };
    default:
      return { title: "404", content: "Page not found" };
  }
}
```

### Renderer Navigation

```javascript
// In WebView
window.host.send("fetch-data", { page: "/dashboard" });

window.host.on("page-data", ({ page, data }) => {
  renderPage(data);
});
```

## Architecture

```text
WebView (React/Svelte/Vue)     Deno Backend
       |                            |
       |-- fetch-data("/about") --> |
       |                            | fetchPageData()
       | <-- page-data ------------ |
       |                            |
```

## Extending

Add real API calls for production data:

```toml
[capabilities.net]
fetch = ["https://api.myservice.com/*"]
```

```typescript
async function fetchPageData(page: string): Promise<unknown> {
  const response = await fetch(`https://api.myservice.com${page}`);
  return response.json();
}
```
