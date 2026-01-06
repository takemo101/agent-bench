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
            vars.insert("POMODORO_TASK_NAME".to_string(), Self::sanitize_value(name));
        }

        vars.insert(
            "POMODORO_PHASE".to_string(),
            Self::sanitize_value(&self.phase),
        );
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

    /// 値をサニタイズする（シェルで安全に使用できるように）
    fn sanitize_value(value: &str) -> String {
        value
            .chars()
            .filter(|c| c.is_alphanumeric() || [' ', '-', '_', '.', ':', '/'].contains(c))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use std::str::FromStr;

    fn create_test_context() -> HookContext {
        HookContext {
            event: HookEvent::WorkStart,
            task_name: Some("Test Task".to_string()),
            phase: "Work".to_string(),
            duration_secs: 1500,
            elapsed_secs: 0,
            remaining_secs: 1500,
            cycle: 1,
            total_cycles: 4,
            timestamp: Utc.timestamp_opt(1672531200, 0).unwrap(), // 2023-01-01 00:00:00 UTC
            session_id: Uuid::from_str("00000000-0000-0000-0000-000000000000").unwrap(),
        }
    }

    #[test]
    fn test_to_env_vars_basic() {
        let context = create_test_context();
        let vars = context.to_env_vars();

        assert_eq!(vars.get("POMODORO_EVENT"), Some(&"work_start".to_string()));
        assert_eq!(
            vars.get("POMODORO_TASK_NAME"),
            Some(&"Test Task".to_string())
        );
        assert_eq!(vars.get("POMODORO_PHASE"), Some(&"Work".to_string()));
        assert_eq!(
            vars.get("POMODORO_DURATION_SECS"),
            Some(&"1500".to_string())
        );
        assert_eq!(vars.get("POMODORO_ELAPSED_SECS"), Some(&"0".to_string()));
        assert_eq!(
            vars.get("POMODORO_REMAINING_SECS"),
            Some(&"1500".to_string())
        );
        assert_eq!(vars.get("POMODORO_CYCLE"), Some(&"1".to_string()));
        assert_eq!(vars.get("POMODORO_TOTAL_CYCLES"), Some(&"4".to_string()));
        assert_eq!(
            vars.get("POMODORO_TIMESTAMP"),
            Some(&"2023-01-01T00:00:00+00:00".to_string())
        );
        assert_eq!(
            vars.get("POMODORO_SESSION_ID"),
            Some(&"00000000-0000-0000-0000-000000000000".to_string())
        );
    }

    #[test]
    fn test_to_env_vars_no_task_name() {
        let mut context = create_test_context();
        context.task_name = None;
        let vars = context.to_env_vars();

        assert!(!vars.contains_key("POMODORO_TASK_NAME"));
    }

    #[test]
    fn test_sanitize_values() {
        let mut context = create_test_context();
        context.task_name = Some("Task; rm -rf /".to_string());
        context.phase = "Phase `echo hack`".to_string();

        let vars = context.to_env_vars();

        // セミコロンやバッククォートが除去されていることを確認
        assert_eq!(
            vars.get("POMODORO_TASK_NAME"),
            Some(&"Task rm -rf /".to_string())
        );
        assert_eq!(
            vars.get("POMODORO_PHASE"),
            Some(&"Phase echo hack".to_string())
        );
    }

    #[test]
    fn test_sanitize_shell_vars() {
        let mut context = create_test_context();
        context.task_name = Some("$VAR ${VAR}".to_string());

        let vars = context.to_env_vars();

        // $ や {} が除去されていること
        assert_eq!(vars.get("POMODORO_TASK_NAME"), Some(&"VAR VAR".to_string()));
    }
}
