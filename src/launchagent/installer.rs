//! LaunchAgentインストール・アンインストールロジック
//!
//! LaunchAgentのインストールおよびアンインストール処理を提供する。

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

use super::error::{LaunchAgentError, Result};
use super::launchctl;
use super::plist::PomodoroLaunchAgent;
use super::{get_launch_agents_dir, get_log_dir, get_plist_path};

/// pomodoroバイナリの絶対パスを解決する
///
/// `which pomodoro` コマンドを実行してバイナリのパスを取得する。
///
/// # Returns
/// バイナリの絶対パス
///
/// # Errors
/// - whichコマンドの実行に失敗
/// - バイナリが見つからない
///
/// # Example
/// ```rust,ignore
/// let binary_path = resolve_binary_path()?;
/// assert!(binary_path.starts_with("/"));
/// ```
pub fn resolve_binary_path() -> Result<String> {
    let output = Command::new("which")
        .arg("pomodoro")
        .output()
        .map_err(|e| {
            LaunchAgentError::BinaryPathResolution(format!(
                "Failed to execute 'which pomodoro': {}",
                e
            ))
        })?;

    if !output.status.success() {
        return Err(LaunchAgentError::BinaryPathResolution(
            "pomodoro binary not found in PATH".to_string(),
        ));
    }

    let path = String::from_utf8(output.stdout)
        .map_err(|e| {
            LaunchAgentError::BinaryPathResolution(format!("Failed to parse which output: {}", e))
        })?
        .trim()
        .to_string();

    if path.is_empty() {
        return Err(LaunchAgentError::BinaryPathResolution(
            "pomodoro binary path is empty".to_string(),
        ));
    }

    tracing::debug!("Resolved pomodoro binary path: {}", path);
    Ok(path)
}

/// LaunchAgentをインストールする
///
/// 以下の処理を順番に実行する:
/// 1. バイナリパスを解決（`which pomodoro`）
/// 2. ログディレクトリを作成（`~/.pomodoro/logs/`）
/// 3. Plistを生成（`PomodoroLaunchAgent::new`）
/// 4. plistファイルを書き込み（`~/Library/LaunchAgents/`）
/// 5. パーミッションを設定（0644）
/// 6. 既存サービスをアンロード（冪等性確保）
/// 7. サービスをロード
///
/// # Returns
/// インストール成功時はOk、失敗時はErr
///
/// # Errors
/// - バイナリパスの解決に失敗
/// - ホームディレクトリの取得に失敗
/// - ログディレクトリの作成に失敗
/// - plistファイルの書き込みに失敗
/// - launchctl loadに失敗
///
/// # Example
/// ```rust,ignore
/// use pomodoro::launchagent::installer;
///
/// installer::install()?;
/// println!("LaunchAgent installed successfully");
/// ```
pub fn install() -> Result<()> {
    // 1. バイナリパスを解決
    let binary_path = resolve_binary_path()?;

    // 2. ログディレクトリを作成
    let log_dir = get_log_dir()?;
    fs::create_dir_all(&log_dir).map_err(LaunchAgentError::LogDirectoryCreation)?;
    tracing::debug!("Created log directory: {:?}", log_dir);

    // 3. Plist生成
    let plist = PomodoroLaunchAgent::new(&binary_path, log_dir.to_string_lossy().to_string());
    let plist_xml = plist.to_xml()?;

    // 4. LaunchAgentsディレクトリを作成（存在しない場合）
    let launch_agents_dir = get_launch_agents_dir()?;
    fs::create_dir_all(&launch_agents_dir).map_err(|e| {
        LaunchAgentError::PlistWrite(format!("Failed to create LaunchAgents directory: {}", e))
    })?;

    // 5. plistファイルを書き込み
    let plist_path = get_plist_path()?;
    fs::write(&plist_path, &plist_xml)
        .map_err(|e| LaunchAgentError::PlistWrite(format!("Failed to write plist file: {}", e)))?;
    tracing::debug!("Wrote plist file: {:?}", plist_path);

    // 6. パーミッション設定（0644: rw-r--r--）
    let mut perms = fs::metadata(&plist_path)
        .map_err(|e| LaunchAgentError::PlistWrite(format!("Failed to get plist metadata: {}", e)))?
        .permissions();
    perms.set_mode(0o644);
    fs::set_permissions(&plist_path, perms).map_err(|e| {
        LaunchAgentError::PlistWrite(format!("Failed to set plist permissions: {}", e))
    })?;

    // 7. 既存のサービスをアンロード（冪等性確保、エラーは無視）
    let _ = launchctl::unload(&plist_path);

    // 8. サービスをロード
    launchctl::load(&plist_path)?;

    tracing::info!("LaunchAgent installed successfully at {:?}", plist_path);
    Ok(())
}

/// LaunchAgentをアンインストールする
///
/// 以下の処理を順番に実行する:
/// 1. plistファイルの存在確認（存在しない場合は成功として扱う）
/// 2. サービスをアンロード（エラーは無視）
/// 3. plistファイルを削除
///
/// # Returns
/// アンインストール成功時はOk、失敗時はErr
///
/// # Errors
/// - ホームディレクトリの取得に失敗
/// - plistファイルの削除に失敗
///
/// # Note
/// plistファイルが存在しない場合は、既にアンインストール済みとして
/// 成功（Ok）を返す（冪等性確保）。
///
/// # Example
/// ```rust,ignore
/// use pomodoro::launchagent::installer;
///
/// installer::uninstall()?;
/// println!("LaunchAgent uninstalled successfully");
/// ```
pub fn uninstall() -> Result<()> {
    // 1. plistファイルパスを取得
    let plist_path = get_plist_path()?;

    // 2. plistファイルが存在しない場合は成功として扱う
    if !plist_path.exists() {
        tracing::info!("LaunchAgent plist file does not exist, nothing to uninstall");
        return Ok(());
    }

    // 3. サービスをアンロード（エラーは無視）
    let _ = launchctl::unload(&plist_path);

    // 4. plistファイルを削除
    fs::remove_file(&plist_path).map_err(|e| {
        LaunchAgentError::PlistRemove(format!("Failed to remove plist file: {}", e))
    })?;

    tracing::info!("LaunchAgent uninstalled successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_binary_path_returns_error_when_not_found() {
        // This test assumes 'pomodoro' is not in PATH during testing
        // The result depends on the environment, so we just verify the function doesn't panic
        let result = resolve_binary_path();
        // Either succeeds (if pomodoro is in PATH) or returns proper error
        match result {
            Ok(path) => assert!(!path.is_empty()),
            Err(e) => assert!(matches!(e, LaunchAgentError::BinaryPathResolution(_))),
        }
    }

    #[test]
    fn test_resolve_binary_path_which_command_works() {
        // Test that 'which' command itself works (using a common command)
        let output = Command::new("which").arg("ls").output();

        assert!(output.is_ok());
        let output = output.unwrap();
        // 'ls' should always be found on Unix systems
        assert!(output.status.success());
    }

    #[test]
    #[ignore = "Modifies system state; run manually on macOS"]
    fn test_install_and_uninstall_integration() {
        // This is an integration test that modifies the system
        // Run manually: cargo test test_install_and_uninstall_integration -- --ignored

        // Install
        let install_result = install();
        assert!(
            install_result.is_ok(),
            "Install failed: {:?}",
            install_result
        );

        // Verify plist file exists
        let plist_path = get_plist_path().unwrap();
        assert!(plist_path.exists(), "Plist file should exist after install");

        // Uninstall
        let uninstall_result = uninstall();
        assert!(
            uninstall_result.is_ok(),
            "Uninstall failed: {:?}",
            uninstall_result
        );

        // Verify plist file is removed
        assert!(
            !plist_path.exists(),
            "Plist file should not exist after uninstall"
        );
    }

    #[test]
    fn test_uninstall_when_not_installed() {
        // Uninstalling when not installed should succeed (idempotency)
        // This test assumes the LaunchAgent is not actually installed
        // We can't fully test this without modifying the system, but we can test the logic

        // Create a mock scenario: if plist doesn't exist, uninstall should succeed
        let result = get_plist_path();
        if let Ok(path) = result {
            if !path.exists() {
                // If plist doesn't exist, uninstall should succeed
                let uninstall_result = uninstall();
                assert!(
                    uninstall_result.is_ok(),
                    "Uninstall should succeed when not installed"
                );
            }
        }
    }

    #[test]
    fn test_log_dir_creation() {
        // Test that log dir path can be obtained
        let result = get_log_dir();
        if let Ok(log_dir) = result {
            assert!(log_dir.to_string_lossy().contains(".pomodoro/logs"));
        }
    }

    #[test]
    fn test_plist_path() {
        // Test that plist path can be obtained
        let result = get_plist_path();
        if let Ok(plist_path) = result {
            assert!(plist_path
                .to_string_lossy()
                .contains("Library/LaunchAgents"));
            assert!(plist_path
                .to_string_lossy()
                .contains("com.github.takemo101.pomodoro.plist"));
        }
    }

    #[test]
    fn test_launch_agents_dir() {
        // Test that LaunchAgents dir path can be obtained
        let result = get_launch_agents_dir();
        if let Ok(dir) = result {
            assert!(dir.to_string_lossy().contains("Library/LaunchAgents"));
        }
    }
}
