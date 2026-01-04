use pomodoro::menubar::TrayIconManager;

#[test]
#[cfg_attr(
    target_os = "macos",
    ignore = "Requires main thread for UI operations on macOS"
)]
fn test_tray_icon_manager_structure() {
    let _ = TrayIconManager::new();
}
