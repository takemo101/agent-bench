//! 通知リクエストの作成

use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NotificationRequestId(String);

impl NotificationRequestId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn from_string(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for NotificationRequestId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for NotificationRequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct NotificationRequest {
    pub id: NotificationRequestId,
    pub title: String,
    pub subtitle: Option<String>,
    pub body: String,
    pub category_id: String,
    pub play_sound: bool,
}

impl NotificationRequest {
    pub fn new(
        title: impl Into<String>,
        body: impl Into<String>,
        category_id: impl Into<String>,
    ) -> Self {
        Self {
            id: NotificationRequestId::new(),
            title: title.into(),
            subtitle: None,
            body: body.into(),
            category_id: category_id.into(),
            play_sound: true,
        }
    }

    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn with_sound(mut self, play_sound: bool) -> Self {
        self.play_sound = play_sound;
        self
    }

    pub fn with_id(mut self, id: NotificationRequestId) -> Self {
        self.id = id;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_request_id_new() {
        let id1 = NotificationRequestId::new();
        let id2 = NotificationRequestId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_notification_request_id_from_string() {
        let id = NotificationRequestId::from_string("test-id-123".to_string());
        assert_eq!(id.as_str(), "test-id-123");
    }

    #[test]
    fn test_notification_request_id_display() {
        let id = NotificationRequestId::from_string("display-test".to_string());
        assert_eq!(format!("{}", id), "display-test");
    }

    #[test]
    fn test_notification_request_id_default() {
        let id = NotificationRequestId::default();
        assert!(!id.as_str().is_empty());
    }

    #[test]
    fn test_notification_request_new() {
        let request = NotificationRequest::new("タイトル", "本文", "CATEGORY_ID");
        assert_eq!(request.title, "タイトル");
        assert_eq!(request.body, "本文");
        assert_eq!(request.category_id, "CATEGORY_ID");
        assert!(request.subtitle.is_none());
        assert!(request.play_sound);
    }

    #[test]
    fn test_notification_request_with_subtitle() {
        let request =
            NotificationRequest::new("タイトル", "本文", "CATEGORY").with_subtitle("サブタイトル");
        assert_eq!(request.subtitle, Some("サブタイトル".to_string()));
    }

    #[test]
    fn test_notification_request_with_sound() {
        let request = NotificationRequest::new("タイトル", "本文", "CATEGORY").with_sound(false);
        assert!(!request.play_sound);
    }

    #[test]
    fn test_notification_request_with_custom_id() {
        let custom_id = NotificationRequestId::from_string("custom-123".to_string());
        let request =
            NotificationRequest::new("タイトル", "本文", "CATEGORY").with_id(custom_id.clone());
        assert_eq!(request.id, custom_id);
    }

    #[test]
    fn test_notification_request_builder_chain() {
        let request = NotificationRequest::new("作業完了", "休憩してください", "WORK_COMPLETE")
            .with_subtitle("API実装")
            .with_sound(true);

        assert_eq!(request.title, "作業完了");
        assert_eq!(request.body, "休憩してください");
        assert_eq!(request.category_id, "WORK_COMPLETE");
        assert_eq!(request.subtitle, Some("API実装".to_string()));
        assert!(request.play_sound);
    }

    #[test]
    fn test_uuid_format() {
        let id = NotificationRequestId::new();
        let uuid_str = id.as_str();
        assert_eq!(uuid_str.len(), 36);
        assert_eq!(uuid_str.chars().nth(8), Some('-'));
        assert_eq!(uuid_str.chars().nth(13), Some('-'));
        assert_eq!(uuid_str.chars().nth(14), Some('4'));
        assert_eq!(uuid_str.chars().nth(18), Some('-'));
        assert_eq!(uuid_str.chars().nth(23), Some('-'));
    }

    #[test]
    fn test_uuid_uniqueness() {
        let mut uuids = std::collections::HashSet::new();
        for _ in 0..100 {
            let id = NotificationRequestId::new();
            assert!(
                uuids.insert(id.as_str().to_string()),
                "UUID collision detected"
            );
        }
    }
}
