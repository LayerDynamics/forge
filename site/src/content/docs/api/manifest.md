---
title: Manifest Reference
description: The manifest.app.toml file defines your Forge application's metadata, defaults, bundle settings, and permissions.
slug: api/manifest
---

The manifest file defines your Forge application's metadata, default window configuration, bundling options, and permissions (capabilities).

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
crash_reporting = false           # Optional: enable crash reporting
crash_report_dir = "./crashes"    # Optional: custom crash dump directory
```

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Display name shown in title bar/menus |
| `identifier` | Yes | Unique reverse-domain identifier |
| `version` | Yes | Semantic version (major.minor.patch) |
| `crash_reporting` | No | Enable crash report capture (default: false) |
| `crash_report_dir` | No | Override crash dump directory |

---

## Windows Section

Default window configuration used when you omit sizing in code:

```toml
[windows]
width = 800              # Default width
height = 600             # Default height
resizable = true         # Default resizable behavior
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
format = "dmg"                                # dmg (default), app, zip, pkg
sign = true                                   
notarize = true                                
team_id = "ABCD1234"                           
signing_identity = "Developer ID Application: My Company (TEAMID)"
entitlements = "entitlements.plist"
category = "public.app-category.developer-tools"
minimum_system_version = "12.0"
```

#### Windows

```toml
[bundle.windows]
format = "msix"                                
sign = true                                    
certificate = "cert.pfx"                       
password = "$CERT_PASSWORD"                    
publisher = "CN=My Company, O=My Company"      
min_version = "10.0.17763.0"                   
capabilities = ["internetClient", "webcam"]    
```

#### Linux

```toml
[bundle.linux]
format = "appimage"                            # appimage (default) or tarball
categories = ["Development", "Utility"]
generic_name = "My Application"
comment = "A useful application"
mime_types = ["text/plain"]
terminal = false
```

---

## Permissions / Capabilities

Permissions define what system resources your app can access. Use either `[permissions]` (preferred) or `[capabilities]` – both map to the same structure. In dev mode (`forge dev`), all permissions are allowed; production enforces them.

### File System

```toml
[permissions.fs]
read = ["./data/**", "~/.myapp/*"]
write = ["./data/**"]
```

Pattern syntax:
- `~` expands to the user home directory
- `*` matches any characters except `/`
- `**` matches across directories

### Network

```toml
[permissions.net]
allow = ["https://api.example.com/*", "http://localhost:*"]
deny = ["http://*.bad.com/*"]
listen = [3000, 8080]  # Ports allowed for servers
```

### UI

```toml
[permissions.ui]
windows = true    # Default: true
menus = true      # Default: true
dialogs = true    # Default: true
tray = false      # Default: false (must opt in)
channels = ["app:*", "ui:*"]  # Optional default channel allowlist for windows
```

### System

```toml
[permissions.sys]
clipboard = true
notify = true
power = true

[permissions.sys.env]
read = ["APP_*", "PATH"]
write = ["APP_DEBUG"]
```

### Process

```toml
[permissions.process]
allow = ["git", "node", "/usr/bin/*"]
env = ["NODE_ENV", "PATH"]   # Env vars allowed when spawning
max_processes = 10           # Default: 10
```

### WebAssembly

```toml
[permissions.wasm]
load = ["./wasm/**"]
preopens = ["./data/**"]
max_instances = 10           # Default: 10
```

### Code Signing

```toml
[permissions.codesign]
sign = true
list_identities = true
```

---

## Complete Example

```toml
[app]
name = "Forge Notes"
identifier = "com.forge.notes"
version = "1.0.0"

[windows]
width = 900
height = 700
resizable = true

[bundle]
icon = "assets/icon"

[bundle.macos]
category = "public.app-category.productivity"
minimum_system_version = "12.0"

# Permissions (capabilities)
[permissions.fs]
read = ["~/.forge-notes/**", "~/Documents/**"]
write = ["~/.forge-notes/**"]

[permissions.net]
allow = ["https://sync.forgenotes.com/*"]

[permissions.sys]
clipboard = true
notify = true

[permissions.process]
allow = ["code", "vim", "nano"]

[permissions.ui]
tray = true
channels = ["notes:*", "sync:*", "settings:*", "ui:*"]
```

---

## Security Best Practices

### Principle of Least Privilege

Only request permissions you actually need:

```toml
# BAD - Too permissive
[permissions.fs]
read = ["/**"]
write = ["/**"]

# GOOD - Specific paths
[permissions.fs]
read = ["~/.myapp/config.json"]
write = ["~/.myapp/data/*"]
```

### Specific Network Hosts

```toml
# BAD - Allows any host
[permissions.net]
allow = ["https://**"]

# GOOD - Specific hosts
[permissions.net]
allow = ["https://api.myservice.com/*"]
```

### Explicit Process Binaries

```toml
# BAD - Allows any process
[permissions.process]
allow = ["*"]

# GOOD - Specific binaries
[permissions.process]
allow = ["git", "npm"]
```

### Channel Restrictions

```toml
# Development only
[permissions.ui]
channels = ["*"]

# Production - explicit channels
[permissions.ui]
channels = ["app:state", "user:action", "file:open"]
```

---

## Default Values

If a section is omitted, these defaults apply:

| Section | Default |
|---------|---------|
| `windows.width` | 800 |
| `windows.height` | 600 |
| `windows.resizable` | true |
| `permissions.fs` | No access |
| `permissions.net` | No access |
| `permissions.ui.windows` | true |
| `permissions.ui.menus` | true |
| `permissions.ui.dialogs` | true |
| `permissions.ui.tray` | false |
| `permissions.sys.clipboard` | false |
| `permissions.sys.notify` | false |
| `permissions.sys.power` | false |
| `permissions.process.allow` | No access |
| `permissions.process.max_processes` | 10 |
| `permissions.wasm.max_instances` | 10 |
