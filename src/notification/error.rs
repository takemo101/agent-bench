//! 通知システムのエラー型定義

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationError {
    AuthorizationFailed(String),
    SendFailed(String),
    PermissionDenied,
    UnsignedBinary,
    InitializationFailed(String),
    InvalidInput(String),
    /// アプリバンドルコンテキストがない（CLI直接実行時）
    NoBundleContext,
}

impl fmt::Display for NotificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotificationError::AuthorizationFailed(msg) => {
                write!(f, "通知許可の取得に失敗しました: {}", msg)
            }
            NotificationError::SendFailed(msg) => {
                write!(f, "通知の送信に失敗しました: {}", msg)
            }
            NotificationError::PermissionDenied => {
                write!(f, "通知許可が拒否されています")
            }
            NotificationError::UnsignedBinary => {
                write!(
                    f,
                    "バイナリが署名されていません。codesignで署名してください"
                )
            }
            NotificationError::InitializationFailed(msg) => {
                write!(f, "通知システムの初期化に失敗しました: {}", msg)
            }
            NotificationError::InvalidInput(msg) => {
                write!(f, "無効な入力: {}", msg)
            }
            NotificationError::NoBundleContext => {
                write!(
                    f,
                    "アプリバンドルコンテキストがありません。通知機能は無効です"
                )
            }
        }
    }
}

impl std::error::Error for NotificationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authorization_failed_display() {
        let error = NotificationError::AuthorizationFailed("timeout".to_string());
        assert_eq!(error.to_string(), "通知許可の取得に失敗しました: timeout");
    }

    #[test]
    fn test_send_failed_display() {
        let error = NotificationError::SendFailed("network error".to_string());
        assert_eq!(error.to_string(), "通知の送信に失敗しました: network error");
    }

    #[test]
    fn test_permission_denied_display() {
        let error = NotificationError::PermissionDenied;
        assert_eq!(error.to_string(), "通知許可が拒否されています");
    }

    #[test]
    fn test_unsigned_binary_display() {
        let error = NotificationError::UnsignedBinary;
        assert!(error.to_string().contains("codesign"));
    }

    #[test]
    fn test_initialization_failed_display() {
        let error = NotificationError::InitializationFailed("missing framework".to_string());
        assert_eq!(
            error.to_string(),
            "通知システムの初期化に失敗しました: missing framework"
        );
    }

    #[test]
    fn test_invalid_input_display() {
        let error = NotificationError::InvalidInput("タスク名が長すぎます".to_string());
        assert_eq!(error.to_string(), "無効な入力: タスク名が長すぎます");
    }

    #[test]
    fn test_error_equality() {
        let error1 = NotificationError::PermissionDenied;
        let error2 = NotificationError::PermissionDenied;
        assert_eq!(error1, error2);

        let error3 = NotificationError::UnsignedBinary;
        assert_ne!(error1, error3);
    }

    #[test]
    fn test_error_clone() {
        let error = NotificationError::SendFailed("test".to_string());
        let cloned = error.clone();
        assert_eq!(error, cloned);
    }

    #[test]
    fn test_error_debug() {
        let error = NotificationError::PermissionDenied;
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("PermissionDenied"));
    }
}
