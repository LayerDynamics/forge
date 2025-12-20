//! Extension Registry - Centralized management of all Forge runtime extensions
//!
//! This module provides:
//! - `ExtensionRegistry`: Central registry for all ext_* extensions
//! - `ExtensionDescriptor`: Metadata about each extension
//! - `ExtensionInitContext`: Context for state initialization
//!
//! All extensions are available at both build time (for TypeScript binding generation)
//! and runtime (for app execution). Users import only what they need via `runtime:*` specifiers.

use crate::capabilities::{Capabilities, CapabilityAdapters};
use deno_core::{Extension, OpState};
use std::sync::Arc;
use tokio::sync::mpsc;

// Re-export extension types needed for context
pub use ext_app::AppInfo;
pub use ext_ipc::{IpcEvent, ToRendererCmd};
pub use ext_window::{MenuEvent as WinMenuEvent, WindowCmd, WindowSystemEvent};

// ============================================================================
// Core Types
// ============================================================================

/// Extension initialization complexity tier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionTier {
    /// No state initialization required - just register the extension
    ExtensionOnly,
    /// Simple state initialization (no external dependencies)
    SimpleState,
    /// Requires capability adapter injection
    CapabilityBased,
    /// Requires complex runtime context (channels, app info, etc.)
    ComplexContext,
}

/// Metadata describing an extension
#[allow(dead_code)]
pub struct ExtensionDescriptor {
    /// Extension name (e.g., "fs", "window")
    pub name: &'static str,
    /// Runtime specifier (e.g., "runtime:fs") - used for documentation and future filtering
    pub specifier: &'static str,
    /// Initialization complexity tier
    pub tier: ExtensionTier,
    /// Factory function to create the Extension
    pub extension_fn: fn() -> Extension,
    /// Whether this extension is required for core functionality
    pub required: bool,
}

/// Channels for IPC communication (Deno <-> Renderer)
/// Note: Currently initialized manually via init_ipc_manually due to channel consumption
#[allow(dead_code)]
pub struct IpcChannels {
    pub to_renderer_tx: mpsc::Sender<ToRendererCmd>,
    pub to_deno_rx: mpsc::Receiver<IpcEvent>,
}

/// Channels for window operations
/// Note: Currently initialized manually via init_window_manually due to channel consumption
#[allow(dead_code)]
pub struct WindowChannels {
    pub cmd_tx: mpsc::Sender<WindowCmd>,
    pub events_rx: mpsc::Receiver<WindowSystemEvent>,
    pub menu_events_rx: mpsc::Receiver<WinMenuEvent>,
}

/// Context for extension state initialization
///
/// This struct provides all possible initialization data that extensions might need.
/// Extensions only use the fields relevant to their tier.
#[allow(dead_code)]
pub struct ExtensionInitContext {
    /// Capability adapters for permission checking
    pub adapters: Option<CapabilityAdapters>,

    /// Raw capabilities (for limit checks like max_processes)
    pub capabilities: Option<Arc<Capabilities>>,

    /// IPC channels (for ext_ipc) - currently consumed manually
    pub ipc: Option<IpcChannels>,

    /// Window channels (for ext_window) - currently consumed manually
    pub window: Option<WindowChannels>,

    /// App information (for ext_app, ext_storage)
    pub app_info: Option<AppInfo>,

    /// Whether running in dev mode
    pub dev_mode: bool,
}

// ============================================================================
// Error Types
// ============================================================================

/// Errors during extension initialization
#[derive(Debug, thiserror::Error)]
pub enum InitError {
    #[error("Missing required context for extension '{extension}': {field}")]
    MissingContext { extension: String, field: String },

    #[error("Initialization failed for extension '{extension}': {reason}")]
    Failed { extension: String, reason: String },
}

// ============================================================================
// Extension Registry Implementation
// ============================================================================

/// Central registry for all Forge runtime extensions
pub struct ExtensionRegistry {
    descriptors: Vec<ExtensionDescriptor>,
}

impl ExtensionRegistry {
    /// Create a new registry with all available extensions
    pub fn new() -> Self {
        Self {
            descriptors: create_all_descriptors(),
        }
    }

    /// Get all extension descriptors
    #[allow(dead_code)]
    pub fn all(&self) -> &[ExtensionDescriptor] {
        &self.descriptors
    }

    /// Get extensions matching a specific tier
    pub fn by_tier(&self, tier: ExtensionTier) -> Vec<&ExtensionDescriptor> {
        self.descriptors.iter().filter(|d| d.tier == tier).collect()
    }

    /// Get all required extensions
    #[allow(dead_code)]
    pub fn required(&self) -> Vec<&ExtensionDescriptor> {
        self.descriptors.iter().filter(|d| d.required).collect()
    }

    /// Get extension count
    pub fn count(&self) -> usize {
        self.descriptors.len()
    }

    /// Build the extensions vector for JsRuntime
    ///
    /// If `enabled` is None, all extensions are included.
    /// If `enabled` is Some, only required extensions and those in the list are included.
    pub fn build_extensions(&self, enabled: Option<&[&str]>) -> Vec<Extension> {
        self.descriptors
            .iter()
            .filter(|d| d.required || enabled.map(|e| e.contains(&d.name)).unwrap_or(true))
            .map(|d| (d.extension_fn)())
            .collect()
    }

    /// Initialize all extension states in the correct order
    ///
    /// State initialization happens in tier order:
    /// 1. Simple state (no dependencies)
    /// 2. Capability-based state (needs adapters)
    /// 3. Complex state (needs channels, app info, etc.)
    ///
    /// Note: Some complex extensions (ipc, window) require manual initialization
    /// via separate functions and will be skipped here. Any errors from these
    /// expected skips are returned but don't prevent other extensions from initializing.
    pub fn init_all_states(
        &self,
        state: &mut OpState,
        ctx: &ExtensionInitContext,
    ) -> Result<(), InitError> {
        // Tier 1: Simple state
        for desc in self.by_tier(ExtensionTier::SimpleState) {
            init_simple_state(desc.name, state)?;
        }

        // Tier 2: Capability-based state
        for desc in self.by_tier(ExtensionTier::CapabilityBased) {
            init_capability_state(desc.name, state, ctx)?;
        }

        // Tier 3: Complex state - continue even if some fail (ipc/window need manual init)
        let mut first_error: Option<InitError> = None;
        for desc in self.by_tier(ExtensionTier::ComplexContext) {
            tracing::debug!("Initializing complex state for: {}", desc.name);
            match init_complex_state(desc.name, state, ctx) {
                Ok(()) => tracing::debug!("Successfully initialized: {}", desc.name),
                Err(e) => {
                    tracing::debug!("Failed to initialize {}: {}", desc.name, e);
                    // Store the first error but continue initializing other extensions
                    if first_error.is_none() {
                        first_error = Some(e);
                    }
                }
            }
        }

        // Return the first error (typically from ipc/window needing manual init)
        if let Some(e) = first_error {
            return Err(e);
        }

        Ok(())
    }
}

impl Default for ExtensionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Static Extension Registration
// ============================================================================

/// Create all extension descriptors
fn create_all_descriptors() -> Vec<ExtensionDescriptor> {
    vec![
        // =====================================================================
        // Tier -1: Core Polyfills (MUST be first - sets up globals)
        // =====================================================================
        ExtensionDescriptor {
            name: "encoding",
            specifier: "runtime:encoding",
            tier: ExtensionTier::ExtensionOnly,
            extension_fn: ext_encoding::encoding_extension,
            required: true, // Required - provides TextEncoder/TextDecoder globals
        },
        // =====================================================================
        // Tier 0: Extension Only (no state initialization)
        // =====================================================================
        ExtensionDescriptor {
            name: "database",
            specifier: "runtime:database",
            tier: ExtensionTier::ComplexContext,
            extension_fn: ext_database::database_extension,
            required: false,
        },
        // debugger moved to ComplexContext (needs state for breakpoints, events)
        ExtensionDescriptor {
            name: "devtools",
            specifier: "runtime:devtools",
            tier: ExtensionTier::ExtensionOnly,
            extension_fn: ext_devtools::devtools_extension,
            required: false,
        },
        ExtensionDescriptor {
            name: "display",
            specifier: "runtime:display",
            tier: ExtensionTier::SimpleState,
            extension_fn: ext_display::display_extension,
            required: false,
        },
        ExtensionDescriptor {
            name: "lock",
            specifier: "runtime:lock",
            tier: ExtensionTier::ExtensionOnly,
            extension_fn: ext_lock::lock_extension,
            required: false,
        },
        ExtensionDescriptor {
            name: "log",
            specifier: "runtime:log",
            tier: ExtensionTier::ExtensionOnly,
            extension_fn: ext_log::log_extension,
            required: false,
        },
        // =====================================================================
        // Tier 1: Simple State (no external dependencies) - moved from Tier 0
        // =====================================================================
        ExtensionDescriptor {
            name: "monitor",
            specifier: "runtime:monitor",
            tier: ExtensionTier::SimpleState,
            extension_fn: ext_monitor::monitor_extension,
            required: false,
        },
        ExtensionDescriptor {
            name: "os_compat",
            specifier: "runtime:os_compat",
            tier: ExtensionTier::ExtensionOnly,
            extension_fn: ext_os_compat::os_compat_extension,
            required: false,
        },
        ExtensionDescriptor {
            name: "path",
            specifier: "runtime:path",
            tier: ExtensionTier::ExtensionOnly,
            extension_fn: ext_path::path_extension,
            required: false,
        },
        // protocol moved to ComplexContext (needs state for invocation events)
        ExtensionDescriptor {
            name: "shortcuts",
            specifier: "runtime:shortcuts",
            tier: ExtensionTier::ComplexContext,
            extension_fn: ext_shortcuts::shortcuts_extension,
            required: false,
        },
        ExtensionDescriptor {
            name: "signals",
            specifier: "runtime:signals",
            tier: ExtensionTier::ExtensionOnly,
            extension_fn: ext_signals::signals_extension,
            required: false,
        },
        ExtensionDescriptor {
            name: "updater",
            specifier: "runtime:updater",
            tier: ExtensionTier::SimpleState,
            extension_fn: ext_updater::updater_extension,
            required: false,
        },
        ExtensionDescriptor {
            name: "webview",
            specifier: "runtime:webview",
            tier: ExtensionTier::ExtensionOnly,
            extension_fn: ext_webview::webview_extension,
            required: false,
        },
        // =====================================================================
        // Tier 1: Simple State (no external dependencies)
        // =====================================================================
        ExtensionDescriptor {
            name: "timers",
            specifier: "runtime:timers",
            tier: ExtensionTier::SimpleState,
            extension_fn: ext_timers::timers_extension,
            required: false,
        },
        ExtensionDescriptor {
            name: "trace",
            specifier: "runtime:trace",
            tier: ExtensionTier::SimpleState,
            extension_fn: ext_trace::trace_extension,
            required: false,
        },
        ExtensionDescriptor {
            name: "weld",
            specifier: "forge:weld",
            tier: ExtensionTier::SimpleState,
            extension_fn: ext_weld::weld_extension,
            required: false,
        },
        ExtensionDescriptor {
            name: "bundler",
            specifier: "forge:bundler",
            tier: ExtensionTier::SimpleState,
            extension_fn: ext_bundler::bundler_extension,
            required: false,
        },
        ExtensionDescriptor {
            name: "etcher",
            specifier: "forge:etcher",
            tier: ExtensionTier::SimpleState,
            extension_fn: ext_etcher::etcher_extension,
            required: false,
        },
        ExtensionDescriptor {
            name: "svelte",
            specifier: "runtime:svelte",
            tier: ExtensionTier::SimpleState,
            extension_fn: ext_svelte::svelte_extension,
            required: false,
        },
        ExtensionDescriptor {
            name: "dock",
            specifier: "runtime:dock",
            tier: ExtensionTier::SimpleState,
            extension_fn: ext_dock::dock_extension,
            required: false,
        },
        ExtensionDescriptor {
            name: "image_tools",
            specifier: "runtime:image_tools",
            tier: ExtensionTier::SimpleState,
            extension_fn: ext_image_tools::image_tools_extension,
            required: false,
        },
        // =====================================================================
        // Tier 2: Capability-Based State (needs adapters)
        // =====================================================================
        ExtensionDescriptor {
            name: "fs",
            specifier: "runtime:fs",
            tier: ExtensionTier::CapabilityBased,
            extension_fn: ext_fs::fs_extension,
            required: true,
        },
        ExtensionDescriptor {
            name: "net",
            specifier: "runtime:net",
            tier: ExtensionTier::CapabilityBased,
            extension_fn: ext_net::net_extension,
            required: false,
        },
        ExtensionDescriptor {
            name: "sys",
            specifier: "runtime:sys",
            tier: ExtensionTier::CapabilityBased,
            extension_fn: ext_sys::sys_extension,
            required: false,
        },
        ExtensionDescriptor {
            name: "crypto",
            specifier: "runtime:crypto",
            tier: ExtensionTier::CapabilityBased,
            extension_fn: ext_crypto::crypto_extension,
            required: false,
        },
        ExtensionDescriptor {
            name: "storage",
            specifier: "runtime:storage",
            tier: ExtensionTier::CapabilityBased,
            extension_fn: ext_storage::storage_extension,
            required: false,
        },
        ExtensionDescriptor {
            name: "codesign",
            specifier: "runtime:codesign",
            tier: ExtensionTier::CapabilityBased,
            extension_fn: ext_codesign::codesign_extension,
            required: false,
        },
        // =====================================================================
        // Tier 3: Complex Context (channels, app info, etc.)
        // =====================================================================
        ExtensionDescriptor {
            name: "ipc",
            specifier: "runtime:ipc",
            tier: ExtensionTier::ComplexContext,
            extension_fn: ext_ipc::ipc_extension,
            required: true,
        },
        ExtensionDescriptor {
            name: "window",
            specifier: "runtime:window",
            tier: ExtensionTier::ComplexContext,
            extension_fn: ext_window::window_extension,
            required: true,
        },
        ExtensionDescriptor {
            name: "process",
            specifier: "runtime:process",
            tier: ExtensionTier::ComplexContext,
            extension_fn: ext_process::process_extension,
            required: false,
        },
        ExtensionDescriptor {
            name: "wasm",
            specifier: "runtime:wasm",
            tier: ExtensionTier::ComplexContext,
            extension_fn: ext_wasm::wasm_extension,
            required: false,
        },
        ExtensionDescriptor {
            name: "app",
            specifier: "runtime:app",
            tier: ExtensionTier::ComplexContext,
            extension_fn: ext_app::app_extension,
            required: true,
        },
        ExtensionDescriptor {
            name: "shell",
            specifier: "runtime:shell",
            tier: ExtensionTier::ComplexContext,
            extension_fn: ext_shell::shell_extension,
            required: false,
        },
        ExtensionDescriptor {
            name: "debugger",
            specifier: "runtime:debugger",
            tier: ExtensionTier::ComplexContext,
            extension_fn: ext_debugger::debugger_extension,
            required: false,
        },
        ExtensionDescriptor {
            name: "protocol",
            specifier: "runtime:protocol",
            tier: ExtensionTier::ComplexContext,
            extension_fn: ext_protocol::protocol_extension,
            required: false,
        },
        ExtensionDescriptor {
            name: "web_inspector",
            specifier: "runtime:web_inspector",
            tier: ExtensionTier::SimpleState,
            extension_fn: ext_web_inspector::web_inspector_extension,
            required: false,
        },
    ]
}

// ============================================================================
// State Initialization Dispatchers
// ============================================================================

/// Initialize simple state extensions (Tier 1)
fn init_simple_state(name: &str, state: &mut OpState) -> Result<(), InitError> {
    match name {
        "timers" => {
            ext_timers::init_timer_state(state);
        }
        "trace" => {
            ext_trace::init_trace_state(state);
        }
        "weld" => {
            ext_weld::init_weld_state(state);
        }
        "bundler" => {
            ext_bundler::init_bundler_state(state);
        }
        "etcher" => {
            ext_etcher::init_etcher_state(state);
        }
        "svelte" => {
            ext_svelte::init_svelte_state(state);
        }
        "dock" => {
            ext_dock::init_dock_state(state);
        }
        "image_tools" => {
            ext_image_tools::init_image_tools_state(state);
        }
        "monitor" => {
            ext_monitor::init_monitor_state(state);
        }
        "display" => {
            ext_display::init_display_state(state);
        }
        "web_inspector" => {
            ext_web_inspector::init_web_inspector_state(state);
        }
        "updater" => {
            ext_updater::init_updater_state(state);
        }
        _ => {
            return Err(InitError::Failed {
                extension: name.to_string(),
                reason: format!("Unknown simple state extension: {}", name),
            });
        }
    }
    Ok(())
}

/// Initialize capability-based state extensions (Tier 2)
fn init_capability_state(
    name: &str,
    state: &mut OpState,
    ctx: &ExtensionInitContext,
) -> Result<(), InitError> {
    let adapters = ctx.adapters.as_ref();

    match name {
        "fs" => {
            ext_fs::init_fs_state(state, adapters.map(|a| a.fs.clone()));
        }
        "net" => {
            ext_net::init_net_state(state, adapters.map(|a| a.net.clone()));
        }
        "sys" => {
            ext_sys::init_sys_state(state, adapters.map(|a| a.sys.clone()));
        }
        "crypto" => {
            // Crypto has no capability checker - all ops are safe
            ext_crypto::init_crypto_state(state, None);
        }
        "storage" => {
            let app_id = ctx
                .app_info
                .as_ref()
                .map(|a| a.identifier.clone())
                .unwrap_or_else(|| "forge-app".to_string());
            ext_storage::init_storage_state(state, app_id, None);
        }
        "codesign" => {
            ext_codesign::init_codesign_state(state, adapters.map(|a| a.codesign.clone()));
        }
        _ => {
            return Err(InitError::Failed {
                extension: name.to_string(),
                reason: format!("Unknown capability state extension: {}", name),
            });
        }
    }
    Ok(())
}

/// Initialize complex context state extensions (Tier 3)
fn init_complex_state(
    name: &str,
    state: &mut OpState,
    ctx: &ExtensionInitContext,
) -> Result<(), InitError> {
    match name {
        "ipc" => {
            // IPC requires consuming channel receivers which can't be done through the generic registry.
            // Use ext_registry::init_ipc_manually() instead.
            let _ipc = ctx.ipc.as_ref().ok_or_else(|| InitError::MissingContext {
                extension: "ipc".to_string(),
                field: "ipc channels".to_string(),
            })?;
            return Err(InitError::Failed {
                extension: "ipc".to_string(),
                reason: "IPC requires channels that must be consumed; use init_ipc_manually()"
                    .to_string(),
            });
        }
        "window" => {
            // Window also requires consuming channel receivers
            return Err(InitError::Failed {
                extension: "window".to_string(),
                reason: "Window requires channels that must be consumed; initialize separately"
                    .to_string(),
            });
        }
        "process" => {
            let max = ctx
                .capabilities
                .as_ref()
                .map(|c| c.get_max_processes())
                .unwrap_or(10);
            let checker = ctx.adapters.as_ref().map(|a| a.process.clone());
            ext_process::init_process_state(state, checker, Some(max));
        }
        "wasm" => {
            let max = ctx
                .capabilities
                .as_ref()
                .map(|c| c.get_max_wasm_instances())
                .unwrap_or(10);
            let checker = ctx.adapters.as_ref().map(|a| a.wasm.clone());
            ext_wasm::init_wasm_state(state, checker, Some(max));
        }
        "app" => {
            let app_info = ctx
                .app_info
                .clone()
                .ok_or_else(|| InitError::MissingContext {
                    extension: "app".to_string(),
                    field: "app_info".to_string(),
                })?;
            ext_app::init_app_state::<ext_app::DefaultAppCapabilityChecker>(
                state, app_info, None, None,
            );
        }
        "shell" => {
            ext_shell::init_shell_state::<ext_shell::DefaultShellCapabilityChecker>(state, None);
        }
        "debugger" => {
            // Initialize debugger state with broadcast channels for events
            ext_debugger::init_debugger_state(state);
        }
        "protocol" => {
            // Initialize protocol state with launch URL from command line args and event channel
            let launch_url = std::env::args()
                .find(|arg| {
                    // Check if arg looks like a custom protocol URL (scheme://...)
                    // Skip common URLs like http, https, file
                    arg.contains("://")
                        && !arg.starts_with("http://")
                        && !arg.starts_with("https://")
                        && !arg.starts_with("file://")
                })
                .map(|s| s.to_string());

            // Get app info from context for protocol registration
            let (app_id, app_name) = ctx
                .app_info
                .as_ref()
                .map(|info| (info.identifier.clone(), info.name.clone()))
                .unwrap_or_else(|| ("forge-app".to_string(), "Forge App".to_string()));

            let exe_path = std::env::current_exe()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();

            ext_protocol::init_protocol_state(state, app_id, app_name, exe_path, launch_url, None);
        }
        "database" => {
            // Initialize database state with app identifier for isolated storage
            let app_id = ctx
                .app_info
                .as_ref()
                .map(|a| a.identifier.clone())
                .unwrap_or_else(|| "forge-app".to_string());
            ext_database::init_database_state(state, app_id, None, None);
        }
        "shortcuts" => {
            // Initialize shortcuts state with app identifier for persistence
            let app_id = ctx
                .app_info
                .as_ref()
                .map(|a| a.identifier.clone())
                .unwrap_or_else(|| "forge-app".to_string());
            ext_shortcuts::init_shortcuts_state(state, app_id);
        }
        _ => {
            return Err(InitError::Failed {
                extension: name.to_string(),
                reason: format!("Unknown complex state extension: {}", name),
            });
        }
    }
    Ok(())
}

// ============================================================================
// Manual initialization helpers for extensions that need consumed channels
// ============================================================================

/// Initialize IPC state manually (requires consuming the channel receiver)
pub fn init_ipc_manually(
    state: &mut OpState,
    to_renderer_tx: mpsc::Sender<ToRendererCmd>,
    to_deno_rx: mpsc::Receiver<IpcEvent>,
    adapters: Option<&CapabilityAdapters>,
) {
    ext_ipc::init_ipc_state(state, to_renderer_tx, to_deno_rx);
    if let Some(adapters) = adapters {
        ext_ipc::init_ipc_capabilities(state, Some(adapters.ipc.clone()));
    }
}

/// Initialize Window state manually (requires consuming channel receivers)
pub fn init_window_manually(
    state: &mut OpState,
    cmd_tx: mpsc::Sender<WindowCmd>,
    events_rx: mpsc::Receiver<WindowSystemEvent>,
    menu_events_rx: mpsc::Receiver<WinMenuEvent>,
    adapters: Option<&CapabilityAdapters>,
) {
    ext_window::init_window_state(state, cmd_tx, events_rx, menu_events_rx);
    if let Some(adapters) = adapters {
        ext_window::init_window_capabilities(state, Some(adapters.window.clone()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = ExtensionRegistry::new();
        // Should have 30 extensions registered (all ext_* crates with implemented extensions)
        assert!(
            registry.count() >= 30,
            "Expected at least 30 extensions, got {}",
            registry.count()
        );
    }

    #[test]
    fn test_tier_filtering() {
        let registry = ExtensionRegistry::new();

        let tier0 = registry.by_tier(ExtensionTier::ExtensionOnly);
        let tier1 = registry.by_tier(ExtensionTier::SimpleState);
        let tier2 = registry.by_tier(ExtensionTier::CapabilityBased);
        let tier3 = registry.by_tier(ExtensionTier::ComplexContext);

        assert!(!tier0.is_empty(), "Should have Tier 0 extensions");
        assert!(!tier1.is_empty(), "Should have Tier 1 extensions");
        assert!(!tier2.is_empty(), "Should have Tier 2 extensions");
        assert!(!tier3.is_empty(), "Should have Tier 3 extensions");

        // Verify total
        assert_eq!(
            tier0.len() + tier1.len() + tier2.len() + tier3.len(),
            registry.count()
        );
    }

    #[test]
    fn test_required_extensions() {
        let registry = ExtensionRegistry::new();
        let required = registry.required();

        // Should include fs, ipc, window, app as required
        let required_names: Vec<&str> = required.iter().map(|d| d.name).collect();
        assert!(required_names.contains(&"fs"), "fs should be required");
        assert!(required_names.contains(&"ipc"), "ipc should be required");
        assert!(
            required_names.contains(&"window"),
            "window should be required"
        );
        assert!(required_names.contains(&"app"), "app should be required");
    }

    #[test]
    fn test_build_extensions() {
        let registry = ExtensionRegistry::new();

        // Build all extensions
        let all = registry.build_extensions(None);
        assert_eq!(all.len(), registry.count());

        // Build only specific extensions (required + enabled)
        let enabled = registry.build_extensions(Some(&["log", "crypto"]));
        // Should have required + log + crypto
        assert!(
            enabled.len() >= 4,
            "Should have at least required extensions"
        );
    }
}
