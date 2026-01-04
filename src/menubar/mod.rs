pub mod event;
pub mod icon;
pub mod menu;

use crate::types::TimerState;
use event::{EventHandler, MenuAction};
use icon::IconManager;
use menu::MenuBuilder;
use thiserror::Error;
use tray_icon::{TrayIcon, TrayIconBuilder};

#[derive(Debug, Error)]
pub enum MenubarError {
    #[error("Failed to build tray icon: {0}")]
    BuildError(String),
    #[error("Tray icon not initialized")]
    NotInitialized,
    #[error("Menu error: {0}")]
    MenuError(String),
    #[error("Update error: {0}")]
    UpdateError(String),
}

pub struct TrayIconManager {
    tray_icon: Option<TrayIcon>,
    icon_manager: IconManager,
    menu_builder: MenuBuilder,
    event_handler: EventHandler,
}

impl TrayIconManager {
    /// 新しいTrayIconManagerを作成
    pub fn new() -> Result<Self, MenubarError> {
        let icon_manager = IconManager::new()?;
        let menu_builder = MenuBuilder::new()?;
        let event_handler = EventHandler::new();

        Ok(Self {
            tray_icon: None,
            icon_manager,
            menu_builder,
            event_handler,
        })
    }

    /// トレイアイコンを初期化して表示
    pub fn initialize(&mut self) -> Result<(), MenubarError> {
        // アイコンとメニューの初期状態を取得（デフォルトの停止状態などを想定）
        // ここではまだTimerStateがないので、デフォルトアイコンを使用
        // 本来はTimerStateを受け取るべきかもしれないが、initializeは起動時に呼ばれるので
        // デフォルト状態で構築する。

        // デフォルトのTimerStateを作成するコストが高い場合は、
        // IconManagerにデフォルトアイコンを返すメソッドがあると良いが、
        // get_iconにはTimerPhaseが必要。

        let working_phase = crate::types::TimerPhase::Stopped;
        let icon = self.icon_manager.get_icon(&working_phase);

        // メニューはまだ構築していない（update_stateで構築する）
        // ただし、空でもメニューを設定しないと表示されない可能性がある。
        // buildでデフォルト状態のメニューを作成する。
        let default_state = TimerState::new(crate::types::PomodoroConfig::default());
        self.menu_builder.build(&default_state)?;

        let tray_icon = TrayIconBuilder::new()
            .with_tooltip("Pomodoro Timer")
            .with_icon(icon.clone())
            .with_menu(Box::new(self.menu_builder.menu().clone()))
            .build()
            .map_err(|e| MenubarError::BuildError(e.to_string()))?;

        self.tray_icon = Some(tray_icon);

        Ok(())
    }

    /// タイマー状態に基づいてトレイアイコンとメニューを更新
    pub fn update_state(&mut self, state: &TimerState) -> Result<(), MenubarError> {
        if let Some(tray_icon) = &mut self.tray_icon {
            // アイコンの更新
            let icon = self.icon_manager.get_icon(&state.phase);
            tray_icon
                .set_icon(Some(icon.clone()))
                .map_err(|e| MenubarError::UpdateError(e.to_string()))?;

            // タイトルの更新（残り時間など）
            let title = IconManager::generate_title(state);
            tray_icon.set_title(Some(title));

            // メニューの再構築と更新
            self.menu_builder.build(state)?;
            tray_icon.set_menu(Some(Box::new(self.menu_builder.menu().clone())));

            // ツールチップの更新（オプション）
            // tray_icon.set_tooltip(Some(format!("State: {:?}", state.phase))) ...

            Ok(())
        } else {
            Err(MenubarError::NotInitialized)
        }
    }

    /// イベントを確認してアクションを返す
    pub fn check_events(&self) -> Option<MenuAction> {
        self.event_handler.check_events(&self.menu_builder)
    }

    /// トレイアイコンを破棄（アプリケーション終了時）
    pub fn shutdown(&mut self) -> Result<(), MenubarError> {
        // TrayIconはDropで破棄されるが、明示的に非表示にしたい場合など
        // set_visible(false) があれば呼ぶ（tray-icon 0.19にあるか確認）
        if let Some(tray_icon) = &mut self.tray_icon {
            let _ = tray_icon.set_visible(false);
        }
        self.tray_icon = None;
        Ok(())
    }
}
