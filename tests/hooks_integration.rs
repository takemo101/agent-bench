use anyhow::Result;
use pomodoro::{
    daemon::timer::TimerEngine,
    hooks::{HookConfig, HookExecutor},
    types::{PomodoroConfig, StartParams},
};
use std::{fs, io::Write, sync::Arc, time::Duration};
use tempfile::NamedTempFile;
use tokio::sync::mpsc;

#[tokio::test]
async fn test_timer_engine_fires_hooks() -> Result<()> {
    // 1. Setup temporary files
    let verify_file = NamedTempFile::new()?;
    let verify_path = verify_file.path().to_str().unwrap().to_string();

    let mut script_file = NamedTempFile::new()?;
    let script_path = script_file.path().to_str().unwrap().to_string();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        // Write script content
        writeln!(script_file, "#!/bin/sh\necho 'executed' > {}", verify_path)?;
        script_file.flush()?;

        // Make executable
        let mut perms = fs::metadata(&script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms)?;
    }

    // Windows support omitted for this specific test as it relies on sh
    #[cfg(not(unix))]
    return Ok(());

    // 2. Setup HookConfig
    let mut config_file = NamedTempFile::new()?;
    let config_json = format!(
        r#"{{
            "version": "1.0",
            "hooks": [
                {{
                    "name": "integration_test_hook",
                    "event": "work_start",
                    "script": "{}",
                    "timeout_secs": 5,
                    "enabled": true
                }}
            ]
        }}"#,
        script_path
    );
    config_file.write_all(config_json.as_bytes())?;
    config_file.flush()?;

    let hook_config = HookConfig::load_from_path(config_file.path())?;

    // 3. Create TimerEngine with HookExecutor
    let hook_executor = Arc::new(HookExecutor::with_config(hook_config));
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let config = PomodoroConfig::default();

    let mut engine = TimerEngine::new_with_hook_executor(config, event_tx, hook_executor);

    // 4. Start timer (triggers WorkStart -> Hook)
    let params = StartParams {
        task_name: Some("Integration Test".to_string()),
        ..Default::default()
    };
    engine.start(&params)?;

    // Consume TimerEvent
    let _ = event_rx.recv().await;

    // 5. Verify hook execution
    // Hooks are async/spawned, so we need to wait a bit
    let mut success = false;
    for _ in 0..20 {
        tokio::time::sleep(Duration::from_millis(100)).await;
        if let Ok(content) = fs::read_to_string(&verify_path) {
            if content.trim() == "executed" {
                success = true;
                break;
            }
        }
    }

    assert!(
        success,
        "Hook script was not executed within timeout (2 seconds)"
    );

    Ok(())
}
