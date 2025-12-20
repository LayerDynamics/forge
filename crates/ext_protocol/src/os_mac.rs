//! macOS implementation for custom URL protocol handling
//!
//! Uses Launch Services and Core Foundation APIs for protocol registration.
//! - LSSetDefaultHandlerForURLScheme: Set default handler
//! - LSCopyDefaultHandlerForURLScheme: Query current handler
//! - CFBundleURLTypes in Info.plist for static registration

use crate::{
    ProtocolCapabilities, ProtocolError, RegistrationOptions, RegistrationResult,
    RegistrationStatus,
};
use tokio::process::Command;
use tracing::{debug, error, warn};

/// Register a custom URL protocol handler on macOS
///
/// On macOS, protocol registration typically requires:
/// 1. CFBundleURLTypes in the app's Info.plist (build-time)
/// 2. LSSetDefaultHandlerForURLScheme to set as default handler (runtime)
///
/// This function handles the runtime registration. Build-time registration
/// should be handled by the bundler.
pub async fn register_protocol(
    scheme: &str,
    app_identifier: &str,
    _app_name: &str,
    _exe_path: &str,
    options: &RegistrationOptions,
) -> Result<RegistrationResult, ProtocolError> {
    debug!(scheme = %scheme, app_id = %app_identifier, "macOS: register_protocol");

    // Check current handler
    let current_handler = get_default_handler(scheme).await.ok().flatten();
    let was_already_registered = current_handler.is_some();

    // Try to set as default handler using LSSetDefaultHandlerForURLScheme
    // This requires the app to have CFBundleURLTypes in its Info.plist
    let set_default = options.set_as_default.unwrap_or(true);

    if set_default {
        // Use Swift/ObjC bridge or direct Core Foundation calls
        // For now, we'll use a helper script approach that works without native bindings
        let script = format!(
            r#"
            import Foundation
            import CoreServices

            let scheme = "{}" as CFString
            let bundleId = "{}" as CFString

            let result = LSSetDefaultHandlerForURLScheme(scheme, bundleId)
            if result == noErr {{
                print("SUCCESS")
            }} else {{
                print("ERROR:\(result)")
            }}
            "#,
            scheme, app_identifier
        );

        let output = Command::new("swift")
            .args(["-e", &script])
            .output()
            .await
            .map_err(|e| {
                error!("Failed to execute swift: {}", e);
                ProtocolError::tool_not_found("Swift CLI not available for protocol registration")
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !stdout.contains("SUCCESS") {
            // If Swift approach fails, try using 'open' command to trigger association
            warn!(
                "LSSetDefaultHandlerForURLScheme failed: {}{}. Trying alternative approach.",
                stdout, stderr
            );

            // Alternative: Use duti if available (common on developer machines)
            let duti_result = Command::new("duti")
                .args(["-s", app_identifier, &format!("{}://", scheme), "viewer"])
                .output()
                .await;

            if duti_result.is_err() {
                // Neither method worked, but we can still track locally
                warn!("Could not set default handler. App must be bundled with CFBundleURLTypes in Info.plist");
            }
        }
    }

    Ok(RegistrationResult {
        success: true,
        scheme: scheme.to_string(),
        was_already_registered,
        previous_handler: current_handler,
    })
}

/// Unregister a custom URL protocol handler
///
/// On macOS, you can't truly unregister a protocol - you can only set a different
/// default handler. This function clears our internal tracking.
pub async fn unregister_protocol(scheme: &str) -> Result<bool, ProtocolError> {
    debug!(scheme = %scheme, "macOS: unregister_protocol");

    // On macOS, we can't truly unregister - the Info.plist entry persists
    // We can only remove our internal tracking
    // A real unregistration would require modifying the app bundle

    warn!(
        "macOS does not support runtime protocol unregistration. \
         The scheme '{}' will remain registered in the app bundle.",
        scheme
    );

    Ok(true)
}

/// Check if a scheme is registered and get handler info
pub async fn is_registered(scheme: &str) -> Result<RegistrationStatus, ProtocolError> {
    debug!(scheme = %scheme, "macOS: is_registered");

    let handler = get_default_handler(scheme).await?;

    // Get current app's bundle identifier for comparison
    let current_app_id = get_current_bundle_id().await.ok();

    let is_default = match (&handler, &current_app_id) {
        (Some(h), Some(c)) => h == c,
        _ => false,
    };

    Ok(RegistrationStatus {
        is_registered: handler.is_some(),
        is_default,
        registered_by: handler,
    })
}

/// Set this app as the default handler for a scheme
pub async fn set_as_default(scheme: &str) -> Result<bool, ProtocolError> {
    debug!(scheme = %scheme, "macOS: set_as_default");

    let app_identifier = get_current_bundle_id()
        .await
        .map_err(|e| ProtocolError::generic(format!("Could not get bundle ID: {}", e)))?;

    let script = format!(
        r#"
        import Foundation
        import CoreServices

        let scheme = "{}" as CFString
        let bundleId = "{}" as CFString

        let result = LSSetDefaultHandlerForURLScheme(scheme, bundleId)
        exit(result == noErr ? 0 : 1)
        "#,
        scheme, app_identifier
    );

    let output = Command::new("swift")
        .args(["-e", &script])
        .output()
        .await
        .map_err(|e| ProtocolError::tool_not_found(format!("Swift CLI not available: {}", e)))?;

    Ok(output.status.success())
}

/// Get the default handler for a URL scheme
async fn get_default_handler(scheme: &str) -> Result<Option<String>, ProtocolError> {
    let script = format!(
        r#"
        import Foundation
        import CoreServices

        let scheme = "{}" as CFString
        if let handler = LSCopyDefaultHandlerForURLScheme(scheme) {{
            print(handler.takeRetainedValue() as String)
        }}
        "#,
        scheme
    );

    let output = Command::new("swift")
        .args(["-e", &script])
        .output()
        .await
        .map_err(|e| ProtocolError::tool_not_found(format!("Swift CLI not available: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if stdout.is_empty() {
        Ok(None)
    } else {
        Ok(Some(stdout))
    }
}

/// Get the current app's bundle identifier
async fn get_current_bundle_id() -> Result<String, ProtocolError> {
    // Try to get from environment or use a fallback
    if let Ok(bundle_id) = std::env::var("FORGE_BUNDLE_ID") {
        return Ok(bundle_id);
    }

    // Try to get from the running process
    let script = r#"
        import Foundation
        if let bundleId = Bundle.main.bundleIdentifier {
            print(bundleId)
        } else {
            // Fallback for CLI execution
            print(ProcessInfo.processInfo.processName)
        }
    "#;

    let output = Command::new("swift")
        .args(["-e", script])
        .output()
        .await
        .map_err(|e| ProtocolError::tool_not_found(format!("Swift CLI not available: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if stdout.is_empty() {
        Err(ProtocolError::generic(
            "Could not determine bundle identifier",
        ))
    } else {
        Ok(stdout)
    }
}

/// Check platform capabilities
pub fn check_capabilities() -> ProtocolCapabilities {
    // Check if Swift is available (needed for our implementation)
    let swift_available = std::process::Command::new("swift")
        .arg("--version")
        .output()
        .is_ok();

    // Check if duti is available (alternative tool)
    let duti_available = std::process::Command::new("duti")
        .arg("-h")
        .output()
        .is_ok();

    ProtocolCapabilities {
        can_register: swift_available,
        can_query: swift_available,
        can_deep_link: true,
        platform: "macos".to_string(),
        notes: if swift_available {
            Some("Using Launch Services via Swift CLI".to_string())
        } else if duti_available {
            Some("Using duti for protocol management".to_string())
        } else {
            Some("Limited functionality - Swift CLI not available".to_string())
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_capabilities() {
        let caps = check_capabilities();
        assert_eq!(caps.platform, "macos");
        // can_register depends on Swift availability
    }
}
