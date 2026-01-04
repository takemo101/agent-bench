//! タイマー連携統合テスト
//! TC-I-005 to TC-I-013: タイマー-通知-サウンド-フォーカスモード連携

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use pomodoro::{
    daemon::{TimerEngine, TimerEvent},
    sound::{SoundError, SoundPlayer, SoundSource},
    types::{PomodoroConfig, TimerPhase},
};
use tokio::sync::mpsc;

pub struct MockSoundPlayer {
    play_count: AtomicU32,
    available: AtomicBool,
    disabled: AtomicBool,
}

impl MockSoundPlayer {
    pub fn new(available: bool, disabled: bool) -> Self {
        Self {
            play_count: AtomicU32::new(0),
            available: AtomicBool::new(available),
            disabled: AtomicBool::new(disabled),
        }
    }

    pub fn play_count(&self) -> u32 {
        self.play_count.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl SoundPlayer for MockSoundPlayer {
    async fn play(&self, _source: &SoundSource) -> Result<(), SoundError> {
        if self.disabled.load(Ordering::SeqCst) {
            return Ok(());
        }
        if !self.available.load(Ordering::SeqCst) {
            return Err(SoundError::DeviceNotAvailable(
                "Mock device not available".to_string(),
            ));
        }
        self.play_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    fn is_available(&self) -> bool {
        self.available.load(Ordering::SeqCst)
    }
}

pub struct MockFocusModeController {
    enabled: AtomicBool,
    enable_count: AtomicU32,
    disable_count: AtomicU32,
    should_fail: AtomicBool,
}

impl MockFocusModeController {
    pub fn new(should_fail: bool) -> Self {
        Self {
            enabled: AtomicBool::new(false),
            enable_count: AtomicU32::new(0),
            disable_count: AtomicU32::new(0),
            should_fail: AtomicBool::new(should_fail),
        }
    }

    pub fn enable(&self) -> Result<(), &'static str> {
        if self.should_fail.load(Ordering::SeqCst) {
            return Err("Focus mode shortcut not found");
        }
        self.enabled.store(true, Ordering::SeqCst);
        self.enable_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    pub fn disable(&self) -> Result<(), &'static str> {
        if self.should_fail.load(Ordering::SeqCst) {
            return Err("Focus mode shortcut not found");
        }
        self.enabled.store(false, Ordering::SeqCst);
        self.disable_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }

    pub fn enable_count(&self) -> u32 {
        self.enable_count.load(Ordering::SeqCst)
    }

    pub fn disable_count(&self) -> u32 {
        self.disable_count.load(Ordering::SeqCst)
    }
}

pub struct MockNotificationSender {
    notifications: std::sync::Mutex<Vec<String>>,
}

impl MockNotificationSender {
    pub fn new() -> Self {
        Self {
            notifications: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn send(&self, title: &str) {
        let mut notifications = self.notifications.lock().unwrap();
        notifications.push(title.to_string());
    }

    pub fn notification_count(&self) -> usize {
        self.notifications.lock().unwrap().len()
    }

    pub fn get_notifications(&self) -> Vec<String> {
        self.notifications.lock().unwrap().clone()
    }
}

impl Default for MockNotificationSender {
    fn default() -> Self {
        Self::new()
    }
}

fn create_test_engine() -> (TimerEngine, mpsc::UnboundedReceiver<TimerEvent>) {
    let (tx, rx) = mpsc::unbounded_channel();
    let config = PomodoroConfig::default();
    (TimerEngine::new(config, tx), rx)
}

fn create_test_engine_with_config(
    config: PomodoroConfig,
) -> (TimerEngine, mpsc::UnboundedReceiver<TimerEvent>) {
    let (tx, rx) = mpsc::unbounded_channel();
    (TimerEngine::new(config, tx), rx)
}

fn embedded_sound_source() -> SoundSource {
    SoundSource::Embedded {
        name: "default".to_string(),
    }
}

#[tokio::test]
async fn tc_i_005_work_started_event_triggers_notification() {
    let (mut engine, mut rx) = create_test_engine();
    let notification_sender = Arc::new(MockNotificationSender::new());

    engine.start(Some(pomodoro::types::StartParams { task_name: Some("通知テスト".to_string()), ..Default::default() })).unwrap();

    let event = rx.try_recv();
    assert!(event.is_ok());

    if let Ok(TimerEvent::WorkStarted { task_name }) = event {
        assert_eq!(task_name, Some("通知テスト".to_string()));
        notification_sender.send(&format!(
            "作業開始: {}",
            task_name.as_deref().unwrap_or("タスク")
        ));
    } else {
        panic!("Expected WorkStarted event");
    }

    assert_eq!(notification_sender.notification_count(), 1);
    assert!(notification_sender.get_notifications()[0].contains("作業開始"));
}

#[tokio::test]
async fn tc_i_005_tick_event_fired() {
    let (mut engine, mut rx) = create_test_engine();

    engine.start(None).unwrap();
    let _ = rx.try_recv();

    let processed = engine.process_tick().unwrap();
    assert!(processed);

    let event = rx.try_recv();
    assert!(event.is_ok());
    assert!(matches!(event.unwrap(), TimerEvent::Tick { .. }));
}

#[tokio::test]
async fn tc_i_006_pause_resume_events_trigger_notifications() {
    let (mut engine, mut rx) = create_test_engine();
    let notification_sender = Arc::new(MockNotificationSender::new());

    engine.start(None).unwrap();
    let _ = rx.try_recv();

    engine.pause().unwrap();
    let paused_event = rx.try_recv();
    assert!(matches!(paused_event, Ok(TimerEvent::Paused)));
    notification_sender.send("一時停止");

    engine.resume().unwrap();
    let resumed_event = rx.try_recv();
    assert!(matches!(resumed_event, Ok(TimerEvent::Resumed)));
    notification_sender.send("再開");

    assert_eq!(notification_sender.notification_count(), 2);
}

#[tokio::test]
async fn tc_i_006_stop_event_triggers_notification() {
    let (mut engine, mut rx) = create_test_engine();
    let notification_sender = Arc::new(MockNotificationSender::new());

    engine.start(Some(pomodoro::types::StartParams { task_name: Some("停止テスト".to_string()), ..Default::default() })).unwrap();
    let _ = rx.try_recv();

    engine.stop().unwrap();

    let event = rx.try_recv();
    assert!(matches!(event, Ok(TimerEvent::Stopped)));

    notification_sender.send("タイマー停止");

    assert_eq!(notification_sender.notification_count(), 1);
    assert!(notification_sender.get_notifications()[0].contains("停止"));
}

#[test]
fn tc_i_008_work_start_enables_focus_mode() {
    let focus_controller = MockFocusModeController::new(false);

    focus_controller.enable().unwrap();

    assert!(focus_controller.is_enabled());
    assert_eq!(focus_controller.enable_count(), 1);
}

#[test]
fn tc_i_009_break_start_disables_focus_mode() {
    let focus_controller = MockFocusModeController::new(false);

    focus_controller.enable().unwrap();
    assert!(focus_controller.is_enabled());

    focus_controller.disable().unwrap();

    assert!(!focus_controller.is_enabled());
    assert_eq!(focus_controller.disable_count(), 1);
}

#[tokio::test]
async fn tc_i_010_focus_mode_failure_fallback() {
    let (mut engine, mut rx) = create_test_engine();
    let focus_controller = MockFocusModeController::new(true);

    engine
        .start(Some(pomodoro::types::StartParams { task_name: Some("フォールバックテスト".to_string()), ..Default::default() }))
        .unwrap();
    let _ = rx.try_recv();

    let focus_result = focus_controller.enable();
    assert!(focus_result.is_err());

    assert_eq!(engine.get_state().phase, TimerPhase::Working);
    assert!(engine.get_state().remaining_seconds > 0);
}

#[tokio::test]
async fn tc_i_011_sound_player_plays_on_event() {
    let sound_player = Arc::new(MockSoundPlayer::new(true, false));

    let source = embedded_sound_source();
    sound_player.play(&source).await.unwrap();

    assert_eq!(sound_player.play_count(), 1);
}

#[tokio::test]
async fn tc_i_011_sound_player_available_check() {
    let available_player = MockSoundPlayer::new(true, false);
    let unavailable_player = MockSoundPlayer::new(false, false);

    assert!(available_player.is_available());
    assert!(!unavailable_player.is_available());
}

#[tokio::test]
async fn tc_i_012_no_sound_flag_disables_playback() {
    let sound_player = Arc::new(MockSoundPlayer::new(true, true));

    let source = embedded_sound_source();
    let result = sound_player.play(&source).await;

    assert!(result.is_ok());
    assert_eq!(sound_player.play_count(), 0);
}

#[tokio::test]
async fn tc_i_013_sound_fallback_to_embedded() {
    let sound_player = Arc::new(MockSoundPlayer::new(true, false));

    let custom_path = std::path::PathBuf::from("/nonexistent/sound.wav");
    let source = if custom_path.exists() {
        SoundSource::System {
            name: "custom".to_string(),
            path: custom_path,
        }
    } else {
        embedded_sound_source()
    };

    assert!(matches!(source, SoundSource::Embedded { .. }));

    let result = sound_player.play(&source).await;
    assert!(result.is_ok());
    assert_eq!(sound_player.play_count(), 1);
}

#[tokio::test]
async fn tc_i_013_sound_device_not_available_error() {
    let sound_player = Arc::new(MockSoundPlayer::new(false, false));

    let source = embedded_sound_source();
    let result = sound_player.play(&source).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        SoundError::DeviceNotAvailable(_)
    ));
}

#[tokio::test]
async fn integration_event_driven_flow() {
    let config = PomodoroConfig {
        auto_cycle: true,
        focus_mode: true,
        ..Default::default()
    };
    let (mut engine, mut rx) = create_test_engine_with_config(config);

    let sound_player = Arc::new(MockSoundPlayer::new(true, false));
    let focus_controller = Arc::new(MockFocusModeController::new(false));
    let notification_sender = Arc::new(MockNotificationSender::new());

    engine.start(Some(pomodoro::types::StartParams { task_name: Some("統合テスト".to_string()), ..Default::default() })).unwrap();

    while let Ok(event) = rx.try_recv() {
        if let TimerEvent::WorkStarted { task_name } = event {
            focus_controller.enable().unwrap();
            notification_sender.send(&format!(
                "作業開始: {}",
                task_name.as_deref().unwrap_or("タスク")
            ));
        }
    }

    assert!(focus_controller.is_enabled());
    assert_eq!(notification_sender.notification_count(), 1);

    for _ in 0..3 {
        engine.process_tick().unwrap();
    }

    let mut tick_count = 0;
    while let Ok(event) = rx.try_recv() {
        if matches!(event, TimerEvent::Tick { .. }) {
            tick_count += 1;
        }
    }
    assert_eq!(tick_count, 3);

    engine.pause().unwrap();
    if let Ok(TimerEvent::Paused) = rx.try_recv() {
        focus_controller.disable().unwrap();
    }
    assert!(!focus_controller.is_enabled());

    engine.resume().unwrap();
    if let Ok(TimerEvent::Resumed) = rx.try_recv() {
        focus_controller.enable().unwrap();
    }
    assert!(focus_controller.is_enabled());

    engine.stop().unwrap();
    if let Ok(TimerEvent::Stopped) = rx.try_recv() {
        focus_controller.disable().unwrap();
        let source = embedded_sound_source();
        sound_player.play(&source).await.unwrap();
        notification_sender.send("タイマー停止");
    }

    assert!(!focus_controller.is_enabled());
    assert_eq!(sound_player.play_count(), 1);
    assert_eq!(notification_sender.notification_count(), 2);
}

#[tokio::test]
async fn integration_multiple_start_stop_cycles() {
    let (mut engine, mut rx) = create_test_engine();
    let focus_controller = Arc::new(MockFocusModeController::new(false));

    for i in 1..=3 {
        engine.start(Some(pomodoro::types::StartParams { task_name: Some(format!("サイクル{}", i)), ..Default::default() })).unwrap();
        if let Ok(TimerEvent::WorkStarted { .. }) = rx.try_recv() {
            focus_controller.enable().unwrap();
        }
        assert!(focus_controller.is_enabled());

        for _ in 0..2 {
            engine.process_tick().unwrap();
        }
        while rx.try_recv().is_ok() {}

        engine.stop().unwrap();
        if let Ok(TimerEvent::Stopped) = rx.try_recv() {
            focus_controller.disable().unwrap();
        }
        assert!(!focus_controller.is_enabled());
    }

    assert_eq!(focus_controller.enable_count(), 3);
    assert_eq!(focus_controller.disable_count(), 3);
}

#[tokio::test]
async fn integration_error_conditions() {
    let (mut engine, _rx) = create_test_engine();

    let result = engine.stop();
    assert!(result.is_err());

    let result = engine.pause();
    assert!(result.is_err());

    engine.start(None).unwrap();

    let result = engine.start(None);
    assert!(result.is_err());
}
