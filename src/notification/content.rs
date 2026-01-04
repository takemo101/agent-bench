use objc2::rc::Retained;
use objc2_foundation::NSString;
use objc2_user_notifications::{UNMutableNotificationContent, UNNotificationSound};

use super::category_ids;

pub struct NotificationContentBuilder {
    content: Retained<UNMutableNotificationContent>,
}

impl NotificationContentBuilder {
    pub fn new() -> Self {
        let content = UNMutableNotificationContent::new();
        Self { content }
    }

    pub fn title(self, title: &str) -> Self {
        let title = NSString::from_str(title);
        self.content.setTitle(&title);
        self
    }

    pub fn subtitle(self, subtitle: &str) -> Self {
        let subtitle = NSString::from_str(subtitle);
        self.content.setSubtitle(&subtitle);
        self
    }

    pub fn body(self, body: &str) -> Self {
        let body = NSString::from_str(body);
        self.content.setBody(&body);
        self
    }

    pub fn category_identifier(self, category_id: &str) -> Self {
        let category_id = NSString::from_str(category_id);
        self.content.setCategoryIdentifier(&category_id);
        self
    }

    pub fn with_default_sound(self) -> Self {
        let sound = UNNotificationSound::defaultSound();
        self.content.setSound(Some(&sound));
        self
    }

    pub fn build(self) -> Retained<UNMutableNotificationContent> {
        self.content
    }
}

impl Default for NotificationContentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub fn create_work_complete_content(
    task_name: Option<&str>,
) -> Retained<UNMutableNotificationContent> {
    let mut builder = NotificationContentBuilder::new()
        .title("🍅 ポモドーロタイマー")
        .body("作業時間が終了しました。休憩してください。")
        .category_identifier(category_ids::WORK_COMPLETE)
        .with_default_sound();

    if let Some(task) = task_name {
        builder = builder.subtitle(task);
    }

    builder.build()
}

pub fn create_break_complete_content(
    task_name: Option<&str>,
) -> Retained<UNMutableNotificationContent> {
    let mut builder = NotificationContentBuilder::new()
        .title("☕ ポモドーロタイマー")
        .body("休憩時間が終了しました。作業を再開してください。")
        .category_identifier(category_ids::BREAK_COMPLETE)
        .with_default_sound();

    if let Some(task) = task_name {
        builder = builder.subtitle(task);
    }

    builder.build()
}

pub fn create_long_break_complete_content(
    task_name: Option<&str>,
) -> Retained<UNMutableNotificationContent> {
    let mut builder = NotificationContentBuilder::new()
        .title("☕ ポモドーロタイマー")
        .body("長い休憩時間が終了しました。作業を再開してください。")
        .category_identifier(category_ids::LONG_BREAK_COMPLETE)
        .with_default_sound();

    if let Some(task) = task_name {
        builder = builder.subtitle(task);
    }

    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_content_builder_default() {
        let _builder = NotificationContentBuilder::default();
    }

    #[test]
    fn test_notification_content_builder_chain() {
        let content = NotificationContentBuilder::new()
            .title("Test Title")
            .body("Test Body")
            .subtitle("Test Subtitle")
            .category_identifier("TEST_CATEGORY")
            .with_default_sound()
            .build();

        assert_eq!(content.title().to_string(), "Test Title");
        assert_eq!(content.body().to_string(), "Test Body");
        assert_eq!(content.subtitle().to_string(), "Test Subtitle");
        assert_eq!(content.categoryIdentifier().to_string(), "TEST_CATEGORY");
        assert!(content.sound().is_some());
    }

    #[test]
    fn test_create_work_complete_content_without_task() {
        let content = create_work_complete_content(None);

        assert_eq!(content.title().to_string(), "🍅 ポモドーロタイマー");
        assert!(content.body().to_string().contains("作業時間が終了"));
        assert_eq!(
            content.categoryIdentifier().to_string(),
            category_ids::WORK_COMPLETE
        );
    }

    #[test]
    fn test_create_work_complete_content_with_task() {
        let content = create_work_complete_content(Some("API実装"));

        assert_eq!(content.title().to_string(), "🍅 ポモドーロタイマー");
        assert_eq!(content.subtitle().to_string(), "API実装");
    }

    #[test]
    fn test_create_break_complete_content_without_task() {
        let content = create_break_complete_content(None);

        assert_eq!(content.title().to_string(), "☕ ポモドーロタイマー");
        assert!(content.body().to_string().contains("休憩時間が終了"));
        assert_eq!(
            content.categoryIdentifier().to_string(),
            category_ids::BREAK_COMPLETE
        );
    }

    #[test]
    fn test_create_break_complete_content_with_task() {
        let content = create_break_complete_content(Some("レビュー対応"));

        assert_eq!(content.subtitle().to_string(), "レビュー対応");
    }

    #[test]
    fn test_create_long_break_complete_content_without_task() {
        let content = create_long_break_complete_content(None);

        assert_eq!(content.title().to_string(), "☕ ポモドーロタイマー");
        assert!(content.body().to_string().contains("長い休憩時間が終了"));
        assert_eq!(
            content.categoryIdentifier().to_string(),
            category_ids::LONG_BREAK_COMPLETE
        );
    }

    #[test]
    fn test_create_long_break_complete_content_with_task() {
        let content = create_long_break_complete_content(Some("ドキュメント作成"));

        assert_eq!(content.subtitle().to_string(), "ドキュメント作成");
    }

    #[test]
    fn test_all_content_has_sound() {
        assert!(create_work_complete_content(None).sound().is_some());
        assert!(create_break_complete_content(None).sound().is_some());
        assert!(create_long_break_complete_content(None).sound().is_some());
    }
}
