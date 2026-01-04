//! タイマーエンジン
//!
//! ポモドーロタイマーのコアロジックを提供する。
//! 状態遷移、カウントダウン、イベント発火を担当する。

use anyhow::{Context, Result};
use tokio::sync::mpsc;
use tokio::time::{interval, Duration, Interval, MissedTickBehavior};

use crate::types::{PomodoroConfig, TimerPhase, TimerState};

/// タイマーイベント
///
/// タイマーエンジンが発火するイベント。
/// 通知システム、サウンド再生、メニューバーUI更新などに使用。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimerEvent {
    /// 作業開始
    WorkStarted { task_name: Option<String> },
    /// 作業完了
    WorkCompleted {
        pomodoro_count: u32,
        task_name: Option<String>,
    },
    /// 休憩開始
    BreakStarted { is_long_break: bool },
    /// 休憩完了
    BreakCompleted { is_long_break: bool },
    /// 一時停止
    Paused,
    /// 再開
    Resumed,
    /// 停止
    Stopped,
    /// ティック（1秒経過）
    Tick { remaining_seconds: u32 },
}

/// タイマーエンジン
///
/// ポモドーロタイマーのコアロジックを担当する。
/// 1秒ごとにティックを発火し、フェーズ完了時にイベントを送信する。
///
/// # 使用方法
///
/// ```ignore
/// let (event_tx, mut event_rx) = mpsc::unbounded_channel();
/// let mut engine = TimerEngine::new(config, event_tx);
/// let mut ticker = engine.create_ticker();
///
/// // メインループ（Daemon側で実装）
/// loop {
///     tokio::select! {
///         _ = ticker.tick() => {
///             engine.process_tick()?;
///         }
///         Some(cmd) = ipc_rx.recv() => {
///             // コマンド処理
///         }
///     }
/// }
/// ```
pub struct TimerEngine {
    /// タイマー状態
    state: TimerState,
    /// イベント送信チャネル
    event_tx: mpsc::UnboundedSender<TimerEvent>,
}

impl TimerEngine {
    /// 新しいTimerEngineを作成
    pub fn new(config: PomodoroConfig, event_tx: mpsc::UnboundedSender<TimerEvent>) -> Self {
        Self {
            state: TimerState::new(config),
            event_tx,
        }
    }

    /// タイマー用のIntervalを作成
    ///
    /// 1秒間隔でティックを発生させるIntervalを返す。
    /// `MissedTickBehavior::Skip` を設定済み。
    pub fn create_ticker() -> Interval {
        let mut ticker = interval(Duration::from_secs(1));
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
        ticker
    }

    /// タイマーを開始
    pub fn start(&mut self, task_name: Option<String>) -> Result<()> {
        if self.state.is_running() {
            anyhow::bail!("タイマーは既に実行中です");
        }

        self.state.start_working(task_name.clone());

        self.event_tx
            .send(TimerEvent::WorkStarted { task_name })
            .context("Failed to send work started event")?;

        Ok(())
    }

    /// タイマーを一時停止
    pub fn pause(&mut self) -> Result<()> {
        if !self.state.is_running() {
            anyhow::bail!("タイマーは実行されていません");
        }

        self.state.pause();

        self.event_tx
            .send(TimerEvent::Paused)
            .context("Failed to send paused event")?;

        Ok(())
    }

    /// タイマーを再開
    pub fn resume(&mut self) -> Result<()> {
        if !self.state.is_paused() {
            anyhow::bail!("タイマーは一時停止していません");
        }

        self.state.resume();

        self.event_tx
            .send(TimerEvent::Resumed)
            .context("Failed to send resumed event")?;

        Ok(())
    }

    /// タイマーを停止
    pub fn stop(&mut self) -> Result<()> {
        if !self.state.is_running() && !self.state.is_paused() {
            anyhow::bail!("タイマーは実行されていません");
        }

        self.state.stop();

        self.event_tx
            .send(TimerEvent::Stopped)
            .context("Failed to send stopped event")?;

        Ok(())
    }

    /// 現在の状態を取得
    pub fn get_state(&self) -> &TimerState {
        &self.state
    }

    /// 1ティック（1秒）を処理
    ///
    /// タイマーが実行中の場合、残り時間を1秒減らし、Tickイベントを送信する。
    /// タイマーが完了した場合、フェーズ遷移を行う。
    ///
    /// # 戻り値
    ///
    /// - `Ok(true)`: タイマーが実行中でティックを処理した
    /// - `Ok(false)`: タイマーが実行中ではない（停止中または一時停止中）
    /// - `Err(...)`: イベント送信に失敗
    pub fn process_tick(&mut self) -> Result<bool> {
        if !self.state.is_running() {
            return Ok(false);
        }

        let completed = self.state.tick();

        // Tickイベントを送信
        self.event_tx
            .send(TimerEvent::Tick {
                remaining_seconds: self.state.remaining_seconds,
            })
            .context("Failed to send tick event")?;

        if completed {
            self.handle_timer_complete()?;
        }

        Ok(true)
    }

    /// タイマー完了時の処理
    fn handle_timer_complete(&mut self) -> Result<()> {
        match self.state.phase {
            TimerPhase::Working => {
                // ポモドーロカウントを増加
                self.state.pomodoro_count += 1;

                // 作業完了イベントを送信
                self.event_tx
                    .send(TimerEvent::WorkCompleted {
                        pomodoro_count: self.state.pomodoro_count,
                        task_name: self.state.task_name.clone(),
                    })
                    .context("Failed to send work completed event")?;

                // 休憩を開始
                self.state.start_breaking();

                // 休憩開始イベントを送信
                self.event_tx
                    .send(TimerEvent::BreakStarted {
                        is_long_break: self.state.phase == TimerPhase::LongBreaking,
                    })
                    .context("Failed to send break started event")?;
            }
            TimerPhase::Breaking | TimerPhase::LongBreaking => {
                let is_long_break = self.state.phase == TimerPhase::LongBreaking;

                // 休憩完了イベントを送信
                self.event_tx
                    .send(TimerEvent::BreakCompleted { is_long_break })
                    .context("Failed to send break completed event")?;

                // 自動サイクルが有効な場合は次の作業を開始
                if self.state.config.auto_cycle {
                    let task_name = self.state.task_name.clone();
                    self.state.start_working(task_name.clone());

                    self.event_tx
                        .send(TimerEvent::WorkStarted { task_name })
                        .context("Failed to send work started event")?;
                } else {
                    self.state.stop();
                }
            }
            _ => {}
        }

        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------------
    // Helper Functions
    // ------------------------------------------------------------------------

    fn create_test_engine() -> (TimerEngine, mpsc::UnboundedReceiver<TimerEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let config = PomodoroConfig::default();
        let engine = TimerEngine::new(config, tx);
        (engine, rx)
    }

    fn create_test_engine_with_config(
        config: PomodoroConfig,
    ) -> (TimerEngine, mpsc::UnboundedReceiver<TimerEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let engine = TimerEngine::new(config, tx);
        (engine, rx)
    }

    // ------------------------------------------------------------------------
    // TimerEvent Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_timer_event_equality() {
        let event1 = TimerEvent::WorkStarted {
            task_name: Some("タスク".to_string()),
        };
        let event2 = TimerEvent::WorkStarted {
            task_name: Some("タスク".to_string()),
        };
        assert_eq!(event1, event2);
    }

    #[test]
    fn test_timer_event_debug() {
        let event = TimerEvent::Tick {
            remaining_seconds: 100,
        };
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("Tick"));
        assert!(debug_str.contains("100"));
    }

    // ------------------------------------------------------------------------
    // TimerEngine Creation Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_timer_engine_new() {
        let (engine, _rx) = create_test_engine();
        let state = engine.get_state();

        assert_eq!(state.phase, TimerPhase::Stopped);
        assert_eq!(state.remaining_seconds, 0);
        assert_eq!(state.pomodoro_count, 0);
        assert!(state.task_name.is_none());
    }

    #[test]
    fn test_create_ticker() {
        let ticker = TimerEngine::create_ticker();
        // Ticker should be created successfully
        assert!(ticker.period() == Duration::from_secs(1));
    }

    // ------------------------------------------------------------------------
    // TimerEngine Start Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_timer_engine_start() {
        let (mut engine, mut rx) = create_test_engine();

        let result = engine.start(Some("テスト".to_string()));
        assert!(result.is_ok());

        let state = engine.get_state();
        assert_eq!(state.phase, TimerPhase::Working);
        assert_eq!(state.remaining_seconds, 25 * 60);
        assert_eq!(state.task_name, Some("テスト".to_string()));

        // Check event was sent
        let event = rx.try_recv().unwrap();
        assert_eq!(
            event,
            TimerEvent::WorkStarted {
                task_name: Some("テスト".to_string())
            }
        );
    }

    #[test]
    fn test_timer_engine_start_without_task_name() {
        let (mut engine, mut rx) = create_test_engine();

        let result = engine.start(None);
        assert!(result.is_ok());

        let state = engine.get_state();
        assert_eq!(state.phase, TimerPhase::Working);
        assert!(state.task_name.is_none());

        let event = rx.try_recv().unwrap();
        assert_eq!(event, TimerEvent::WorkStarted { task_name: None });
    }

    #[test]
    fn test_timer_engine_start_already_running() {
        let (mut engine, _rx) = create_test_engine();

        engine.start(None).unwrap();
        let result = engine.start(None);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("タイマーは既に実行中です"));
    }

    // ------------------------------------------------------------------------
    // TimerEngine Pause Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_timer_engine_pause() {
        let (mut engine, mut rx) = create_test_engine();

        engine.start(None).unwrap();
        rx.try_recv().unwrap(); // consume WorkStarted

        let result = engine.pause();
        assert!(result.is_ok());

        let state = engine.get_state();
        assert_eq!(state.phase, TimerPhase::Paused);

        let event = rx.try_recv().unwrap();
        assert_eq!(event, TimerEvent::Paused);
    }

    #[test]
    fn test_timer_engine_pause_not_running() {
        let (mut engine, _rx) = create_test_engine();

        let result = engine.pause();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("タイマーは実行されていません"));
    }

    // ------------------------------------------------------------------------
    // TimerEngine Resume Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_timer_engine_resume() {
        let (mut engine, mut rx) = create_test_engine();

        engine.start(None).unwrap();
        rx.try_recv().unwrap(); // consume WorkStarted
        engine.pause().unwrap();
        rx.try_recv().unwrap(); // consume Paused

        let result = engine.resume();
        assert!(result.is_ok());

        let state = engine.get_state();
        assert_eq!(state.phase, TimerPhase::Working);

        let event = rx.try_recv().unwrap();
        assert_eq!(event, TimerEvent::Resumed);
    }

    #[test]
    fn test_timer_engine_resume_not_paused() {
        let (mut engine, _rx) = create_test_engine();

        engine.start(None).unwrap();

        let result = engine.resume();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("タイマーは一時停止していません"));
    }

    // ------------------------------------------------------------------------
    // TimerEngine Stop Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_timer_engine_stop() {
        let (mut engine, mut rx) = create_test_engine();

        engine.start(Some("タスク".to_string())).unwrap();
        rx.try_recv().unwrap(); // consume WorkStarted

        let result = engine.stop();
        assert!(result.is_ok());

        let state = engine.get_state();
        assert_eq!(state.phase, TimerPhase::Stopped);
        assert_eq!(state.remaining_seconds, 0);
        assert!(state.task_name.is_none());

        let event = rx.try_recv().unwrap();
        assert_eq!(event, TimerEvent::Stopped);
    }

    #[test]
    fn test_timer_engine_stop_when_paused() {
        let (mut engine, mut rx) = create_test_engine();

        engine.start(None).unwrap();
        rx.try_recv().unwrap(); // consume WorkStarted
        engine.pause().unwrap();
        rx.try_recv().unwrap(); // consume Paused

        let result = engine.stop();
        assert!(result.is_ok());

        let state = engine.get_state();
        assert_eq!(state.phase, TimerPhase::Stopped);

        let event = rx.try_recv().unwrap();
        assert_eq!(event, TimerEvent::Stopped);
    }

    #[test]
    fn test_timer_engine_stop_not_running() {
        let (mut engine, _rx) = create_test_engine();

        let result = engine.stop();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("タイマーは実行されていません"));
    }

    // ------------------------------------------------------------------------
    // TimerEngine Process Tick Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_process_tick_when_stopped() {
        let (mut engine, _rx) = create_test_engine();

        let result = engine.process_tick();
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Returns false when not running
    }

    #[test]
    fn test_process_tick_when_paused() {
        let (mut engine, mut rx) = create_test_engine();

        engine.start(None).unwrap();
        rx.try_recv().unwrap(); // consume WorkStarted
        engine.pause().unwrap();
        rx.try_recv().unwrap(); // consume Paused

        let result = engine.process_tick();
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Returns false when paused
    }

    #[test]
    fn test_process_tick_when_running() {
        let (mut engine, mut rx) = create_test_engine();

        engine.start(None).unwrap();
        rx.try_recv().unwrap(); // consume WorkStarted

        let initial_remaining = engine.get_state().remaining_seconds;

        let result = engine.process_tick();
        assert!(result.is_ok());
        assert!(result.unwrap()); // Returns true when running

        // Check remaining time decreased
        assert_eq!(
            engine.get_state().remaining_seconds,
            initial_remaining - 1
        );

        // Check Tick event was sent
        let event = rx.try_recv().unwrap();
        assert!(matches!(event, TimerEvent::Tick { .. }));
    }

    #[test]
    fn test_process_tick_triggers_completion() {
        let (mut engine, mut rx) = create_test_engine();

        engine.start(Some("タスク".to_string())).unwrap();
        rx.try_recv().unwrap(); // consume WorkStarted

        // Set remaining_seconds to 1 so next tick will complete
        engine.state.remaining_seconds = 1;

        let result = engine.process_tick();
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Check events: Tick, WorkCompleted, BreakStarted
        let tick = rx.try_recv().unwrap();
        assert!(matches!(tick, TimerEvent::Tick { remaining_seconds: 0 }));

        let work_completed = rx.try_recv().unwrap();
        assert!(matches!(work_completed, TimerEvent::WorkCompleted { .. }));

        let break_started = rx.try_recv().unwrap();
        assert!(matches!(break_started, TimerEvent::BreakStarted { .. }));
    }

    // ------------------------------------------------------------------------
    // TimerEngine Complete Handler Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_handle_timer_complete_work_finished() {
        let (mut engine, mut rx) = create_test_engine();

        // Start working and simulate completion
        engine.start(Some("タスク".to_string())).unwrap();
        rx.try_recv().unwrap(); // consume WorkStarted

        // Manually set remaining_seconds to 0 and phase to Working
        engine.state.remaining_seconds = 0;

        let result = engine.handle_timer_complete();
        assert!(result.is_ok());

        // Check pomodoro count increased
        assert_eq!(engine.state.pomodoro_count, 1);

        // Check phase changed to Breaking
        assert_eq!(engine.state.phase, TimerPhase::Breaking);
        assert_eq!(engine.state.remaining_seconds, 5 * 60);

        // Check events
        let event1 = rx.try_recv().unwrap();
        assert_eq!(
            event1,
            TimerEvent::WorkCompleted {
                pomodoro_count: 1,
                task_name: Some("タスク".to_string())
            }
        );

        let event2 = rx.try_recv().unwrap();
        assert_eq!(
            event2,
            TimerEvent::BreakStarted {
                is_long_break: false
            }
        );
    }

    #[test]
    fn test_handle_timer_complete_long_break_after_4_pomodoros() {
        let (mut engine, mut rx) = create_test_engine();

        engine.start(None).unwrap();
        rx.try_recv().unwrap(); // consume WorkStarted

        // Set pomodoro_count to 3 (next will be 4th)
        engine.state.pomodoro_count = 3;
        engine.state.remaining_seconds = 0;

        let result = engine.handle_timer_complete();
        assert!(result.is_ok());

        // Check pomodoro count is now 4
        assert_eq!(engine.state.pomodoro_count, 4);

        // Check phase is LongBreaking
        assert_eq!(engine.state.phase, TimerPhase::LongBreaking);
        assert_eq!(engine.state.remaining_seconds, 15 * 60);

        // Check events
        let _work_completed = rx.try_recv().unwrap();
        let break_started = rx.try_recv().unwrap();
        assert_eq!(
            break_started,
            TimerEvent::BreakStarted {
                is_long_break: true
            }
        );
    }

    #[test]
    fn test_handle_timer_complete_break_finished_no_auto_cycle() {
        let (mut engine, mut rx) = create_test_engine();

        engine.start(None).unwrap();
        rx.try_recv().unwrap(); // consume WorkStarted

        // Simulate work complete and start breaking
        engine.state.pomodoro_count = 1;
        engine.state.phase = TimerPhase::Breaking;
        engine.state.remaining_seconds = 0;

        let result = engine.handle_timer_complete();
        assert!(result.is_ok());

        // Should stop (auto_cycle is false by default)
        assert_eq!(engine.state.phase, TimerPhase::Stopped);

        let event = rx.try_recv().unwrap();
        assert_eq!(
            event,
            TimerEvent::BreakCompleted {
                is_long_break: false
            }
        );
    }

    #[test]
    fn test_handle_timer_complete_break_finished_with_auto_cycle() {
        let config = PomodoroConfig {
            auto_cycle: true,
            ..Default::default()
        };
        let (mut engine, mut rx) = create_test_engine_with_config(config);

        engine.start(Some("継続タスク".to_string())).unwrap();
        rx.try_recv().unwrap(); // consume WorkStarted

        // Simulate work complete and start breaking
        engine.state.pomodoro_count = 1;
        engine.state.phase = TimerPhase::Breaking;
        engine.state.remaining_seconds = 0;

        let result = engine.handle_timer_complete();
        assert!(result.is_ok());

        // Should start working again
        assert_eq!(engine.state.phase, TimerPhase::Working);
        assert_eq!(engine.state.remaining_seconds, 25 * 60);

        // Check events
        let break_completed = rx.try_recv().unwrap();
        assert_eq!(
            break_completed,
            TimerEvent::BreakCompleted {
                is_long_break: false
            }
        );

        let work_started = rx.try_recv().unwrap();
        assert_eq!(
            work_started,
            TimerEvent::WorkStarted {
                task_name: Some("継続タスク".to_string())
            }
        );
    }

    #[test]
    fn test_handle_timer_complete_long_break_finished() {
        let (mut engine, mut rx) = create_test_engine();

        engine.start(None).unwrap();
        rx.try_recv().unwrap(); // consume WorkStarted

        // Simulate long break
        engine.state.phase = TimerPhase::LongBreaking;
        engine.state.remaining_seconds = 0;

        let result = engine.handle_timer_complete();
        assert!(result.is_ok());

        let event = rx.try_recv().unwrap();
        assert_eq!(
            event,
            TimerEvent::BreakCompleted {
                is_long_break: true
            }
        );
    }

    // ------------------------------------------------------------------------
    // Get State Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_get_state_returns_reference() {
        let (engine, _rx) = create_test_engine();
        let state = engine.get_state();

        assert_eq!(state.phase, TimerPhase::Stopped);
        assert_eq!(state.config.work_minutes, 25);
    }

    // ------------------------------------------------------------------------
    // Integration-style Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_full_pomodoro_cycle_simulation() {
        let (mut engine, mut rx) = create_test_engine();

        // Start work
        engine.start(Some("集中作業".to_string())).unwrap();
        assert_eq!(engine.get_state().phase, TimerPhase::Working);
        rx.try_recv().unwrap(); // WorkStarted

        // Simulate several ticks
        for _ in 0..5 {
            let processed = engine.process_tick().unwrap();
            assert!(processed);
            rx.try_recv().unwrap(); // Tick
        }

        // Simulate completion by setting remaining to 1 and ticking
        engine.state.remaining_seconds = 1;
        engine.process_tick().unwrap();
        rx.try_recv().unwrap(); // Tick
        rx.try_recv().unwrap(); // WorkCompleted
        rx.try_recv().unwrap(); // BreakStarted

        // Should be in break now
        assert_eq!(engine.get_state().phase, TimerPhase::Breaking);

        // Complete break
        engine.state.remaining_seconds = 1;
        engine.process_tick().unwrap();

        // Should be stopped (no auto_cycle)
        assert_eq!(engine.get_state().phase, TimerPhase::Stopped);
    }

    #[test]
    fn test_pause_resume_preserves_remaining_time() {
        let (mut engine, mut rx) = create_test_engine();

        engine.start(None).unwrap();
        rx.try_recv().unwrap(); // WorkStarted

        // Simulate some time passing
        engine.state.remaining_seconds = 1000;

        // Pause
        engine.pause().unwrap();
        rx.try_recv().unwrap(); // Paused
        assert_eq!(engine.state.remaining_seconds, 1000);

        // Resume
        engine.resume().unwrap();
        rx.try_recv().unwrap(); // Resumed
        assert_eq!(engine.state.remaining_seconds, 1000);
    }

    #[test]
    fn test_multiple_pomodoro_cycles() {
        let config = PomodoroConfig {
            auto_cycle: true,
            ..Default::default()
        };
        let (mut engine, mut rx) = create_test_engine_with_config(config);

        engine.start(None).unwrap();
        rx.try_recv().unwrap(); // WorkStarted

        // Complete 4 pomodoros
        for i in 1..=4 {
            // Complete work
            engine.state.remaining_seconds = 1;
            engine.process_tick().unwrap();
            assert_eq!(engine.state.pomodoro_count, i);

            // Drain events
            while rx.try_recv().is_ok() {}

            // Complete break
            engine.state.remaining_seconds = 1;
            engine.process_tick().unwrap();

            // Drain events
            while rx.try_recv().is_ok() {}

            // Should be working again (auto_cycle)
            assert_eq!(engine.state.phase, TimerPhase::Working);
        }

        assert_eq!(engine.state.pomodoro_count, 4);
    }

    #[test]
    fn test_interleaved_commands_and_ticks() {
        let (mut engine, mut rx) = create_test_engine();

        // Start
        engine.start(Some("タスク".to_string())).unwrap();
        rx.try_recv().unwrap(); // WorkStarted

        // Process a tick
        assert!(engine.process_tick().unwrap());
        rx.try_recv().unwrap(); // Tick

        // Pause (can be called while timer is running)
        engine.pause().unwrap();
        rx.try_recv().unwrap(); // Paused

        // Process tick while paused - should return false
        assert!(!engine.process_tick().unwrap());

        // Resume
        engine.resume().unwrap();
        rx.try_recv().unwrap(); // Resumed

        // Process tick again
        assert!(engine.process_tick().unwrap());
        rx.try_recv().unwrap(); // Tick

        // Stop
        engine.stop().unwrap();
        rx.try_recv().unwrap(); // Stopped

        // Process tick while stopped - should return false
        assert!(!engine.process_tick().unwrap());
    }
}
