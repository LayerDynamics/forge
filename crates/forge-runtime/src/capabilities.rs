//! Capabilities/permissions system for Forge apps
//!
//! In dev mode, all operations are allowed.
//! In production mode, operations are checked against manifest permissions.

use globset::{GlobSet, GlobSetBuilder};
use serde::Deserialize;
use std::path::Path;

/// Permissions section from manifest.app.toml
#[derive(Debug, Deserialize, Clone, Default)]
pub struct Permissions {
    pub fs: Option<FsPermissions>,
    pub net: Option<NetPermissions>,
    pub ui: Option<UiPermissions>,
    pub sys: Option<SysPermissions>,
    pub process: Option<ProcessPermissions>,
    pub wasm: Option<WasmPermissions>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct FsPermissions {
    pub read: Option<Vec<String>>,
    pub write: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct NetPermissions {
    pub allow: Option<Vec<String>>,
    pub deny: Option<Vec<String>>,
    /// Allowed ports for listening (binds to 0.0.0.0)
    pub listen: Option<Vec<u16>>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct UiPermissions {
    pub windows: Option<bool>,
    pub menus: Option<bool>,
    pub dialogs: Option<bool>,
    pub tray: Option<bool>,
    /// Default channel allowlist for new windows (if not specified per-window)
    pub channels: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct SysPermissions {
    pub clipboard: Option<bool>,
    pub notify: Option<bool>,
    pub power: Option<bool>,
    pub env: Option<EnvPermissions>,
}

/// Environment variable permissions
#[derive(Debug, Deserialize, Clone, Default)]
pub struct EnvPermissions {
    /// Glob patterns for env vars allowed to read
    pub read: Option<Vec<String>>,
    /// Glob patterns for env vars allowed to write
    pub write: Option<Vec<String>>,
}

/// Process spawning permissions
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ProcessPermissions {
    /// Glob patterns for allowed binaries/commands
    pub allow: Option<Vec<String>>,
    /// Glob patterns for env vars that can be passed to spawned processes
    pub env: Option<Vec<String>>,
    /// Maximum concurrent child processes (default: 10)
    pub max_processes: Option<usize>,
}

/// WebAssembly permissions
#[derive(Debug, Deserialize, Clone, Default)]
pub struct WasmPermissions {
    /// Glob patterns for allowed WASM file paths
    pub load: Option<Vec<String>>,
    /// Glob patterns for allowed WASI preopened directories
    pub preopens: Option<Vec<String>>,
    /// Maximum concurrent WASM instances (default: 10)
    pub max_instances: Option<usize>,
}

/// Runtime capabilities checker
#[derive(Debug, Clone)]
pub struct Capabilities {
    pub dev_mode: bool,
    fs_read: Option<GlobSet>,
    fs_write: Option<GlobSet>,
    net_allow_patterns: Option<GlobSet>,
    net_deny_patterns: Option<GlobSet>,
    net_listen_ports: Option<Vec<u16>>,
    ui_windows: bool,
    ui_menus: bool,
    ui_dialogs: bool,
    ui_tray: bool,
    /// Default channel allowlist for windows (None = all channels allowed in dev mode, empty = no channels in prod)
    pub ui_channels: Option<Vec<String>>,
    sys_clipboard: bool,
    sys_notify: bool,
    sys_power: bool,
    env_read_patterns: Option<GlobSet>,
    env_write_patterns: Option<GlobSet>,
    process_allow_patterns: Option<GlobSet>,
    process_env_patterns: Option<GlobSet>,
    pub process_max_processes: usize,
    wasm_load_patterns: Option<GlobSet>,
    wasm_preopen_patterns: Option<GlobSet>,
    pub wasm_max_instances: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum CapabilityError {
    #[error("Permission denied: {capability} for {resource}")]
    Denied {
        capability: String,
        resource: String,
    },

    #[error("Invalid glob pattern: {0}")]
    InvalidPattern(String),
}

impl Capabilities {
    /// Create capabilities from manifest permissions
    pub fn from_permissions(
        permissions: Option<&Permissions>,
        dev_mode: bool,
    ) -> Result<Self, CapabilityError> {
        let permissions = permissions.cloned().unwrap_or_default();

        let fs_read =
            Self::compile_patterns(permissions.fs.as_ref().and_then(|f| f.read.as_ref()))?;

        let fs_write =
            Self::compile_patterns(permissions.fs.as_ref().and_then(|f| f.write.as_ref()))?;

        let net = permissions.net.unwrap_or_default();
        let ui = permissions.ui.unwrap_or_default();
        let sys = permissions.sys.unwrap_or_default();
        let process = permissions.process.unwrap_or_default();
        let wasm = permissions.wasm.unwrap_or_default();

        // Compile network host patterns (for wildcard matching like *.example.com)
        let net_allow_patterns = Self::compile_host_patterns(net.allow.as_ref())?;
        let net_deny_patterns = Self::compile_host_patterns(net.deny.as_ref())?;

        // Compile env var patterns
        let env_read_patterns =
            Self::compile_simple_patterns(sys.env.as_ref().and_then(|e| e.read.as_ref()))?;
        let env_write_patterns =
            Self::compile_simple_patterns(sys.env.as_ref().and_then(|e| e.write.as_ref()))?;

        // Compile process patterns
        let process_allow_patterns = Self::compile_simple_patterns(process.allow.as_ref())?;
        let process_env_patterns = Self::compile_simple_patterns(process.env.as_ref())?;

        // Compile WASM patterns
        let wasm_load_patterns = Self::compile_patterns(wasm.load.as_ref())?;
        let wasm_preopen_patterns = Self::compile_patterns(wasm.preopens.as_ref())?;

        Ok(Self {
            dev_mode,
            fs_read,
            fs_write,
            net_allow_patterns,
            net_deny_patterns,
            net_listen_ports: net.listen,
            ui_windows: ui.windows.unwrap_or(true), // windows allowed by default
            ui_menus: ui.menus.unwrap_or(true),
            ui_dialogs: ui.dialogs.unwrap_or(true),
            ui_tray: ui.tray.unwrap_or(false), // tray requires explicit permission
            ui_channels: ui.channels.clone(),
            sys_clipboard: sys.clipboard.unwrap_or(false),
            sys_notify: sys.notify.unwrap_or(false),
            sys_power: sys.power.unwrap_or(false),
            env_read_patterns,
            env_write_patterns,
            process_allow_patterns,
            process_env_patterns,
            process_max_processes: process.max_processes.unwrap_or(10),
            wasm_load_patterns,
            wasm_preopen_patterns,
            wasm_max_instances: wasm.max_instances.unwrap_or(10),
        })
    }

    /// Compile glob patterns for filesystem paths (uses literal_separator)
    fn compile_patterns(
        patterns: Option<&Vec<String>>,
    ) -> Result<Option<GlobSet>, CapabilityError> {
        match patterns {
            None => Ok(None),
            Some(pats) if pats.is_empty() => Ok(None),
            Some(pats) => {
                let mut builder = GlobSetBuilder::new();
                for pat in pats {
                    // Use literal_separator so * doesn't match path separators
                    // (only ** should match across directories)
                    let glob = globset::GlobBuilder::new(pat)
                        .literal_separator(true)
                        .build()
                        .map_err(|e| CapabilityError::InvalidPattern(e.to_string()))?;
                    builder.add(glob);
                }
                Ok(Some(builder.build().map_err(|e| {
                    CapabilityError::InvalidPattern(e.to_string())
                })?))
            }
        }
    }

    /// Compile glob patterns for hostnames (supports *.example.com style wildcards)
    fn compile_host_patterns(
        patterns: Option<&Vec<String>>,
    ) -> Result<Option<GlobSet>, CapabilityError> {
        match patterns {
            None => Ok(None),
            Some(pats) if pats.is_empty() => Ok(None),
            Some(pats) => {
                let mut builder = GlobSetBuilder::new();
                for pat in pats {
                    // For hosts, we don't use literal_separator since . is part of the domain
                    // * matches anything, *.example.com matches subdomains
                    let glob = globset::GlobBuilder::new(pat)
                        .literal_separator(false)
                        .build()
                        .map_err(|e| CapabilityError::InvalidPattern(e.to_string()))?;
                    builder.add(glob);
                }
                Ok(Some(builder.build().map_err(|e| {
                    CapabilityError::InvalidPattern(e.to_string())
                })?))
            }
        }
    }

    /// Compile simple patterns (env vars, binary names) without path separator handling
    fn compile_simple_patterns(
        patterns: Option<&Vec<String>>,
    ) -> Result<Option<GlobSet>, CapabilityError> {
        match patterns {
            None => Ok(None),
            Some(pats) if pats.is_empty() => Ok(None),
            Some(pats) => {
                let mut builder = GlobSetBuilder::new();
                for pat in pats {
                    let glob = globset::GlobBuilder::new(pat)
                        .literal_separator(false)
                        .build()
                        .map_err(|e| CapabilityError::InvalidPattern(e.to_string()))?;
                    builder.add(glob);
                }
                Ok(Some(builder.build().map_err(|e| {
                    CapabilityError::InvalidPattern(e.to_string())
                })?))
            }
        }
    }

    /// Check if filesystem read is allowed for the given path
    pub fn check_fs_read(&self, path: &str) -> Result<(), CapabilityError> {
        if self.dev_mode {
            return Ok(());
        }

        match &self.fs_read {
            None => Err(CapabilityError::Denied {
                capability: "fs.read".to_string(),
                resource: path.to_string(),
            }),
            Some(patterns) => {
                let p = Path::new(path);
                if patterns.is_match(p) {
                    Ok(())
                } else {
                    Err(CapabilityError::Denied {
                        capability: "fs.read".to_string(),
                        resource: path.to_string(),
                    })
                }
            }
        }
    }

    /// Check if filesystem write is allowed for the given path
    pub fn check_fs_write(&self, path: &str) -> Result<(), CapabilityError> {
        if self.dev_mode {
            return Ok(());
        }

        match &self.fs_write {
            None => Err(CapabilityError::Denied {
                capability: "fs.write".to_string(),
                resource: path.to_string(),
            }),
            Some(patterns) => {
                let p = Path::new(path);
                if patterns.is_match(p) {
                    Ok(())
                } else {
                    Err(CapabilityError::Denied {
                        capability: "fs.write".to_string(),
                        resource: path.to_string(),
                    })
                }
            }
        }
    }

    /// Check if network access is allowed for the given host
    pub fn check_net(&self, host: &str) -> Result<(), CapabilityError> {
        if self.dev_mode {
            return Ok(());
        }

        // Check deny patterns first (takes precedence)
        if let Some(deny_patterns) = &self.net_deny_patterns {
            if deny_patterns.is_match(host) {
                return Err(CapabilityError::Denied {
                    capability: "net".to_string(),
                    resource: host.to_string(),
                });
            }
        }

        // Check allow patterns
        match &self.net_allow_patterns {
            None => Err(CapabilityError::Denied {
                capability: "net".to_string(),
                resource: host.to_string(),
            }),
            Some(allow_patterns) => {
                if allow_patterns.is_match(host) {
                    Ok(())
                } else {
                    Err(CapabilityError::Denied {
                        capability: "net".to_string(),
                        resource: host.to_string(),
                    })
                }
            }
        }
    }

    /// Check if network listen is allowed for the given port
    pub fn check_net_listen(&self, port: u16) -> Result<(), CapabilityError> {
        if self.dev_mode {
            return Ok(());
        }

        match &self.net_listen_ports {
            None => Err(CapabilityError::Denied {
                capability: "net.listen".to_string(),
                resource: port.to_string(),
            }),
            Some(ports) => {
                // Port 0 in the list means any port is allowed
                if ports.contains(&0) || ports.contains(&port) {
                    Ok(())
                } else {
                    Err(CapabilityError::Denied {
                        capability: "net.listen".to_string(),
                        resource: port.to_string(),
                    })
                }
            }
        }
    }

    /// Check if window creation is allowed
    pub fn check_ui_windows(&self) -> Result<(), CapabilityError> {
        if self.dev_mode || self.ui_windows {
            Ok(())
        } else {
            Err(CapabilityError::Denied {
                capability: "ui.windows".to_string(),
                resource: "window".to_string(),
            })
        }
    }

    /// Check if menu operations are allowed
    pub fn check_ui_menus(&self) -> Result<(), CapabilityError> {
        if self.dev_mode || self.ui_menus {
            Ok(())
        } else {
            Err(CapabilityError::Denied {
                capability: "ui.menus".to_string(),
                resource: "menu".to_string(),
            })
        }
    }

    /// Check if dialog operations are allowed
    pub fn check_ui_dialogs(&self) -> Result<(), CapabilityError> {
        if self.dev_mode || self.ui_dialogs {
            Ok(())
        } else {
            Err(CapabilityError::Denied {
                capability: "ui.dialogs".to_string(),
                resource: "dialog".to_string(),
            })
        }
    }

    /// Check if tray operations are allowed
    pub fn check_ui_tray(&self) -> Result<(), CapabilityError> {
        if self.dev_mode || self.ui_tray {
            Ok(())
        } else {
            Err(CapabilityError::Denied {
                capability: "ui.tray".to_string(),
                resource: "tray".to_string(),
            })
        }
    }

    /// Check if clipboard operations are allowed
    pub fn check_sys_clipboard(&self) -> Result<(), CapabilityError> {
        if self.dev_mode || self.sys_clipboard {
            Ok(())
        } else {
            Err(CapabilityError::Denied {
                capability: "sys.clipboard".to_string(),
                resource: "clipboard".to_string(),
            })
        }
    }

    /// Check if notification operations are allowed
    pub fn check_sys_notify(&self) -> Result<(), CapabilityError> {
        if self.dev_mode || self.sys_notify {
            Ok(())
        } else {
            Err(CapabilityError::Denied {
                capability: "sys.notify".to_string(),
                resource: "notification".to_string(),
            })
        }
    }

    /// Check if power/battery info access is allowed
    pub fn check_sys_power(&self) -> Result<(), CapabilityError> {
        if self.dev_mode || self.sys_power {
            Ok(())
        } else {
            Err(CapabilityError::Denied {
                capability: "sys.power".to_string(),
                resource: "power info".to_string(),
            })
        }
    }

    /// Check if reading an environment variable is allowed
    pub fn check_env_read(&self, key: &str) -> Result<(), CapabilityError> {
        if self.dev_mode {
            return Ok(());
        }

        match &self.env_read_patterns {
            // If no env permissions specified, allow all reads by default (backwards compatible)
            None => Ok(()),
            Some(patterns) => {
                if patterns.is_match(key) {
                    Ok(())
                } else {
                    Err(CapabilityError::Denied {
                        capability: "sys.env.read".to_string(),
                        resource: key.to_string(),
                    })
                }
            }
        }
    }

    /// Check if writing an environment variable is allowed
    pub fn check_env_write(&self, key: &str) -> Result<(), CapabilityError> {
        if self.dev_mode {
            return Ok(());
        }

        match &self.env_write_patterns {
            // If no env permissions specified, deny all writes by default
            None => Err(CapabilityError::Denied {
                capability: "sys.env.write".to_string(),
                resource: key.to_string(),
            }),
            Some(patterns) => {
                if patterns.is_match(key) {
                    Ok(())
                } else {
                    Err(CapabilityError::Denied {
                        capability: "sys.env.write".to_string(),
                        resource: key.to_string(),
                    })
                }
            }
        }
    }

    /// Check if spawning a process is allowed
    pub fn check_process_spawn(&self, binary: &str) -> Result<(), CapabilityError> {
        if self.dev_mode {
            return Ok(());
        }

        match &self.process_allow_patterns {
            None => Err(CapabilityError::Denied {
                capability: "process.spawn".to_string(),
                resource: binary.to_string(),
            }),
            Some(patterns) => {
                if patterns.is_match(binary) {
                    Ok(())
                } else {
                    Err(CapabilityError::Denied {
                        capability: "process.spawn".to_string(),
                        resource: binary.to_string(),
                    })
                }
            }
        }
    }

    /// Check if passing an env var to a spawned process is allowed
    pub fn check_process_env(&self, key: &str) -> Result<(), CapabilityError> {
        if self.dev_mode {
            return Ok(());
        }

        match &self.process_env_patterns {
            // If no process env permissions specified, allow inherited vars only
            None => Err(CapabilityError::Denied {
                capability: "process.env".to_string(),
                resource: key.to_string(),
            }),
            Some(patterns) => {
                if patterns.is_match(key) {
                    Ok(())
                } else {
                    Err(CapabilityError::Denied {
                        capability: "process.env".to_string(),
                        resource: key.to_string(),
                    })
                }
            }
        }
    }

    /// Check if a channel is allowed for IPC communication
    /// Returns true if the channel is allowed, false otherwise
    pub fn check_channel(
        &self,
        channel: &str,
        window_channels: Option<&[String]>,
    ) -> Result<(), CapabilityError> {
        // In dev mode, all channels are allowed
        if self.dev_mode {
            return Ok(());
        }

        // If window has specific channel allowlist, use that
        if let Some(window_channels) = window_channels {
            if window_channels.iter().any(|c| c == "*" || c == channel) {
                return Ok(());
            }
            return Err(CapabilityError::Denied {
                capability: "ui.channel".to_string(),
                resource: channel.to_string(),
            });
        }

        // Otherwise, use the default channel allowlist from manifest
        if let Some(default_channels) = &self.ui_channels {
            if default_channels.iter().any(|c| c == "*" || c == channel) {
                return Ok(());
            }
            return Err(CapabilityError::Denied {
                capability: "ui.channel".to_string(),
                resource: channel.to_string(),
            });
        }

        // Default-deny: If no channel list specified in manifest, deny all channels
        // Apps must explicitly opt-in with ["*"] for all channels or specific channel names
        Err(CapabilityError::Denied {
            capability: "ui.channel".to_string(),
            resource: channel.to_string(),
        })
    }

    /// Get the default channel allowlist
    pub fn get_default_channels(&self) -> Option<Vec<String>> {
        self.ui_channels.clone()
    }

    /// Get the maximum number of concurrent child processes
    pub fn get_max_processes(&self) -> usize {
        self.process_max_processes
    }

    /// Check if loading WASM from a path is allowed
    pub fn check_wasm_load(&self, path: &str) -> Result<(), CapabilityError> {
        if self.dev_mode {
            return Ok(());
        }

        match &self.wasm_load_patterns {
            None => Err(CapabilityError::Denied {
                capability: "wasm.load".to_string(),
                resource: path.to_string(),
            }),
            Some(patterns) => {
                let p = Path::new(path);
                if patterns.is_match(p) {
                    Ok(())
                } else {
                    Err(CapabilityError::Denied {
                        capability: "wasm.load".to_string(),
                        resource: path.to_string(),
                    })
                }
            }
        }
    }

    /// Check if preopening a directory for WASI is allowed
    pub fn check_wasm_preopen(&self, host_path: &str) -> Result<(), CapabilityError> {
        if self.dev_mode {
            return Ok(());
        }

        match &self.wasm_preopen_patterns {
            None => Err(CapabilityError::Denied {
                capability: "wasm.preopen".to_string(),
                resource: host_path.to_string(),
            }),
            Some(patterns) => {
                let p = Path::new(host_path);
                if patterns.is_match(p) {
                    Ok(())
                } else {
                    Err(CapabilityError::Denied {
                        capability: "wasm.preopen".to_string(),
                        resource: host_path.to_string(),
                    })
                }
            }
        }
    }

    /// Get the maximum number of concurrent WASM instances
    pub fn get_max_wasm_instances(&self) -> usize {
        self.wasm_max_instances
    }
}

// ============================================================================
// Adapters implementing extension capability checker traits
// ============================================================================

use std::sync::Arc;

/// Struct containing all capability adapter Arc pointers
pub struct CapabilityAdapters {
    pub fs: Arc<dyn ext_fs::FsCapabilityChecker>,
    pub ipc: Arc<dyn ext_ipc::IpcCapabilityChecker>,
    pub net: Arc<dyn ext_net::NetCapabilityChecker>,
    pub sys: Arc<dyn ext_sys::SysCapabilityChecker>,
    pub window: Arc<dyn ext_window::WindowCapabilityChecker>,
    pub process: Arc<dyn ext_process::ProcessCapabilityChecker>,
    pub wasm: Arc<dyn ext_wasm::WasmCapabilityChecker>,
}

/// Adapter that implements ext_fs::FsCapabilityChecker using Capabilities
pub struct FsCapabilityAdapter {
    capabilities: Arc<Capabilities>,
}

impl FsCapabilityAdapter {
    pub fn new(capabilities: Arc<Capabilities>) -> Self {
        Self { capabilities }
    }
}

impl ext_fs::FsCapabilityChecker for FsCapabilityAdapter {
    fn check_read(&self, path: &str) -> Result<(), String> {
        self.capabilities
            .check_fs_read(path)
            .map_err(|e| e.to_string())
    }

    fn check_write(&self, path: &str) -> Result<(), String> {
        self.capabilities
            .check_fs_write(path)
            .map_err(|e| e.to_string())
    }
}

/// Adapter that implements ext_net::NetCapabilityChecker using Capabilities
pub struct NetCapabilityAdapter {
    capabilities: Arc<Capabilities>,
}

impl NetCapabilityAdapter {
    pub fn new(capabilities: Arc<Capabilities>) -> Self {
        Self { capabilities }
    }
}

impl ext_net::NetCapabilityChecker for NetCapabilityAdapter {
    fn check_connect(&self, host: &str) -> Result<(), String> {
        self.capabilities.check_net(host).map_err(|e| e.to_string())
    }

    fn check_listen(&self, port: u16) -> Result<(), String> {
        self.capabilities
            .check_net_listen(port)
            .map_err(|e| e.to_string())
    }
}

/// Adapter that implements ext_sys::SysCapabilityChecker using Capabilities
pub struct SysCapabilityAdapter {
    capabilities: Arc<Capabilities>,
}

impl SysCapabilityAdapter {
    pub fn new(capabilities: Arc<Capabilities>) -> Self {
        Self { capabilities }
    }
}

impl ext_sys::SysCapabilityChecker for SysCapabilityAdapter {
    fn check_clipboard_read(&self) -> Result<(), String> {
        self.capabilities
            .check_sys_clipboard()
            .map_err(|e| e.to_string())
    }

    fn check_clipboard_write(&self) -> Result<(), String> {
        self.capabilities
            .check_sys_clipboard()
            .map_err(|e| e.to_string())
    }

    fn check_notify(&self) -> Result<(), String> {
        self.capabilities
            .check_sys_notify()
            .map_err(|e| e.to_string())
    }

    fn check_env(&self, key: &str) -> Result<(), String> {
        self.capabilities
            .check_env_read(key)
            .map_err(|e| e.to_string())
    }

    fn check_env_write(&self, key: &str) -> Result<(), String> {
        self.capabilities
            .check_env_write(key)
            .map_err(|e| e.to_string())
    }

    fn check_power(&self) -> Result<(), String> {
        self.capabilities
            .check_sys_power()
            .map_err(|e| e.to_string())
    }
}

/// Adapter that implements ext_window::WindowCapabilityChecker using Capabilities
pub struct WindowCapabilityAdapter {
    capabilities: Arc<Capabilities>,
}

impl WindowCapabilityAdapter {
    pub fn new(capabilities: Arc<Capabilities>) -> Self {
        Self { capabilities }
    }
}

impl ext_window::WindowCapabilityChecker for WindowCapabilityAdapter {
    fn check_windows(&self) -> Result<(), String> {
        self.capabilities
            .check_ui_windows()
            .map_err(|e| e.to_string())
    }

    fn check_menus(&self) -> Result<(), String> {
        self.capabilities
            .check_ui_menus()
            .map_err(|e| e.to_string())
    }

    fn check_dialogs(&self) -> Result<(), String> {
        self.capabilities
            .check_ui_dialogs()
            .map_err(|e| e.to_string())
    }

    fn check_tray(&self) -> Result<(), String> {
        self.capabilities.check_ui_tray().map_err(|e| e.to_string())
    }

    fn check_native_handle(&self) -> Result<(), String> {
        // Native handle access requires windows permission
        self.capabilities
            .check_ui_windows()
            .map_err(|e| e.to_string())
    }
}

/// Adapter that implements ext_ipc::IpcCapabilityChecker using Capabilities
pub struct IpcCapabilityAdapter {
    capabilities: Arc<Capabilities>,
}

impl IpcCapabilityAdapter {
    pub fn new(capabilities: Arc<Capabilities>) -> Self {
        Self { capabilities }
    }
}

impl ext_ipc::IpcCapabilityChecker for IpcCapabilityAdapter {
    fn check_channel(
        &self,
        channel: &str,
        window_channels: Option<&[String]>,
    ) -> Result<(), String> {
        self.capabilities
            .check_channel(channel, window_channels)
            .map_err(|e| e.to_string())
    }
}

/// Adapter that implements ext_process::ProcessCapabilityChecker using Capabilities
pub struct ProcessCapabilityAdapter {
    capabilities: Arc<Capabilities>,
}

impl ProcessCapabilityAdapter {
    pub fn new(capabilities: Arc<Capabilities>) -> Self {
        Self { capabilities }
    }
}

impl ext_process::ProcessCapabilityChecker for ProcessCapabilityAdapter {
    fn check_spawn(&self, binary: &str) -> Result<(), String> {
        self.capabilities
            .check_process_spawn(binary)
            .map_err(|e| e.to_string())
    }

    fn check_env(&self, key: &str) -> Result<(), String> {
        self.capabilities
            .check_process_env(key)
            .map_err(|e| e.to_string())
    }
}

/// Adapter that implements ext_wasm::WasmCapabilityChecker using Capabilities
pub struct WasmCapabilityAdapter {
    capabilities: Arc<Capabilities>,
}

impl WasmCapabilityAdapter {
    pub fn new(capabilities: Arc<Capabilities>) -> Self {
        Self { capabilities }
    }
}

impl ext_wasm::WasmCapabilityChecker for WasmCapabilityAdapter {
    fn check_load(&self, path: &str) -> Result<(), String> {
        self.capabilities
            .check_wasm_load(path)
            .map_err(|e| e.to_string())
    }

    fn check_preopen(&self, host_path: &str) -> Result<(), String> {
        self.capabilities
            .check_wasm_preopen(host_path)
            .map_err(|e| e.to_string())
    }
}

/// Create all capability adapters from Capabilities
pub fn create_capability_adapters(capabilities: Capabilities) -> CapabilityAdapters {
    let caps = Arc::new(capabilities);
    CapabilityAdapters {
        fs: Arc::new(FsCapabilityAdapter::new(caps.clone())),
        ipc: Arc::new(IpcCapabilityAdapter::new(caps.clone())),
        net: Arc::new(NetCapabilityAdapter::new(caps.clone())),
        sys: Arc::new(SysCapabilityAdapter::new(caps.clone())),
        window: Arc::new(WindowCapabilityAdapter::new(caps.clone())),
        process: Arc::new(ProcessCapabilityAdapter::new(caps.clone())),
        wasm: Arc::new(WasmCapabilityAdapter::new(caps)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dev_mode_allows_all() {
        let caps = Capabilities::from_permissions(None, true).unwrap();
        assert!(caps.check_fs_read("/any/path").is_ok());
        assert!(caps.check_fs_write("/any/path").is_ok());
        assert!(caps.check_net("any.host.com").is_ok());
        assert!(caps.check_net_listen(8080).is_ok());
        assert!(caps.check_ui_windows().is_ok());
        assert!(caps.check_ui_menus().is_ok());
        assert!(caps.check_ui_dialogs().is_ok());
        assert!(caps.check_ui_tray().is_ok());
        assert!(caps.check_sys_clipboard().is_ok());
        assert!(caps.check_sys_notify().is_ok());
        assert!(caps.check_sys_power().is_ok());
        assert!(caps.check_env_read("PATH").is_ok());
        assert!(caps.check_env_write("MY_VAR").is_ok());
        assert!(caps.check_process_spawn("ls").is_ok());
        assert!(caps.check_process_env("HOME").is_ok());
        assert!(caps.check_channel("any-channel", None).is_ok());
    }

    #[test]
    fn test_no_permissions_denies_all() {
        let caps = Capabilities::from_permissions(None, false).unwrap();
        assert!(caps.check_fs_read("/any/path").is_err());
        assert!(caps.check_fs_write("/any/path").is_err());
        assert!(caps.check_net("any.host.com").is_err());
        assert!(caps.check_net_listen(8080).is_err());
        // UI defaults: windows, menus, dialogs allowed; tray denied
        assert!(caps.check_ui_windows().is_ok());
        assert!(caps.check_ui_menus().is_ok());
        assert!(caps.check_ui_dialogs().is_ok());
        assert!(caps.check_ui_tray().is_err());
        // sys defaults: clipboard, notify, power denied
        assert!(caps.check_sys_clipboard().is_err());
        assert!(caps.check_sys_notify().is_err());
        assert!(caps.check_sys_power().is_err());
        // env reads allowed by default, writes denied
        assert!(caps.check_env_read("PATH").is_ok());
        assert!(caps.check_env_write("MY_VAR").is_err());
        // process spawn denied without permissions
        assert!(caps.check_process_spawn("ls").is_err());
        assert!(caps.check_process_env("HOME").is_err());
    }

    #[test]
    fn test_glob_patterns() {
        let perms = Permissions {
            fs: Some(FsPermissions {
                read: Some(vec!["./data/**".to_string()]),
                write: Some(vec!["./data/*.txt".to_string()]),
            }),
            ..Default::default()
        };
        let caps = Capabilities::from_permissions(Some(&perms), false).unwrap();

        assert!(caps.check_fs_read("./data/foo.txt").is_ok());
        assert!(caps.check_fs_read("./data/sub/bar.txt").is_ok());
        assert!(caps.check_fs_read("./other/foo.txt").is_err());

        assert!(caps.check_fs_write("./data/foo.txt").is_ok());
        assert!(caps.check_fs_write("./data/sub/bar.txt").is_err()); // doesn't match *.txt
    }

    #[test]
    fn test_net_allow_deny() {
        let perms = Permissions {
            net: Some(NetPermissions {
                allow: Some(vec![
                    "api.example.com".to_string(),
                    "*.trusted.com".to_string(),
                ]),
                deny: Some(vec!["evil.com".to_string()]),
                listen: None,
            }),
            ..Default::default()
        };
        let caps = Capabilities::from_permissions(Some(&perms), false).unwrap();

        assert!(caps.check_net("api.example.com").is_ok());
        assert!(caps.check_net("sub.trusted.com").is_ok()); // matches wildcard
        assert!(caps.check_net("deep.sub.trusted.com").is_ok()); // also matches
        assert!(caps.check_net("random.com").is_err()); // not in allow list
        assert!(caps.check_net("evil.com").is_err()); // explicitly denied
    }

    #[test]
    fn test_net_listen() {
        let perms = Permissions {
            net: Some(NetPermissions {
                allow: None,
                deny: None,
                listen: Some(vec![8080, 3000]),
            }),
            ..Default::default()
        };
        let caps = Capabilities::from_permissions(Some(&perms), false).unwrap();

        assert!(caps.check_net_listen(8080).is_ok());
        assert!(caps.check_net_listen(3000).is_ok());
        assert!(caps.check_net_listen(9000).is_err()); // not in list
    }

    #[test]
    fn test_net_listen_any_port() {
        let perms = Permissions {
            net: Some(NetPermissions {
                allow: None,
                deny: None,
                listen: Some(vec![0]), // 0 means any port
            }),
            ..Default::default()
        };
        let caps = Capabilities::from_permissions(Some(&perms), false).unwrap();

        assert!(caps.check_net_listen(8080).is_ok());
        assert!(caps.check_net_listen(9000).is_ok());
        assert!(caps.check_net_listen(443).is_ok());
    }

    #[test]
    fn test_net_deny_takes_precedence() {
        let perms = Permissions {
            net: Some(NetPermissions {
                allow: Some(vec!["*".to_string()]),       // allow all
                deny: Some(vec!["evil.com".to_string()]), // but deny evil.com
                listen: None,
            }),
            ..Default::default()
        };
        let caps = Capabilities::from_permissions(Some(&perms), false).unwrap();

        assert!(caps.check_net("good.com").is_ok());
        assert!(caps.check_net("evil.com").is_err()); // deny takes precedence
    }

    #[test]
    fn test_sys_permissions() {
        let perms = Permissions {
            sys: Some(SysPermissions {
                clipboard: Some(true),
                notify: Some(false),
                power: Some(true),
                env: None,
            }),
            ..Default::default()
        };
        let caps = Capabilities::from_permissions(Some(&perms), false).unwrap();

        assert!(caps.check_sys_clipboard().is_ok());
        assert!(caps.check_sys_notify().is_err());
        assert!(caps.check_sys_power().is_ok());
    }

    #[test]
    fn test_env_permissions() {
        let perms = Permissions {
            sys: Some(SysPermissions {
                clipboard: None,
                notify: None,
                power: None,
                env: Some(EnvPermissions {
                    read: Some(vec![
                        "HOME".to_string(),
                        "PATH".to_string(),
                        "MY_*".to_string(),
                    ]),
                    write: Some(vec!["MY_*".to_string()]),
                }),
            }),
            ..Default::default()
        };
        let caps = Capabilities::from_permissions(Some(&perms), false).unwrap();

        // Read permissions
        assert!(caps.check_env_read("HOME").is_ok());
        assert!(caps.check_env_read("PATH").is_ok());
        assert!(caps.check_env_read("MY_VAR").is_ok()); // matches wildcard
        assert!(caps.check_env_read("SECRET").is_err()); // not in list

        // Write permissions
        assert!(caps.check_env_write("MY_VAR").is_ok()); // matches wildcard
        assert!(caps.check_env_write("HOME").is_err()); // not in write list
    }

    #[test]
    fn test_process_permissions() {
        let perms = Permissions {
            process: Some(ProcessPermissions {
                allow: Some(vec![
                    "ls".to_string(),
                    "cat".to_string(),
                    "/usr/bin/*".to_string(),
                ]),
                env: Some(vec!["PATH".to_string(), "HOME".to_string()]),
                max_processes: Some(5),
            }),
            ..Default::default()
        };
        let caps = Capabilities::from_permissions(Some(&perms), false).unwrap();

        // Spawn permissions
        assert!(caps.check_process_spawn("ls").is_ok());
        assert!(caps.check_process_spawn("cat").is_ok());
        assert!(caps.check_process_spawn("/usr/bin/grep").is_ok()); // matches wildcard
        assert!(caps.check_process_spawn("rm").is_err()); // not allowed

        // Process env permissions
        assert!(caps.check_process_env("PATH").is_ok());
        assert!(caps.check_process_env("HOME").is_ok());
        assert!(caps.check_process_env("SECRET").is_err()); // not allowed

        // Max processes
        assert_eq!(caps.process_max_processes, 5);
    }

    #[test]
    fn test_ui_permissions() {
        let perms = Permissions {
            ui: Some(UiPermissions {
                windows: Some(true),
                menus: Some(false),
                dialogs: Some(true),
                tray: Some(true),
                channels: Some(vec!["app:*".to_string()]),
            }),
            ..Default::default()
        };
        let caps = Capabilities::from_permissions(Some(&perms), false).unwrap();

        assert!(caps.check_ui_windows().is_ok());
        assert!(caps.check_ui_menus().is_err());
        assert!(caps.check_ui_dialogs().is_ok());
        assert!(caps.check_ui_tray().is_ok());
    }

    #[test]
    fn test_channel_filtering() {
        let perms = Permissions {
            ui: Some(UiPermissions {
                channels: Some(vec!["app:config".to_string(), "app:data".to_string()]),
                ..Default::default()
            }),
            ..Default::default()
        };
        let caps = Capabilities::from_permissions(Some(&perms), false).unwrap();

        // Using manifest default channels
        assert!(caps.check_channel("app:config", None).is_ok());
        assert!(caps.check_channel("app:data", None).is_ok());
        assert!(caps.check_channel("app:secret", None).is_err());

        // Using window-specific channels (overrides manifest)
        let window_channels = vec!["private:*".to_string()];
        assert!(caps
            .check_channel("private:*", Some(&window_channels))
            .is_ok());
        assert!(caps
            .check_channel("app:config", Some(&window_channels))
            .is_err());
    }

    #[test]
    fn test_channel_wildcard() {
        let perms = Permissions {
            ui: Some(UiPermissions {
                channels: Some(vec!["*".to_string()]),
                ..Default::default()
            }),
            ..Default::default()
        };
        let caps = Capabilities::from_permissions(Some(&perms), false).unwrap();

        assert!(caps.check_channel("any:channel", None).is_ok());
        assert!(caps.check_channel("other:channel", None).is_ok());
    }

    #[test]
    fn test_error_messages() {
        let caps = Capabilities::from_permissions(None, false).unwrap();

        let err = caps.check_fs_read("/secret/file").unwrap_err();
        assert!(err.to_string().contains("fs.read"));
        assert!(err.to_string().contains("/secret/file"));

        let err = caps.check_net("blocked.com").unwrap_err();
        assert!(err.to_string().contains("net"));
        assert!(err.to_string().contains("blocked.com"));
    }

    #[test]
    fn test_adapters() {
        let caps = Capabilities::from_permissions(None, true).unwrap();
        let adapters = create_capability_adapters(caps);

        // Test FS adapter
        assert!(adapters.fs.check_read("/any/path").is_ok());
        assert!(adapters.fs.check_write("/any/path").is_ok());

        // Test Net adapter
        assert!(adapters.net.check_connect("any.host.com").is_ok());
        assert!(adapters.net.check_listen(8080).is_ok());

        // Test Sys adapter
        assert!(adapters.sys.check_clipboard_read().is_ok());
        assert!(adapters.sys.check_clipboard_write().is_ok());
        assert!(adapters.sys.check_notify().is_ok());
        assert!(adapters.sys.check_env("PATH").is_ok());
        assert!(adapters.sys.check_env_write("MY_VAR").is_ok());
        assert!(adapters.sys.check_power().is_ok());

        // Test Window adapter (covers UI capabilities)
        assert!(adapters.window.check_windows().is_ok());
        assert!(adapters.window.check_menus().is_ok());
        assert!(adapters.window.check_dialogs().is_ok());
        assert!(adapters.window.check_tray().is_ok());

        // Test Process adapter
        assert!(adapters.process.check_spawn("ls").is_ok());
        assert!(adapters.process.check_env("PATH").is_ok());

        // Test WASM adapter
        assert!(adapters.wasm.check_load("/any/path.wasm").is_ok());
        assert!(adapters.wasm.check_preopen("/any/dir").is_ok());
    }
}
