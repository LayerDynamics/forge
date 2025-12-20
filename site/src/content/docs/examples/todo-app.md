---
title: "todo-app"
description: Todo list demonstrating file persistence with runtime:fs
slug: examples/todo-app
---

A todo list application demonstrating file system persistence and IPC patterns.

## Overview

This example shows:
- Reading/writing JSON files with `runtime:fs`
- Capability-based file access permissions
- Bidirectional IPC for state sync
- Application menus

## Features

- Add, complete, and delete todos
- Persistent storage in `~/.forge-todo.json`
- Custom application menu
- Real-time sync between Deno and UI

## Running

```bash
forge dev examples/todo-app
```

## Capabilities

```toml
[capabilities.fs]
read = ["~/.forge-todo.json"]
write = ["~/.forge-todo.json"]

[capabilities.channels]
allowed = ["*"]
```

Note: File access is scoped to a single file for security.

## Key Patterns

### File Persistence

```typescript
import { readTextFile, writeTextFile, exists } from "runtime:fs";

// Load todos on startup
if (await exists(todoPath)) {
  const content = await readTextFile(todoPath);
  todos = JSON.parse(content);
}

// Save on changes
await writeTextFile(todoPath, JSON.stringify(todos, null, 2));
```

### IPC for State Sync

```typescript
// Deno side - send state to UI
import { sendToWindow } from "runtime:ipc";
sendToWindow(windowId, "todos:update", todos);

// WebView side - receive updates
window.host.on("todos:update", (todos) => {
  renderTodos(todos);
});
```

## Extending

Add more capabilities as needed:

```toml
# Add notifications
[capabilities.sys]
notifications = true

# Add more file locations
[capabilities.fs]
read = ["~/.forge-todo.json", "~/Documents/todos/*"]
write = ["~/.forge-todo.json", "~/Documents/todos/*"]
```
