//! Focus mode integration via Shortcuts.app
//!
//! This module provides integration with macOS Focus Mode using Shortcuts.app.
//! It enables automatic focus mode activation when work timer starts and
//! deactivation when break timer starts.

#[cfg(target_os = "macos")]
use std::process::Command;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::time::timeout;
use tracing::{error, info, warn};

/// Focus mode configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusModeConfig {
    /// Whether focus mode integration is enabled
    pub enabled: bool,
    /// Shortcut name to enable focus mode
    pub enable_shortcut_name: String,
    /// Shortcut name to disable focus mode
    pub disable_shortcut_name: String,
}

impl Default for FocusModeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            enable_shortcut_name: "Enable Work Focus".to_string(),
            disable_shortcut_name: "Disable Work Focus".to_string(),
        }
    }
}

/// Focus mode errors
#[derive(Debug, Error)]
pub enum FocusModeError {
    /// Shortcuts.app not found (macOS 12+ required)
    #[error("Shortcuts.app not found. macOS 12+ required.")]
    ShortcutsNotFound,

    /// Shortcut not found in Shortcuts.app
    #[error("Shortcut '{0}' not found.")]
    ShortcutNotFound(String),

    /// Shortcut execution timed out
    #[error("Shortcut '{0}' execution timed out ({1}s)")]
    ExecutionTimeout(String, u64),

    /// Shortcut execution failed
    #[error("Shortcut '{0}' execution failed: {1}")]
    ExecutionFailed(String, String),

    /// Other focus mode error
    #[error("Focus mode error: {0}")]
    Other(String),
}

/// Check if Shortcuts.app exists on the system
///
/// Returns `true` if `/usr/bin/shortcuts` exists (macOS 12+), `false` otherwise.
pub fn shortcuts_exists() -> bool {
    #[cfg(target_os = "macos")]
    {
        std::path::Path::new("/usr/bin/shortcuts").exists()
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

/// Enable focus mode via Shortcuts.app
///
/// Executes the specified shortcut to enable focus mode.
/// Times out after 5 seconds to prevent blocking the timer.
///
/// # Arguments
/// * `shortcut_name` - Name of the shortcut to run (e.g., "Enable Work Focus")
///
/// # Returns
/// * `Ok(())` - Focus mode enabled successfully
/// * `Err(FocusModeError)` - Failed to enable focus mode
pub async fn enable_focus(shortcut_name: &str) -> Result<(), FocusModeError> {
    info!("Enabling focus mode: {}", shortcut_name);

    if !shortcuts_exists() {
        warn!("Shortcuts.app not found. Skipping focus mode.");
        return Err(FocusModeError::ShortcutsNotFound);
    }

    let result = timeout(Duration::from_secs(5), execute_shortcut(shortcut_name)).await;

    match result {
        Ok(Ok(())) => {
            info!("Focus mode enabled.");
            Ok(())
        }
        Ok(Err(e)) => {
            error!("Failed to enable focus mode: {}", e);
            Err(e)
        }
        Err(_) => {
            error!("Focus mode enable timed out.");
            Err(FocusModeError::ExecutionTimeout(
                shortcut_name.to_string(),
                5,
            ))
        }
    }
}

/// Disable focus mode via Shortcuts.app
///
/// Executes the specified shortcut to disable focus mode.
/// Times out after 5 seconds to prevent blocking the timer.
///
/// # Arguments
/// * `shortcut_name` - Name of the shortcut to run (e.g., "Disable Work Focus")
///
/// # Returns
/// * `Ok(())` - Focus mode disabled successfully
/// * `Err(FocusModeError)` - Failed to disable focus mode
pub async fn disable_focus(shortcut_name: &str) -> Result<(), FocusModeError> {
    info!("Disabling focus mode: {}", shortcut_name);

    if !shortcuts_exists() {
        warn!("Shortcuts.app not found. Skipping focus mode.");
        return Err(FocusModeError::ShortcutsNotFound);
    }

    let result = timeout(Duration::from_secs(5), execute_shortcut(shortcut_name)).await;

    match result {
        Ok(Ok(())) => {
            info!("Focus mode disabled.");
            Ok(())
        }
        Ok(Err(e)) => {
            error!("Failed to disable focus mode: {}", e);
            Err(e)
        }
        Err(_) => {
            error!("Focus mode disable timed out.");
            Err(FocusModeError::ExecutionTimeout(
                shortcut_name.to_string(),
                5,
            ))
        }
    }
}

/// Execute a shortcut via `/usr/bin/shortcuts run`
///
/// # Arguments
/// * `shortcut_name` - Name of the shortcut to run
///
/// # Returns
/// * `Ok(())` - Shortcut executed successfully
/// * `Err(FocusModeError)` - Shortcut execution failed
async fn execute_shortcut(shortcut_name: &str) -> Result<(), FocusModeError> {
    #[cfg(not(target_os = "macos"))]
    {
        let _ = shortcut_name;
        Err(FocusModeError::ShortcutsNotFound)
    }

    #[cfg(target_os = "macos")]
    {
        let shortcut_name_owned = shortcut_name.to_string();
        let shortcut_name_for_error = shortcut_name_owned.clone();
        let output = tokio::task::spawn_blocking(move || {
            Command::new("/usr/bin/shortcuts")
                .arg("run")
                .arg(&shortcut_name_owned)
                .output()
        })
        .await
        .map_err(|e| FocusModeError::Other(format!("Task join error: {}", e)))?
        .map_err(|e| FocusModeError::Other(format!("Command execution error: {}", e)))?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("not found") || stderr.contains("does not exist") {
                Err(FocusModeError::ShortcutNotFound(shortcut_name_for_error))
            } else {
                Err(FocusModeError::ExecutionFailed(
                    shortcut_name_for_error,
                    stderr.to_string(),
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_mode_config_default() {
        let config = FocusModeConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.enable_shortcut_name, "Enable Work Focus");
        assert_eq!(config.disable_shortcut_name, "Disable Work Focus");
    }

    #[test]
    fn test_shortcuts_exists_returns_false_on_non_macos() {
        // On non-macOS platforms, shortcuts_exists() should return false
        #[cfg(not(target_os = "macos"))]
        {
            assert!(!shortcuts_exists());
        }
    }

    #[tokio::test]
    async fn test_enable_focus_returns_error_when_shortcuts_not_found() {
        // On non-macOS platforms, enable_focus should return ShortcutsNotFound
        #[cfg(not(target_os = "macos"))]
        {
            let result = enable_focus("Test Shortcut").await;
            assert!(matches!(result, Err(FocusModeError::ShortcutsNotFound)));
        }
    }

    #[tokio::test]
    async fn test_disable_focus_returns_error_when_shortcuts_not_found() {
        // On non-macOS platforms, disable_focus should return ShortcutsNotFound
        #[cfg(not(target_os = "macos"))]
        {
            let result = disable_focus("Test Shortcut").await;
            assert!(matches!(result, Err(FocusModeError::ShortcutsNotFound)));
        }
    }

    #[test]
    fn test_focus_mode_error_display() {
        let err = FocusModeError::ShortcutsNotFound;
        assert_eq!(
            err.to_string(),
            "Shortcuts.app not found. macOS 12+ required."
        );

        let err = FocusModeError::ShortcutNotFound("Test".to_string());
        assert_eq!(err.to_string(), "Shortcut 'Test' not found.");

        let err = FocusModeError::ExecutionTimeout("Test".to_string(), 5);
        assert_eq!(err.to_string(), "Shortcut 'Test' execution timed out (5s)");

        let err = FocusModeError::ExecutionFailed("Test".to_string(), "error".to_string());
        assert_eq!(err.to_string(), "Shortcut 'Test' execution failed: error");
    }
}
