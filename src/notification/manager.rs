use std::sync::mpsc::{channel, Receiver, RecvError, TryRecvError};

use objc2::rc::Retained;
use objc2::MainThreadMarker;
use objc2_foundation::{NSSet, NSString};
use objc2_user_notifications::{UNMutableNotificationContent, UNNotificationRequest};

use super::actions::create_categories;
use super::center::NotificationCenter;
use super::content::{
    create_break_complete_content, create_long_break_complete_content, create_work_complete_content,
};
use super::delegate::{NotificationActionEvent, NotificationDelegate};
use super::error::NotificationError;
use super::request::NotificationRequestId;

fn create_un_request(
    request_id: &NotificationRequestId,
    content: &UNMutableNotificationContent,
) -> Retained<UNNotificationRequest> {
    let identifier = NSString::from_str(request_id.as_str());
    UNNotificationRequest::requestWithIdentifier_content_trigger(&identifier, content, None)
}

pub struct NotificationManager {
    center: NotificationCenter,
    _delegate: Retained<NotificationDelegate>,
    action_receiver: Receiver<NotificationActionEvent>,
}

impl NotificationManager {
    pub fn new(mtm: MainThreadMarker) -> Result<Self, NotificationError> {
        let center = NotificationCenter::try_shared()?;

        let (auth_tx, auth_rx) = channel();
        center.request_authorization(auth_tx);

        let granted = auth_rx
            .recv()
            .map_err(|_| NotificationError::InitializationFailed("チャネル受信エラー".to_string()))?
            .map_err(|e| NotificationError::AuthorizationFailed(e.to_string()))?;

        if !granted {
            return Err(NotificationError::PermissionDenied);
        }

        let (action_tx, action_rx) = channel();
        let delegate = NotificationDelegate::new(mtm, action_tx);

        center.set_delegate(delegate.as_protocol());

        let categories = create_categories();
        let category_set =
            NSSet::from_retained_slice(categories.iter().collect::<Vec<_>>().as_slice());
        center.set_notification_categories(&category_set);

        Ok(Self {
            center,
            _delegate: delegate,
            action_receiver: action_rx,
        })
    }

    pub fn send_work_complete_notification(
        &self,
        task_name: Option<&str>,
    ) -> Result<NotificationRequestId, NotificationError> {
        let content = create_work_complete_content(task_name);
        let request_id = NotificationRequestId::new();
        let request = create_un_request(&request_id, &content);

        let (tx, rx) = channel();
        self.center.add_notification_request(&request, tx);

        rx.recv()
            .map_err(|_| NotificationError::SendFailed("チャネル受信エラー".to_string()))??;

        Ok(request_id)
    }

    pub fn send_break_complete_notification(
        &self,
        task_name: Option<&str>,
    ) -> Result<NotificationRequestId, NotificationError> {
        let content = create_break_complete_content(task_name);
        let request_id = NotificationRequestId::new();
        let request = create_un_request(&request_id, &content);

        let (tx, rx) = channel();
        self.center.add_notification_request(&request, tx);

        rx.recv()
            .map_err(|_| NotificationError::SendFailed("チャネル受信エラー".to_string()))??;

        Ok(request_id)
    }

    pub fn send_long_break_complete_notification(
        &self,
        task_name: Option<&str>,
    ) -> Result<NotificationRequestId, NotificationError> {
        let content = create_long_break_complete_content(task_name);
        let request_id = NotificationRequestId::new();
        let request = create_un_request(&request_id, &content);

        let (tx, rx) = channel();
        self.center.add_notification_request(&request, tx);

        rx.recv()
            .map_err(|_| NotificationError::SendFailed("チャネル受信エラー".to_string()))??;

        Ok(request_id)
    }

    pub fn try_recv_action(&self) -> Result<NotificationActionEvent, TryRecvError> {
        self.action_receiver.try_recv()
    }

    pub fn recv_action(&self) -> Result<NotificationActionEvent, RecvError> {
        self.action_receiver.recv()
    }

    pub fn is_authorized(&self) -> bool {
        self.center.is_authorized_sync()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires signed app bundle context and main thread"]
    fn test_notification_manager_new() {
        let mtm = MainThreadMarker::new().expect("must be on main thread");
        let _manager = NotificationManager::new(mtm).expect("failed to create manager");
    }

    #[test]
    #[ignore = "requires signed app bundle context and main thread"]
    fn test_send_work_complete_notification() {
        let mtm = MainThreadMarker::new().expect("must be on main thread");
        let manager = NotificationManager::new(mtm).expect("failed to create manager");
        let result = manager.send_work_complete_notification(Some("テストタスク"));
        assert!(result.is_ok());
    }

    #[test]
    #[ignore = "requires signed app bundle context and main thread"]
    fn test_send_break_complete_notification() {
        let mtm = MainThreadMarker::new().expect("must be on main thread");
        let manager = NotificationManager::new(mtm).expect("failed to create manager");
        let result = manager.send_break_complete_notification(None);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore = "requires signed app bundle context and main thread"]
    fn test_try_recv_action_empty() {
        let mtm = MainThreadMarker::new().expect("must be on main thread");
        let manager = NotificationManager::new(mtm).expect("failed to create manager");
        let result = manager.try_recv_action();
        assert!(result.is_err());
    }
}
