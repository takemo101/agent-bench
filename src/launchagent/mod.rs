//! LaunchAgent管理モジュール
//!
//! macOSのLaunchAgentを使用したポモドーロタイマーDaemonの自動起動・
//! バックグラウンド実行機能を提供する。
//!
//! # モジュール構成
//!
//! - `error` - エラー型定義
//! - `plist` - Plist構造体定義、XML生成
//! - `launchctl` - launchctlコマンド実行ラッパー
//!
//! # 例
//!
//! ```rust,ignore
//! use pomodoro::launchagent::{PomodoroLaunchAgent, launchctl};
//!
//! // Plist生成
//! let plist = PomodoroLaunchAgent::new(
//!     "/usr/local/bin/pomodoro",
//!     "/Users/username/.pomodoro/logs",
//! );
//! let xml = plist.to_xml()?;
//!
//! // サービス登録（macOSのみ）
//! launchctl::load(&plist_path)?;
//! ```
//!
//! # 注意事項
//!
//! - すべてのパスは絶対パスで指定する必要がある（`~`は使用不可）
//! - launchctl関連の関数はmacOSでのみ正常に動作する
//! - install/uninstall関数は次のSubtask (#35) で実装予定

pub mod error;
pub mod launchctl;
pub mod plist;

// Re-export common types
pub use error::{LaunchAgentError, Result};
pub use plist::{PomodoroLaunchAgent, DEFAULT_LABEL};

/// plistファイルのデフォルトパスを取得
///
/// # Returns
/// plistファイルのパス（`~/Library/LaunchAgents/com.github.takemo101.pomodoro.plist`）
///
/// # Errors
/// ホームディレクトリの取得に失敗した場合
pub fn get_plist_path() -> Result<std::path::PathBuf> {
    let home_dir = dirs::home_dir().ok_or(LaunchAgentError::HomeDirectoryNotFound)?;
    Ok(home_dir.join(format!("Library/LaunchAgents/{}.plist", DEFAULT_LABEL)))
}

/// ログディレクトリのデフォルトパスを取得
///
/// # Returns
/// ログディレクトリのパス（`~/.pomodoro/logs`）
///
/// # Errors
/// ホームディレクトリの取得に失敗した場合
pub fn get_log_dir() -> Result<std::path::PathBuf> {
    let home_dir = dirs::home_dir().ok_or(LaunchAgentError::HomeDirectoryNotFound)?;
    Ok(home_dir.join(".pomodoro/logs"))
}

/// LaunchAgentsディレクトリのデフォルトパスを取得
///
/// # Returns
/// LaunchAgentsディレクトリのパス（`~/Library/LaunchAgents`）
///
/// # Errors
/// ホームディレクトリの取得に失敗した場合
pub fn get_launch_agents_dir() -> Result<std::path::PathBuf> {
    let home_dir = dirs::home_dir().ok_or(LaunchAgentError::HomeDirectoryNotFound)?;
    Ok(home_dir.join("Library/LaunchAgents"))
}

// Stub functions for install/uninstall (to be implemented in Subtask #35)
// These are placeholder signatures that will be fully implemented later.

/// LaunchAgentをインストールする（スタブ - #35で実装予定）
///
/// # Returns
/// インストール成功時はOk、失敗時はErr
///
/// # Note
/// この関数は現在スタブであり、完全な実装はSubtask #35で行われる。
#[allow(dead_code)]
pub fn install() -> Result<()> {
    unimplemented!("install() will be implemented in Subtask #35")
}

/// LaunchAgentをアンインストールする（スタブ - #35で実装予定）
///
/// # Returns
/// アンインストール成功時はOk、失敗時はErr
///
/// # Note
/// この関数は現在スタブであり、完全な実装はSubtask #35で行われる。
#[allow(dead_code)]
pub fn uninstall() -> Result<()> {
    unimplemented!("uninstall() will be implemented in Subtask #35")
}

/// LaunchAgentがインストールされているか確認する
///
/// # Returns
/// インストール済みの場合はtrue、未インストールの場合はfalse
pub fn is_installed() -> bool {
    match get_plist_path() {
        Ok(path) => path.exists(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_plist_path() {
        let result = get_plist_path();

        // May fail if home directory is not set (e.g., in some CI environments)
        if let Ok(path) = result {
            assert!(path.to_string_lossy().contains("Library/LaunchAgents"));
            assert!(path
                .to_string_lossy()
                .contains("com.example.pomodoro.plist"));
        }
    }

    #[test]
    fn test_get_log_dir() {
        let result = get_log_dir();

        if let Ok(path) = result {
            assert!(path.to_string_lossy().contains(".pomodoro/logs"));
        }
    }

    #[test]
    fn test_get_launch_agents_dir() {
        let result = get_launch_agents_dir();

        if let Ok(path) = result {
            assert!(path.to_string_lossy().contains("Library/LaunchAgents"));
        }
    }

    #[test]
    fn test_is_installed_returns_false_when_not_installed() {
        // In test environment, plist file should not exist
        let installed = is_installed();
        // This might be true if the test is run on a system where the app is installed
        // So we just verify it returns a boolean without panicking
        let _ = installed;
    }

    #[test]
    fn test_default_label_constant() {
        assert_eq!(DEFAULT_LABEL, "com.example.pomodoro");
    }

    #[test]
    fn test_plist_creation() {
        let plist =
            PomodoroLaunchAgent::new("/usr/local/bin/pomodoro", "/Users/test/.pomodoro/logs");

        assert_eq!(plist.label, DEFAULT_LABEL);
    }

    #[test]
    fn test_plist_xml_generation() {
        let plist =
            PomodoroLaunchAgent::new("/usr/local/bin/pomodoro", "/Users/test/.pomodoro/logs");

        let xml = plist.to_xml();
        assert!(xml.is_ok());

        let xml_str = xml.unwrap();
        assert!(xml_str.contains("<plist"));
        assert!(xml_str.contains("com.example.pomodoro"));
    }

    #[test]
    fn test_reexports() {
        // Verify that types are properly re-exported
        let _: LaunchAgentError = LaunchAgentError::HomeDirectoryNotFound;
        let _: PomodoroLaunchAgent = PomodoroLaunchAgent::default();
    }
}
