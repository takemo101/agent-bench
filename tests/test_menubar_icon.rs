use pomodoro::menubar::icon::IconManager;
use pomodoro::types::{TimerState, PomodoroConfig};

#[test]
fn test_generate_title_working() {
    let mut state = TimerState::new(PomodoroConfig::default());
    state.start_working(None); // Set to Working phase
    state.remaining_seconds = 930; // 15:30
    
    let title = IconManager::generate_title(&state);
    assert_eq!(title, "🍅 15:30");
}

#[test]
fn test_generate_title_breaking() {
    let mut state = TimerState::new(PomodoroConfig::default());
    state.start_breaking(); // Set to Breaking phase
    state.remaining_seconds = 270; // 04:30
    
    let title = IconManager::generate_title(&state);
    assert_eq!(title, "☕ 04:30");
}

#[test]
fn test_generate_title_paused() {
    let mut state = TimerState::new(PomodoroConfig::default());
    state.start_working(None);
    state.pause(); // Set to Paused phase
    
    let title = IconManager::generate_title(&state);
    assert_eq!(title, "⏸ 一時停止");
}

#[test]
fn test_generate_title_stopped() {
    let mut state = TimerState::new(PomodoroConfig::default());
    state.stop(); // Set to Stopped phase
    
    let title = IconManager::generate_title(&state);
    assert_eq!(title, "⏸ 停止中");
}

#[test]
fn test_icon_manager_creation() {
    let manager = IconManager::new();
    assert!(manager.is_ok());
}
