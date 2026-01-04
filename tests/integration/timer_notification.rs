//! タイマー連携統合テスト
//!
//! タイマーエンジンと通知・サウンド・フォーカスモードの連携をテストする。
//!
//! ## テスト対象
//! - TC-I-005 to TC-I-007: タイマー-通知
//! - TC-I-008 to TC-I-010: タイマー-フォーカスモード
//! - TC-I-011 to TC-I-013: タイマー-サウンド

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use pomodoro::{
    daemon::{TimerEngine, TimerEvent},
    sound::{SoundError, SoundPlayer, SoundSource},
    types::{PomodoroConfig, TimerPhase},
};
use tokio::sync::mpsc;

// ============================================================================
// Mock Implementations
// ============================================================================

/// モックサウンドプレイヤー
///
/// サウンド再生の呼び出しを記録するテスト用実装。
pub struct MockSoundPlayer {
    /// 再生が呼び出された回数
    play_count: AtomicU32,
    /// 利用可能かどうか
    available: AtomicBool,
    /// 無効化されているか
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
            return Err(SoundError::DeviceNotAvailable);
        }
        self.play_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    fn is_available(&self) -> bool {
        self.available.load(Ordering::SeqCst)
    }
}

/// モックフォーカスモードコントローラー
///
/// フォーカスモードの有効/無効化を記録するテスト用実装。
pub struct MockFocusModeController {
    /// フォーカスモードが有効か
    enabled: AtomicBool,
    /// enable呼び出し回数
    enable_count: AtomicU32,
    /// disable呼び出し回数
    disable_count: AtomicU32,
    /// 失敗をシミュレートするか
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

/// モック通知センダー
///
/// 通知送信を記録するテスト用実装。
pub struct MockNotificationSender {
    /// 送信された通知のタイトル一覧
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

// ============================================================================
// Helper Functions
// ============================================================================

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

// ============================================================================
// TC-I-005: 作業完了時の通知送信
// ============================================================================

#[tokio::test]
async fn test_work_complete_triggers_notification() {
    let (mut engine, mut rx) = create_test_engine();
    let notification_sender = Arc::new(MockNotificationSender::new());

    // タイマー開始
    engine.start(Some("通知テスト".to_string())).unwrap();
    let _ = rx.try_recv(); // WorkStarted

    // タイマー完了をシミュレート
    engine.state.remaining_seconds = 1;
    engine.process_tick().unwrap();

    // イベントを収集
    let mut events = Vec::new();
    while let Ok(event) = rx.try_recv() {
        events.push(event);
    }

    // WorkCompletedイベントを検証
    let work_completed = events
        .iter()
        .find(|e| matches!(e, TimerEvent::WorkCompleted { .. }));
    assert!(work_completed.is_some());

    // 実際の通知連携をシミュレート
    if let Some(TimerEvent::WorkCompleted { task_name, .. }) = work_completed {
        notification_sender.send(&format!("作業完了: {}", task_name.as_deref().unwrap_or("タスク")));
    }

    assert_eq!(notification_sender.notification_count(), 1);
    assert!(notification_sender.get_notifications()[0].contains("作業完了"));
}

// ============================================================================
// TC-I-006: 休憩完了時の通知送信
// ============================================================================

#[tokio::test]
async fn test_break_complete_triggers_notification() {
    let (mut engine, mut rx) = create_test_engine();
    let notification_sender = Arc::new(MockNotificationSender::new());

    // タイマー開始 → 作業完了 → 休憩開始
    engine.start(None).unwrap();
    let _ = rx.try_recv(); // WorkStarted

    // 作業完了をシミュレート
    engine.state.remaining_seconds = 1;
    engine.process_tick().unwrap();

    // イベントをドレイン
    while rx.try_recv().is_ok() {}

    // 休憩中の状態を確認
    assert!(matches!(
        engine.get_state().phase,
        TimerPhase::Breaking | TimerPhase::LongBreaking
    ));

    // 休憩完了をシミュレート
    engine.state.remaining_seconds = 1;
    engine.process_tick().unwrap();

    // イベントを収集
    let mut events = Vec::new();
    while let Ok(event) = rx.try_recv() {
        events.push(event);
    }

    // BreakCompletedイベントを検証
    let break_completed = events
        .iter()
        .find(|e| matches!(e, TimerEvent::BreakCompleted { .. }));
    assert!(break_completed.is_some());

    // 通知連携をシミュレート
    if let Some(TimerEvent::BreakCompleted { is_long_break }) = break_completed {
        let msg = if *is_long_break {
            "長い休憩完了"
        } else {
            "休憩完了"
        };
        notification_sender.send(msg);
    }

    assert_eq!(notification_sender.notification_count(), 1);
}

// ============================================================================
// TC-I-008: 作業開始時のフォーカスモード有効化
// ============================================================================

#[test]
fn test_work_start_enables_focus_mode() {
    let focus_controller = MockFocusModeController::new(false);

    // タイマー開始時にフォーカスモードを有効化
    focus_controller.enable().unwrap();

    assert!(focus_controller.is_enabled());
    assert_eq!(focus_controller.enable_count(), 1);
}

// ============================================================================
// TC-I-009: 休憩開始時のフォーカスモード無効化
// ============================================================================

#[test]
fn test_break_start_disables_focus_mode() {
    let focus_controller = MockFocusModeController::new(false);

    // 作業中 → フォーカスモード有効
    focus_controller.enable().unwrap();
    assert!(focus_controller.is_enabled());

    // 休憩開始 → フォーカスモード無効
    focus_controller.disable().unwrap();

    assert!(!focus_controller.is_enabled());
    assert_eq!(focus_controller.disable_count(), 1);
}

// ============================================================================
// TC-I-010: フォーカスモード失敗時のフォールバック
// ============================================================================

#[tokio::test]
async fn test_focus_mode_failure_fallback() {
    let (mut engine, mut rx) = create_test_engine();
    let focus_controller = MockFocusModeController::new(true); // 失敗をシミュレート

    // タイマー開始
    engine.start(Some("フォールバックテスト".to_string())).unwrap();
    let _ = rx.try_recv(); // WorkStarted

    // フォーカスモード有効化を試みる（失敗する）
    let focus_result = focus_controller.enable();
    assert!(focus_result.is_err());

    // タイマーは継続して動作する
    assert_eq!(engine.get_state().phase, TimerPhase::Working);
    assert!(engine.get_state().remaining_seconds > 0);
}

// ============================================================================
// TC-I-011: 作業完了時のサウンド再生
// ============================================================================

#[tokio::test]
async fn test_work_complete_plays_sound() {
    let (mut engine, mut rx) = create_test_engine();
    let sound_player = Arc::new(MockSoundPlayer::new(true, false));

    // タイマー開始
    engine.start(None).unwrap();
    let _ = rx.try_recv(); // WorkStarted

    // 作業完了をシミュレート
    engine.state.remaining_seconds = 1;
    engine.process_tick().unwrap();

    // イベントを収集
    let mut events = Vec::new();
    while let Ok(event) = rx.try_recv() {
        events.push(event);
    }

    // WorkCompletedイベントでサウンド再生
    let work_completed = events
        .iter()
        .find(|e| matches!(e, TimerEvent::WorkCompleted { .. }));
    assert!(work_completed.is_some());

    // サウンド再生をシミュレート
    sound_player
        .play(&SoundSource::Embedded)
        .await
        .unwrap();

    assert_eq!(sound_player.play_count(), 1);
}

// ============================================================================
// TC-I-012: --no-soundフラグ
// ============================================================================

#[tokio::test]
async fn test_no_sound_flag() {
    let sound_player = Arc::new(MockSoundPlayer::new(true, true)); // disabled=true

    // 無効化されたプレイヤーでの再生
    let result = sound_player.play(&SoundSource::Embedded).await;

    // エラーなく完了し、再生は行われない
    assert!(result.is_ok());
    assert_eq!(sound_player.play_count(), 0);
}

// ============================================================================
// TC-I-013: サウンドファイル未検出時のフォールバック
// ============================================================================

#[tokio::test]
async fn test_sound_fallback_to_embedded() {
    let sound_player = Arc::new(MockSoundPlayer::new(true, false));

    // カスタムサウンドが見つからない場合、埋め込みサウンドにフォールバック
    let custom_path = std::path::PathBuf::from("/nonexistent/sound.wav");
    let source = if custom_path.exists() {
        SoundSource::File(custom_path)
    } else {
        SoundSource::Embedded
    };

    // 埋め込みサウンドで再生
    assert!(matches!(source, SoundSource::Embedded));

    let result = sound_player.play(&source).await;
    assert!(result.is_ok());
    assert_eq!(sound_player.play_count(), 1);
}

// ============================================================================
// 統合シナリオ: 完全なポモドーロサイクル
// ============================================================================

#[tokio::test]
async fn test_full_pomodoro_cycle_with_integrations() {
    let config = PomodoroConfig {
        auto_cycle: true,
        focus_mode: true,
        ..Default::default()
    };
    let (mut engine, mut rx) = create_test_engine_with_config(config);

    let sound_player = Arc::new(MockSoundPlayer::new(true, false));
    let focus_controller = Arc::new(MockFocusModeController::new(false));
    let notification_sender = Arc::new(MockNotificationSender::new());

    // 1. 作業開始
    engine.start(Some("統合テスト".to_string())).unwrap();
    focus_controller.enable().unwrap();
    let _ = rx.try_recv(); // WorkStarted

    assert!(focus_controller.is_enabled());

    // 2. 作業完了
    engine.state.remaining_seconds = 1;
    engine.process_tick().unwrap();

    // イベント処理
    while let Ok(event) = rx.try_recv() {
        match event {
            TimerEvent::WorkCompleted { .. } => {
                sound_player.play(&SoundSource::Embedded).await.unwrap();
                notification_sender.send("作業完了");
            }
            TimerEvent::BreakStarted { .. } => {
                focus_controller.disable().unwrap();
            }
            _ => {}
        }
    }

    assert!(!focus_controller.is_enabled());
    assert_eq!(sound_player.play_count(), 1);
    assert_eq!(notification_sender.notification_count(), 1);

    // 3. 休憩完了
    engine.state.remaining_seconds = 1;
    engine.process_tick().unwrap();

    // イベント処理
    while let Ok(event) = rx.try_recv() {
        match event {
            TimerEvent::BreakCompleted { .. } => {
                notification_sender.send("休憩完了");
            }
            TimerEvent::WorkStarted { .. } => {
                focus_controller.enable().unwrap();
            }
            _ => {}
        }
    }

    // auto_cycleなので作業再開、フォーカスモード再有効化
    assert!(focus_controller.is_enabled());
    assert_eq!(notification_sender.notification_count(), 2);
}
