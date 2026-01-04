//! メニュー構築モジュール
//!
//! トレイアイコンのドロップダウンメニューを構築・管理する。

use super::MenubarError;
use crate::types::{TimerPhase, TimerState};
use tray_icon::menu::{Menu, MenuId, MenuItem, PredefinedMenuItem};

/// メニュー項目のID管理
#[derive(Debug, Clone)]
pub struct MenuItemIds {
    pub pause: MenuId,
    pub resume: MenuId,
    pub stop: MenuId,
    pub quit: MenuId,
}

impl Default for MenuItemIds {
    fn default() -> Self {
        Self {
            pause: MenuId::new("pause"),
            resume: MenuId::new("resume"),
            stop: MenuId::new("stop"),
            quit: MenuId::new("quit"),
        }
    }
}

/// メニュー構築
pub struct MenuBuilder {
    /// メニューインスタンス
    menu: Menu,
    /// メニュー項目ID
    item_ids: MenuItemIds,
}

impl MenuBuilder {
    /// 新しいMenuBuilderを作成
    pub fn new() -> Result<Self, MenubarError> {
        let menu = Menu::new();
        let item_ids = MenuItemIds::default();

        Ok(Self { menu, item_ids })
    }

    /// メニューを取得
    pub fn menu(&self) -> &Menu {
        &self.menu
    }

    /// メニュー項目IDを取得
    pub fn item_ids(&self) -> &MenuItemIds {
        &self.item_ids
    }

    /// 状態に基づいてメニューを構築
    ///
    /// # Arguments
    /// * `state` - 現在のタイマー状態
    pub fn build(&mut self, state: &TimerState) -> Result<(), MenubarError> {
        // メニューをクリア（tray-icon 0.19にはclearがないかもしれないので、既存のアイテムを削除するか、
        // 毎回新しいMenuを作る必要があるか確認が必要だが、Menu::new()で作ったメニューにappendしていくスタイルにする）
        // tray-iconのMenuは内部可変性を持っているので、参照だけで操作可能。
        // ただし、項目を全削除するAPIがあるか確認。
        // ない場合は、remove_all的なことをするか、Menuごと作り直す必要がある。
        // ここでは、設計書に従い、appendしていくが、実際にはupdate_menu_stateで有効無効を切り替える運用にするのが良さそう。
        // しかし、"残り時間"などのテキストが変わるアイテムは再生成が必要。

        // メニュー項目の全削除はAPIにないので、items()で取得して削除するなどが考えられるが、
        // 単純にメニュー全体を作り直して set_menu する方が簡単かもしれない。
        // しかし、TrayIconManager側で set_menu するには、MenuBuilderが新しいMenuを返す必要がある。

        // 設計書では `pub fn build(&mut self, state: &TimerState) -> Result<&Menu>` となっている。
        // 毎回Menuを作り直して返す実装にする。

        self.menu = Menu::new();

        // タイトル（無効化）
        let title = MenuItem::with_id(MenuId::new("title"), "ポモドーロタイマー", false, None);
        self.menu
            .append(&title)
            .map_err(|e| MenubarError::MenuError(e.to_string()))?;

        self.menu
            .append(&PredefinedMenuItem::separator())
            .map_err(|e| MenubarError::MenuError(e.to_string()))?;

        // 状態表示（無効化）
        if let Some(task_name) = &state.task_name {
            let status = MenuItem::with_id(
                MenuId::new("status_task"),
                format!("作業中: {}", task_name),
                false,
                None,
            );
            self.menu
                .append(&status)
                .map_err(|e| MenubarError::MenuError(e.to_string()))?;
        }

        let remaining_minutes = state.remaining_seconds / 60;
        let remaining_seconds = state.remaining_seconds % 60;
        let remaining = MenuItem::with_id(
            MenuId::new("status_time"),
            format!(
                "残り時間: {:02}:{:02}",
                remaining_minutes, remaining_seconds
            ),
            false,
            None,
        );
        self.menu
            .append(&remaining)
            .map_err(|e| MenubarError::MenuError(e.to_string()))?;

        let count = MenuItem::with_id(
            MenuId::new("status_count"),
            format!("ポモドーロ: #{}", state.pomodoro_count),
            false,
            None,
        );
        self.menu
            .append(&count)
            .map_err(|e| MenubarError::MenuError(e.to_string()))?;

        self.menu
            .append(&PredefinedMenuItem::separator())
            .map_err(|e| MenubarError::MenuError(e.to_string()))?;

        // 操作メニュー
        let pause_enabled = matches!(
            state.phase,
            TimerPhase::Working | TimerPhase::Breaking | TimerPhase::LongBreaking
        );
        let pause = MenuItem::with_id(
            self.item_ids.pause.clone(),
            "⏸ 一時停止",
            pause_enabled,
            None,
        );
        self.menu
            .append(&pause)
            .map_err(|e| MenubarError::MenuError(e.to_string()))?;

        let resume_enabled = matches!(state.phase, TimerPhase::Paused);
        let resume =
            MenuItem::with_id(self.item_ids.resume.clone(), "▶ 再開", resume_enabled, None);
        self.menu
            .append(&resume)
            .map_err(|e| MenubarError::MenuError(e.to_string()))?;

        let stop_enabled = !matches!(state.phase, TimerPhase::Stopped);
        let stop = MenuItem::with_id(self.item_ids.stop.clone(), "⏹ 停止", stop_enabled, None);
        self.menu
            .append(&stop)
            .map_err(|e| MenubarError::MenuError(e.to_string()))?;

        self.menu
            .append(&PredefinedMenuItem::separator())
            .map_err(|e| MenubarError::MenuError(e.to_string()))?;

        // 終了
        let quit = MenuItem::with_id(self.item_ids.quit.clone(), "終了", true, None);
        self.menu
            .append(&quit)
            .map_err(|e| MenubarError::MenuError(e.to_string()))?;

        Ok(())
    }
}
