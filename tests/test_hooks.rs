use chrono::Utc;
use pomodoro::hooks::{HookContext, HookExecutor};
use pomodoro::types::HookEvent;
use uuid::Uuid;

#[test]
fn test_hook_event_as_str() {
    assert_eq!(HookEvent::WorkStart.as_str(), "work_start");
    assert_eq!(HookEvent::WorkEnd.as_str(), "work_end");
    assert_eq!(HookEvent::Stop.as_str(), "stop");
}

#[test]
fn test_hook_context_to_env_vars() {
    let context = HookContext {
        event: HookEvent::WorkStart,
        task_name: Some("Test Task".to_string()),
        phase: "working".to_string(),
        duration_secs: 1500,
        elapsed_secs: 0,
        remaining_secs: 1500,
        cycle: 1,
        total_cycles: 4,
        timestamp: Utc::now(),
        session_id: Uuid::new_v4(),
    };

    let vars = context.to_env_vars();

    assert_eq!(vars.get("POMODORO_EVENT").unwrap(), "work_start");
    assert_eq!(vars.get("POMODORO_TASK_NAME").unwrap(), "Test Task");
    assert_eq!(vars.get("POMODORO_PHASE").unwrap(), "working");
    assert_eq!(vars.get("POMODORO_DURATION_SECS").unwrap(), "1500");
}

#[tokio::test]
async fn test_hook_executor_creation() {
    // Should handle missing config gracefully
    let _executor = HookExecutor::new();
    // Verify it doesn't panic
}
