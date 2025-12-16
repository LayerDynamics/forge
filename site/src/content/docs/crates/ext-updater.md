---
title: "ext_updater"
description: Auto-update functionality extension providing the runtime:updater module.
slug: crates/ext-updater
---

The `ext_updater` crate provides automatic update functionality for Forge applications through the `runtime:updater` module.

## Overview

ext_updater handles:

- **Update checking** - Check for available updates
- **Download management** - Download update packages
- **Installation** - Apply updates with restart
- **Rollback** - Revert to previous version
- **Progress tracking** - Monitor update progress

## Module: `runtime:updater`

```typescript
import {
  checkForUpdates,
  downloadUpdate,
  installUpdate,
  getUpdateInfo,
  onUpdateAvailable,
  onDownloadProgress
} from "runtime:updater";
```

## Key Types

### Error Types

```rust
enum UpdaterErrorCode {
    Generic = 10100,
    CheckFailed = 10101,
    DownloadFailed = 10102,
    InstallFailed = 10103,
    NoUpdateAvailable = 10104,
    VerificationFailed = 10105,
    RollbackFailed = 10106,
}

struct UpdaterError {
    code: UpdaterErrorCode,
    message: String,
}
```

### Update Types

```rust
struct UpdateInfo {
    version: String,
    current_version: String,
    release_date: String,
    release_notes: Option<String>,
    mandatory: bool,
    download_url: String,
    size_bytes: u64,
    signature: String,
}

struct DownloadProgress {
    downloaded_bytes: u64,
    total_bytes: u64,
    percent: f64,
}

struct UpdaterConfig {
    endpoint: String,
    channel: UpdateChannel,
    auto_check: bool,
    auto_download: bool,
    auto_install: bool,
}

enum UpdateChannel {
    Stable,
    Beta,
    Alpha,
}

struct UpdaterState {
    config: UpdaterConfig,
    current_version: String,
    pending_update: Option<UpdateInfo>,
}
```

## Operations

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_updater_check` | `checkForUpdates()` | Check for updates |
| `op_updater_download` | `downloadUpdate()` | Download available update |
| `op_updater_install` | `installUpdate()` | Install and restart |
| `op_updater_get_info` | `getUpdateInfo()` | Get pending update info |
| `op_updater_on_available` | `onUpdateAvailable(callback)` | Update available event |
| `op_updater_on_progress` | `onDownloadProgress(callback)` | Download progress |
| `op_updater_rollback` | `rollback()` | Rollback to previous |

## Usage Examples

### Check for Updates

```typescript
import { checkForUpdates, getUpdateInfo } from "runtime:updater";

const hasUpdate = await checkForUpdates();
if (hasUpdate) {
  const info = await getUpdateInfo();
  console.log(`New version available: ${info.version}`);
  console.log(`Current version: ${info.current_version}`);
  console.log(`Release notes: ${info.release_notes}`);
}
```

### Download and Install

```typescript
import {
  checkForUpdates,
  downloadUpdate,
  installUpdate,
  onDownloadProgress
} from "runtime:updater";

// Check for updates
if (await checkForUpdates()) {
  // Track progress
  onDownloadProgress((progress) => {
    console.log(`Downloading: ${progress.percent.toFixed(1)}%`);
    updateProgressBar(progress.percent);
  });

  // Download
  await downloadUpdate();

  // Prompt user
  if (await confirmUpdate()) {
    // Install and restart
    await installUpdate();
    // App will restart with new version
  }
}
```

### Auto-Update Flow

```typescript
import {
  checkForUpdates,
  downloadUpdate,
  installUpdate,
  onUpdateAvailable
} from "runtime:updater";

// Listen for updates in background
onUpdateAvailable(async (info) => {
  console.log(`Update ${info.version} available`);

  // Auto-download in background
  await downloadUpdate();

  // Notify user
  showNotification({
    title: "Update Ready",
    body: `Version ${info.version} is ready to install`,
    action: "Restart to Update"
  });
});

// Initial check on startup
checkForUpdates();
```

### Manual Update Check

```typescript
import { checkForUpdates, getUpdateInfo } from "runtime:updater";

async function checkForUpdatesManually() {
  try {
    const hasUpdate = await checkForUpdates();

    if (hasUpdate) {
      const info = await getUpdateInfo();
      showUpdateDialog(info);
    } else {
      showMessage("You're running the latest version!");
    }
  } catch (e) {
    showError("Failed to check for updates");
  }
}
```

### Update with Confirmation

```typescript
import { checkForUpdates, getUpdateInfo, downloadUpdate, installUpdate } from "runtime:updater";

async function promptForUpdate() {
  const hasUpdate = await checkForUpdates();
  if (!hasUpdate) return;

  const info = await getUpdateInfo();

  const result = await showDialog({
    title: "Update Available",
    message: `Version ${info.version} is available. Would you like to update?`,
    detail: info.release_notes,
    buttons: ["Update Now", "Later"]
  });

  if (result === "Update Now") {
    await downloadUpdate();
    await installUpdate();
  }
}
```

## Update Server Response

```json
{
  "version": "2.0.0",
  "release_date": "2024-01-15",
  "release_notes": "Bug fixes and performance improvements",
  "mandatory": false,
  "platforms": {
    "darwin-x86_64": {
      "url": "https://releases.example.com/app-2.0.0-darwin-x64.tar.gz",
      "signature": "abc123...",
      "size": 45678901
    },
    "darwin-aarch64": {
      "url": "https://releases.example.com/app-2.0.0-darwin-arm64.tar.gz",
      "signature": "def456...",
      "size": 43567890
    }
  }
}
```

## File Structure

```text
crates/ext_updater/
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
use forge_weld_macro::{weld_op, weld_struct, weld_enum};
use serde::{Deserialize, Serialize};

#[weld_enum]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateChannel {
    Stable,
    Beta,
    Alpha,
}

#[weld_struct]
#[derive(Debug, Serialize)]
pub struct UpdateInfo {
    pub version: String,
    pub current_version: String,
    pub release_notes: Option<String>,
    pub mandatory: bool,
}

#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_updater_check(
    state: Rc<RefCell<OpState>>,
) -> Result<bool, UpdaterError> {
    // implementation
}
```

## Build Configuration

```rust
// build.rs
use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_updater", "runtime:updater")
        .ts_path("ts/init.ts")
        .ops(&["op_updater_check", "op_updater_download", "op_updater_install", /* ... */])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build runtime_updater extension");
}
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `deno_core` | Op definitions |
| `reqwest` | HTTP downloads |
| `semver` | Version comparison |
| `ring` | Signature verification |
| `tokio` | Async runtime |
| `serde` | Serialization |
| `forge-weld` | Build-time code generation |
| `forge-weld-macro` | `#[weld_op]`, `#[weld_struct]`, `#[weld_enum]` macros |
| `linkme` | Compile-time symbol collection |

## Security

- Updates are verified using Ed25519 signatures
- Download URLs must use HTTPS
- Checksums verified before installation
- Rollback available if update fails

## Related

- [ext_app](/docs/crates/ext-app) - Application lifecycle
- [Architecture](/docs/architecture) - Full system architecture
