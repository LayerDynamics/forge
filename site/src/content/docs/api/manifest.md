---
title: Manifest Reference
description: The manifest.app.toml file defines your Forge application's metadata, window configuration, and capability declarations.
slug: api/manifest
---

The manifest file defines your Forge application's metadata, window configuration, and capability declarations.

## Location

The manifest must be at the root of your app directory:

```
my-app/
├── manifest.app.toml   # <-- Here
├── src/
└── web/
```

---

## App Section

Basic application metadata:

```toml
[app]
name = "My Application"           # Display name
identifier = "com.example.myapp"  # Reverse domain identifier
version = "1.0.0"                 # Semantic version
description = "A great app"       # Optional description
```

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Display name shown in title bar, menus |
| `identifier` | Yes | Unique reverse-domain identifier |
| `version` | Yes | Semantic version (major.minor.patch) |
| `description` | No | Brief description of the app |

---

## Windows Section

Default window configuration:

```toml
[windows]
width = 800              # Initial width in pixels
height = 600             # Initial height in pixels
min_width = 400          # Minimum width (optional)
min_height = 300         # Minimum height (optional)
max_width = 1920         # Maximum width (optional)
max_height = 1080        # Maximum height (optional)
resizable = true         # Allow resizing
decorations = true       # Show window decorations (title bar, etc.)
transparent = false      # Transparent window background
always_on_top = false    # Keep above other windows
fullscreen = false       # Start in fullscreen
title = "My App"         # Window title (defaults to app.name)
```

---

## Bundle Section

Configuration for bundling your app into distributable packages (`.app`, `.dmg`, `.msix`, `.AppImage`):

```toml
[bundle]
icon = "assets/icon"         # Path to icon (without extension)
```

### App Icon

**Required for bundling.** Your app must have an icon that meets these requirements:

| Requirement | Value |
|-------------|-------|
| **Format** | PNG with transparency (RGBA) |
| **Size** | 1024x1024 pixels (minimum 512x512) |
| **Aspect Ratio** | Square (1:1) |

**Recommended location:** `assets/icon.png`

The bundler looks for icons in this order:
1. `bundle.icon` path from manifest (with `.png`, `.icns`, `.ico` extensions)
2. `assets/icon.png`
3. `assets/icon.icns`
4. `assets/icon.ico`
5. `icon.png` (in app root)

**CLI Tools:**

```bash
# Create a placeholder icon
forge icon create my-app/assets/icon.png

# Validate your icon
forge icon validate my-app
```

### Platform-Specific Bundle Options

#### macOS

```toml
[bundle.macos]
sign = true                                    # Enable code signing
notarize = true                                # Enable notarization (requires signing)
team_id = "ABCD1234"                           # Apple Developer Team ID
signing_identity = "Developer ID Application: My Company (TEAMID)"
entitlements = "entitlements.plist"            # Custom entitlements file
category = "public.app-category.developer-tools"
minimum_system_version = "12.0"
```

#### Windows

```toml
[bundle.windows]
format = "msix"                                # Package format
sign = true                                    # Enable code signing
certificate = "cert.pfx"                       # Path to certificate file
password = "$CERT_PASSWORD"                    # Password (supports $ENV_VAR)
publisher = "CN=My Company, O=My Company"      # Publisher DN
min_version = "10.0.17763.0"                   # Minimum Windows version
capabilities = ["internetClient", "webcam"]    # Additional capabilities
```

#### Linux

```toml
[bundle.linux]
format = "appimage"                            # "appimage" or "tarball"
categories = ["Development", "Utility"]        # Desktop entry categories
generic_name = "My Application"
comment = "A useful application"
mime_types = ["text/plain"]                    # Supported file types
terminal = false                               # Run in terminal
```

---

## Capabilities Section

Capabilities define what system resources your app can access. Forge uses a capability-based security model - you must explicitly declare permissions.

### File System

```toml
[capabilities.fs]
read = [
  "~/.myapp/*",           # Read from home directory
  "./data/**",            # Read from app data directory
  "/tmp/myapp-*"          # Read specific temp files
]
write = [
  "~/.myapp/*",           # Write to home directory
  "./data/**"             # Write to app data directory
]
```

**Pattern syntax:**
- `~` expands to user's home directory
- `*` matches any characters except `/`
- `**` matches any characters including `/`
- `.` relative to app directory

### Network

```toml
[capabilities.net]
fetch = [
  "https://api.example.com/*",      # HTTPS API
  "https://cdn.example.com/**",     # CDN with subpaths
  "http://localruntime:*"              # Local development
]
```

### System

```toml
[capabilities.sys]
clipboard = true        # Read/write clipboard
notifications = true    # Show system notifications
info = true            # Access system information (always allowed)
```

### Process

```toml
[capabilities.process]
spawn = [
  "git",                 # Allow git commands
  "npm",                 # Allow npm
  "node",                # Allow node
  "/usr/bin/*"           # Allow any binary in /usr/bin
]
```

### IPC Channels

```toml
[capabilities.channels]
allowed = ["*"]           # Allow all channels (development)
# OR
allowed = [
  "user:*",               # All user channels
  "app:state",            # Specific channel
  "system:ready"
]
```

**Channel patterns:**
- `*` allows all channels
- `namespace:*` allows all channels in namespace
- `exact:channel` allows specific channel
- Empty `[]` denies all channels

### UI Capabilities

Control access to UI features. Most are enabled by default, but tray icons require explicit permission:

```toml
[capabilities.ui]
windows = true      # Create and manage windows (default: true)
menus = true        # Set app menu and context menus (default: true)
dialogs = true      # Show file/message dialogs (default: true)
tray = false        # Create system tray icons (default: false)
```

| Capability | Default | Description |
|------------|---------|-------------|
| `windows` | `true` | Create windows via `openWindow()` |
| `menus` | `true` | Use `setAppMenu()` and `showContextMenu()` |
| `dialogs` | `true` | Use `dialog.open()`, `dialog.save()`, `dialog.message()` |
| `tray` | `false` | Use `createTray()` for system tray icons |

**Example with tray enabled:**

```toml
[capabilities.ui]
tray = true  # Enable tray icons (other UI capabilities default to true)
```

**Note:** In development mode (`forge dev`), all capabilities are enabled regardless of manifest settings.

---

## Complete Example

```toml
[app]
name = "Forge Notes"
identifier = "com.forge.notes"
version = "1.0.0"
description = "A simple note-taking app"

[windows]
width = 900
height = 700
min_width = 400
min_height = 300
resizable = true
title = "Forge Notes"

[bundle]
icon = "assets/icon"

[bundle.macos]
category = "public.app-category.productivity"
minimum_system_version = "12.0"

[capabilities]

# File system access for notes storage
[capabilities.fs]
read = ["~/.forge-notes/**", "~/Documents/**"]
write = ["~/.forge-notes/**"]

# Network for sync (optional)
[capabilities.net]
fetch = ["https://sync.forgenotes.com/*"]

# System features
[capabilities.sys]
clipboard = true
notifications = true

# Process spawning (for opening files in editor)
[capabilities.process]
spawn = ["code", "vim", "nano"]

# IPC channels
[capabilities.channels]
allowed = ["notes:*", "sync:*", "settings:*", "ui:*"]

# UI capabilities (tray must be explicitly enabled)
[capabilities.ui]
tray = true
```

---

## Security Best Practices

### Principle of Least Privilege

Only request capabilities you actually need:

```toml
# BAD - Too permissive
[capabilities.fs]
read = ["/**"]
write = ["/**"]

# GOOD - Specific paths
[capabilities.fs]
read = ["~/.myapp/config.json"]
write = ["~/.myapp/data/*"]
```

### Specific Network Hosts

```toml
# BAD - Allows any host
[capabilities.net]
fetch = ["https://**"]

# GOOD - Specific hosts
[capabilities.net]
fetch = ["https://api.myservice.com/*"]
```

### Explicit Process Binaries

```toml
# BAD - Allows any process
[capabilities.process]
spawn = ["*"]

# GOOD - Specific binaries
[capabilities.process]
spawn = ["git", "npm"]
```

### Channel Restrictions

```toml
# Development only
[capabilities.channels]
allowed = ["*"]

# Production - explicit channels
[capabilities.channels]
allowed = ["app:state", "user:action", "file:open"]
```

---

## Default Values

If a section is omitted, these defaults apply:

| Section | Default |
|---------|---------|
| `windows.width` | 800 |
| `windows.height` | 600 |
| `windows.resizable` | true |
| `windows.decorations` | true |
| `capabilities.ui.windows` | true |
| `capabilities.ui.menus` | true |
| `capabilities.ui.dialogs` | true |
| `capabilities.ui.tray` | false |
| `capabilities.fs` | No access |
| `capabilities.net` | No access |
| `capabilities.sys.clipboard` | false |
| `capabilities.sys.notifications` | false |
| `capabilities.process` | No access |
| `capabilities.channels` | Deny all |

---

## Environment Variables

Some manifest values can reference environment variables:

```toml
[app]
version = "${CARGO_PKG_VERSION}"  # From build environment
```
