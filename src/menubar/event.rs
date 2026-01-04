//! イベントハンドラモジュール
//!
//! トレイアイコンとメニューのイベントを処理する。

use super::menu::MenuBuilder;
use tray_icon::menu::MenuEvent;
use tray_icon::TrayIconEvent;

/// メニューアクション
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    /// 一時停止
    Pause,
    /// 再開
    Resume,
    /// 停止
    Stop,
    /// アプリケーション終了
    Quit,
}

/// イベントハンドラ
pub struct EventHandler;

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl EventHandler {
    /// 新しいEventHandlerを作成
    pub fn new() -> Self {
        Self
    }

    /// イベントを処理してアクションを返す
    ///
    /// トレイアイコンクリックイベントとメニュークリックイベントを確認し、
    /// 対応するアクションがあれば返す。ノンブロッキングで動作する。
    ///
    /// # Arguments
    /// * `menu_builder` - メニュービルダー（ID照合用）
    ///
    /// # Returns
    /// 実行すべきアクション（あれば）
    pub fn check_events(&self, menu_builder: &MenuBuilder) -> Option<MenuAction> {
        // トレイアイコンイベントの処理
        if let Ok(event) = TrayIconEvent::receiver().try_recv() {
            // 左クリックでメニューを表示（macOSでは自動的に処理されることが多いが、
            // tray-icon crateの仕様によっては明示的な処理が必要な場合もある。
            // しかし、通常はOSがメニューを表示してくれる。
            // ここではログ出力や特定のインタラクションが必要な場合に処理を追加する。
            // 現時点ではアクションは生成しない。
            tracing::debug!("Tray icon event: {:?}", event);
        }

        // メニューイベントの処理
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            let ids = menu_builder.item_ids();

            if event.id == ids.pause {
                return Some(MenuAction::Pause);
            } else if event.id == ids.resume {
                return Some(MenuAction::Resume);
            } else if event.id == ids.stop {
                return Some(MenuAction::Stop);
            } else if event.id == ids.quit {
                return Some(MenuAction::Quit);
            }
        }

        None
    }
}
