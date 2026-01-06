//! フック設定管理モジュール
//!
//! フック設定ファイル (`~/.pomodoro/hooks.json`) の読み込み・検証を担当する。

use crate::types::HookEvent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// 許可されるイベント名
const VALID_EVENTS: &[&str] = &[
    "work_start",
    "work_end",
    "break_start",
    "break_end",
    "long_break_start",
    "long_break_end",
    "pause",
    "resume",
    "stop",
];

/// 1イベントあたりの最大フック数
const MAX_HOOKS_PER_EVENT: usize = 10;

/// タイムアウトの最小値（秒）
const MIN_TIMEOUT_SECS: u64 = 1;

/// タイムアウトの最大値（秒）
const MAX_TIMEOUT_SECS: u64 = 300;

/// フック設定エラー
#[derive(Debug, Error)]
pub enum HookConfigError {
    /// E030: 設定ファイルが見つかりません
    #[error("[E030] 設定ファイルが見つかりません: {0}")]
    FileNotFound(PathBuf),

    /// E031: 設定ファイルの解析に失敗しました
    #[error("[E031] 設定ファイルの解析に失敗しました: {0}")]
    ParseError(String),

    /// バリデーションエラー
    #[error("バリデーションエラー: {0}")]
    ValidationError(String),

    /// IOエラー
    #[error("IOエラー: {0}")]
    IoError(#[from] std::io::Error),
}

/// フック設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookConfig {
    /// 設定ファイルのバージョン
    #[serde(default = "default_version")]
    pub version: String,

    /// フック定義のリスト
    #[serde(default)]
    pub hooks: Vec<HookDefinition>,

    /// デフォルト設定
    #[serde(default)]
    pub defaults: HookDefaults,

    /// 内部用: イベント名 -> フック定義リストのマップ（検証・正規化後に構築）
    #[serde(skip)]
    hooks_by_event: HashMap<String, Vec<HookDefinition>>,
}

fn default_version() -> String {
    "1.0".to_string()
}

/// フック定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookDefinition {
    /// フック名
    pub name: String,

    /// トリガーするイベント名
    pub event: String,

    /// 実行するスクリプトのパス
    pub script: PathBuf,

    /// タイムアウト（秒）
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    /// 有効かどうか
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_timeout() -> u64 {
    30
}

fn default_enabled() -> bool {
    true
}

/// デフォルト設定
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HookDefaults {
    /// デフォルトタイムアウト（秒）
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

impl Default for HookConfig {
    fn default() -> Self {
        Self {
            version: default_version(),
            hooks: Vec::new(),
            defaults: HookDefaults::default(),
            hooks_by_event: HashMap::new(),
        }
    }
}

impl HookConfig {
    /// デフォルトパス (`~/.pomodoro/hooks.json`) から設定を読み込む
    ///
    /// ファイルが存在しない場合は `Err(HookConfigError::FileNotFound)` を返す。
    pub fn load() -> Result<Self, HookConfigError> {
        let path = Self::default_config_path()?;
        Self::load_from_path(&path)
    }

    /// 指定されたパスから設定を読み込む
    pub fn load_from_path(path: &Path) -> Result<Self, HookConfigError> {
        if !path.exists() {
            return Err(HookConfigError::FileNotFound(path.to_path_buf()));
        }

        let content = fs::read_to_string(path)?;
        Self::parse_and_validate(&content)
    }

    /// JSON文字列をパースして検証する
    pub fn parse_and_validate(content: &str) -> Result<Self, HookConfigError> {
        let mut config: HookConfig = serde_json::from_str(content)
            .map_err(|e| HookConfigError::ParseError(e.to_string()))?;

        config.validate()?;
        config.normalize_paths()?;
        config.build_event_map();

        Ok(config)
    }

    /// デフォルトの設定ファイルパスを取得
    pub fn default_config_path() -> Result<PathBuf, HookConfigError> {
        dirs::home_dir()
            .map(|h| h.join(".pomodoro").join("hooks.json"))
            .ok_or_else(|| {
                HookConfigError::IoError(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "ホームディレクトリが見つかりません",
                ))
            })
    }

    /// 設定を検証する
    fn validate(&self) -> Result<(), HookConfigError> {
        // イベントごとのフック数をカウント
        let mut event_counts: HashMap<&str, usize> = HashMap::new();

        for hook in &self.hooks {
            // イベント名の検証
            if !VALID_EVENTS.contains(&hook.event.as_str()) {
                return Err(HookConfigError::ValidationError(format!(
                    "無効なイベント名: '{}'. 許可されるイベント: {:?}",
                    hook.event, VALID_EVENTS
                )));
            }

            // タイムアウトの検証
            if hook.timeout_secs < MIN_TIMEOUT_SECS || hook.timeout_secs > MAX_TIMEOUT_SECS {
                return Err(HookConfigError::ValidationError(format!(
                    "フック '{}' のタイムアウト値 {} が範囲外です (許可: {}-{}秒)",
                    hook.name, hook.timeout_secs, MIN_TIMEOUT_SECS, MAX_TIMEOUT_SECS
                )));
            }

            // イベントごとのフック数をカウント
            *event_counts.entry(hook.event.as_str()).or_insert(0) += 1;
        }

        // 1イベントあたりの最大フック数チェック
        for (event, count) in &event_counts {
            if *count > MAX_HOOKS_PER_EVENT {
                return Err(HookConfigError::ValidationError(format!(
                    "イベント '{}' のフック数 {} が上限 {} を超えています",
                    event, count, MAX_HOOKS_PER_EVENT
                )));
            }
        }

        // デフォルトタイムアウトの検証
        if self.defaults.timeout_secs < MIN_TIMEOUT_SECS
            || self.defaults.timeout_secs > MAX_TIMEOUT_SECS
        {
            return Err(HookConfigError::ValidationError(format!(
                "デフォルトタイムアウト値 {} が範囲外です (許可: {}-{}秒)",
                self.defaults.timeout_secs, MIN_TIMEOUT_SECS, MAX_TIMEOUT_SECS
            )));
        }

        Ok(())
    }

    /// スクリプトパスを正規化する（~展開、絶対パス化）
    fn normalize_paths(&mut self) -> Result<(), HookConfigError> {
        for hook in &mut self.hooks {
            hook.script = Self::normalize_path(&hook.script)?;
        }
        Ok(())
    }

    /// パスを正規化する
    fn normalize_path(path: &Path) -> Result<PathBuf, HookConfigError> {
        let path_str = path.to_string_lossy();

        // チルダ展開
        let expanded = if path_str.starts_with("~/") {
            let home = dirs::home_dir().ok_or_else(|| {
                HookConfigError::IoError(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "ホームディレクトリが見つかりません",
                ))
            })?;
            home.join(&path_str[2..])
        } else {
            path.to_path_buf()
        };

        // 絶対パス化（既に絶対パスならそのまま）
        if expanded.is_absolute() {
            Ok(expanded)
        } else {
            // 相対パスの場合はエラー（セキュリティ対策）
            Err(HookConfigError::ValidationError(format!(
                "スクリプトパスは絶対パスまたは~/で始まる必要があります: {:?}",
                path
            )))
        }
    }

    /// イベントごとのフックマップを構築
    fn build_event_map(&mut self) {
        self.hooks_by_event.clear();
        for hook in &self.hooks {
            if hook.enabled {
                self.hooks_by_event
                    .entry(hook.event.clone())
                    .or_default()
                    .push(hook.clone());
            }
        }
    }

    /// 指定されたイベントに対応するフックを取得
    pub fn get_hooks_for_event(&self, event: &HookEvent) -> Option<&Vec<HookDefinition>> {
        self.hooks_by_event.get(event.as_str())
    }

    /// 指定されたイベント名に対応するフックを取得
    pub fn get_hooks_for_event_name(&self, event_name: &str) -> Option<&Vec<HookDefinition>> {
        self.hooks_by_event.get(event_name)
    }

    /// 有効なフックが存在するかどうか
    pub fn has_hooks(&self) -> bool {
        !self.hooks_by_event.is_empty()
    }

    /// デフォルトタイムアウトを取得
    pub fn default_timeout(&self) -> u64 {
        self.defaults.timeout_secs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_config(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file
    }

    #[test]
    fn test_parse_valid_config() {
        let json = r#"{
            "version": "1.0",
            "hooks": [
                {
                    "name": "slack-notify",
                    "event": "work_end",
                    "script": "/usr/local/bin/notify.sh",
                    "timeout_secs": 30,
                    "enabled": true
                }
            ],
            "defaults": {
                "timeout_secs": 30
            }
        }"#;

        let config = HookConfig::parse_and_validate(json).unwrap();
        assert_eq!(config.version, "1.0");
        assert_eq!(config.hooks.len(), 1);
        assert_eq!(config.hooks[0].name, "slack-notify");
        assert_eq!(config.hooks[0].event, "work_end");
    }

    #[test]
    fn test_parse_invalid_event() {
        let json = r#"{
            "version": "1.0",
            "hooks": [
                {
                    "name": "test",
                    "event": "invalid_event",
                    "script": "/usr/local/bin/test.sh"
                }
            ]
        }"#;

        let result = HookConfig::parse_and_validate(json);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("無効なイベント名"));
    }

    #[test]
    fn test_parse_timeout_too_low() {
        let json = r#"{
            "version": "1.0",
            "hooks": [
                {
                    "name": "test",
                    "event": "work_end",
                    "script": "/usr/local/bin/test.sh",
                    "timeout_secs": 0
                }
            ]
        }"#;

        let result = HookConfig::parse_and_validate(json);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("タイムアウト値"));
    }

    #[test]
    fn test_parse_timeout_too_high() {
        let json = r#"{
            "version": "1.0",
            "hooks": [
                {
                    "name": "test",
                    "event": "work_end",
                    "script": "/usr/local/bin/test.sh",
                    "timeout_secs": 500
                }
            ]
        }"#;

        let result = HookConfig::parse_and_validate(json);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("タイムアウト値"));
    }

    #[test]
    fn test_parse_too_many_hooks_per_event() {
        let hooks: Vec<String> = (0..11)
            .map(|i| {
                format!(
                    r#"{{
                    "name": "hook{}",
                    "event": "work_end",
                    "script": "/usr/local/bin/hook{}.sh"
                }}"#,
                    i, i
                )
            })
            .collect();

        let json = format!(
            r#"{{
            "version": "1.0",
            "hooks": [{}]
        }}"#,
            hooks.join(",")
        );

        let result = HookConfig::parse_and_validate(&json);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("上限"));
    }

    #[test]
    fn test_load_from_file() {
        let json = r#"{
            "version": "1.0",
            "hooks": [
                {
                    "name": "test",
                    "event": "work_start",
                    "script": "/usr/local/bin/test.sh"
                }
            ]
        }"#;

        let file = create_temp_config(json);
        let config = HookConfig::load_from_path(file.path()).unwrap();
        assert_eq!(config.hooks.len(), 1);
    }

    #[test]
    fn test_file_not_found_error() {
        let result = HookConfig::load_from_path(Path::new("/nonexistent/path/hooks.json"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("[E030]"));
    }

    #[test]
    fn test_parse_error() {
        let invalid_json = "{ invalid json }";
        let result = HookConfig::parse_and_validate(invalid_json);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("[E031]"));
    }

    #[test]
    fn test_normalize_tilde_path() {
        let path = Path::new("~/scripts/test.sh");
        let normalized = HookConfig::normalize_path(path).unwrap();
        assert!(normalized.is_absolute());
        assert!(normalized.to_string_lossy().contains("scripts/test.sh"));
    }

    #[test]
    fn test_reject_relative_path() {
        let path = Path::new("scripts/test.sh");
        let result = HookConfig::normalize_path(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_default_values() {
        let json = r#"{
            "hooks": [
                {
                    "name": "test",
                    "event": "work_end",
                    "script": "/usr/local/bin/test.sh"
                }
            ]
        }"#;

        let config = HookConfig::parse_and_validate(json).unwrap();
        assert_eq!(config.version, "1.0");
        assert_eq!(config.hooks[0].timeout_secs, 30);
        assert!(config.hooks[0].enabled);
    }

    #[test]
    fn test_get_hooks_for_event() {
        let json = r#"{
            "version": "1.0",
            "hooks": [
                {
                    "name": "hook1",
                    "event": "work_end",
                    "script": "/usr/local/bin/hook1.sh"
                },
                {
                    "name": "hook2",
                    "event": "work_start",
                    "script": "/usr/local/bin/hook2.sh"
                }
            ]
        }"#;

        let config = HookConfig::parse_and_validate(json).unwrap();
        let work_end_hooks = config.get_hooks_for_event(&HookEvent::WorkEnd);
        assert!(work_end_hooks.is_some());
        assert_eq!(work_end_hooks.unwrap().len(), 1);
    }

    #[test]
    fn test_disabled_hooks_not_in_map() {
        let json = r#"{
            "version": "1.0",
            "hooks": [
                {
                    "name": "disabled-hook",
                    "event": "work_end",
                    "script": "/usr/local/bin/hook.sh",
                    "enabled": false
                }
            ]
        }"#;

        let config = HookConfig::parse_and_validate(json).unwrap();
        let hooks = config.get_hooks_for_event(&HookEvent::WorkEnd);
        assert!(hooks.is_none());
    }

    #[test]
    fn test_all_valid_events() {
        for event in VALID_EVENTS {
            let json = format!(
                r#"{{
                "version": "1.0",
                "hooks": [
                    {{
                        "name": "test",
                        "event": "{}",
                        "script": "/usr/local/bin/test.sh"
                    }}
                ]
            }}"#,
                event
            );

            let result = HookConfig::parse_and_validate(&json);
            assert!(result.is_ok(), "Event '{}' should be valid", event);
        }
    }

    #[test]
    fn test_has_hooks() {
        let json = r#"{
            "version": "1.0",
            "hooks": [
                {
                    "name": "test",
                    "event": "work_end",
                    "script": "/usr/local/bin/test.sh"
                }
            ]
        }"#;

        let config = HookConfig::parse_and_validate(json).unwrap();
        assert!(config.has_hooks());

        let empty_config = HookConfig::default();
        assert!(!empty_config.has_hooks());
    }
}
