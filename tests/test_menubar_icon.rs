//! IconManager統合テスト
//!
//! メニューバーアイコン管理機能のテスト

use pomodoro_timer::menubar::icon::IconManager;
use pomodoro_timer::types::{PomodoroConfig, TimerPhase, TimerState};

// =============================================================================
// generate_title テスト
// =============================================================================

#[test]
fn test_generate_title_working_standard() {
    let mut state = TimerState::new(PomodoroConfig::default());
    state.phase = TimerPhase::Working;
    state.remaining_seconds = 1500; // 25:00

    let title = IconManager::generate_title(&state);
    assert_eq!(title, "🍅 25:00");
}

#[test]
fn test_generate_title_working_mid_session() {
    let mut state = TimerState::new(PomodoroConfig::default());
    state.phase = TimerPhase::Working;
    state.remaining_seconds = 930; // 15:30

    let title = IconManager::generate_title(&state);
    assert_eq!(title, "🍅 15:30");
}

#[test]
fn test_generate_title_working_final_minute() {
    let mut state = TimerState::new(PomodoroConfig::default());
    state.phase = TimerPhase::Working;
    state.remaining_seconds = 59; // 00:59

    let title = IconManager::generate_title(&state);
    assert_eq!(title, "🍅 00:59");
}

#[test]
fn test_generate_title_breaking_short() {
    let mut state = TimerState::new(PomodoroConfig::default());
    state.phase = TimerPhase::Breaking;
    state.remaining_seconds = 300; // 05:00

    let title = IconManager::generate_title(&state);
    assert_eq!(title, "☕ 05:00");
}

#[test]
fn test_generate_title_long_breaking() {
    let mut state = TimerState::new(PomodoroConfig::default());
    state.phase = TimerPhase::LongBreaking;
    state.remaining_seconds = 900; // 15:00

    let title = IconManager::generate_title(&state);
    assert_eq!(title, "☕ 15:00");
}

#[test]
fn test_generate_title_paused_ignores_remaining() {
    let mut state = TimerState::new(PomodoroConfig::default());
    state.phase = TimerPhase::Paused;
    state.remaining_seconds = 1234; // 任意の値

    let title = IconManager::generate_title(&state);
    assert_eq!(title, "⏸ 一時停止");
}

#[test]
fn test_generate_title_stopped() {
    let state = TimerState::new(PomodoroConfig::default());

    let title = IconManager::generate_title(&state);
    assert_eq!(title, "⏸ 停止中");
}

// =============================================================================
// IconManager構造体テスト
// =============================================================================

#[test]
fn test_icon_manager_creation() {
    let manager = IconManager::new();
    assert!(
        manager.is_ok(),
        "IconManager should be created successfully"
    );
}

#[test]
fn test_icon_manager_get_icon_all_phases() {
    let manager = IconManager::new().expect("Failed to create IconManager");

    // 全フェーズでアイコンが取得できることを確認
    let phases = [
        TimerPhase::Working,
        TimerPhase::Breaking,
        TimerPhase::LongBreaking,
        TimerPhase::Paused,
        TimerPhase::Stopped,
    ];

    for phase in &phases {
        let _icon = manager.get_icon(phase);
        // アイコンが取得できれば成功（内容の検証は困難）
    }
}

// =============================================================================
// 境界値テスト
// =============================================================================

#[test]
fn test_generate_title_zero_remaining() {
    let mut state = TimerState::new(PomodoroConfig::default());
    state.phase = TimerPhase::Working;
    state.remaining_seconds = 0;

    let title = IconManager::generate_title(&state);
    assert_eq!(title, "🍅 00:00");
}

#[test]
fn test_generate_title_one_second() {
    let mut state = TimerState::new(PomodoroConfig::default());
    state.phase = TimerPhase::Working;
    state.remaining_seconds = 1;

    let title = IconManager::generate_title(&state);
    assert_eq!(title, "🍅 00:01");
}

#[test]
fn test_generate_title_one_minute() {
    let mut state = TimerState::new(PomodoroConfig::default());
    state.phase = TimerPhase::Working;
    state.remaining_seconds = 60;

    let title = IconManager::generate_title(&state);
    assert_eq!(title, "🍅 01:00");
}

#[test]
fn test_generate_title_max_config_time() {
    let mut state = TimerState::new(PomodoroConfig::default());
    state.phase = TimerPhase::Working;
    state.remaining_seconds = 120 * 60; // 120分 = 7200秒

    let title = IconManager::generate_title(&state);
    assert_eq!(title, "🍅 120:00");
}
