use std::ptr::NonNull;
use std::sync::mpsc::Sender;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::{Bool, ProtocolObject};
use objc2_foundation::{NSError, NSSet};
use objc2_user_notifications::{
    UNAuthorizationOptions, UNAuthorizationStatus, UNNotificationCategory, UNNotificationRequest,
    UNNotificationSettings, UNUserNotificationCenter, UNUserNotificationCenterDelegate,
};

use super::error::NotificationError;

pub struct NotificationCenter {
    inner: Retained<UNUserNotificationCenter>,
}

impl NotificationCenter {
    pub fn shared() -> Self {
        let inner = UNUserNotificationCenter::currentNotificationCenter();
        Self { inner }
    }

    pub fn request_authorization(&self, sender: Sender<Result<bool, NotificationError>>) {
        let options = UNAuthorizationOptions::Alert
            | UNAuthorizationOptions::Sound
            | UNAuthorizationOptions::Badge;

        let block = RcBlock::new(move |granted: Bool, error: *mut NSError| {
            let result = if !error.is_null() {
                let err = unsafe { &*error };
                let description = err.localizedDescription().to_string();
                Err(NotificationError::AuthorizationFailed(description))
            } else {
                Ok(granted.as_bool())
            };
            let _ = sender.send(result);
        });

        self.inner
            .requestAuthorizationWithOptions_completionHandler(options, &block);
    }

    pub fn set_notification_categories(&self, categories: &NSSet<UNNotificationCategory>) {
        self.inner.setNotificationCategories(categories);
    }

    pub fn set_delegate(&self, delegate: &ProtocolObject<dyn UNUserNotificationCenterDelegate>) {
        self.inner.setDelegate(Some(delegate));
    }

    pub fn add_notification_request(
        &self,
        request: &UNNotificationRequest,
        sender: Sender<Result<(), NotificationError>>,
    ) {
        let block = RcBlock::new(move |error: *mut NSError| {
            let result = if !error.is_null() {
                let err = unsafe { &*error };
                let description = err.localizedDescription().to_string();
                Err(NotificationError::SendFailed(description))
            } else {
                Ok(())
            };
            let _ = sender.send(result);
        });

        self.inner
            .addNotificationRequest_withCompletionHandler(request, Some(&block));
    }

    pub fn get_notification_settings(&self, sender: Sender<UNAuthorizationStatus>) {
        let block = RcBlock::new(move |settings: NonNull<UNNotificationSettings>| {
            let status = unsafe { settings.as_ref().authorizationStatus() };
            let _ = sender.send(status);
        });

        self.inner
            .getNotificationSettingsWithCompletionHandler(&block);
    }

    pub fn is_authorized_sync(&self) -> bool {
        let (tx, rx) = std::sync::mpsc::channel();
        self.get_notification_settings(tx);
        match rx.recv() {
            Ok(status) => status == UNAuthorizationStatus::Authorized,
            Err(_) => false,
        }
    }
}

unsafe impl Send for NotificationCenter {}
unsafe impl Sync for NotificationCenter {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires signed app bundle context"]
    fn test_notification_center_shared() {
        let _center = NotificationCenter::shared();
    }

    #[test]
    fn test_notification_center_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<NotificationCenter>();
    }
}
