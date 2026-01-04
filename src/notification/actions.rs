use objc2::rc::Retained;
use objc2_foundation::{NSArray, NSString};
use objc2_user_notifications::{
    UNNotificationAction, UNNotificationActionOptions, UNNotificationCategory,
    UNNotificationCategoryOptions,
};

use super::{action_ids, category_ids};

pub fn create_pause_action() -> Retained<UNNotificationAction> {
    let identifier = NSString::from_str(action_ids::PAUSE_ACTION);
    let title = NSString::from_str("一時停止");

    UNNotificationAction::actionWithIdentifier_title_options(
        &identifier,
        &title,
        UNNotificationActionOptions::empty(),
    )
}

pub fn create_stop_action() -> Retained<UNNotificationAction> {
    let identifier = NSString::from_str(action_ids::STOP_ACTION);
    let title = NSString::from_str("停止");

    UNNotificationAction::actionWithIdentifier_title_options(
        &identifier,
        &title,
        UNNotificationActionOptions::Destructive,
    )
}

pub fn create_actions() -> Retained<NSArray<UNNotificationAction>> {
    let pause_action = create_pause_action();
    let stop_action = create_stop_action();

    NSArray::from_retained_slice(&[pause_action, stop_action])
}

fn create_category(
    identifier: &str,
    actions: &NSArray<UNNotificationAction>,
) -> Retained<UNNotificationCategory> {
    let identifier = NSString::from_str(identifier);
    let intent_identifiers: Retained<NSArray<NSString>> = NSArray::new();

    UNNotificationCategory::categoryWithIdentifier_actions_intentIdentifiers_options(
        &identifier,
        actions,
        &intent_identifiers,
        UNNotificationCategoryOptions::empty(),
    )
}

pub fn create_categories() -> Retained<NSArray<UNNotificationCategory>> {
    let actions = create_actions();

    let work_complete = create_category(category_ids::WORK_COMPLETE, &actions);
    let break_complete = create_category(category_ids::BREAK_COMPLETE, &actions);
    let long_break_complete = create_category(category_ids::LONG_BREAK_COMPLETE, &actions);

    NSArray::from_retained_slice(&[work_complete, break_complete, long_break_complete])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_pause_action() {
        let action = create_pause_action();
        assert_eq!(action.identifier().to_string(), action_ids::PAUSE_ACTION);
        assert_eq!(action.title().to_string(), "一時停止");
    }

    #[test]
    fn test_create_stop_action() {
        let action = create_stop_action();
        assert_eq!(action.identifier().to_string(), action_ids::STOP_ACTION);
        assert_eq!(action.title().to_string(), "停止");
    }

    #[test]
    fn test_create_actions_count() {
        let actions = create_actions();
        assert_eq!(actions.count(), 2);
    }

    #[test]
    fn test_create_categories_count() {
        let categories = create_categories();
        assert_eq!(categories.count(), 3);
    }

    #[test]
    fn test_create_categories_identifiers() {
        let categories = create_categories();
        let identifiers: Vec<String> = categories
            .iter()
            .map(|c| c.identifier().to_string())
            .collect();

        assert!(identifiers.contains(&category_ids::WORK_COMPLETE.to_string()));
        assert!(identifiers.contains(&category_ids::BREAK_COMPLETE.to_string()));
        assert!(identifiers.contains(&category_ids::LONG_BREAK_COMPLETE.to_string()));
    }
}
