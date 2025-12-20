# Forge Troubleshooting Guide

Common issues and solutions for Forge development.

## Table of Contents

1. [Common Build Errors](#common-build-errors)
2. [Runtime Errors](#runtime-errors)
3. [Development Issues](#development-issues)
4. [Platform-Specific Issues](#platform-specific-issues)
5. [Extension Development Issues](#extension-development-issues)
6. [Performance Issues](#performance-issues)

---

## Common Build Errors

### Missing manifest.app.toml

**Error:**
```
Error: manifest.app.toml not found in ./my-app
```

**Cause:** App directory doesn't contain required manifest file

**Solution:**
```bash
# Create manifest.app.toml in your app root
cat > manifest.app.toml << 'EOF'
[app]
name = "My App"
identifier = "com.example.myapp"
version = "1.0.0"

[windows]
width = 800
height = 600
resizable = true
EOF
```

---

### Icon Validation Failures

**Error:**
```
Error: Icon validation failed: Expected PNG, got JPEG
```

**Cause:** App icon must be PNG format with specific dimensions

**Solution:**
```bash
# Convert icon to PNG
convert icon.jpg icon.png

# Or use Forge icon tool
forge icon create assets/icon.png

# Validate icon
forge icon validate .
```

**Requirements:**
- Format: PNG with alpha channel
- Size: 1024×1024px recommended (will be scaled)
- Location: Specified in manifest or default `assets/icon.png`

---

### Framework Detection Issues

**Error:**
```
Warning: Could not detect framework, using minimal configuration
```

**Cause:** Forge couldn't auto-detect your frontend framework

**Solution:**

**React:**
Ensure `package.json` contains:
```json
{
  "dependencies": {
    "react": "^18.0.0",
    "react-dom": "^18.0.0"
  }
}
```

**Svelte:**
Ensure `svelte.config.js` exists:
```javascript
export default {
  // Config here
};
```

**Vue:**
Ensure `package.json` contains:
```json
{
  "dependencies": {
    "vue": "^3.0.0"
  }
}
```

**Manual override:** Create `forge.config.ts`:
```typescript
export default {
  framework: "react" // or "svelte", "vue", "minimal"
};
```

---

### esbuild Bundling Errors

**Error:**
```
Error: Could not resolve "react"
  at src/App.tsx:1:0
```

**Cause:** Missing dependencies or incorrect import paths

**Solution 1 - Install dependencies:**
```bash
npm install react react-dom
# or
deno install npm:react npm:react-dom
```

**Solution 2 - Fix import paths:**
```typescript
// ✗ Bad - Won't work in Deno
import React from "react";

// ✓ Good - Use npm: specifier
import React from "npm:react";

// ✓ Also Good - Use import map in deno.json
// deno.json:
{
  "imports": {
    "react": "npm:react@^18.0.0"
  }
}

// Then in code:
import React from "react";
```

**Solution 3 - Check entry point:**
Ensure `src/main.ts` exists and is valid TypeScript:
```bash
# Test compilation
deno check src/main.ts
```

---

### Build Script Errors

**Error:**
```
error: Uncaught (in promise) TypeError: Cannot read properties of undefined
```

**Cause:** Build script (`build.ts`) has errors

**Solution:**
```bash
# Test build script directly
deno run --allow-read --allow-write build.ts

# Enable debug logging
FORGE_LOG=debug forge build .

# Check for missing permissions in build script
deno run --allow-all build.ts
```

---

## Runtime Errors

### Permission Denied Errors

**Error:**
```
Error: Permission denied: Cannot read file "./data/config.json"
Capability check failed: fs.read
```

**Cause:** Missing permission in `manifest.app.toml`

**Solution:**

Add permission to manifest:
```toml
[permissions.fs]
read = ["./data/**"]
```

**Common permission patterns:**

```toml
[permissions]
# File system
[permissions.fs]
read = ["./data/**", "./config/**"]
write = ["./data/**", "./logs/**"]

# Network
[permissions.net]
allow = ["api.example.com", "*.trusted.com"]
deny = ["evil.com"]

# Process
[permissions.process]
allow_spawn = true
max_processes = 10

# WebAssembly
[permissions.wasm]
allow_wasm = true

# IPC channels
[permissions.ui]
channels = ["app:*", "custom:*"]
```

**Debug permissions:**
```bash
# Run in dev mode (all permissions allowed)
forge dev my-app

# If it works in dev but not production, it's a permission issue
```

---

### Module Not Found

**Error:**
```
error: Module not found "file:///path/to/missing.ts"
```

**Cause:** Import path doesn't resolve correctly

**Solution 1 - Check file exists:**
```bash
ls -la src/missing.ts
```

**Solution 2 - Fix import path:**
```typescript
// ✗ Bad - Relative to wrong location
import { helper } from "./utils/helper.ts";

// ✓ Good - Correct relative path
import { helper } from "../utils/helper.ts";

// ✓ Also Good - Use import map
// deno.json:
{
  "imports": {
    "@/": "./src/"
  }
}

// Code:
import { helper } from "@/utils/helper.ts";
```

**Solution 3 - Runtime module imports:**
```typescript
// ✗ Bad - Wrong specifier
import * as fs from "runtime:filesystem";

// ✓ Good - Correct runtime specifier
import * as fs from "runtime:fs";
```

---

### IPC Communication Failures

**Error (in renderer):**
```
TypeError: window.host is undefined
```

**Cause:** IPC bridge not initialized

**Solution:**

Ensure `web/index.html` loads before sending messages:
```html
<!DOCTYPE html>
<html>
<head>
    <script type="module" src="/bundle.js"></script>
</head>
<body>
    <div id="app"></div>
    <script>
        // Wait for host bridge to be ready
        if (typeof window.host === 'undefined') {
            console.error("Host bridge not available!");
        }

        window.host.on("ready", () => {
            console.log("IPC bridge ready");
        });
    </script>
</body>
</html>
```

**Error (in Deno):**
```
Error: No window with ID 1
```

**Cause:** Trying to send to closed or non-existent window

**Solution:**
```typescript
import * as window from "runtime:window";

// Track window IDs
const windows = new Set<number>();

window.on("created", (id: number) => {
    windows.add(id);
});

window.on("closed", (id: number) => {
    windows.delete(id);
});

// Only send to existing windows
function sendToWindow(id: number, channel: string, data: unknown) {
    if (windows.has(id)) {
        window.sendToWindow(id, channel, data);
    }
}
```

---

### WebView Loading Errors

**Error:**
```
Error: Failed to load app:// URL
net::ERR_FILE_NOT_FOUND
```

**Cause:** Missing `web/index.html` or incorrect asset path

**Solution 1 - Verify web directory:**
```bash
ls -la web/
# Should contain:
# - index.html
# - bundle.js (or other assets)
```

**Solution 2 - Check HTML file:**
```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>My App</title>
</head>
<body>
    <div id="root"></div>
    <script type="module" src="/bundle.js"></script>
</body>
</html>
```

**Solution 3 - Verify asset paths:**
```html
<!-- ✗ Bad - Absolute filesystem path -->
<script src="/Users/me/app/bundle.js"></script>

<!-- ✓ Good - Relative to web/ directory -->
<script src="/bundle.js"></script>

<!-- ✓ Also Good - Relative path -->
<script src="./bundle.js"></script>
```

---

## Development Issues

### HMR (Hot Module Reload) Not Working

**Error:**
```
WebSocket connection failed: ws://localhost:3001
```

**Cause:** HMR server not running or port blocked

**Solution 1 - Ensure dev mode:**
```bash
# ✓ Correct - Starts HMR server
forge dev my-app

# ✗ Wrong - No HMR
forge build my-app
```

**Solution 2 - Check port availability:**
```bash
# See if port 3001 is in use
lsof -i :3001

# Kill conflicting process
kill -9 <PID>
```

**Solution 3 - Configure different port:**
```typescript
// In src/main.ts (dev mode only)
if (Deno.env.get("DEV_MODE")) {
    const ws = new WebSocket("ws://localhost:3002"); // Custom port
}
```

---

### forge-runtime Not Found

**Error:**
```
Error: forge-runtime binary not found
```

**Cause:** Runtime not built or not in PATH

**Solution 1 - Build runtime:**
```bash
cargo build -p forge-runtime
```

**Solution 2 - Set environment variable:**
```bash
# Temporary
export FORGE_RUNTIME_PATH=/path/to/target/debug/forge-runtime
forge dev my-app

# Permanent (add to ~/.zshrc or ~/.bashrc)
echo 'export FORGE_RUNTIME_PATH=/path/to/forge/target/debug/forge-runtime' >> ~/.zshrc
```

**Solution 3 - Add to PATH:**
```bash
# Add workspace target to PATH
export PATH="/path/to/forge/target/debug:$PATH"
```

---

### Extension Build Failures

**Error:**
```
error: could not compile `ext_myext` due to 2 previous errors
```

**Cause:** Compilation errors in extension Rust code

**Solution 1 - Check macro ordering:**
```rust
// ✗ Wrong - op2 before weld_op
#[op2(async)]
#[weld_op(async)]
async fn my_op() -> Result<String, Error> { }

// ✓ Correct - weld_op before op2
#[weld_op(async)]
#[op2(async)]
async fn my_op() -> Result<String, Error> { }
```

**Solution 2 - Verify build.rs:**
```rust
use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_myext", "runtime:myext")
        .ts_path("ts/init.ts")  // Must exist!
        .ops(&["op_my_function"]) // Must match actual op names
        .generate_sdk_module("../../sdk")
        .build()
        .expect("Failed to build extension");
}
```

**Solution 3 - Clean and rebuild:**
```bash
cargo clean -p ext_myext
cargo build -p ext_myext
```

---

### Type Generation Errors

**Error:**
```
error: TypeScript transpilation failed
  Unexpected token in ts/init.ts
```

**Cause:** Syntax error in TypeScript source

**Solution:**
```bash
# Check TypeScript directly
deno check crates/ext_myext/ts/init.ts

# Common issues:

# ✗ Bad - Invalid TypeScript
export function myFunc(arg: unknown): Promise<string> {
    return ops.op_my_func(arg);  // ops not imported
}

# ✓ Good - Import core
const core = Deno.core;
const ops = core.ops;

export function myFunc(arg: unknown): Promise<string> {
    return ops.op_my_func(arg);
}
```

---

## Platform-Specific Issues

### macOS: Code Signing Issues

**Error:**
```
Error: Code signature invalid
"MyApp.app" is damaged and can't be opened
```

**Solution 1 - Sign with certificate:**
```bash
# List available certificates
security find-identity -v -p codesigning

# Sign app
codesign --sign "Developer ID Application: Your Name" \
    --deep \
    --force \
    --options runtime \
    MyApp.app
```

**Solution 2 - Ad-hoc signing (development):**
```bash
codesign --sign - --force --deep MyApp.app
```

**Solution 3 - Remove quarantine (development only):**
```bash
xattr -cr MyApp.app
```

---

### Windows: MSIX Packaging Issues

**Error:**
```
Error: MakeAppx.exe failed: Invalid manifest
```

**Solution:**
```bash
# Ensure Windows SDK is installed
# Download from: https://developer.microsoft.com/windows/downloads/windows-sdk/

# Verify MakeAppx.exe location
where MakeAppx.exe

# If not found, add to PATH:
# C:\Program Files (x86)\Windows Kits\10\bin\10.0.22621.0\x64
```

**Manifest issues:**
```xml
<!-- AppxManifest.xml must have valid publisher -->
<Identity
  Name="com.example.myapp"
  Publisher="CN=YourName"
  Version="1.0.0.0" />
```

---

### Linux: Missing System Dependencies

**Error:**
```
error: failed to run custom build command for `wry`
  /usr/bin/ld: cannot find -lwebkit2gtk-4.1
```

**Solution:**
```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install -y \
    libwebkit2gtk-4.1-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    libxdo-dev

# Fedora
sudo dnf install webkit2gtk4.1-devel gtk3-devel libappindicator-gtk3-devel xdotool

# Arch
sudo pacman -S webkit2gtk gtk3 libappindicator-gtk3 xdotool
```

---

### Linux: AppImage Execution Issues

**Error:**
```
Error: FUSE not available
Cannot mount AppImage
```

**Solution:**
```bash
# Install FUSE
sudo apt-get install fuse libfuse2

# Or extract and run without FUSE
./MyApp.AppImage --appimage-extract
./squashfs-root/AppRun
```

---

## Extension Development Issues

### Inventory Types Not Found

**Error:**
```
error: cannot find type `MyStruct` in the crate root
```

**Cause:** Struct not registered with `#[weld_struct]` macro

**Solution:**
```rust
// Add weld_struct macro
#[weld_struct]
#[derive(Serialize, Deserialize)]
pub struct MyStruct {
    pub field: String,
}

// In build.rs, enable inventory
ExtensionBuilder::new("runtime_myext", "runtime:myext")
    .use_inventory_types()  // Important!
    .build()
```

---

### Op Not Found in Generated SDK

**Error:**
```typescript
// In TypeScript:
import { myFunction } from "runtime:myext";
// Error: myFunction is not exported
```

**Cause:** Op not listed in `build.rs` ops array

**Solution:**
```rust
// In build.rs
ExtensionBuilder::new("runtime_myext", "runtime:myext")
    .ts_path("ts/init.ts")
    .ops(&[
        "op_my_function",  // Must match actual op name
    ])
    .generate_sdk_module("../../sdk")
    .build()
```

---

## Performance Issues

### Slow Startup Time

**Symptom:** App takes >2 seconds to launch

**Diagnosis:**
```bash
# Enable trace logging
FORGE_LOG=trace forge dev my-app 2>&1 | grep -E "(took|ms)"
```

**Common causes:**

1. **Too many extensions:**
```toml
# In manifest - only enable needed extensions
[extensions]
enable = ["fs", "window", "net"]  # Don't load all 40+
```

2. **Synchronous I/O in initialization:**
```typescript
// ✗ Bad - Blocks startup
const config = await loadHugeConfig();
Deno.core.initializeApp(config);

// ✓ Good - Load after startup
setTimeout(async () => {
    const config = await loadHugeConfig();
    applyConfig(config);
}, 0);
```

3. **Large bundle size:**
```bash
# Check bundle size
ls -lh web/bundle.js

# If >1MB, enable code splitting
```

---

### High Memory Usage

**Symptom:** App uses >500MB RAM

**Diagnosis:**
```typescript
// Monitor memory
setInterval(() => {
    const mem = Deno.memoryUsage();
    console.log("Heap:", (mem.heapUsed / 1024 / 1024).toFixed(2), "MB");
}, 5000);
```

**Common causes:**

1. **Memory leaks:**
```typescript
// ✗ Bad - Listeners never removed
window.host.on("data", handleData);

// ✓ Good - Clean up
const listener = window.host.on("data", handleData);
window.addEventListener("unload", () => listener.remove());
```

2. **Large cached data:**
```typescript
// Use WeakMap for caches
const cache = new WeakMap();  // Allows GC
```

---

## Diagnostic Commands

### Check Environment

```bash
# Rust version
rustc --version
cargo --version

# Deno version
deno --version

# Forge CLI version
forge --version

# System info
uname -a
```

### Enable Debug Logging

```bash
# Full debug output
FORGE_LOG=debug forge dev my-app

# Specific module
FORGE_LOG=ext_window=trace forge dev my-app

# Multiple modules
FORGE_LOG=ext_window=trace,ext_ipc=debug forge dev my-app
```

### Check Build Output

```bash
# Verbose cargo build
cargo build -vv -p forge-runtime

# Check generated files
ls -la target/debug/build/ext_*/out/

# View generated SDK
cat sdk/runtime.myext.ts
```

---

## Getting Help

If you're still stuck:

1. **Check existing issues:** [https://github.com/forge-app/forge/issues](https://github.com/forge-app/forge/issues)
2. **Enable debug logging:** `FORGE_LOG=debug forge dev my-app`
3. **Minimal reproduction:** Create smallest possible example that reproduces the issue
4. **Include:**
   - Forge version (`forge --version`)
   - Operating system and version
   - Full error message
   - Relevant configuration (manifest.app.toml, deno.json)

---

## Quick Reference: Error Recovery

| Error Type | First Action |
|------------|--------------|
| Build error | `cargo clean && cargo build` |
| Module not found | Check import paths and `deno.json` |
| Permission denied | Add to `[permissions]` in manifest |
| IPC failure | Verify `window.host` is defined |
| Extension build | Check macro ordering (`#[weld_op]` before `#[op2]`) |
| Slow startup | Enable `FORGE_LOG=trace` and check timings |
| Platform error | Install system dependencies |

---

## See Also

- [DOCUMENTATION.md](./DOCUMENTATION.md) - Full documentation
- [PERFORMANCE.md](./PERFORMANCE.md) - Optimization guide
- [TYPE_MAPPING.md](./TYPE_MAPPING.md) - Rust ↔ TypeScript types
- [GitHub Issues](https://github.com/forge-app/forge/issues) - Report bugs
