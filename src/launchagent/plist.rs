//! Plist構造体定義とXML生成
//!
//! LaunchAgent plistファイルの構造体定義とXMLシリアライズ機能を提供する。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::error::{LaunchAgentError, Result};

/// LaunchAgentのデフォルトラベル
pub const DEFAULT_LABEL: &str = "com.example.pomodoro";

/// LaunchAgent plist構造体
///
/// macOSのLaunchAgentに必要な設定項目を定義する。
/// すべてのパスは絶対パスで指定する必要がある（`~`は使用不可）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PomodoroLaunchAgent {
    /// サービスを一意に識別するラベル（逆ドメイン形式）
    #[serde(rename = "Label")]
    pub label: String,

    /// 実行するプログラムとその引数
    #[serde(rename = "ProgramArguments")]
    pub program_arguments: Vec<String>,

    /// ログイン時に自動起動するか
    #[serde(rename = "RunAtLoad")]
    pub run_at_load: bool,

    /// プロセスが終了した際に自動再起動するか
    #[serde(rename = "KeepAlive")]
    pub keep_alive: bool,

    /// 標準出力のログファイルパス
    #[serde(rename = "StandardOutPath")]
    pub standard_out_path: String,

    /// 標準エラー出力のログファイルパス
    #[serde(rename = "StandardErrorPath")]
    pub standard_error_path: String,

    /// 作業ディレクトリ（オプション）
    #[serde(rename = "WorkingDirectory", skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,

    /// 環境変数（オプション）
    #[serde(
        rename = "EnvironmentVariables",
        skip_serializing_if = "Option::is_none"
    )]
    pub environment_variables: Option<HashMap<String, String>>,
}

impl PomodoroLaunchAgent {
    /// 新しいLaunchAgent設定を作成
    ///
    /// # Arguments
    /// * `binary_path` - pomodoroバイナリの絶対パス（例: "/usr/local/bin/pomodoro"）
    /// * `log_dir` - ログディレクトリの絶対パス（例: "/Users/username/.pomodoro/logs"）
    ///
    /// # Returns
    /// LaunchAgent設定
    ///
    /// # Panics
    /// `binary_path`または`log_dir`が空の場合
    pub fn new(binary_path: impl Into<String>, log_dir: impl Into<String>) -> Self {
        let binary_path = binary_path.into();
        let log_dir = log_dir.into();

        assert!(!binary_path.is_empty(), "binary_path must not be empty");
        assert!(!log_dir.is_empty(), "log_dir must not be empty");

        Self {
            label: DEFAULT_LABEL.to_string(),
            program_arguments: vec![binary_path, "daemon".to_string()],
            run_at_load: true,
            keep_alive: true,
            standard_out_path: format!("{}/stdout.log", log_dir),
            standard_error_path: format!("{}/stderr.log", log_dir),
            working_directory: None,
            environment_variables: None,
        }
    }

    /// カスタムラベルを設定
    ///
    /// # Arguments
    /// * `label` - サービスラベル（逆ドメイン形式推奨）
    ///
    /// # Returns
    /// 更新された自身への参照
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// 作業ディレクトリを設定
    ///
    /// # Arguments
    /// * `working_directory` - 作業ディレクトリの絶対パス
    ///
    /// # Returns
    /// 更新された自身への参照
    pub fn with_working_directory(mut self, working_directory: impl Into<String>) -> Self {
        self.working_directory = Some(working_directory.into());
        self
    }

    /// 環境変数を設定
    ///
    /// # Arguments
    /// * `env_vars` - 環境変数のマップ
    ///
    /// # Returns
    /// 更新された自身への参照
    pub fn with_environment_variables(mut self, env_vars: HashMap<String, String>) -> Self {
        self.environment_variables = Some(env_vars);
        self
    }

    /// KeepAliveを無効化
    ///
    /// # Returns
    /// 更新された自身への参照
    pub fn without_keep_alive(mut self) -> Self {
        self.keep_alive = false;
        self
    }

    /// Plist XML文字列を生成
    ///
    /// # Returns
    /// XML形式のplist文字列
    ///
    /// # Errors
    /// シリアライズに失敗した場合
    pub fn to_xml(&self) -> Result<String> {
        let mut buf = Vec::new();
        plist::to_writer_xml(&mut buf, self)
            .map_err(|e| LaunchAgentError::PlistSerialization(e.to_string()))?;

        String::from_utf8(buf).map_err(|e| {
            LaunchAgentError::PlistSerialization(format!("UTF-8 conversion failed: {}", e))
        })
    }

    /// XML文字列からPlist構造体を復元
    ///
    /// # Arguments
    /// * `xml` - XML形式のplist文字列
    ///
    /// # Returns
    /// パースされたPlist構造体
    ///
    /// # Errors
    /// デシリアライズに失敗した場合
    pub fn from_xml(xml: &str) -> Result<Self> {
        plist::from_bytes(xml.as_bytes())
            .map_err(|e| LaunchAgentError::PlistDeserialization(e.to_string()))
    }
}

impl Default for PomodoroLaunchAgent {
    fn default() -> Self {
        Self::new("/usr/local/bin/pomodoro", "/tmp/.pomodoro/logs")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_valid_plist() {
        let plist =
            PomodoroLaunchAgent::new("/usr/local/bin/pomodoro", "/Users/test/.pomodoro/logs");

        assert_eq!(plist.label, "com.example.pomodoro");
        assert_eq!(
            plist.program_arguments,
            vec!["/usr/local/bin/pomodoro", "daemon"]
        );
        assert!(plist.run_at_load);
        assert!(plist.keep_alive);
        assert_eq!(
            plist.standard_out_path,
            "/Users/test/.pomodoro/logs/stdout.log"
        );
        assert_eq!(
            plist.standard_error_path,
            "/Users/test/.pomodoro/logs/stderr.log"
        );
        assert!(plist.working_directory.is_none());
        assert!(plist.environment_variables.is_none());
    }

    #[test]
    fn test_with_label() {
        let plist = PomodoroLaunchAgent::new("/usr/local/bin/pomodoro", "/tmp/logs")
            .with_label("com.custom.app");

        assert_eq!(plist.label, "com.custom.app");
    }

    #[test]
    fn test_with_working_directory() {
        let plist = PomodoroLaunchAgent::new("/usr/local/bin/pomodoro", "/tmp/logs")
            .with_working_directory("/Users/test");

        assert_eq!(plist.working_directory, Some("/Users/test".to_string()));
    }

    #[test]
    fn test_with_environment_variables() {
        let mut env_vars = HashMap::new();
        env_vars.insert("PATH".to_string(), "/usr/local/bin".to_string());
        env_vars.insert("HOME".to_string(), "/Users/test".to_string());

        let plist = PomodoroLaunchAgent::new("/usr/local/bin/pomodoro", "/tmp/logs")
            .with_environment_variables(env_vars.clone());

        assert_eq!(plist.environment_variables, Some(env_vars));
    }

    #[test]
    fn test_without_keep_alive() {
        let plist =
            PomodoroLaunchAgent::new("/usr/local/bin/pomodoro", "/tmp/logs").without_keep_alive();

        assert!(!plist.keep_alive);
    }

    #[test]
    fn test_to_xml_generates_valid_xml() {
        let plist =
            PomodoroLaunchAgent::new("/usr/local/bin/pomodoro", "/Users/test/.pomodoro/logs");

        let xml = plist.to_xml().unwrap();

        // Check XML declaration and DOCTYPE
        assert!(xml.contains("<?xml version=\"1.0\""));
        assert!(xml.contains("<!DOCTYPE plist"));

        // Check key elements
        assert!(xml.contains("<key>Label</key>"));
        assert!(xml.contains("<string>com.example.pomodoro</string>"));
        assert!(xml.contains("<key>ProgramArguments</key>"));
        assert!(xml.contains("<key>RunAtLoad</key>"));
        assert!(xml.contains("<true/>") || xml.contains("<true />"));
        assert!(xml.contains("<key>KeepAlive</key>"));
        assert!(xml.contains("<key>StandardOutPath</key>"));
        assert!(xml.contains("<key>StandardErrorPath</key>"));
    }

    #[test]
    fn test_to_xml_excludes_none_fields() {
        let plist = PomodoroLaunchAgent::new("/usr/local/bin/pomodoro", "/tmp/logs");
        let xml = plist.to_xml().unwrap();

        // Optional fields should not be included when None
        assert!(!xml.contains("<key>WorkingDirectory</key>"));
        assert!(!xml.contains("<key>EnvironmentVariables</key>"));
    }

    #[test]
    fn test_to_xml_includes_optional_fields() {
        let mut env_vars = HashMap::new();
        env_vars.insert("TEST".to_string(), "value".to_string());

        let plist = PomodoroLaunchAgent::new("/usr/local/bin/pomodoro", "/tmp/logs")
            .with_working_directory("/work")
            .with_environment_variables(env_vars);

        let xml = plist.to_xml().unwrap();

        assert!(xml.contains("<key>WorkingDirectory</key>"));
        assert!(xml.contains("<key>EnvironmentVariables</key>"));
    }

    #[test]
    fn test_from_xml_roundtrip() {
        let original =
            PomodoroLaunchAgent::new("/usr/local/bin/pomodoro", "/Users/test/.pomodoro/logs");

        let xml = original.to_xml().unwrap();
        let parsed = PomodoroLaunchAgent::from_xml(&xml).unwrap();

        assert_eq!(original, parsed);
    }

    #[test]
    fn test_from_xml_with_optional_fields() {
        let mut env_vars = HashMap::new();
        env_vars.insert("KEY".to_string(), "value".to_string());

        let original = PomodoroLaunchAgent::new("/usr/local/bin/pomodoro", "/tmp/logs")
            .with_working_directory("/work")
            .with_environment_variables(env_vars);

        let xml = original.to_xml().unwrap();
        let parsed = PomodoroLaunchAgent::from_xml(&xml).unwrap();

        assert_eq!(original, parsed);
    }

    #[test]
    fn test_from_xml_invalid_xml() {
        let result = PomodoroLaunchAgent::from_xml("invalid xml");
        assert!(result.is_err());
    }

    #[test]
    fn test_default() {
        let plist = PomodoroLaunchAgent::default();
        assert_eq!(plist.label, "com.example.pomodoro");
        assert_eq!(plist.program_arguments[0], "/usr/local/bin/pomodoro");
    }

    #[test]
    #[should_panic(expected = "binary_path must not be empty")]
    fn test_new_panics_on_empty_binary_path() {
        PomodoroLaunchAgent::new("", "/tmp/logs");
    }

    #[test]
    #[should_panic(expected = "log_dir must not be empty")]
    fn test_new_panics_on_empty_log_dir() {
        PomodoroLaunchAgent::new("/usr/local/bin/pomodoro", "");
    }

    #[test]
    fn test_builder_pattern_chaining() {
        let mut env_vars = HashMap::new();
        env_vars.insert("TEST".to_string(), "value".to_string());

        let plist = PomodoroLaunchAgent::new("/usr/local/bin/pomodoro", "/tmp/logs")
            .with_label("com.test.app")
            .with_working_directory("/work")
            .with_environment_variables(env_vars)
            .without_keep_alive();

        assert_eq!(plist.label, "com.test.app");
        assert_eq!(plist.working_directory, Some("/work".to_string()));
        assert!(!plist.keep_alive);
        assert!(plist.environment_variables.is_some());
    }
}
