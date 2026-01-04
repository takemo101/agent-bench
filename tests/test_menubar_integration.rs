//! MenuBar統合テスト
//!
//! TrayIconManagerの統合動作を検証する。

use pomodoro::menubar::TrayIconManager;
use pomodoro::types::{PomodoroConfig, TimerPhase, TimerState};

#[test]
#[cfg_attr(
    target_os = "macos",
    ignore = "Requires main thread for UI operations on macOS"
)]
fn test_tray_icon_manager_instantiation() {
    let manager = TrayIconManager::new();
    if cfg!(not(target_os = "macos")) {
        assert!(manager.is_ok());
    }
}

#[test]
#[cfg_attr(
    target_os = "macos",
    ignore = "Requires main thread for UI operations on macOS"
)]
fn test_tray_icon_manager_lifecycle() {
    if let Ok(mut manager) = TrayIconManager::new() {
        // Initialize (might fail on CI due to headless environment)
        if manager.initialize().is_err() {
            return;
        }

        let mut state = TimerState::new(PomodoroConfig::default());
        state.phase = TimerPhase::Working;

        // Update state
        let result = manager.update_state(&state);
        assert!(result.is_ok());

        // Check events (should be None initially)
        let action = manager.check_events();
        assert_eq!(action, None);

        // Shutdown
        let result = manager.shutdown();
        assert!(result.is_ok());
    }
}
