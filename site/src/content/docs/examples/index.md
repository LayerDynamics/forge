---
title: "Examples"
description: Sample applications demonstrating Forge capabilities
slug: examples
---

Forge includes several example applications demonstrating different framework capabilities. Each example is a complete, runnable app you can use as a starting point for your own projects.

## Running Examples

```bash
# Run any example with the CLI
forge dev examples/todo-app

# Or using cargo during development
cargo run -p forge_cli -- dev examples/todo-app
```

## Example Applications

### Beginner

| Example | Description | Key Features |
|---------|-------------|--------------|
| [example-deno-app](/docs/examples/example-deno-app) | Minimal starter app | Basic window, IPC |
| [react-app](/docs/examples/react-app) | React + TypeScript starter | React integration, bundling |
| [nextjs-app](/docs/examples/nextjs-app) | Next.js integration | SSR patterns |

### Intermediate

| Example | Description | Key Features |
|---------|-------------|--------------|
| [todo-app](/docs/examples/todo-app) | Todo list with persistence | runtime:fs, file storage |
| [weather-app](/docs/examples/weather-app) | Weather with API calls | runtime:net, runtime:sys |
| [text-editor](/docs/examples/text-editor) | Simple text editor | Dialogs, clipboard, menus |

### Advanced

| Example | Description | Key Features |
|---------|-------------|--------------|
| [system-monitor](/docs/examples/system-monitor) | System resource monitor | Multi-window, tray, process |
| [svelte-app](/docs/examples/svelte-app) | Secure vault with SvelteKit | runtime:svelte, encryption |
| [wasm-forge-example](/docs/examples/wasm-forge-example) | WebAssembly integration | runtime:wasm |
| [developer-toolkit](/docs/examples/developer-toolkit) | Full-featured dev tools | Code signing, crypto, shell |

## Creating a New App

The quickest way to start a new Forge app is copying an example:

```bash
# Copy the todo-app as a starting point
cp -r examples/todo-app my-new-app

# Edit the manifest
nano my-new-app/manifest.app.toml

# Run your app
forge dev my-new-app
```

## App Structure

All examples follow the standard Forge app structure:

```text
my-app/
├── manifest.app.toml   # App configuration and capabilities
├── deno.json           # Deno configuration (TypeScript, imports)
├── src/
│   └── main.ts         # Deno entry point (app logic)
└── web/
    └── index.html      # WebView content (UI)
```

## Capability Patterns

Examples demonstrate different capability configurations:

### Minimal (example-deno-app)
```toml
[capabilities.channels]
allowed = ["*"]
```

### File Access (todo-app)
```toml
[capabilities.fs]
read = ["~/.forge-todo.json"]
write = ["~/.forge-todo.json"]
```

### Network + Notifications (weather-app)
```toml
[capabilities.net]
fetch = ["https://api.open-meteo.com/*"]

[capabilities.sys]
notifications = true
```

### Full Access (developer-toolkit)
```toml
[permissions.fs]
read = ["**/*"]
write = ["**/*"]

[permissions.process]
allow = ["codesign", "signtool", "openssl"]
```
