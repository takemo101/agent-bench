use crate::hooks::{HookConfig, HookConfigError, HookContext, HookDefinition};
use std::fs;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

/// フック実行機能
#[derive(Debug, Clone)]
pub struct HookExecutor {
    config: HookConfig,
    enabled: bool,
}

impl Default for HookExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl HookExecutor {
    /// 新しいHookExecutorを作成
    ///
    /// ~/.pomodoro/hooks.json から設定を読み込む。
    /// ファイルが存在しない、または読み込みエラーの場合は無効状態で初期化する。
    pub fn new() -> Self {
        match HookConfig::load() {
            Ok(config) => {
                let enabled = config.has_hooks();
                debug!("フック設定を読み込みました: {} フック登録", config.hooks.len());
                Self { config, enabled }
            }
            Err(HookConfigError::FileNotFound(path)) => {
                debug!("フック設定ファイルが見つかりません: {:?}", path);
                Self {
                    config: HookConfig::default(),
                    enabled: false,
                }
            }
            Err(e) => {
                warn!("フック設定の読み込みに失敗しました: {}", e);
                Self {
                    config: HookConfig::default(),
                    enabled: false,
                }
            }
        }
    }

    /// テスト用に設定を指定して作成
    #[cfg(test)]
    pub fn with_config(config: HookConfig) -> Self {
        Self {
            config,
            enabled: true,
        }
    }

    /// フックを実行（非同期・Fire-and-forget）
    pub fn execute(&self, context: HookContext) {
        if !self.enabled {
            return;
        }

        let event_name = context.event.as_str().to_string();
        if let Some(hooks) = self.config.hooks.get(&event_name) {
            if hooks.is_empty() {
                return;
            }

            let hooks = hooks.clone();
            let global_timeout = self.config.global_timeout;
            let context = context.clone();

            // Fire-and-forget execution
            tokio::spawn(async move {
                for hook in hooks {
                    if !hook.enabled {
                        continue;
                    }

                    if let Err(e) = Self::execute_single_hook(&hook, &context, global_timeout).await
                    {
                        error!("フック実行エラー ({}): {}", hook.name, e);
                    }
                }
            });
        }
    }

    /// 単一のフックを実行
    async fn execute_single_hook(
        hook: &HookDefinition,
        context: &HookContext,
        global_timeout: u64,
    ) -> Result<(), String> {
        Self::validate_script(&hook.script)?;

        let timeout_secs = hook.timeout.unwrap_or(global_timeout);
        let env_vars = context.to_env_vars();

        info!("フック実行開始: {} (timeout: {}s)", hook.name, timeout_secs);

        let mut command = Command::new(&hook.script);
        command
            .envs(&env_vars)
            .env("POMODORO_HOOK_NAME", &hook.name)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Execute with timeout
        let child_result = timeout(
            Duration::from_secs(timeout_secs),
            command
                .spawn()
                .map_err(|e| e.to_string())?
                .wait_with_output(),
        )
        .await;

        match child_result {
            Ok(Ok(output)) => {
                let status = output.status;
                Self::log_output(&hook.name, "stdout", &output.stdout);
                Self::log_output(&hook.name, "stderr", &output.stderr);

                if status.success() {
                    info!("フック実行成功: {}", hook.name);
                    Ok(())
                } else {
                    Err(format!(
                        "スクリプトが非ゼロの終了コードで終了しました: {:?}",
                        status.code()
                    ))
                }
            }
            Ok(Err(e)) => Err(format!("プロセス実行エラー: {}", e)),
            Err(_) => Err(format!("タイムアウトしました ({}秒)", timeout_secs)),
        }
    }

    /// スクリプトの検証
    fn validate_script(path: &Path) -> Result<(), String> {
        if !path.is_absolute() {
            return Err(format!("絶対パスを指定してください: {:?}", path));
        }
        if !path.exists() {
            return Err(format!("ファイルが存在しません: {:?}", path));
        }
        // 実行権限チェック (Unix)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = fs::metadata(path) {
                if metadata.permissions().mode() & 0o111 == 0 {
                    return Err(format!("実行権限がありません: {:?}", path));
                }
            }
        }
        Ok(())
    }

    /// 出力ログ記録（最大10KB）
    fn log_output(hook_name: &str, stream_name: &str, data: &[u8]) {
        if data.is_empty() {
            return;
        }

        const MAX_LOG_SIZE: usize = 10 * 1024; // 10KB

        let log_content = if data.len() > MAX_LOG_SIZE {
            let truncated = &data[..MAX_LOG_SIZE];
            format!("{}... (truncated)", String::from_utf8_lossy(truncated))
        } else {
            String::from_utf8_lossy(data).to_string()
        };

        if stream_name == "stderr" {
            warn!("[Hook: {}] stderr: {}", hook_name, log_content);
        } else {
            info!("[Hook: {}] stdout: {}", hook_name, log_content);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_validate_script_success() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();

        // On Unix, we need to set executable permission
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(path, perms).unwrap();
        }

        assert!(HookExecutor::validate_script(path).is_ok());
    }

    #[test]
    fn test_validate_script_not_exists() {
        let path = Path::new("/tmp/non_existent_script_12345");
        let result = HookExecutor::validate_script(path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("ファイルが存在しません"));
    }

    #[test]
    fn test_validate_script_not_absolute() {
        let path = Path::new("script.sh");
        let result = HookExecutor::validate_script(path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("絶対パスを指定してください"));
    }

    #[cfg(unix)]
    #[test]
    fn test_validate_script_no_permission() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();

        // Remove executable permission
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path).unwrap().permissions();
        perms.set_mode(0o644);
        fs::set_permissions(path, perms).unwrap();

        let result = HookExecutor::validate_script(path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("実行権限がありません"));
    }
}
