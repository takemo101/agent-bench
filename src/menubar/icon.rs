//! アイコン管理モジュール
//!
//! メニューバーアイコンの生成・管理と残り時間テキストの動的生成を担当する。

use crate::types::{TimerPhase, TimerState};

use super::MenubarError;
use tray_icon::Icon;

/// アイコン管理
///
/// 状態に応じたアイコン画像の管理と、残り時間テキストの生成を行う。
pub struct IconManager {
    /// 作業中アイコン（🍅）
    working_icon: Icon,
    /// 休憩中アイコン（☕）
    breaking_icon: Icon,
    /// 停止中アイコン（グレー）
    stopped_icon: Icon,
}

impl IconManager {
    /// 新しいIconManagerを作成
    ///
    /// アイコンリソースを読み込み、IconManagerを初期化する。
    /// アイコンファイルが見つからない場合はデフォルトアイコンを使用する。
    pub fn new() -> Result<Self, MenubarError> {
        let default_icon = Self::create_default_icon()?;

        Ok(Self {
            working_icon: default_icon.clone(),
            breaking_icon: default_icon.clone(),
            stopped_icon: default_icon,
        })
    }

    /// 状態に応じたアイコンテキストを生成
    ///
    /// # テキスト形式
    /// - 作業中: `🍅 15:30`
    /// - 休憩中/長い休憩中: `☕ 04:30`
    /// - 一時停止中: `⏸ 一時停止`
    /// - 停止中: `⏸ 停止中`
    ///
    /// # Arguments
    /// * `state` - 現在のタイマー状態
    ///
    /// # Returns
    /// メニューバーに表示するテキスト
    pub fn generate_title(state: &TimerState) -> String {
        let minutes = state.remaining_seconds / 60;
        let seconds = state.remaining_seconds % 60;

        match state.phase {
            TimerPhase::Working => format!("🍅 {:02}:{:02}", minutes, seconds),
            TimerPhase::Breaking | TimerPhase::LongBreaking => {
                format!("☕ {:02}:{:02}", minutes, seconds)
            }
            TimerPhase::Paused => "⏸ 一時停止".to_string(),
            TimerPhase::Stopped => "⏸ 停止中".to_string(),
        }
    }

    /// 状態に応じたアイコンを取得
    ///
    /// # Arguments
    /// * `phase` - 現在のタイマーフェーズ
    ///
    /// # Returns
    /// 対応するアイコンへの参照
    pub fn get_icon(&self, phase: &TimerPhase) -> &Icon {
        match phase {
            TimerPhase::Working => &self.working_icon,
            TimerPhase::Breaking | TimerPhase::LongBreaking => &self.breaking_icon,
            TimerPhase::Stopped | TimerPhase::Paused => &self.stopped_icon,
        }
    }

    /// デフォルトアイコンを作成
    ///
    /// アイコンリソースが利用できない場合に使用するフォールバックアイコン。
    /// 22x22ピクセルのグレー単色アイコンを生成する。
    fn create_default_icon() -> Result<Icon, MenubarError> {
        let width = 22u32;
        let height = 22u32;
        // RGBA: 4 bytes per pixel. Gray color.
        let rgba: Vec<u8> = [128, 128, 128, 255].repeat((width * height) as usize);

        Icon::from_rgba(rgba, width, height).map_err(|e| MenubarError::BuildError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PomodoroConfig;

    // =========================================================================
    // IconManager::generate_title テスト
    // =========================================================================

    #[test]
    fn test_generate_title_working() {
        let mut state = TimerState::new(PomodoroConfig::default());
        state.phase = TimerPhase::Working;
        state.remaining_seconds = 930; // 15:30

        let title = IconManager::generate_title(&state);
        assert_eq!(title, "🍅 15:30");
    }

    #[test]
    fn test_generate_title_working_zero_padded() {
        let mut state = TimerState::new(PomodoroConfig::default());
        state.phase = TimerPhase::Working;
        state.remaining_seconds = 65; // 01:05

        let title = IconManager::generate_title(&state);
        assert_eq!(title, "🍅 01:05");
    }

    #[test]
    fn test_generate_title_breaking() {
        let mut state = TimerState::new(PomodoroConfig::default());
        state.phase = TimerPhase::Breaking;
        state.remaining_seconds = 270; // 04:30

        let title = IconManager::generate_title(&state);
        assert_eq!(title, "☕ 04:30");
    }

    #[test]
    fn test_generate_title_long_breaking() {
        let mut state = TimerState::new(PomodoroConfig::default());
        state.phase = TimerPhase::LongBreaking;
        state.remaining_seconds = 600; // 10:00

        let title = IconManager::generate_title(&state);
        assert_eq!(title, "☕ 10:00");
    }

    #[test]
    fn test_generate_title_paused() {
        let mut state = TimerState::new(PomodoroConfig::default());
        state.phase = TimerPhase::Paused;
        state.remaining_seconds = 500; // 残り時間は無視される

        let title = IconManager::generate_title(&state);
        assert_eq!(title, "⏸ 一時停止");
    }

    #[test]
    fn test_generate_title_stopped() {
        let state = TimerState::new(PomodoroConfig::default());
        // デフォルトはStopped, remaining_seconds = 0

        let title = IconManager::generate_title(&state);
        assert_eq!(title, "⏸ 停止中");
    }

    #[test]
    fn test_generate_title_zero_seconds() {
        let mut state = TimerState::new(PomodoroConfig::default());
        state.phase = TimerPhase::Working;
        state.remaining_seconds = 0;

        let title = IconManager::generate_title(&state);
        assert_eq!(title, "🍅 00:00");
    }

    #[test]
    fn test_generate_title_large_time() {
        let mut state = TimerState::new(PomodoroConfig::default());
        state.phase = TimerPhase::Working;
        state.remaining_seconds = 7200; // 120:00 (2時間)

        let title = IconManager::generate_title(&state);
        assert_eq!(title, "🍅 120:00");
    }

    // =========================================================================
    // IconManager::new テスト
    // =========================================================================

    #[test]
    fn test_icon_manager_new() {
        let result = IconManager::new();
        assert!(result.is_ok());
    }

    // =========================================================================
    // IconManager::get_icon テスト
    // =========================================================================

    #[test]
    fn test_get_icon_working() {
        let manager = IconManager::new().unwrap();
        let _icon = manager.get_icon(&TimerPhase::Working);
        // アイコンが取得できることを確認（内容の検証は困難）
    }

    #[test]
    fn test_get_icon_breaking() {
        let manager = IconManager::new().unwrap();
        let _icon = manager.get_icon(&TimerPhase::Breaking);
    }

    #[test]
    fn test_get_icon_long_breaking() {
        let manager = IconManager::new().unwrap();
        let _icon = manager.get_icon(&TimerPhase::LongBreaking);
    }

    #[test]
    fn test_get_icon_paused() {
        let manager = IconManager::new().unwrap();
        let _icon = manager.get_icon(&TimerPhase::Paused);
    }

    #[test]
    fn test_get_icon_stopped() {
        let manager = IconManager::new().unwrap();
        let _icon = manager.get_icon(&TimerPhase::Stopped);
    }

    // =========================================================================
    // IconManager::create_default_icon テスト
    // =========================================================================

    #[test]
    fn test_create_default_icon() {
        let result = IconManager::create_default_icon();
        assert!(result.is_ok());
    }
}
