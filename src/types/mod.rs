//! ポモドーロタイマーのデータ型定義
//!
//! タイマーの状態管理とIPC通信に使用するデータ型を提供する。

use serde::{Deserialize, Serialize};

/// タイマーのフェーズ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimerPhase {
    /// 停止中
    Stopped,
    /// 作業中
    Working,
    /// 短い休憩中
    Breaking,
    /// 長い休憩中
    LongBreaking,
    /// 一時停止中
    Paused,
}

impl TimerPhase {
    /// フェーズ名を取得
    pub fn as_str(&self) -> &'static str {
        match self {
            TimerPhase::Stopped => "stopped",
            TimerPhase::Working => "working",
            TimerPhase::Breaking => "breaking",
            TimerPhase::LongBreaking => "long_breaking",
            TimerPhase::Paused => "paused",
        }
    }

    /// 実行中のフェーズかどうか
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            TimerPhase::Working | TimerPhase::Breaking | TimerPhase::LongBreaking
        )
    }
}

impl std::str::FromStr for TimerPhase {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "working" => Ok(TimerPhase::Working),
            "breaking" => Ok(TimerPhase::Breaking),
            "long_breaking" => Ok(TimerPhase::LongBreaking),
            "paused" => Ok(TimerPhase::Paused),
            "stopped" => Ok(TimerPhase::Stopped),
            _ => Err(()),
        }
    }
}

/// タイマー設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomodoroConfig {
    /// 作業時間（分）: 1-120
    pub work_minutes: u32,
    /// 短い休憩時間（分）: 1-60
    pub break_minutes: u32,
    /// 長い休憩時間（分）: 1-60
    pub long_break_minutes: u32,
    /// 自動サイクル有効化
    pub auto_cycle: bool,
    /// フォーカスモード連携有効化
    pub focus_mode: bool,
}

impl Default for PomodoroConfig {
    fn default() -> Self {
        Self {
            work_minutes: 25,
            break_minutes: 5,
            long_break_minutes: 15,
            auto_cycle: false,
            focus_mode: false,
        }
    }
}

impl PomodoroConfig {
    /// 設定を検証
    pub fn validate(&self) -> Result<(), String> {
        if self.work_minutes < 1 || self.work_minutes > 120 {
            return Err("作業時間は1-120分の範囲で指定してください".to_string());
        }
        if self.break_minutes < 1 || self.break_minutes > 60 {
            return Err("休憩時間は1-60分の範囲で指定してください".to_string());
        }
        if self.long_break_minutes < 1 || self.long_break_minutes > 60 {
            return Err("長い休憩時間は1-60分の範囲で指定してください".to_string());
        }
        Ok(())
    }

    /// StartParamsから設定を更新
    ///
    /// 指定されたパラメータのみを更新する（Noneのフィールドは更新しない）
    pub fn update_from_params(&mut self, params: &StartParams) {
        if let Some(work_minutes) = params.work_minutes {
            self.work_minutes = work_minutes;
        }
        if let Some(break_minutes) = params.break_minutes {
            self.break_minutes = break_minutes;
        }
        if let Some(long_break_minutes) = params.long_break_minutes {
            self.long_break_minutes = long_break_minutes;
        }
        if let Some(auto_cycle) = params.auto_cycle {
            self.auto_cycle = auto_cycle;
        }
        if let Some(focus_mode) = params.focus_mode {
            self.focus_mode = focus_mode;
        }
    }
}

/// タイマーの現在状態
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerState {
    /// 現在のフェーズ
    pub phase: TimerPhase,
    /// 残り時間（秒）
    pub remaining_seconds: u32,
    /// 完了したポモドーロ回数
    pub pomodoro_count: u32,
    /// 現在のタスク名
    pub task_name: Option<String>,
    /// タイマー設定
    pub config: PomodoroConfig,
    /// 一時停止前のフェーズ（再開時に使用）
    #[serde(skip)]
    previous_phase: Option<TimerPhase>,
}

impl TimerState {
    /// 新しいTimerStateを作成（停止状態）
    pub fn new(config: PomodoroConfig) -> Self {
        Self {
            phase: TimerPhase::Stopped,
            remaining_seconds: 0,
            pomodoro_count: 0,
            task_name: None,
            config,
            previous_phase: None,
        }
    }

    /// 作業フェーズを開始
    pub fn start_working(&mut self, task_name: Option<String>) {
        self.phase = TimerPhase::Working;
        self.remaining_seconds = self.config.work_minutes * 60;
        self.task_name = task_name;
        self.previous_phase = None;
    }

    /// 休憩フェーズを開始
    pub fn start_breaking(&mut self) {
        // 4ポモドーロごとに長い休憩
        if self.pomodoro_count > 0 && self.pomodoro_count % 4 == 0 {
            self.phase = TimerPhase::LongBreaking;
            self.remaining_seconds = self.config.long_break_minutes * 60;
        } else {
            self.phase = TimerPhase::Breaking;
            self.remaining_seconds = self.config.break_minutes * 60;
        }
        self.previous_phase = None;
    }

    /// 一時停止
    pub fn pause(&mut self) {
        if matches!(
            self.phase,
            TimerPhase::Working | TimerPhase::Breaking | TimerPhase::LongBreaking
        ) {
            self.previous_phase = Some(self.phase);
            self.phase = TimerPhase::Paused;
        }
    }

    /// 再開
    pub fn resume(&mut self) {
        if self.phase == TimerPhase::Paused {
            if let Some(previous) = self.previous_phase.take() {
                self.phase = previous;
            } else {
                // フォールバック: 前のフェーズが不明な場合は作業中に戻す
                self.phase = TimerPhase::Working;
            }
        }
    }

    /// 停止
    pub fn stop(&mut self) {
        self.phase = TimerPhase::Stopped;
        self.remaining_seconds = 0;
        self.task_name = None;
        self.previous_phase = None;
    }

    /// 1秒経過
    /// 戻り値: タイマーが完了したかどうか
    pub fn tick(&mut self) -> bool {
        if self.remaining_seconds > 0 {
            self.remaining_seconds -= 1;
        }
        self.remaining_seconds == 0
    }

    /// 実行中かどうか
    pub fn is_running(&self) -> bool {
        matches!(
            self.phase,
            TimerPhase::Working | TimerPhase::Breaking | TimerPhase::LongBreaking
        )
    }

    /// 一時停止中かどうか
    pub fn is_paused(&self) -> bool {
        self.phase == TimerPhase::Paused
    }

    /// 現在のフェーズの合計時間（秒）を取得
    pub fn current_duration(&self) -> u32 {
        let phase = if self.phase == TimerPhase::Paused {
            self.previous_phase.unwrap_or(TimerPhase::Stopped)
        } else {
            self.phase
        };

        match phase {
            TimerPhase::Working => self.config.work_minutes * 60,
            TimerPhase::Breaking => self.config.break_minutes * 60,
            TimerPhase::LongBreaking => self.config.long_break_minutes * 60,
            _ => 0,
        }
    }
}

// ============================================================================
// IPC Types
// ============================================================================

/// IPCリクエスト
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "lowercase")]
pub enum IpcRequest {
    /// タイマー開始
    Start {
        #[serde(flatten)]
        params: StartParams,
    },
    /// タイマー一時停止
    Pause,
    /// タイマー再開
    Resume,
    /// タイマー停止
    Stop,
    /// ステータス確認
    Status,
}

/// 開始パラメータ
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StartParams {
    #[serde(rename = "workMinutes", skip_serializing_if = "Option::is_none")]
    pub work_minutes: Option<u32>,
    #[serde(rename = "breakMinutes", skip_serializing_if = "Option::is_none")]
    pub break_minutes: Option<u32>,
    #[serde(rename = "longBreakMinutes", skip_serializing_if = "Option::is_none")]
    pub long_break_minutes: Option<u32>,
    #[serde(rename = "taskName", skip_serializing_if = "Option::is_none")]
    pub task_name: Option<String>,
    #[serde(rename = "autoCycle", skip_serializing_if = "Option::is_none")]
    pub auto_cycle: Option<bool>,
    #[serde(rename = "focusMode", skip_serializing_if = "Option::is_none")]
    pub focus_mode: Option<bool>,
}

/// IPCレスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcResponse {
    /// ステータス（success/error）
    pub status: String,
    /// メッセージ
    pub message: String,
    /// データ
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<ResponseData>,
}

/// レスポンスデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(rename = "remainingSeconds", skip_serializing_if = "Option::is_none")]
    pub remaining_seconds: Option<u32>,
    #[serde(rename = "pomodoroCount", skip_serializing_if = "Option::is_none")]
    pub pomodoro_count: Option<u32>,
    #[serde(rename = "taskName", skip_serializing_if = "Option::is_none")]
    pub task_name: Option<String>,
    #[serde(rename = "duration", skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
}

impl IpcResponse {
    /// 成功レスポンスを作成
    pub fn success(message: impl Into<String>, data: Option<ResponseData>) -> Self {
        Self {
            status: "success".to_string(),
            message: message.into(),
            data,
        }
    }

    /// エラーレスポンスを作成
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            status: "error".to_string(),
            message: message.into(),
            data: None,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------------
    // TimerPhase Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_timer_phase_as_str() {
        assert_eq!(TimerPhase::Stopped.as_str(), "stopped");
        assert_eq!(TimerPhase::Working.as_str(), "working");
        assert_eq!(TimerPhase::Breaking.as_str(), "breaking");
        assert_eq!(TimerPhase::LongBreaking.as_str(), "long_breaking");
        assert_eq!(TimerPhase::Paused.as_str(), "paused");
    }

    #[test]
    fn test_timer_phase_is_active() {
        assert!(!TimerPhase::Stopped.is_active());
        assert!(TimerPhase::Working.is_active());
        assert!(TimerPhase::Breaking.is_active());
        assert!(TimerPhase::LongBreaking.is_active());
        assert!(!TimerPhase::Paused.is_active());
    }

    #[test]
    fn test_timer_phase_serialize() {
        let json = serde_json::to_string(&TimerPhase::Working).unwrap();
        assert_eq!(json, "\"working\"");
    }

    #[test]
    fn test_timer_phase_deserialize() {
        let phase: TimerPhase = serde_json::from_str("\"breaking\"").unwrap();
        assert_eq!(phase, TimerPhase::Breaking);
    }

    // ------------------------------------------------------------------------
    // PomodoroConfig Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_pomodoro_config_default() {
        let config = PomodoroConfig::default();
        assert_eq!(config.work_minutes, 25);
        assert_eq!(config.break_minutes, 5);
        assert_eq!(config.long_break_minutes, 15);
        assert!(!config.auto_cycle);
        assert!(!config.focus_mode);
    }

    #[test]
    fn test_pomodoro_config_validate_success() {
        let config = PomodoroConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_pomodoro_config_validate_work_minutes_too_low() {
        let config = PomodoroConfig {
            work_minutes: 0,
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "作業時間は1-120分の範囲で指定してください"
        );
    }

    #[test]
    fn test_pomodoro_config_validate_work_minutes_too_high() {
        let config = PomodoroConfig {
            work_minutes: 121,
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "作業時間は1-120分の範囲で指定してください"
        );
    }

    #[test]
    fn test_pomodoro_config_validate_break_minutes_too_low() {
        let config = PomodoroConfig {
            break_minutes: 0,
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "休憩時間は1-60分の範囲で指定してください"
        );
    }

    #[test]
    fn test_pomodoro_config_validate_break_minutes_too_high() {
        let config = PomodoroConfig {
            break_minutes: 61,
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "休憩時間は1-60分の範囲で指定してください"
        );
    }

    #[test]
    fn test_pomodoro_config_validate_long_break_minutes_too_low() {
        let config = PomodoroConfig {
            long_break_minutes: 0,
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "長い休憩時間は1-60分の範囲で指定してください"
        );
    }

    #[test]
    fn test_pomodoro_config_validate_long_break_minutes_too_high() {
        let config = PomodoroConfig {
            long_break_minutes: 61,
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "長い休憩時間は1-60分の範囲で指定してください"
        );
    }

    // ------------------------------------------------------------------------
    // TimerState Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_timer_state_new() {
        let config = PomodoroConfig::default();
        let state = TimerState::new(config);
        assert_eq!(state.phase, TimerPhase::Stopped);
        assert_eq!(state.remaining_seconds, 0);
        assert_eq!(state.pomodoro_count, 0);
        assert_eq!(state.task_name, None);
    }

    #[test]
    fn test_timer_state_start_working() {
        let config = PomodoroConfig::default();
        let mut state = TimerState::new(config);
        state.start_working(Some("API実装".to_string()));

        assert_eq!(state.phase, TimerPhase::Working);
        assert_eq!(state.remaining_seconds, 25 * 60); // 1500 seconds
        assert_eq!(state.task_name, Some("API実装".to_string()));
    }

    #[test]
    fn test_timer_state_start_breaking_normal() {
        let config = PomodoroConfig::default();
        let mut state = TimerState::new(config);
        state.pomodoro_count = 1;
        state.start_breaking();

        assert_eq!(state.phase, TimerPhase::Breaking);
        assert_eq!(state.remaining_seconds, 5 * 60); // 300 seconds
    }

    #[test]
    fn test_timer_state_start_breaking_long() {
        let config = PomodoroConfig::default();
        let mut state = TimerState::new(config);
        state.pomodoro_count = 4;
        state.start_breaking();

        assert_eq!(state.phase, TimerPhase::LongBreaking);
        assert_eq!(state.remaining_seconds, 15 * 60); // 900 seconds
    }

    #[test]
    fn test_timer_state_pause() {
        let config = PomodoroConfig::default();
        let mut state = TimerState::new(config);
        state.start_working(None);
        state.remaining_seconds = 1000;
        state.pause();

        assert_eq!(state.phase, TimerPhase::Paused);
        assert_eq!(state.remaining_seconds, 1000); // 残り時間は保持
    }

    #[test]
    fn test_timer_state_resume() {
        let config = PomodoroConfig::default();
        let mut state = TimerState::new(config);
        state.start_working(None);
        state.remaining_seconds = 1000;
        state.pause();
        state.resume();

        assert_eq!(state.phase, TimerPhase::Working);
        assert_eq!(state.remaining_seconds, 1000); // 残り時間は保持
    }

    #[test]
    fn test_timer_state_stop() {
        let config = PomodoroConfig::default();
        let mut state = TimerState::new(config);
        state.start_working(Some("タスク".to_string()));
        state.remaining_seconds = 1000;
        state.stop();

        assert_eq!(state.phase, TimerPhase::Stopped);
        assert_eq!(state.remaining_seconds, 0);
        assert_eq!(state.task_name, None);
    }

    #[test]
    fn test_timer_state_tick_not_completed() {
        let config = PomodoroConfig::default();
        let mut state = TimerState::new(config);
        state.remaining_seconds = 10;

        let completed = state.tick();

        assert!(!completed);
        assert_eq!(state.remaining_seconds, 9);
    }

    #[test]
    fn test_timer_state_tick_completed() {
        let config = PomodoroConfig::default();
        let mut state = TimerState::new(config);
        state.remaining_seconds = 1;

        let completed = state.tick();

        assert!(completed);
        assert_eq!(state.remaining_seconds, 0);
    }

    #[test]
    fn test_timer_state_is_running() {
        let config = PomodoroConfig::default();
        let mut state = TimerState::new(config);

        assert!(!state.is_running());

        state.start_working(None);
        assert!(state.is_running());

        state.pause();
        assert!(!state.is_running());
    }

    #[test]
    fn test_timer_state_is_paused() {
        let config = PomodoroConfig::default();
        let mut state = TimerState::new(config);

        assert!(!state.is_paused());

        state.start_working(None);
        state.pause();
        assert!(state.is_paused());

        state.resume();
        assert!(!state.is_paused());
    }

    // ------------------------------------------------------------------------
    // IPC Types Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_ipc_request_start_serialize() {
        let request = IpcRequest::Start {
            params: StartParams {
                work_minutes: Some(30),
                task_name: Some("テスト".to_string()),
                ..Default::default()
            },
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"command\":\"start\""));
        assert!(json.contains("\"workMinutes\":30"));
        assert!(json.contains("\"taskName\":\"テスト\""));
    }

    #[test]
    fn test_ipc_request_pause_deserialize() {
        let json = r#"{"command":"pause"}"#;
        let request: IpcRequest = serde_json::from_str(json).unwrap();
        assert!(matches!(request, IpcRequest::Pause));
    }

    #[test]
    fn test_ipc_request_status_serialize() {
        let request = IpcRequest::Status;
        let json = serde_json::to_string(&request).unwrap();
        assert_eq!(json, r#"{"command":"status"}"#);
    }

    #[test]
    fn test_ipc_response_success() {
        let data = ResponseData {
            state: Some("working".to_string()),
            remaining_seconds: Some(1500),
            pomodoro_count: Some(3),
            task_name: Some("開発".to_string()),
            duration: Some(1500),
        };
        let response = IpcResponse::success("タイマーを開始しました", Some(data));

        assert_eq!(response.status, "success");
        assert_eq!(response.message, "タイマーを開始しました");
        assert!(response.data.is_some());
    }

    #[test]
    fn test_ipc_response_error() {
        let response = IpcResponse::error("タイマーは既に実行中です");

        assert_eq!(response.status, "error");
        assert_eq!(response.message, "タイマーは既に実行中です");
        assert!(response.data.is_none());
    }

    #[test]
    fn test_ipc_response_serialize() {
        let response = IpcResponse::success("OK", None);
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("\"status\":\"success\""));
        assert!(json.contains("\"message\":\"OK\""));
        // data should be omitted when None
        assert!(!json.contains("\"data\""));
    }

    #[test]
    fn test_start_params_default() {
        let params = StartParams::default();
        assert!(params.work_minutes.is_none());
        assert!(params.break_minutes.is_none());
        assert!(params.long_break_minutes.is_none());
        assert!(params.task_name.is_none());
        assert!(params.auto_cycle.is_none());
        assert!(params.focus_mode.is_none());
    }

    // ------------------------------------------------------------------------
    // PomodoroConfig update_from_params Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_pomodoro_config_update_from_params_work_minutes() {
        let mut config = PomodoroConfig::default();
        let params = StartParams {
            work_minutes: Some(2),
            ..Default::default()
        };

        config.update_from_params(&params);

        assert_eq!(config.work_minutes, 2);
        assert_eq!(config.break_minutes, 5);
        assert_eq!(config.long_break_minutes, 15);
    }

    #[test]
    fn test_pomodoro_config_update_from_params_all_fields() {
        let mut config = PomodoroConfig::default();
        let params = StartParams {
            work_minutes: Some(30),
            break_minutes: Some(10),
            long_break_minutes: Some(20),
            auto_cycle: Some(true),
            focus_mode: Some(true),
            task_name: Some("テスト".to_string()),
        };

        config.update_from_params(&params);

        assert_eq!(config.work_minutes, 30);
        assert_eq!(config.break_minutes, 10);
        assert_eq!(config.long_break_minutes, 20);
        assert!(config.auto_cycle);
        assert!(config.focus_mode);
    }

    #[test]
    fn test_pomodoro_config_update_from_params_empty() {
        let mut config = PomodoroConfig::default();
        let original = config.clone();
        let params = StartParams::default();

        config.update_from_params(&params);

        assert_eq!(config.work_minutes, original.work_minutes);
        assert_eq!(config.break_minutes, original.break_minutes);
        assert_eq!(config.long_break_minutes, original.long_break_minutes);
        assert_eq!(config.auto_cycle, original.auto_cycle);
        assert_eq!(config.focus_mode, original.focus_mode);
    }

    #[test]
    fn test_pomodoro_config_update_from_params_partial() {
        let mut config = PomodoroConfig {
            work_minutes: 25,
            break_minutes: 5,
            long_break_minutes: 15,
            auto_cycle: false,
            focus_mode: true,
        };
        let params = StartParams {
            work_minutes: Some(10),
            focus_mode: Some(false),
            ..Default::default()
        };

        config.update_from_params(&params);

        assert_eq!(config.work_minutes, 10);
        assert_eq!(config.break_minutes, 5);
        assert_eq!(config.long_break_minutes, 15);
        assert!(!config.auto_cycle);
        assert!(!config.focus_mode);
    }
}
