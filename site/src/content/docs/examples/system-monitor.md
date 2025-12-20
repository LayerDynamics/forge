---
title: "system-monitor"
description: System monitor demonstrating multi-window, tray icons, and process APIs
slug: examples/system-monitor
---

A system resource monitor demonstrating advanced Forge capabilities.

## Overview

This example shows:
- Multi-window management
- System tray icon with menu
- Process listing with `runtime:process`
- System info with `runtime:sys`
- Real-time updates

## Features

- CPU and memory usage display
- Running processes list
- System tray icon with context menu
- Multiple detachable windows

## Running

```bash
forge dev examples/system-monitor
```

## Capabilities

```toml
[capabilities.sys]
info = true           # System information
notifications = true  # Desktop notifications

[capabilities.process]
list = true    # List running processes
spawn = false  # No process spawning

[capabilities.channels]
allowed = ["*"]
```

## Key Patterns

### Multi-Window

```typescript
import { createWindow, closeWindow } from "runtime:window";

// Open a new details window
const detailWindow = await createWindow({
  title: "Process Details",
  width: 400,
  height: 300,
  url: "app://details.html"
});
```

### System Tray

```typescript
import { createTray, setTrayMenu } from "runtime:sys";

await createTray({
  icon: "./assets/tray-icon.png",
  tooltip: "Forge Monitor"
});

await setTrayMenu([
  { id: "show", label: "Show Monitor" },
  { id: "sep", type: "separator" },
  { id: "quit", label: "Quit" }
]);
```

### Process Listing

```typescript
import { listProcesses } from "runtime:process";

const processes = await listProcesses();
for (const proc of processes) {
  console.log(`${proc.pid}: ${proc.name} (${proc.memory}KB)`);
}
```

### System Info

```typescript
import { getSystemInfo } from "runtime:sys";

const info = await getSystemInfo();
console.log(`OS: ${info.os} ${info.osVersion}`);
console.log(`Memory: ${info.totalMemory / 1024 / 1024}MB`);
```

## Architecture

```
┌─────────────────┐     ┌─────────────────┐
│   Main Window   │     │  Detail Window  │
│   (dashboard)   │     │   (per-process) │
└────────┬────────┘     └────────┬────────┘
         │                       │
         └───────────┬───────────┘
                     │ IPC
              ┌──────┴──────┐
              │  Deno Core  │
              │ - sys info  │
              │ - processes │
              │ - tray      │
              └─────────────┘
```
