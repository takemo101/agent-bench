use std::sync::mpsc::Sender;

use block2::Block;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{define_class, msg_send, DefinedClass, MainThreadMarker, MainThreadOnly};
use objc2_foundation::{NSObject, NSObjectProtocol};
use objc2_user_notifications::{
    UNNotification, UNNotificationPresentationOptions, UNNotificationResponse,
    UNUserNotificationCenter, UNUserNotificationCenterDelegate,
};

use super::action_ids;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationActionEvent {
    Pause,
    Stop,
}

#[derive(Clone)]
pub struct NotificationDelegateIvars {
    action_sender: Option<Sender<NotificationActionEvent>>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "PomodoroNotificationDelegate"]
    #[ivars = NotificationDelegateIvars]
    pub struct NotificationDelegate;

    unsafe impl NSObjectProtocol for NotificationDelegate {}

    unsafe impl UNUserNotificationCenterDelegate for NotificationDelegate {
        #[unsafe(method(userNotificationCenter:didReceiveNotificationResponse:withCompletionHandler:))]
        fn did_receive_notification_response(
            &self,
            _center: &UNUserNotificationCenter,
            response: &UNNotificationResponse,
            completion_handler: &Block<dyn Fn()>,
        ) {
            let action_identifier = response.actionIdentifier().to_string();

            let event = match action_identifier.as_str() {
                action_ids::PAUSE_ACTION => Some(NotificationActionEvent::Pause),
                action_ids::STOP_ACTION => Some(NotificationActionEvent::Stop),
                _ => None,
            };

            if let Some(event) = event {
                if let Some(sender) = &self.ivars().action_sender {
                    let _ = sender.send(event);
                }
            }

            completion_handler.call(());
        }

        #[unsafe(method(userNotificationCenter:willPresentNotification:withCompletionHandler:))]
        fn will_present_notification(
            &self,
            _center: &UNUserNotificationCenter,
            _notification: &UNNotification,
            completion_handler: &Block<dyn Fn(UNNotificationPresentationOptions)>,
        ) {
            let options = UNNotificationPresentationOptions::Banner
                | UNNotificationPresentationOptions::Sound
                | UNNotificationPresentationOptions::List;

            completion_handler.call((options,));
        }
    }
);

impl NotificationDelegate {
    pub fn new(
        mtm: MainThreadMarker,
        action_sender: Sender<NotificationActionEvent>,
    ) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(NotificationDelegateIvars {
            action_sender: Some(action_sender),
        });
        unsafe { msg_send![super(this), init] }
    }

    pub fn as_protocol(&self) -> &ProtocolObject<dyn UNUserNotificationCenterDelegate> {
        ProtocolObject::from_ref(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_action_event_debug() {
        let pause = NotificationActionEvent::Pause;
        let stop = NotificationActionEvent::Stop;
        assert_eq!(format!("{:?}", pause), "Pause");
        assert_eq!(format!("{:?}", stop), "Stop");
    }

    #[test]
    fn test_notification_action_event_clone() {
        let pause = NotificationActionEvent::Pause;
        let cloned = pause.clone();
        assert_eq!(pause, cloned);
    }

    #[test]
    fn test_notification_action_event_equality() {
        assert_eq!(
            NotificationActionEvent::Pause,
            NotificationActionEvent::Pause
        );
        assert_eq!(NotificationActionEvent::Stop, NotificationActionEvent::Stop);
        assert_ne!(
            NotificationActionEvent::Pause,
            NotificationActionEvent::Stop
        );
    }
}
