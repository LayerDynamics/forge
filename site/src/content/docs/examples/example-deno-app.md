---
title: "example-deno-app"
description: Minimal Forge app demonstrating basic window creation and IPC
slug: examples/example-deno-app
---

The simplest Forge app - demonstrates basic window creation and IPC communication.

## Overview

This example shows:
- Minimal `manifest.app.toml` configuration
- Basic window creation
- Deno ↔ WebView IPC pattern

## Project Structure

```text
example-deno-app/
├── manifest.app.toml
├── deno.json
├── src/
│   └── main.ts
└── web/
    └── index.html
```

## Running

```bash
forge dev examples/example-deno-app
```

## manifest.app.toml

```toml
[app]
name = "ExampleDenoApp"
identifier = "com.example.denoapp"
version = "0.1.0"

[windows]
width = 960
height = 600
resizable = true

[capabilities.channels]
allowed = ["*"]  # Allow all IPC channels
```

## Key Concepts

### Window Configuration

The `[windows]` section defines default window properties:
- `width`/`height` - Initial dimensions
- `resizable` - Whether user can resize

### IPC Channels

The `capabilities.channels.allowed = ["*"]` permits all IPC communication between Deno and WebView.

## Use As Template

This is the ideal starting point for new apps:

```bash
cp -r examples/example-deno-app my-app
# Edit manifest.app.toml with your app details
forge dev my-app
```
