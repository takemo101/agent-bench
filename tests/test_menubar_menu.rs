//! MenuBuilder統合テスト
//!
//! メニューバーメニュー構築機能のテスト

use pomodoro::menubar::menu::{MenuBuilder, MenuItemIds};
use pomodoro::types::{PomodoroConfig, TimerPhase, TimerState};

#[test]
fn test_menu_item_ids_default() {
    let ids = MenuItemIds::default();
    assert_eq!(ids.pause.as_ref(), "pause");
    assert_eq!(ids.resume.as_ref(), "resume");
    assert_eq!(ids.stop.as_ref(), "stop");
    assert_eq!(ids.quit.as_ref(), "quit");
}

// 注意: MenuBuilder::new() や build() は内部で tray_icon::menu::Menu::new() を呼び出す。
// macOSなど一部のプラットフォームでは、メニュー作成はメインスレッドで行う必要があるため、
// 標準的な `cargo test` (マルチスレッド) 環境ではパニックする可能性がある。
// 以下のテストは構造とロジックの確認用だが、CI環境等でパニックする場合は無視する。

#[test]
#[ignore = "Requires main thread execution for UI components on macOS"]
fn test_menu_builder_lifecycle() {
    // このテストはメインスレッドで実行する必要がある
    if let Ok(mut builder) = MenuBuilder::new() {
        let mut state = TimerState::new(PomodoroConfig::default());
        state.phase = TimerPhase::Working;

        // ビルドが成功するか
        let result = builder.build(&state);
        assert!(result.is_ok());

        // 別の状態でも成功するか
        state.phase = TimerPhase::Paused;
        let result = builder.build(&state);
        assert!(result.is_ok());
    }
}
