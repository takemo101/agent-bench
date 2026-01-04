//! launchctlコマンド実行ラッパー
//!
//! macOSのlaunchctlコマンドを実行するラッパー関数を提供する。
//! load/unloadは従来のAPI、bootstrap/bootoutはmacOS 10.10+の新しいAPIである。

use std::path::Path;
use std::process::Command;

use super::error::{LaunchAgentError, Result};

/// launchctl loadを実行してサービスを登録する
///
/// # Arguments
/// * `plist_path` - plistファイルの絶対パス
///
/// # Returns
/// 成功時はOk、失敗時はErr
///
/// # Errors
/// - launchctlコマンドの実行に失敗
/// - サービスの登録に失敗
///
/// # Note
/// このAPIはmacOSでのみ正常に動作する。
pub fn load(plist_path: &Path) -> Result<()> {
    let output = Command::new("launchctl")
        .arg("load")
        .arg(plist_path)
        .output()
        .map_err(|e| {
            LaunchAgentError::LaunchctlExecution(format!(
                "Failed to execute 'launchctl load': {}",
                e
            ))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(LaunchAgentError::ServiceLoad(stderr.to_string()));
    }

    tracing::debug!("launchctl load succeeded for {:?}", plist_path);
    Ok(())
}

/// launchctl unloadを実行してサービスを解除する
///
/// # Arguments
/// * `plist_path` - plistファイルの絶対パス
///
/// # Returns
/// 成功時はOk、失敗時はErr
///
/// # Errors
/// - launchctlコマンドの実行に失敗
/// - サービスの解除に失敗
///
/// # Note
/// サービスが既に解除されている場合もエラーを返すため、
/// 呼び出し側でエラーを無視することを推奨。
pub fn unload(plist_path: &Path) -> Result<()> {
    let output = Command::new("launchctl")
        .arg("unload")
        .arg(plist_path)
        .output()
        .map_err(|e| {
            LaunchAgentError::LaunchctlExecution(format!(
                "Failed to execute 'launchctl unload': {}",
                e
            ))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::warn!(
            "launchctl unload failed (may be already unloaded): {}",
            stderr
        );
        return Err(LaunchAgentError::ServiceUnload(stderr.to_string()));
    }

    tracing::debug!("launchctl unload succeeded for {:?}", plist_path);
    Ok(())
}

/// launchctl bootstrap（macOS 10.10+の新しいAPI）
///
/// # Arguments
/// * `domain` - ドメイン（例: "gui/501"）
/// * `plist_path` - plistファイルの絶対パス
///
/// # Returns
/// 成功時はOk、失敗時はErr
///
/// # Errors
/// - launchctlコマンドの実行に失敗
/// - サービスの登録に失敗
///
/// # Note
/// macOS 10.10以降で推奨されるAPI。互換性のため`load`も併用可能。
pub fn bootstrap(domain: &str, plist_path: &Path) -> Result<()> {
    let output = Command::new("launchctl")
        .arg("bootstrap")
        .arg(domain)
        .arg(plist_path)
        .output()
        .map_err(|e| {
            LaunchAgentError::LaunchctlExecution(format!(
                "Failed to execute 'launchctl bootstrap': {}",
                e
            ))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(LaunchAgentError::ServiceLoad(format!(
            "bootstrap failed: {}",
            stderr
        )));
    }

    tracing::debug!("launchctl bootstrap succeeded for {:?}", plist_path);
    Ok(())
}

/// launchctl bootout（macOS 10.10+の新しいAPI）
///
/// # Arguments
/// * `domain` - ドメイン（例: "gui/501"）
/// * `label` - サービスラベル（例: "com.example.pomodoro"）
///
/// # Returns
/// 成功時はOk、失敗時はErr
///
/// # Errors
/// - launchctlコマンドの実行に失敗
/// - サービスの解除に失敗
pub fn bootout(domain: &str, label: &str) -> Result<()> {
    let service_target = format!("{}/{}", domain, label);

    let output = Command::new("launchctl")
        .arg("bootout")
        .arg(&service_target)
        .output()
        .map_err(|e| {
            LaunchAgentError::LaunchctlExecution(format!(
                "Failed to execute 'launchctl bootout': {}",
                e
            ))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::warn!(
            "launchctl bootout failed (may be already stopped): {}",
            stderr
        );
        return Err(LaunchAgentError::ServiceUnload(format!(
            "bootout failed: {}",
            stderr
        )));
    }

    tracing::debug!("launchctl bootout succeeded for {}", service_target);
    Ok(())
}

/// launchctl list でサービス情報を取得
///
/// # Arguments
/// * `label` - サービスラベル（例: "com.example.pomodoro"）
///
/// # Returns
/// サービス情報の文字列（存在する場合）
///
/// # Errors
/// - launchctlコマンドの実行に失敗
/// - サービスが存在しない場合
pub fn list(label: &str) -> Result<String> {
    let output = Command::new("launchctl")
        .arg("list")
        .arg(label)
        .output()
        .map_err(|e| {
            LaunchAgentError::LaunchctlExecution(format!(
                "Failed to execute 'launchctl list': {}",
                e
            ))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(LaunchAgentError::LaunchctlExecution(format!(
            "Service not found or not running: {}",
            stderr
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.to_string())
}

/// ユーザードメインを取得（gui/{uid}形式）
///
/// # Returns
/// ユーザードメイン文字列（例: "gui/501"）
///
/// # Note
/// Linux環境ではダミー値を返す（macOS専用機能のため）
#[cfg(target_os = "macos")]
pub fn get_user_domain() -> String {
    use std::process::Command;

    let uid = Command::new("id")
        .arg("-u")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "501".to_string());

    format!("gui/{}", uid)
}

/// ユーザードメインを取得（gui/{uid}形式）
///
/// # Returns
/// ユーザードメイン文字列（例: "gui/1000"）
///
/// # Note
/// 非macOS環境ではダミー値を返す
#[cfg(not(target_os = "macos"))]
pub fn get_user_domain() -> String {
    // Linux環境ではuidを取得してダミードメインを返す
    use std::process::Command;

    let uid = Command::new("id")
        .arg("-u")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "1000".to_string());

    format!("gui/{}", uid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // Note: These tests verify the API signatures and error handling.
    // Actual launchctl execution tests should be run on macOS or in CI with macOS runner.

    #[test]
    fn test_get_user_domain_format() {
        let domain = get_user_domain();
        assert!(domain.starts_with("gui/"));
        // UID should be a number
        let uid_str = domain.strip_prefix("gui/").unwrap();
        assert!(
            uid_str.parse::<u32>().is_ok(),
            "UID should be a valid number"
        );
    }

    #[test]
    #[ignore = "launchctl behavior varies by environment; skipped in CI"]
    fn test_load_returns_error_on_nonexistent_path() {
        // launchctl is not available in container, so this will fail with execution error
        let path = PathBuf::from("/nonexistent/path.plist");
        let result = load(&path);

        // Either execution error (launchctl not found) or service load error
        assert!(result.is_err());
    }

    #[test]
    #[ignore = "launchctl behavior varies by environment; skipped in CI"]
    fn test_unload_returns_error_on_nonexistent_path() {
        let path = PathBuf::from("/nonexistent/path.plist");
        let result = unload(&path);

        assert!(result.is_err());
    }

    #[test]
    fn test_bootstrap_returns_error_on_nonexistent_path() {
        let path = PathBuf::from("/nonexistent/path.plist");
        let result = bootstrap("gui/1000", &path);

        assert!(result.is_err());
    }

    #[test]
    fn test_bootout_returns_error_on_nonexistent_service() {
        let result = bootout("gui/1000", "com.nonexistent.service");

        assert!(result.is_err());
    }

    #[test]
    fn test_list_returns_error_on_nonexistent_service() {
        let result = list("com.nonexistent.service");

        assert!(result.is_err());
    }

    // Tests for error types
    #[test]
    #[ignore = "launchctl behavior varies by environment; skipped in CI"]
    fn test_load_error_type() {
        let path = PathBuf::from("/nonexistent/path.plist");
        let result = load(&path);

        match result {
            Err(LaunchAgentError::LaunchctlExecution(_))
            | Err(LaunchAgentError::ServiceLoad(_)) => (),
            _ => panic!("Expected LaunchctlExecution or ServiceLoad error"),
        }
    }

    #[test]
    #[ignore = "launchctl behavior varies by environment; skipped in CI"]
    fn test_unload_error_type() {
        let path = PathBuf::from("/nonexistent/path.plist");
        let result = unload(&path);

        match result {
            Err(LaunchAgentError::LaunchctlExecution(_))
            | Err(LaunchAgentError::ServiceUnload(_)) => (),
            _ => panic!("Expected LaunchctlExecution or ServiceUnload error"),
        }
    }
}
