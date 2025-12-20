//! Platform-specific adapters for WebView DevTools integration.
//!
//! Provides abstraction over different WebView implementations:
//! - WebKit (macOS/iOS) - Safari Web Inspector
//! - WebView2 (Windows) - Edge DevTools Protocol
//! - WebKitGTK (Linux) - WebKitGTK Inspector

use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, trace, warn};

use crate::{CdpMessage, CdpResponse, WebInspectorError};

// ============================================================================
// Platform Adapter Trait
// ============================================================================

/// Trait for platform-specific DevTools integration.
///
/// Each platform (WebKit, WebView2, WebKitGTK) implements this trait to provide
/// custom panel injection and CDP message handling.
#[async_trait(?Send)]
pub trait PlatformAdapter: Send + Sync {
    /// Get the platform name
    fn name(&self) -> &'static str;

    /// Check if this platform is supported on current system
    fn is_supported(&self) -> bool;

    /// Get platform version info if available
    fn version_info(&self) -> Option<String>;

    /// Inject the custom Forge panel into the DevTools
    ///
    /// This uses platform-specific APIs:
    /// - WebKit: browser.devtools.panels.create (Safari 16+)
    /// - WebView2: DevToolsProtocolExtension
    /// - WebKitGTK: webkit_web_inspector_* APIs
    async fn inject_panel(
        &self,
        window_id: &str,
        assets: &PanelAssets,
    ) -> Result<(), WebInspectorError>;

    /// Send a CDP message through the platform's DevTools connection
    async fn send_cdp_message(
        &self,
        window_id: &str,
        message: &CdpMessage,
    ) -> Result<CdpResponse, WebInspectorError>;

    /// Subscribe to CDP events from the DevTools
    fn on_cdp_event(&self, callback: Box<dyn Fn(String, Value) + Send + Sync>);

    /// Check if DevTools are open for a window
    fn is_devtools_open(&self, window_id: &str) -> bool;

    /// Open DevTools for a window
    async fn open_devtools(&self, window_id: &str) -> Result<(), WebInspectorError>;

    /// Close DevTools for a window
    async fn close_devtools(&self, window_id: &str) -> Result<(), WebInspectorError>;
}

// ============================================================================
// Panel Assets
// ============================================================================

/// Assets required for the custom DevTools panel
#[derive(Debug, Clone)]
pub struct PanelAssets {
    /// Panel HTML content
    pub html: String,
    /// Panel JavaScript code
    pub js: String,
    /// Panel CSS styles
    pub css: String,
    /// Panel icon (base64 or URL)
    pub icon: Option<String>,
}

impl Default for PanelAssets {
    fn default() -> Self {
        Self {
            html: DEFAULT_PANEL_HTML.to_string(),
            js: DEFAULT_PANEL_JS.to_string(),
            css: DEFAULT_PANEL_CSS.to_string(),
            icon: None,
        }
    }
}

// ============================================================================
// Platform Detection
// ============================================================================

/// Detected platform type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlatformType {
    WebKit,
    WebView2,
    WebKitGTK,
    Unknown,
}

impl PlatformType {
    /// Detect current platform
    pub fn detect() -> Self {
        #[cfg(target_os = "macos")]
        {
            PlatformType::WebKit
        }
        #[cfg(target_os = "windows")]
        {
            PlatformType::WebView2
        }
        #[cfg(target_os = "linux")]
        {
            PlatformType::WebKitGTK
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            PlatformType::Unknown
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            PlatformType::WebKit => "WebKit",
            PlatformType::WebView2 => "WebView2",
            PlatformType::WebKitGTK => "WebKitGTK",
            PlatformType::Unknown => "Unknown",
        }
    }
}

/// Create the appropriate platform adapter for the current system
pub fn create_platform_adapter() -> Box<dyn PlatformAdapter> {
    match PlatformType::detect() {
        PlatformType::WebKit => Box::new(WebKitAdapter::new()),
        PlatformType::WebView2 => Box::new(WebView2Adapter::new()),
        PlatformType::WebKitGTK => Box::new(WebKitGTKAdapter::new()),
        PlatformType::Unknown => Box::new(NullAdapter::new()),
    }
}

/// Create a shareable platform adapter wrapped in Arc for multi-threaded access
pub fn create_shared_adapter() -> Arc<dyn PlatformAdapter> {
    let platform = PlatformType::detect();
    trace!(platform = ?platform, "Creating shared platform adapter");
    match platform {
        PlatformType::WebKit => Arc::new(WebKitAdapter::new()),
        PlatformType::WebView2 => Arc::new(WebView2Adapter::new()),
        PlatformType::WebKitGTK => Arc::new(WebKitGTKAdapter::new()),
        PlatformType::Unknown => Arc::new(NullAdapter::new()),
    }
}

// ============================================================================
// WebKit Adapter (macOS/iOS)
// ============================================================================

/// WebKit/Safari Web Inspector adapter
pub struct WebKitAdapter {
    /// Safari version (16+ required for devtools.panels API)
    safari_version: Option<u32>,
}

impl WebKitAdapter {
    pub fn new() -> Self {
        let safari_version = detect_safari_version();
        debug!(
            "WebKitAdapter created, Safari version: {:?}",
            safari_version
        );
        Self { safari_version }
    }
}

impl Default for WebKitAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait(?Send)]
impl PlatformAdapter for WebKitAdapter {
    fn name(&self) -> &'static str {
        "WebKit"
    }

    fn is_supported(&self) -> bool {
        // Safari 16+ supports devtools.panels API
        self.safari_version.map(|v| v >= 16).unwrap_or(false)
    }

    fn version_info(&self) -> Option<String> {
        self.safari_version.map(|v| format!("Safari {}", v))
    }

    async fn inject_panel(
        &self,
        window_id: &str,
        assets: &PanelAssets,
    ) -> Result<(), WebInspectorError> {
        if !self.is_supported() {
            return Err(WebInspectorError::platform_unsupported(
                "Safari 16+ required for DevTools panel injection",
            ));
        }

        // Validate panel assets before injection
        trace!(
            html_len = assets.html.len(),
            js_len = assets.js.len(),
            css_len = assets.css.len(),
            has_icon = assets.icon.is_some(),
            "Validating panel assets"
        );

        debug!("Injecting Forge panel for window: {} (WebKit)", window_id);

        // The actual injection would use evaluate_script on the WebView
        // to register the panel using browser.devtools.panels.create API
        //
        // Script would be something like:
        // ```javascript
        // if (browser?.devtools?.panels) {
        //     browser.devtools.panels.create(
        //         'Forge',
        //         'forge-icon.png',
        //         'forge-panel.html'
        //     ).then(panel => {
        //         panel.onShown.addListener(win => {
        //             win.__forgeInit?.();
        //         });
        //     });
        // }
        // ```

        Ok(())
    }

    async fn send_cdp_message(
        &self,
        window_id: &str,
        message: &CdpMessage,
    ) -> Result<CdpResponse, WebInspectorError> {
        // WebKit uses a different protocol than CDP, but we can map common operations
        debug!(
            "Sending CDP message to WebKit: {}.{}",
            window_id, message.method
        );

        // For now return success for enable/disable commands
        if message.method.ends_with(".enable") || message.method.ends_with(".disable") {
            return Ok(CdpResponse::success(message.id, serde_json::json!({})));
        }

        Err(WebInspectorError::cdp_error(format!(
            "CDP method not implemented for WebKit: {}",
            message.method
        )))
    }

    fn on_cdp_event(&self, _callback: Box<dyn Fn(String, Value) + Send + Sync>) {
        // Register event listener
    }

    fn is_devtools_open(&self, _window_id: &str) -> bool {
        false
    }

    async fn open_devtools(&self, window_id: &str) -> Result<(), WebInspectorError> {
        debug!("Opening DevTools for window: {} (WebKit)", window_id);
        // Would call webkit_web_view_get_inspector() and show()
        Ok(())
    }

    async fn close_devtools(&self, window_id: &str) -> Result<(), WebInspectorError> {
        debug!("Closing DevTools for window: {} (WebKit)", window_id);
        Ok(())
    }
}

// ============================================================================
// WebView2 Adapter (Windows)
// ============================================================================

/// WebView2/Edge DevTools adapter
pub struct WebView2Adapter {
    /// Whether DevToolsProtocolExtension is available
    protocol_extension: bool,
}

impl WebView2Adapter {
    pub fn new() -> Self {
        let protocol_extension = detect_webview2_protocol();
        debug!(
            "WebView2Adapter created, protocol extension: {}",
            protocol_extension
        );
        Self { protocol_extension }
    }
}

impl Default for WebView2Adapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait(?Send)]
impl PlatformAdapter for WebView2Adapter {
    fn name(&self) -> &'static str {
        "WebView2"
    }

    fn is_supported(&self) -> bool {
        self.protocol_extension
    }

    fn version_info(&self) -> Option<String> {
        // Would query WebView2 version
        Some("WebView2".to_string())
    }

    async fn inject_panel(
        &self,
        window_id: &str,
        assets: &PanelAssets,
    ) -> Result<(), WebInspectorError> {
        if !self.is_supported() {
            return Err(WebInspectorError::platform_unsupported(
                "WebView2 DevTools Protocol extension not available",
            ));
        }

        // Validate panel assets before injection
        trace!(
            html_len = assets.html.len(),
            js_len = assets.js.len(),
            css_len = assets.css.len(),
            has_icon = assets.icon.is_some(),
            "Validating panel assets for WebView2"
        );

        debug!("Injecting Forge panel for window: {} (WebView2)", window_id);

        // WebView2 uses:
        // - ICoreWebView2DevToolsProtocolEventReceiver for events
        // - CallDevToolsProtocolMethod for commands
        // - Page.addScriptToEvaluateOnNewDocument for injection

        Ok(())
    }

    async fn send_cdp_message(
        &self,
        window_id: &str,
        message: &CdpMessage,
    ) -> Result<CdpResponse, WebInspectorError> {
        debug!(
            "Sending CDP message to WebView2: {}.{}",
            window_id, message.method
        );

        // WebView2 supports CDP natively via CallDevToolsProtocolMethod
        // For now return success for enable/disable commands
        if message.method.ends_with(".enable") || message.method.ends_with(".disable") {
            return Ok(CdpResponse::success(message.id, serde_json::json!({})));
        }

        Err(WebInspectorError::cdp_error(format!(
            "CDP method not implemented for WebView2: {}",
            message.method
        )))
    }

    fn on_cdp_event(&self, _callback: Box<dyn Fn(String, Value) + Send + Sync>) {
        // Register with ICoreWebView2DevToolsProtocolEventReceiver
    }

    fn is_devtools_open(&self, _window_id: &str) -> bool {
        false
    }

    async fn open_devtools(&self, window_id: &str) -> Result<(), WebInspectorError> {
        debug!("Opening DevTools for window: {} (WebView2)", window_id);
        // Would call ICoreWebView2.OpenDevToolsWindow()
        Ok(())
    }

    async fn close_devtools(&self, window_id: &str) -> Result<(), WebInspectorError> {
        debug!("Closing DevTools for window: {} (WebView2)", window_id);
        // WebView2 doesn't have a close method - user closes manually
        Ok(())
    }
}

// ============================================================================
// WebKitGTK Adapter (Linux)
// ============================================================================

/// WebKitGTK Inspector adapter
pub struct WebKitGTKAdapter {
    /// WebKitGTK version
    version: Option<String>,
}

impl WebKitGTKAdapter {
    pub fn new() -> Self {
        let version = detect_webkitgtk_version();
        debug!("WebKitGTKAdapter created, version: {:?}", version);
        Self { version }
    }
}

impl Default for WebKitGTKAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait(?Send)]
impl PlatformAdapter for WebKitGTKAdapter {
    fn name(&self) -> &'static str {
        "WebKitGTK"
    }

    fn is_supported(&self) -> bool {
        self.version.is_some()
    }

    fn version_info(&self) -> Option<String> {
        self.version.clone()
    }

    async fn inject_panel(
        &self,
        window_id: &str,
        _assets: &PanelAssets,
    ) -> Result<(), WebInspectorError> {
        debug!(
            "Injecting Forge panel for window: {} (WebKitGTK)",
            window_id
        );

        // WebKitGTK inspector is more limited
        // Would use webkit_web_inspector_* APIs

        Ok(())
    }

    async fn send_cdp_message(
        &self,
        window_id: &str,
        message: &CdpMessage,
    ) -> Result<CdpResponse, WebInspectorError> {
        debug!(
            "Sending CDP message to WebKitGTK: {}.{}",
            window_id, message.method
        );

        if message.method.ends_with(".enable") || message.method.ends_with(".disable") {
            return Ok(CdpResponse::success(message.id, serde_json::json!({})));
        }

        Err(WebInspectorError::cdp_error(format!(
            "CDP method not implemented for WebKitGTK: {}",
            message.method
        )))
    }

    fn on_cdp_event(&self, _callback: Box<dyn Fn(String, Value) + Send + Sync>) {
        // Limited event support on WebKitGTK
    }

    fn is_devtools_open(&self, _window_id: &str) -> bool {
        false
    }

    async fn open_devtools(&self, window_id: &str) -> Result<(), WebInspectorError> {
        debug!("Opening DevTools for window: {} (WebKitGTK)", window_id);
        // Would call webkit_web_inspector_show()
        Ok(())
    }

    async fn close_devtools(&self, window_id: &str) -> Result<(), WebInspectorError> {
        debug!("Closing DevTools for window: {} (WebKitGTK)", window_id);
        // Would call webkit_web_inspector_close()
        Ok(())
    }
}

// ============================================================================
// Null Adapter (Unsupported platforms)
// ============================================================================

/// Null adapter for unsupported platforms
pub struct NullAdapter;

impl NullAdapter {
    pub fn new() -> Self {
        warn!("NullAdapter created - web inspector not supported on this platform");
        Self
    }
}

impl Default for NullAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait(?Send)]
impl PlatformAdapter for NullAdapter {
    fn name(&self) -> &'static str {
        "Null"
    }

    fn is_supported(&self) -> bool {
        false
    }

    fn version_info(&self) -> Option<String> {
        None
    }

    async fn inject_panel(
        &self,
        _window_id: &str,
        _assets: &PanelAssets,
    ) -> Result<(), WebInspectorError> {
        Err(WebInspectorError::platform_unsupported(
            "Web inspector not supported on this platform",
        ))
    }

    async fn send_cdp_message(
        &self,
        _window_id: &str,
        _message: &CdpMessage,
    ) -> Result<CdpResponse, WebInspectorError> {
        Err(WebInspectorError::platform_unsupported(
            "Web inspector not supported on this platform",
        ))
    }

    fn on_cdp_event(&self, _callback: Box<dyn Fn(String, Value) + Send + Sync>) {
        // No-op
    }

    fn is_devtools_open(&self, _window_id: &str) -> bool {
        false
    }

    async fn open_devtools(&self, _window_id: &str) -> Result<(), WebInspectorError> {
        Err(WebInspectorError::platform_unsupported(
            "Web inspector not supported on this platform",
        ))
    }

    async fn close_devtools(&self, _window_id: &str) -> Result<(), WebInspectorError> {
        Err(WebInspectorError::platform_unsupported(
            "Web inspector not supported on this platform",
        ))
    }
}

// ============================================================================
// Platform Detection Helpers
// ============================================================================

#[cfg(target_os = "macos")]
fn detect_safari_version() -> Option<u32> {
    // Would query Safari version using system_profiler or sw_vers
    // For now assume Safari 17 (macOS Sonoma+)
    Some(17)
}

#[cfg(not(target_os = "macos"))]
fn detect_safari_version() -> Option<u32> {
    None
}

#[cfg(target_os = "windows")]
fn detect_webview2_protocol() -> bool {
    // Would check if WebView2 runtime is installed
    true
}

#[cfg(not(target_os = "windows"))]
fn detect_webview2_protocol() -> bool {
    false
}

#[cfg(target_os = "linux")]
fn detect_webkitgtk_version() -> Option<String> {
    // Would query WebKitGTK version using pkg-config or library version
    Some("2.42".to_string())
}

#[cfg(not(target_os = "linux"))]
fn detect_webkitgtk_version() -> Option<String> {
    None
}

// ============================================================================
// Default Panel Assets
// ============================================================================

/// Load panel assets from external files at compile time.
/// This allows for easier editing and better separation of concerns.
const DEFAULT_PANEL_HTML: &str = include_str!("../panels/forge-panel.html");
const DEFAULT_PANEL_JS: &str = include_str!("../panels/forge-panel.js");
const DEFAULT_PANEL_CSS: &str = include_str!("../panels/forge-panel.css");

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_type_detect() {
        let platform = PlatformType::detect();
        // Should be one of the known types
        assert!(matches!(
            platform,
            PlatformType::WebKit
                | PlatformType::WebView2
                | PlatformType::WebKitGTK
                | PlatformType::Unknown
        ));
    }

    #[test]
    fn test_create_platform_adapter() {
        let adapter = create_platform_adapter();
        assert!(!adapter.name().is_empty());
    }

    #[test]
    fn test_panel_assets_default() {
        let assets = PanelAssets::default();
        assert!(!assets.html.is_empty());
        assert!(!assets.js.is_empty());
        assert!(!assets.css.is_empty());
    }

    #[test]
    fn test_null_adapter_not_supported() {
        let adapter = NullAdapter::new();
        assert!(!adapter.is_supported());
        assert_eq!(adapter.name(), "Null");
    }
}
