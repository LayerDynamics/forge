---
title: "runtime:protocol"
description: "Custom URL protocol handler for deep linking in Forge applications"
slug: docs/api/runtime-protocol
---

Custom URL protocol handler for deep linking in Forge applications. Register custom URL schemes (like `myapp://`) to enable launching your app from browsers, emails, and other applications.

> **Implementation**: TypeScript types are auto-generated from Rust via [forge-weld](/docs/crates/forge-weld). See [ext_protocol](/docs/crates/ext-protocol) for implementation details.

## Features

- Register custom URL schemes (`myapp://`, `forge://`, etc.)
- Handle deep links when app is running or launches from URL
- Parse and build protocol URLs with query parameters
- Check platform capabilities and registration status
- Listen for protocol invocation events

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    External Source                           │
│     Browser / Email / Other App → myapp://path?query        │
└───────────────────────────┬─────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                   Operating System                           │
│   (URL Scheme Registry / Launch Services / xdg-open)        │
└───────────────────────────┬─────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                 Forge Application                            │
│  getLaunchUrl() ← launch │ onInvoke() ← running              │
└─────────────────────────────────────────────────────────────┘
```

## Platform Support

| Platform | Registration | Deep Linking | Notes |
|----------|--------------|--------------|-------|
| macOS | Full | Full | Uses Launch Services, CFBundleURLTypes |
| Windows | Full | Full | Registry-based URL handler |
| Linux | Partial | Full | Uses xdg-open, .desktop files |

## Import

```typescript
import {
  // Registration
  register,
  unregister,
  isRegistered,
  listRegistered,
  setAsDefault,
  // Invocation handling
  getLaunchUrl,
  onInvoke,
  receiveInvocation,
  // URL utilities
  parseUrl,
  buildUrl,
  // Capabilities
  checkCapabilities,
  // Types
  type RegistrationOptions,
  type RegistrationResult,
  type RegistrationStatus,
  type ProtocolInfo,
  type ProtocolInvocation,
  type ParsedProtocolUrl,
  type ProtocolCapabilities,
} from "runtime:protocol";
```

## API Reference

<!-- forge:api -->
<!-- generated from sdk/runtime.protocol.ts — edit signatures in the SDK, run `make docs-api` to refresh -->
```typescript
info(): ExtensionInfo
register( scheme: string, options: RegistrationOptions =
unregister(scheme: string): Promise<boolean>
isRegistered(scheme: string): Promise<RegistrationStatus>
listRegistered(): ProtocolInfo[]
setAsDefault(scheme: string): Promise<boolean>
getLaunchUrl(): string | null
receiveInvocation(): Promise<ProtocolInvocation>
onInvoke( callback: (invocation: ProtocolInvocation) => void ): () => void
parseUrl(url: string): ParsedProtocolUrl
buildUrl( scheme: string, path: string, query?: Record<string, string> ): string
checkCapabilities(): ProtocolCapabilities
```
<!-- /forge:api -->

### Registration

#### register(scheme, options?)

Register a custom URL protocol handler. This allows URLs like `myapp://` to open your application.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `scheme` | `string` | URL scheme to register (without `://`) |
| `options` | `RegistrationOptions` | Optional registration options |

**RegistrationOptions:**

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `description` | `string` | - | Human-readable protocol description |
| `icon_path` | `string` | - | Path to icon file (platform-specific) |
| `set_as_default` | `boolean` | `true` | Set this app as default handler |

**Returns:** `Promise<RegistrationResult>`

**RegistrationResult:**

| Property | Type | Description |
|----------|------|-------------|
| `success` | `boolean` | Whether registration succeeded |
| `scheme` | `string` | The scheme that was registered |
| `was_already_registered` | `boolean` | Whether scheme was already registered |
| `previous_handler` | `string \| null` | Previous handler app identifier |

**Example:**

```typescript
import { register } from "runtime:protocol";

const result = await register("myapp", {
  description: "My Application Protocol",
  set_as_default: true,
});

if (result.success) {
  console.log("Protocol registered successfully");
  if (result.was_already_registered) {
    console.log(`Previously handled by: ${result.previous_handler}`);
  }
}
```

---

#### unregister(scheme)

Unregister a custom URL protocol handler.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `scheme` | `string` | URL scheme to unregister |

**Returns:** `Promise<boolean>` - True if was registered and unregistered

**Example:**

```typescript
import { unregister } from "runtime:protocol";

const wasRegistered = await unregister("myapp");
if (wasRegistered) {
  console.log("Protocol unregistered");
}
```

---

#### isRegistered(scheme)

Check if a URL scheme is registered and get handler information.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `scheme` | `string` | URL scheme to check |

**Returns:** `Promise<RegistrationStatus>`

**RegistrationStatus:**

| Property | Type | Description |
|----------|------|-------------|
| `is_registered` | `boolean` | Whether any handler is registered |
| `is_default` | `boolean` | Whether this app is default handler |
| `registered_by` | `string \| null` | App identifier of current handler |

**Example:**

```typescript
import { isRegistered } from "runtime:protocol";

const status = await isRegistered("myapp");
if (status.is_registered) {
  if (status.is_default) {
    console.log("This app is the default handler");
  } else {
    console.log(`Registered by: ${status.registered_by}`);
  }
}
```

---

#### listRegistered()

List all protocols registered by this app.

**Returns:** `ProtocolInfo[]`

**ProtocolInfo:**

| Property | Type | Description |
|----------|------|-------------|
| `scheme` | `string` | URL scheme (e.g., "myapp") |
| `description` | `string \| null` | Human-readable description |
| `icon_path` | `string \| null` | Path to icon file |
| `is_default` | `boolean` | Whether this app is default handler |
| `registered_by` | `string \| null` | App identifier that registered |

**Example:**

```typescript
import { listRegistered } from "runtime:protocol";

const protocols = listRegistered();
for (const proto of protocols) {
  console.log(`${proto.scheme}:// - ${proto.description || "No description"}`);
  console.log(`  Default: ${proto.is_default}`);
}
```

---

#### setAsDefault(scheme)

Set this app as the default handler for a scheme.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `scheme` | `string` | URL scheme to become default for |

**Returns:** `Promise<boolean>` - True if successful

**Example:**

```typescript
import { setAsDefault, isRegistered } from "runtime:protocol";

const status = await isRegistered("myapp");
if (status.is_registered && !status.is_default) {
  await setAsDefault("myapp");
  console.log("Now the default handler for myapp://");
}
```

### Invocation Handling

#### getLaunchUrl()

Get the URL that launched this app (if app was opened via deep link).

**Returns:** `string | null` - Launch URL or null if not launched via protocol

**Example:**

```typescript
import { getLaunchUrl, parseUrl } from "runtime:protocol";

// Check on app startup
const launchUrl = getLaunchUrl();
if (launchUrl) {
  console.log(`App launched with: ${launchUrl}`);
  const parsed = parseUrl(launchUrl);
  handleDeepLink(parsed);
}
```

---

#### onInvoke(callback)

Listen for protocol URL invocations. Called both when app is launched via URL and when URLs are received while running.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `callback` | `(invocation: ProtocolInvocation) => void` | Handler function |

**Returns:** `() => void` - Cleanup function to stop listening

**ProtocolInvocation:**

| Property | Type | Description |
|----------|------|-------------|
| `id` | `string` | Unique invocation ID |
| `url` | `string` | Full URL that was invoked |
| `scheme` | `string` | URL scheme (e.g., "myapp") |
| `path` | `string` | Path portion of URL |
| `query` | `Record<string, string>` | Query parameters |
| `fragment` | `string \| null` | URL fragment (after #) |
| `timestamp` | `number` | Unix timestamp (milliseconds) |
| `is_launch` | `boolean` | Whether this invocation launched the app |

**Example:**

```typescript
import { onInvoke } from "runtime:protocol";

const cleanup = onInvoke((invocation) => {
  console.log(`Received: ${invocation.url}`);
  console.log(`Path: ${invocation.path}`);
  console.log(`Query: ${JSON.stringify(invocation.query)}`);
  console.log(`Is launch: ${invocation.is_launch}`);

  // Route based on path
  switch (invocation.path) {
    case "/auth/callback":
      handleAuthCallback(invocation.query);
      break;
    case "/open":
      openFile(invocation.query.file);
      break;
    case "/settings":
      navigateToSettings(invocation.fragment);
      break;
    default:
      console.log("Unknown deep link path");
  }
});

// Later, stop listening
cleanup();
```

---

#### receiveInvocation()

Low-level API to receive the next protocol invocation. For most use cases, prefer `onInvoke()`.

**Returns:** `Promise<ProtocolInvocation>`

**Example:**

```typescript
import { receiveInvocation } from "runtime:protocol";

// Manual event loop
async function listenForInvocations() {
  while (true) {
    try {
      const invocation = await receiveInvocation();
      handleInvocation(invocation);
    } catch (err) {
      console.error("Listener error:", err);
      break;
    }
  }
}
```

### URL Utilities

#### parseUrl(url)

Parse a protocol URL into its components.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `url` | `string` | URL to parse |

**Returns:** `ParsedProtocolUrl`

**ParsedProtocolUrl:**

| Property | Type | Description |
|----------|------|-------------|
| `scheme` | `string` | URL scheme |
| `path` | `string` | Path portion |
| `query` | `Record<string, string>` | Query parameters |
| `fragment` | `string \| null` | Fragment, if any |
| `is_valid` | `boolean` | Whether URL is valid |

**Example:**

```typescript
import { parseUrl } from "runtime:protocol";

const parsed = parseUrl("myapp://settings/theme?dark=true#section1");
// {
//   scheme: "myapp",
//   path: "settings/theme",
//   query: { dark: "true" },
//   fragment: "section1",
//   is_valid: true
// }

// Validate before using
if (parsed.is_valid) {
  console.log(`Scheme: ${parsed.scheme}`);
  console.log(`Path: ${parsed.path}`);
} else {
  console.error("Invalid protocol URL");
}
```

---

#### buildUrl(scheme, path, query?)

Build a protocol URL from components.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `scheme` | `string` | URL scheme |
| `path` | `string` | Path portion |
| `query` | `Record<string, string>` | Optional query parameters |

**Returns:** `string` - Constructed URL

**Example:**

```typescript
import { buildUrl } from "runtime:protocol";

// Simple URL
const simple = buildUrl("myapp", "settings");
// "myapp://settings"

// With query parameters
const withQuery = buildUrl("myapp", "auth/callback", {
  token: "abc123",
  redirect: "/dashboard",
});
// "myapp://auth/callback?token=abc123&redirect=%2Fdashboard"

// For sharing
const shareUrl = buildUrl("myapp", "open", {
  file: "document.pdf",
  page: "5",
});
console.log(`Share this link: ${shareUrl}`);
```

### Capabilities

#### checkCapabilities()

Check platform capabilities for protocol handling.

**Returns:** `ProtocolCapabilities`

**ProtocolCapabilities:**

| Property | Type | Description |
|----------|------|-------------|
| `can_register` | `boolean` | Whether registration is supported |
| `can_query` | `boolean` | Whether querying is supported |
| `can_deep_link` | `boolean` | Whether deep linking is supported |
| `platform` | `string` | Current platform identifier |
| `notes` | `string \| null` | Additional capability notes |

**Example:**

```typescript
import { checkCapabilities, register } from "runtime:protocol";

const caps = checkCapabilities();
console.log(`Platform: ${caps.platform}`);

if (caps.can_register) {
  await register("myapp");
} else {
  console.log(`Registration not available: ${caps.notes}`);
}

if (!caps.can_deep_link) {
  console.warn("Deep linking may not work on this platform");
}
```

## Type Definitions

```typescript
/** Options for registering a protocol handler */
interface RegistrationOptions {
  /** Human-readable description of the protocol */
  description?: string;
  /** Path to icon file (platform-specific format) */
  icon_path?: string;
  /** Whether to set this app as the default handler (default: true) */
  set_as_default?: boolean;
}

/** Result of protocol registration */
interface RegistrationResult {
  success: boolean;
  scheme: string;
  was_already_registered: boolean;
  previous_handler: string | null;
}

/** Status of a protocol registration */
interface RegistrationStatus {
  is_registered: boolean;
  is_default: boolean;
  registered_by: string | null;
}

/** Information about a registered protocol */
interface ProtocolInfo {
  scheme: string;
  description: string | null;
  icon_path: string | null;
  is_default: boolean;
  registered_by: string | null;
}

/** A protocol URL invocation event */
interface ProtocolInvocation {
  id: string;
  url: string;
  scheme: string;
  path: string;
  query: Record<string, string>;
  fragment: string | null;
  timestamp: number;
  is_launch: boolean;
}

/** Parsed URL components */
interface ParsedProtocolUrl {
  scheme: string;
  path: string;
  query: Record<string, string>;
  fragment: string | null;
  is_valid: boolean;
}

/** Platform capabilities for protocol handling */
interface ProtocolCapabilities {
  can_register: boolean;
  can_query: boolean;
  can_deep_link: boolean;
  platform: string;
  notes: string | null;
}
```

## Lifecycle Hooks

### onBefore(opName, callback)

```typescript
import { onBefore } from "runtime:protocol";

onBefore("register", (args) => {
  console.log("Registering protocol...");
});
```

### onAfter(opName, callback)

```typescript
import { onAfter } from "runtime:protocol";

onAfter("register", (result) => {
  if (result.success) {
    analytics.track("protocol_registered", { scheme: result.scheme });
  }
});
```

### onError(opName, callback)

```typescript
import { onError } from "runtime:protocol";

onError("register", (error) => {
  console.error("Protocol registration failed:", error.message);
});
```

**Available operation names:** `"info"`, `"register"`, `"unregister"`, `"isRegistered"`, `"listRegistered"`, `"setAsDefault"`, `"getLaunchUrl"`, `"receiveInvocation"`, `"parseUrl"`, `"buildUrl"`, `"checkCapabilities"`

## Complete Examples

### OAuth Callback Handler

```typescript
import { register, onInvoke, buildUrl } from "runtime:protocol";

// Register protocol for OAuth callbacks
await register("myapp", {
  description: "My App OAuth Handler",
});

// Listen for OAuth callbacks
onInvoke((invocation) => {
  if (invocation.path === "/auth/callback") {
    const { code, state, error } = invocation.query;

    if (error) {
      showError(`Authentication failed: ${error}`);
      return;
    }

    if (code && state === sessionState) {
      // Exchange code for tokens
      exchangeCodeForTokens(code).then((tokens) => {
        saveTokens(tokens);
        navigateToDashboard();
      });
    }
  }
});

// Generate OAuth URL
function getOAuthUrl(): string {
  const redirectUri = buildUrl("myapp", "auth/callback");
  return `https://oauth.provider.com/authorize?` +
    `client_id=${CLIENT_ID}&` +
    `redirect_uri=${encodeURIComponent(redirectUri)}&` +
    `response_type=code&` +
    `state=${sessionState}`;
}
```

### File Association Handler

```typescript
import { register, getLaunchUrl, onInvoke, parseUrl } from "runtime:protocol";

// Register for custom file type
await register("myfile", {
  description: "My App File Handler",
});

// Handle launch URL
const launchUrl = getLaunchUrl();
if (launchUrl) {
  const parsed = parseUrl(launchUrl);
  if (parsed.is_valid && parsed.path.startsWith("/open/")) {
    const filePath = decodeURIComponent(parsed.path.substring(6));
    openFile(filePath);
  }
}

// Handle URLs while running
onInvoke((invocation) => {
  if (invocation.path.startsWith("/open/")) {
    const filePath = decodeURIComponent(invocation.path.substring(6));
    openFile(filePath);

    // Bring window to front
    bringWindowToFront();
  }
});

// Create deep link for a file
function createFileLink(filePath: string): string {
  return buildUrl("myfile", `/open/${encodeURIComponent(filePath)}`);
}
```

### Deep Link Router

```typescript
import { register, getLaunchUrl, onInvoke, parseUrl } from "runtime:protocol";

// Define routes
const routes: Record<string, (query: Record<string, string>, fragment?: string) => void> = {
  "/": () => navigateTo("/home"),
  "/settings": (query, fragment) => {
    navigateTo("/settings");
    if (fragment) {
      scrollToSection(fragment);
    }
  },
  "/share": (query) => {
    if (query.content) {
      openShareDialog(decodeURIComponent(query.content));
    }
  },
  "/user/:id": (query) => {
    if (query.id) {
      showUserProfile(query.id);
    }
  },
};

// Route handler
function handleRoute(url: string) {
  const parsed = parseUrl(url);
  if (!parsed.is_valid) {
    console.error("Invalid URL:", url);
    return;
  }

  // Find matching route
  for (const [pattern, handler] of Object.entries(routes)) {
    const match = matchPattern(pattern, parsed.path);
    if (match) {
      handler({ ...parsed.query, ...match.params }, parsed.fragment || undefined);
      return;
    }
  }

  console.log("No route matched:", parsed.path);
}

// Initialize
await register("myapp");

// Handle launch URL
const launchUrl = getLaunchUrl();
if (launchUrl) {
  handleRoute(launchUrl);
}

// Handle runtime invocations
onInvoke((invocation) => {
  handleRoute(invocation.url);
});
```

### Share URL Generator

```typescript
import { buildUrl, register } from "runtime:protocol";

await register("myapp");

// Generate shareable URLs for content
function createShareUrl(contentId: string, options?: {
  highlight?: string;
  autoplay?: boolean;
}): string {
  const query: Record<string, string> = { id: contentId };

  if (options?.highlight) {
    query.highlight = options.highlight;
  }
  if (options?.autoplay) {
    query.autoplay = "true";
  }

  return buildUrl("myapp", "/content/view", query);
}

// Usage
const shareUrl = createShareUrl("doc-123", { highlight: "paragraph-5" });
// "myapp://content/view?id=doc-123&highlight=paragraph-5"

// Copy to clipboard
navigator.clipboard.writeText(shareUrl);
showToast("Link copied!");
```

### Multi-Protocol App

```typescript
import { register, onInvoke, checkCapabilities } from "runtime:protocol";

const caps = checkCapabilities();

if (caps.can_register) {
  // Register multiple protocols
  await Promise.all([
    register("myapp", { description: "Main App Protocol" }),
    register("myapp-auth", { description: "Authentication Protocol" }),
    register("myapp-share", { description: "Content Sharing Protocol" }),
  ]);
}

// Handle all protocols
onInvoke((invocation) => {
  switch (invocation.scheme) {
    case "myapp":
      handleMainProtocol(invocation);
      break;
    case "myapp-auth":
      handleAuthProtocol(invocation);
      break;
    case "myapp-share":
      handleShareProtocol(invocation);
      break;
  }
});

function handleMainProtocol(inv: ProtocolInvocation) {
  // General app navigation
  navigateTo(inv.path);
}

function handleAuthProtocol(inv: ProtocolInvocation) {
  // OAuth/SSO callbacks
  if (inv.path === "/callback") {
    processOAuthCallback(inv.query);
  }
}

function handleShareProtocol(inv: ProtocolInvocation) {
  // Content sharing
  if (inv.path === "/receive") {
    receiveSharedContent(inv.query);
  }
}
```

## Best Practices

### Register on First Run

```typescript
import { isRegistered, register, checkCapabilities } from "runtime:protocol";

async function ensureProtocolRegistered() {
  const caps = checkCapabilities();
  if (!caps.can_register) {
    console.log("Protocol registration not supported");
    return;
  }

  const status = await isRegistered("myapp");
  if (!status.is_registered) {
    await register("myapp", {
      description: "My Application",
    });
  } else if (!status.is_default) {
    // Optionally prompt user to set as default
    const shouldSetDefault = await askUser("Set as default handler?");
    if (shouldSetDefault) {
      await setAsDefault("myapp");
    }
  }
}
```

### Handle Launch URL Early

```typescript
import { getLaunchUrl } from "runtime:protocol";

// Check at app startup, before UI renders
async function initializeApp() {
  const launchUrl = getLaunchUrl();

  if (launchUrl) {
    // Store for processing after app is ready
    globalState.pendingDeepLink = launchUrl;
  }

  // Continue initialization...
  await initializeUI();

  // Process pending deep link
  if (globalState.pendingDeepLink) {
    handleDeepLink(globalState.pendingDeepLink);
  }
}
```

### Validate Protocol URLs

```typescript
import { parseUrl } from "runtime:protocol";

function validateAndHandle(url: string): boolean {
  const parsed = parseUrl(url);

  if (!parsed.is_valid) {
    console.error("Invalid protocol URL");
    return false;
  }

  // Validate scheme
  if (parsed.scheme !== "myapp") {
    console.error("Unknown scheme:", parsed.scheme);
    return false;
  }

  // Validate path
  const allowedPaths = ["/", "/settings", "/share", "/auth/callback"];
  if (!allowedPaths.some(p => parsed.path.startsWith(p))) {
    console.error("Unknown path:", parsed.path);
    return false;
  }

  return true;
}
```

### Clean Up on Exit

```typescript
import { unregister, listRegistered } from "runtime:protocol";

// If app shouldn't handle protocols after uninstall
async function uninstallCleanup() {
  const protocols = listRegistered();
  for (const proto of protocols) {
    await unregister(proto.scheme);
  }
}
```
