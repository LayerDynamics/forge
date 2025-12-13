//! Crash reporting infrastructure for Forge apps
//!
//! Provides panic handling and crash report generation for debugging.

use std::fs;
use std::panic;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{error, info, warn};

/// Whether crash reporting is enabled
static CRASH_REPORTING_ENABLED: AtomicBool = AtomicBool::new(false);

/// Directory to write crash reports
static CRASH_REPORT_DIR: once_cell::sync::OnceCell<PathBuf> = once_cell::sync::OnceCell::new();

/// App name for crash reports
static APP_NAME: once_cell::sync::OnceCell<String> = once_cell::sync::OnceCell::new();

/// Initialize crash reporting with the given configuration
///
/// # Arguments
/// * `enabled` - Whether crash reporting is enabled
/// * `report_dir` - Directory to write crash reports to
/// * `app_name` - Name of the app (for report metadata)
pub fn init_crash_reporting(enabled: bool, report_dir: &str, app_name: &str) {
    CRASH_REPORTING_ENABLED.store(enabled, Ordering::SeqCst);
    let _ = CRASH_REPORT_DIR.set(PathBuf::from(report_dir));
    let _ = APP_NAME.set(app_name.to_string());

    if enabled {
        // Ensure the crash report directory exists
        if let Err(e) = fs::create_dir_all(report_dir) {
            warn!(
                "Failed to create crash report directory {}: {}",
                report_dir, e
            );
        }

        // Set up the panic hook
        let default_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            handle_panic(panic_info);
            // Call the default hook to preserve normal panic behavior
            default_hook(panic_info);
        }));

        info!(
            "Crash reporting initialized, reports will be written to: {}",
            report_dir
        );
    }
}

/// Handle a panic by generating a crash report
fn handle_panic(panic_info: &panic::PanicHookInfo) {
    if !CRASH_REPORTING_ENABLED.load(Ordering::SeqCst) {
        return;
    }

    // Capture backtrace
    let backtrace = backtrace::Backtrace::new();

    // Log the panic
    error!("PANIC: {}", panic_info);
    error!("Backtrace:\n{:?}", backtrace);

    // Write crash report to file
    if let Some(dir) = CRASH_REPORT_DIR.get() {
        if let Err(e) = write_crash_report(dir, panic_info, &backtrace) {
            error!("Failed to write crash report: {}", e);
        }
    }
}

/// Write a crash report to a file
fn write_crash_report(
    dir: &Path,
    panic_info: &panic::PanicHookInfo,
    backtrace: &backtrace::Backtrace,
) -> std::io::Result<PathBuf> {
    let app_name = APP_NAME.get().map(|s| s.as_str()).unwrap_or("forge-app");
    let timestamp = chrono::Utc::now();
    let filename = format!(
        "crash-{}-{}.txt",
        app_name,
        timestamp.format("%Y%m%d-%H%M%S")
    );
    let filepath = dir.join(&filename);

    // Get panic location info
    let location = panic_info
        .location()
        .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
        .unwrap_or_else(|| "unknown location".to_string());

    // Get panic message
    let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
        s.clone()
    } else {
        "Unknown panic payload".to_string()
    };

    // Get system info
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let hostname = hostname::get()
        .ok()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Build the crash report
    let report = format!(
        r#"================================================================================
                          FORGE CRASH REPORT
================================================================================

Application:  {}
Timestamp:    {}
OS:           {}
Arch:         {}
Hostname:     {}

================================================================================
                              PANIC INFO
================================================================================

Location:     {}
Message:      {}

================================================================================
                              BACKTRACE
================================================================================

{:?}

================================================================================
                           END OF REPORT
================================================================================
"#,
        app_name,
        timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
        os,
        arch,
        hostname,
        location,
        message,
        backtrace
    );

    // Write the report
    fs::write(&filepath, report)?;

    info!("Crash report written to: {}", filepath.display());
    Ok(filepath)
}

/// Check if crash reporting is enabled
pub fn is_enabled() -> bool {
    CRASH_REPORTING_ENABLED.load(Ordering::SeqCst)
}

/// Get the crash report directory, if configured
pub fn get_report_dir() -> Option<PathBuf> {
    CRASH_REPORT_DIR.get().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;

    #[test]
    fn test_crash_reporting_disabled_by_default() {
        assert!(!is_enabled());
    }

    #[test]
    fn test_init_crash_reporting() {
        let temp_dir = env::temp_dir().join("forge-crash-test");
        let _ = fs::remove_dir_all(&temp_dir);

        // Note: We can't fully test panic handling without actually panicking,
        // so we just test initialization
        init_crash_reporting(true, temp_dir.to_str().unwrap(), "test-app");

        assert!(is_enabled());
        assert!(temp_dir.exists());

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
