//! LaunchAgentエラー型定義
//!
//! LaunchAgent管理で発生するエラーを定義する。

use std::io;
use thiserror::Error;

/// LaunchAgent管理のエラー型
#[derive(Debug, Error)]
pub enum LaunchAgentError {
    /// バイナリパスの解決に失敗
    #[error("Failed to resolve pomodoro binary path: {0}")]
    BinaryPathResolution(String),

    /// ホームディレクトリの取得に失敗
    #[error("Failed to get home directory")]
    HomeDirectoryNotFound,

    /// ログディレクトリの作成に失敗
    #[error("Failed to create log directory: {0}")]
    LogDirectoryCreation(#[source] io::Error),

    /// plistファイルの書き込みに失敗
    #[error("Failed to write plist file: {0}")]
    PlistWrite(String),

    /// plistファイルの削除に失敗
    #[error("Failed to remove plist file: {0}")]
    PlistRemove(String),

    /// plistのシリアライズに失敗
    #[error("Failed to serialize plist: {0}")]
    PlistSerialization(String),

    /// plistのデシリアライズに失敗
    #[error("Failed to deserialize plist: {0}")]
    PlistDeserialization(String),

    /// launchctlコマンドの実行に失敗
    #[error("Failed to execute launchctl: {0}")]
    LaunchctlExecution(String),

    /// サービスの登録に失敗
    #[error("Failed to load LaunchAgent: {0}")]
    ServiceLoad(String),

    /// サービスの解除に失敗
    #[error("Failed to unload LaunchAgent: {0}")]
    ServiceUnload(String),

    /// 権限エラー
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// I/Oエラー
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
}

/// LaunchAgent操作の結果型
pub type Result<T> = std::result::Result<T, LaunchAgentError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_binary_path_resolution() {
        let error = LaunchAgentError::BinaryPathResolution("not found".to_string());
        assert!(error.to_string().contains("pomodoro binary path"));
        assert!(error.to_string().contains("not found"));
    }

    #[test]
    fn test_error_display_home_directory_not_found() {
        let error = LaunchAgentError::HomeDirectoryNotFound;
        assert!(error.to_string().contains("home directory"));
    }

    #[test]
    fn test_error_display_plist_serialization() {
        let error = LaunchAgentError::PlistSerialization("invalid data".to_string());
        assert!(error.to_string().contains("serialize plist"));
    }

    #[test]
    fn test_error_display_launchctl_execution() {
        let error = LaunchAgentError::LaunchctlExecution("command failed".to_string());
        assert!(error.to_string().contains("launchctl"));
    }

    #[test]
    fn test_error_display_service_load() {
        let error = LaunchAgentError::ServiceLoad("already loaded".to_string());
        assert!(error.to_string().contains("load LaunchAgent"));
    }

    #[test]
    fn test_error_display_service_unload() {
        let error = LaunchAgentError::ServiceUnload("not running".to_string());
        assert!(error.to_string().contains("unload LaunchAgent"));
    }

    #[test]
    fn test_error_from_io_error() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let error: LaunchAgentError = io_error.into();
        assert!(matches!(error, LaunchAgentError::Io(_)));
    }

    #[test]
    fn test_result_type_alias() {
        fn returns_ok() -> Result<()> {
            Ok(())
        }
        assert!(returns_ok().is_ok());

        fn returns_err() -> Result<()> {
            Err(LaunchAgentError::HomeDirectoryNotFound)
        }
        assert!(returns_err().is_err());
    }
}
