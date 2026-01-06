use crate::types::HookEvent;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

/// フック実行コンテキスト
///
/// スクリプト実行時に環境変数として渡される情報
#[derive(Debug, Clone)]
pub struct HookContext {
    /// 発生したイベント
    pub event: HookEvent,
    /// タスク名（あれば）
    pub task_name: Option<String>,
    /// 現在のフェーズ
    pub phase: String,
    /// フェーズの合計時間（秒）
    pub duration_secs: u64,
    /// 経過時間（秒）
    pub elapsed_secs: u64,
    /// 残り時間（秒）
    pub remaining_secs: u64,
    /// 現在のサイクル数
    pub cycle: u32,
    /// 合計サイクル数
    pub total_cycles: u32,
    /// イベント発生時刻
    pub timestamp: DateTime<Utc>,
    /// セッションID
    pub session_id: Uuid,
}

impl HookContext {
    /// 環境変数マップに変換
    pub fn to_env_vars(&self) -> HashMap<String, String> {
        let mut vars = HashMap::new();

        vars.insert(
            "POMODORO_EVENT".to_string(),
            self.event.as_str().to_string(),
        );

        if let Some(ref name) = self.task_name {
            vars.insert("POMODORO_TASK_NAME".to_string(), name.clone());
        }

        vars.insert("POMODORO_PHASE".to_string(), self.phase.clone());
        vars.insert(
            "POMODORO_DURATION_SECS".to_string(),
            self.duration_secs.to_string(),
        );
        vars.insert(
            "POMODORO_ELAPSED_SECS".to_string(),
            self.elapsed_secs.to_string(),
        );
        vars.insert(
            "POMODORO_REMAINING_SECS".to_string(),
            self.remaining_secs.to_string(),
        );
        vars.insert("POMODORO_CYCLE".to_string(), self.cycle.to_string());
        vars.insert(
            "POMODORO_TOTAL_CYCLES".to_string(),
            self.total_cycles.to_string(),
        );
        vars.insert(
            "POMODORO_TIMESTAMP".to_string(),
            self.timestamp.to_rfc3339(),
        );
        vars.insert(
            "POMODORO_SESSION_ID".to_string(),
            self.session_id.to_string(),
        );

        vars
    }
}
