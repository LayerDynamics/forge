---
title: "ext_webview"
description: WebView management extension providing the runtime:webview module.
slug: crates/ext-webview
---

The `ext_webview` crate provides WebView creation and management for Forge applications through the `runtime:webview` module.

## Overview

ext_webview handles:

- **WebView creation** - Create WebView instances
- **Navigation** - Load URLs and HTML content
- **JavaScript execution** - Run JS in WebView context
- **WebView events** - Handle navigation and load events
- **WebView configuration** - User agent, dev tools, etc.

## Module: `runtime:webview`

```typescript
import {
  create,
  navigate,
  loadHtml,
  executeScript,
  close,
  reload
} from "runtime:webview";
```

## Key Types

### Error Types

```rust
enum WebviewErrorCode {
    Generic = 9400,
    CreateFailed = 9401,
    NavigateFailed = 9402,
    ScriptFailed = 9403,
    NotFound = 9404,
    InvalidUrl = 9405,
}

struct WebviewError {
    code: WebviewErrorCode,
    message: String,
}
```

### WebView Types

```rust
struct WebviewHandle {
    id: u32,
}

struct WebviewConfig {
    url: Option<String>,
    html: Option<String>,
    user_agent: Option<String>,
    dev_tools: Option<bool>,
    transparent: Option<bool>,
    autoplay: Option<bool>,
    incognito: Option<bool>,
}

struct WebviewState {
    webviews: HashMap<u32, Webview>,
    next_id: u32,
}

struct NavigationEvent {
    webview_id: u32,
    url: String,
}

struct LoadEvent {
    webview_id: u32,
    success: bool,
}
```

## Operations

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_webview_create` | `create(config?)` | Create new WebView |
| `op_webview_navigate` | `navigate(handle, url)` | Navigate to URL |
| `op_webview_load_html` | `loadHtml(handle, html)` | Load HTML content |
| `op_webview_execute` | `executeScript(handle, js)` | Execute JavaScript |
| `op_webview_close` | `close(handle)` | Close WebView |
| `op_webview_reload` | `reload(handle)` | Reload content |
| `op_webview_go_back` | `goBack(handle)` | Navigate back |
| `op_webview_go_forward` | `goForward(handle)` | Navigate forward |
| `op_webview_get_url` | `getUrl(handle)` | Get current URL |

## Usage Examples

### Creating a WebView

```typescript
import { create, navigate } from "runtime:webview";

const webview = await create({
  url: "https://example.com",
  devTools: true,
  userAgent: "MyApp/1.0"
});
```

### Loading HTML Content

```typescript
import { create, loadHtml } from "runtime:webview";

const webview = await create();

await loadHtml(webview, `
  <!DOCTYPE html>
  <html>
    <head><title>Hello</title></head>
    <body>
      <h1>Hello from Forge!</h1>
    </body>
  </html>
`);
```

### Executing JavaScript

```typescript
import { create, navigate, executeScript } from "runtime:webview";

const webview = await create({ url: "https://example.com" });

// Wait for page load, then execute
const result = await executeScript(webview, `
  document.title
`);
console.log("Page title:", result);

// Modify page content
await executeScript(webview, `
  document.body.style.backgroundColor = "lightblue";
`);
```

### Navigation

```typescript
import { create, navigate, goBack, goForward, reload } from "runtime:webview";

const webview = await create({ url: "https://example.com" });

// Navigate to new page
await navigate(webview, "https://example.com/page2");

// Navigation history
await goBack(webview);
await goForward(webview);

// Reload
await reload(webview);
```

### Cleanup

```typescript
import { create, close } from "runtime:webview";

const webview = await create({ url: "https://example.com" });

// When done
await close(webview);
```

## Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `url` | `string` | - | Initial URL to load |
| `html` | `string` | - | Initial HTML content |
| `userAgent` | `string` | System default | Custom user agent |
| `devTools` | `boolean` | `false` | Enable dev tools |
| `transparent` | `boolean` | `false` | Transparent background |
| `autoplay` | `boolean` | `true` | Allow media autoplay |
| `incognito` | `boolean` | `false` | Incognito/private mode |

## File Structure

```text
crates/ext_webview/
├── src/
│   └── lib.rs        # Extension implementation
├── ts/
│   └── init.ts       # TypeScript module shim
├── build.rs          # forge-weld build configuration
└── Cargo.toml
```

## Rust Implementation

Operations are annotated with forge-weld macros for automatic TypeScript binding generation:

```rust
// src/lib.rs
use deno_core::{op2, Extension, OpState};
use forge_weld_macro::{weld_op, weld_struct};
use serde::{Deserialize, Serialize};

#[weld_struct]
#[derive(Debug, Serialize)]
pub struct WebViewNewResult {
    pub id: String,
}

#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_host_webview_new(
    state: Rc<RefCell<OpState>>,
    #[serde] params: WebViewNewParams,
) -> Result<WebViewNewResult, WebViewError> {
    // implementation
}
```

## Build Configuration

```rust
// build.rs
use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_webview", "runtime:webview")
        .ts_path("ts/init.ts")
        .ops(&["op_host_webview_new", "op_host_webview_exit", "op_host_webview_eval", /* ... */])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build runtime_webview extension");
}
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `deno_core` | Op definitions |
| `wry` | WebView implementation |
| `tokio` | Async runtime |
| `serde` | Serialization |
| `tracing` | Logging |
| `forge-weld` | Build-time code generation |
| `forge-weld-macro` | `#[weld_op]`, `#[weld_struct]` macros |
| `linkme` | Compile-time symbol collection |
| `ext_window` | Window management integration |

## Related

- [ext_window](/docs/crates/ext-window) - Window management
- [ext_devtools](/docs/crates/ext-devtools) - Developer tools
- [ext_ipc](/docs/crates/ext-ipc) - IPC communication
- [Architecture](/docs/architecture) - Full system architecture
