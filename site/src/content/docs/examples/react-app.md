---
title: "react-app"
description: React + TypeScript starter for building Forge applications
slug: examples/react-app
---

A minimal React + TypeScript starter demonstrating how to integrate React with Forge.

## Overview

This example shows:
- Basic window creation with `runtime:window`
- Bidirectional IPC with `runtime:ipc`
- React frontend bundling workflow

## Features

- React 18 with TypeScript
- Window creation and focus management
- Ping/pong IPC demonstration
- Development-friendly bundling

## Running

```bash
forge dev examples/react-app
```

## Capabilities

```toml
[capabilities.channels]
allowed = ["*"]
```

## Key Patterns

### Window Creation

```typescript
import { createWindow } from "runtime:window";

const win = await createWindow({
  url: "app://index.html",
  width: 1024,
  height: 768,
  title: "React App"
});

await win.focus();
```

### IPC Event Handling

```typescript
import { windowEvents, sendToWindow } from "runtime:ipc";

for await (const event of windowEvents()) {
  if (event.channel === "ping") {
    sendToWindow(win.id, "pong", { timestamp: Date.now() });
  }
}
```

### React Integration

The WebView loads static HTML that includes bundled React code:

```html
<div id="root"></div>
<script type="module" src="./bundle.js"></script>
```

## Extending

Build on this starter by adding:

```toml
# Add file system access
[capabilities.fs]
read = ["./data/**"]
write = ["./data/**"]

# Add network access
[capabilities.net]
fetch = ["https://api.example.com/*"]
```
