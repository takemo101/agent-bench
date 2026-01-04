pub mod error;
pub mod request;

#[cfg(target_os = "macos")]
pub mod actions;
#[cfg(target_os = "macos")]
pub mod center;
#[cfg(target_os = "macos")]
pub mod content;
#[cfg(target_os = "macos")]
pub mod delegate;
#[cfg(target_os = "macos")]
mod manager;

pub use error::NotificationError;
pub use request::{NotificationRequest, NotificationRequestId};

#[cfg(target_os = "macos")]
pub use actions::{create_actions, create_categories};
#[cfg(target_os = "macos")]
pub use center::NotificationCenter;
#[cfg(target_os = "macos")]
pub use content::{
    create_break_complete_content, create_long_break_complete_content,
    create_work_complete_content, NotificationContentBuilder,
};
#[cfg(target_os = "macos")]
pub use delegate::{NotificationActionEvent, NotificationDelegate};
#[cfg(target_os = "macos")]
pub use manager::NotificationManager;

pub mod category_ids {
    pub const WORK_COMPLETE: &str = "WORK_COMPLETE";
    pub const BREAK_COMPLETE: &str = "BREAK_COMPLETE";
    pub const LONG_BREAK_COMPLETE: &str = "LONG_BREAK_COMPLETE";
}

pub mod action_ids {
    pub const PAUSE_ACTION: &str = "PAUSE_ACTION";
    pub const STOP_ACTION: &str = "STOP_ACTION";
}

pub mod limits {
    pub const MAX_TASK_NAME_LENGTH: usize = 100;
    pub const MAX_TITLE_LENGTH: usize = 50;
    pub const MAX_BODY_LENGTH: usize = 200;
}

/// # Errors
/// - タスク名が100文字を超える場合
/// - タスク名に制御文字が含まれる場合
pub fn validate_task_name(task_name: &str) -> Result<&str, NotificationError> {
    if task_name.len() > limits::MAX_TASK_NAME_LENGTH {
        return Err(NotificationError::InvalidInput(format!(
            "タスク名は{}文字以内にしてください",
            limits::MAX_TASK_NAME_LENGTH
        )));
    }

    if task_name.chars().any(|c| c.is_control()) {
        return Err(NotificationError::InvalidInput(
            "タスク名に制御文字は使用できません".to_string(),
        ));
    }

    Ok(task_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_ids() {
        assert_eq!(category_ids::WORK_COMPLETE, "WORK_COMPLETE");
        assert_eq!(category_ids::BREAK_COMPLETE, "BREAK_COMPLETE");
        assert_eq!(category_ids::LONG_BREAK_COMPLETE, "LONG_BREAK_COMPLETE");
    }

    #[test]
    fn test_action_ids() {
        assert_eq!(action_ids::PAUSE_ACTION, "PAUSE_ACTION");
        assert_eq!(action_ids::STOP_ACTION, "STOP_ACTION");
    }

    #[test]
    fn test_validate_task_name_valid() {
        let result = validate_task_name("API実装");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "API実装");
    }

    #[test]
    fn test_validate_task_name_too_long() {
        let long_name = "a".repeat(101);
        let result = validate_task_name(&long_name);
        assert!(result.is_err());
        match result {
            Err(NotificationError::InvalidInput(msg)) => {
                assert!(msg.contains("100文字以内"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_validate_task_name_with_control_chars() {
        let name_with_newline = "API\n実装";
        let result = validate_task_name(name_with_newline);
        assert!(result.is_err());
        match result {
            Err(NotificationError::InvalidInput(msg)) => {
                assert!(msg.contains("制御文字"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_validate_task_name_with_tab() {
        let name_with_tab = "API\t実装";
        let result = validate_task_name(name_with_tab);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_task_name_exact_limit() {
        let exact_limit = "a".repeat(100);
        let result = validate_task_name(&exact_limit);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_task_name_empty() {
        let result = validate_task_name("");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_task_name_unicode() {
        let unicode_name = "🍅ポモドーロタスク🎯";
        let result = validate_task_name(unicode_name);
        assert!(result.is_ok());
    }

    #[test]
    fn test_re_exports() {
        let _error = NotificationError::PermissionDenied;
        let _request = NotificationRequest::new("title", "body", "category");
        let _id = NotificationRequestId::new();
    }
}
