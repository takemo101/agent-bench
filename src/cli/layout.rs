//! レイアウトレンダラーモジュール
//!
//! インジケーター、時間表示、アニメーション、タスク名を3行構成のレイアウトに統合する。

use crate::cli::animation::AnimationFrame;
use crate::cli::time_format::TimeDisplay;
use crate::types::TimerPhase;
use colored::Colorize;
use unicode_width::UnicodeWidthStr;

/// 3行レイアウトの構造
#[derive(Debug, Clone)]
pub struct DisplayLayout {
    /// 1行目: フェーズ＋インジケーター＋時間
    pub line1: String,
    /// 2行目: アニメーション
    pub line2: String,
    /// 3行目: タスク名（オプション）
    pub line3: Option<String>,
    /// 行数（2 or 3）
    pub line_count: usize,
}

impl DisplayLayout {
    /// レイアウトを作成
    pub fn new(line1: String, line2: String, line3: Option<String>) -> Self {
        let line_count = if line3.is_some() { 3 } else { 2 };
        Self {
            line1,
            line2,
            line3,
            line_count,
        }
    }

    /// 全行をベクタで取得
    pub fn lines(&self) -> Vec<&str> {
        let mut result = vec![self.line1.as_str(), self.line2.as_str()];
        if let Some(ref line3) = self.line3 {
            result.push(line3.as_str());
        }
        result
    }

    /// 最大行幅を取得
    pub fn max_width(&self) -> usize {
        let mut max = UnicodeWidthStr::width(self.line1.as_str());
        max = max.max(UnicodeWidthStr::width(self.line2.as_str()));
        if let Some(ref line3) = self.line3 {
            max = max.max(UnicodeWidthStr::width(line3.as_str()));
        }
        max
    }
}

impl Default for DisplayLayout {
    fn default() -> Self {
        Self::new(String::new(), String::new(), None)
    }
}

/// レイアウトレンダラー
pub struct LayoutRenderer {
    /// ターミナル幅（80文字未満で簡略表示）
    terminal_width: usize,
    /// プログレスバー幅
    bar_width: usize,
}

impl LayoutRenderer {
    /// 新しいLayoutRendererを作成
    pub fn new(terminal_width: usize) -> Self {
        // プログレスバー幅を計算（ターミナル幅の40%、最大40文字）
        let bar_width = ((terminal_width as f32) * 0.4).min(40.0) as usize;
        Self {
            terminal_width,
            bar_width,
        }
    }

    /// デフォルトのターミナル幅（80文字）で作成
    pub fn with_default_width() -> Self {
        Self::new(80)
    }

    /// ターミナル幅を取得
    pub fn terminal_width(&self) -> usize {
        self.terminal_width
    }

    /// プログレスバー幅を取得
    pub fn bar_width(&self) -> usize {
        self.bar_width
    }

    /// レイアウトを構築
    ///
    /// # Arguments
    /// * `phase` - 現在のフェーズ
    /// * `time_display` - 時間表示情報
    /// * `animation_frame` - アニメーションフレーム
    /// * `task_name` - タスク名（オプション）
    /// * `progress_position` - プログレス位置
    /// * `progress_total` - プログレス合計
    pub fn build_layout(
        &self,
        phase: TimerPhase,
        time_display: &TimeDisplay,
        animation_frame: Option<&AnimationFrame>,
        task_name: Option<&str>,
        progress_position: u64,
        progress_total: u64,
    ) -> DisplayLayout {
        // 1行目: フェーズ + プログレスバー + 時間
        let line1 = self.build_line1(phase, time_display, progress_position, progress_total);

        // 2行目: アニメーション
        let line2 = match animation_frame {
            Some(frame) => frame.content.clone(),
            None => String::new(),
        };

        // 3行目: タスク名
        let line3 = task_name.map(|name| format!("タスク: {}", name.cyan()));

        DisplayLayout::new(line1, line2, line3)
    }

    /// 1行目を構築
    fn build_line1(
        &self,
        phase: TimerPhase,
        time_display: &TimeDisplay,
        position: u64,
        total: u64,
    ) -> String {
        let (icon, label, color) = Self::phase_style(phase);

        // プログレスバーを構築
        let bar = self.build_progress_bar(position, total);

        // 時間表示
        let time_str = time_display.format();

        // フェーズ表示（色付き）
        let phase_str = format!("{} {}", icon, label);
        let colored_phase = match color {
            "red" => phase_str.red().to_string(),
            "green" => phase_str.green().to_string(),
            "blue" => phase_str.blue().to_string(),
            "yellow" => phase_str.yellow().to_string(),
            _ => phase_str.white().to_string(),
        };

        format!("{} {} {}", colored_phase, bar, time_str)
    }

    /// プログレスバーを構築
    fn build_progress_bar(&self, position: u64, total: u64) -> String {
        if total == 0 {
            return format!("[{}]", "░".repeat(self.bar_width));
        }

        let filled = ((position as f64 / total as f64) * self.bar_width as f64) as usize;
        let empty = self.bar_width.saturating_sub(filled);

        format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
    }

    /// フェーズのスタイルを取得
    pub fn phase_style(phase: TimerPhase) -> (&'static str, &'static str, &'static str) {
        match phase {
            TimerPhase::Working => ("🍅", "作業中", "red"),
            TimerPhase::Breaking => ("☕", "休憩中", "green"),
            TimerPhase::LongBreaking => ("🛏️", "長期休憩中", "blue"),
            TimerPhase::Paused => ("⏸️", "一時停止", "yellow"),
            TimerPhase::Stopped => ("⏹", "停止", "white"),
        }
    }

    /// ターミナル幅を更新
    pub fn set_terminal_width(&mut self, width: usize) {
        self.terminal_width = width;
        self.bar_width = ((width as f32) * 0.4).min(40.0) as usize;
    }

    /// 独自レンダリング（indicatif非使用）
    pub fn render_standalone(&self, layout: &DisplayLayout) -> String {
        let mut output = String::new();
        output.push_str(&layout.line1);
        output.push('\n');
        output.push_str(&layout.line2);
        if let Some(ref line3) = layout.line3 {
            output.push('\n');
            output.push_str(line3);
        }
        output
    }
}

impl Default for LayoutRenderer {
    fn default() -> Self {
        Self::with_default_width()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------------
    // DisplayLayout Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_display_layout_new_with_task() {
        let layout = DisplayLayout::new(
            "line1".to_string(),
            "line2".to_string(),
            Some("line3".to_string()),
        );
        assert_eq!(layout.line_count, 3);
        assert_eq!(layout.line1, "line1");
        assert_eq!(layout.line2, "line2");
        assert_eq!(layout.line3, Some("line3".to_string()));
    }

    #[test]
    fn test_display_layout_new_without_task() {
        let layout = DisplayLayout::new("line1".to_string(), "line2".to_string(), None);
        assert_eq!(layout.line_count, 2);
        assert!(layout.line3.is_none());
    }

    #[test]
    fn test_display_layout_lines_with_task() {
        let layout = DisplayLayout::new(
            "a".to_string(),
            "b".to_string(),
            Some("c".to_string()),
        );
        let lines = layout.lines();
        assert_eq!(lines, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_display_layout_lines_without_task() {
        let layout = DisplayLayout::new("a".to_string(), "b".to_string(), None);
        let lines = layout.lines();
        assert_eq!(lines, vec!["a", "b"]);
    }

    #[test]
    fn test_display_layout_max_width() {
        let layout = DisplayLayout::new(
            "short".to_string(),
            "medium text".to_string(),
            Some("longest text here".to_string()),
        );
        assert_eq!(layout.max_width(), 17); // "longest text here" = 17
    }

    #[test]
    fn test_display_layout_max_width_unicode() {
        let layout = DisplayLayout::new(
            "日本語".to_string(), // 6 width (3 chars * 2)
            "test".to_string(),   // 4 width
            None,
        );
        assert_eq!(layout.max_width(), 6);
    }

    #[test]
    fn test_display_layout_default() {
        let layout = DisplayLayout::default();
        assert_eq!(layout.line1, "");
        assert_eq!(layout.line2, "");
        assert!(layout.line3.is_none());
        assert_eq!(layout.line_count, 2);
    }

    // ------------------------------------------------------------------------
    // LayoutRenderer Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_layout_renderer_new() {
        let renderer = LayoutRenderer::new(80);
        assert_eq!(renderer.terminal_width(), 80);
        assert_eq!(renderer.bar_width(), 32); // 80 * 0.4 = 32
    }

    #[test]
    fn test_layout_renderer_new_narrow() {
        let renderer = LayoutRenderer::new(40);
        assert_eq!(renderer.terminal_width(), 40);
        assert_eq!(renderer.bar_width(), 16); // 40 * 0.4 = 16
    }

    #[test]
    fn test_layout_renderer_new_wide() {
        let renderer = LayoutRenderer::new(120);
        assert_eq!(renderer.terminal_width(), 120);
        assert_eq!(renderer.bar_width(), 40); // capped at 40
    }

    #[test]
    fn test_layout_renderer_with_default_width() {
        let renderer = LayoutRenderer::with_default_width();
        assert_eq!(renderer.terminal_width(), 80);
    }

    #[test]
    fn test_layout_renderer_default() {
        let renderer = LayoutRenderer::default();
        assert_eq!(renderer.terminal_width(), 80);
    }

    #[test]
    fn test_layout_renderer_build_layout_working() {
        let renderer = LayoutRenderer::new(80);
        let time_display = TimeDisplay::new(323, 1500);
        let frame = AnimationFrame::new("🏃💨 ─────");

        let layout = renderer.build_layout(
            TimerPhase::Working,
            &time_display,
            Some(&frame),
            Some("テストタスク"),
            323,
            1500,
        );

        assert!(layout.line1.contains("作業中"));
        assert!(layout.line2.contains("🏃"));
        assert!(layout.line3.is_some());
        assert_eq!(layout.line_count, 3);
    }

    #[test]
    fn test_layout_renderer_build_layout_without_animation() {
        let renderer = LayoutRenderer::new(80);
        let time_display = TimeDisplay::new(0, 1500);

        let layout = renderer.build_layout(
            TimerPhase::Working,
            &time_display,
            None,
            None,
            0,
            1500,
        );

        assert!(layout.line1.contains("作業中"));
        assert_eq!(layout.line2, "");
        assert!(layout.line3.is_none());
        assert_eq!(layout.line_count, 2);
    }

    #[test]
    fn test_layout_renderer_build_layout_breaking() {
        let renderer = LayoutRenderer::new(80);
        let time_display = TimeDisplay::new(60, 300);

        let layout = renderer.build_layout(
            TimerPhase::Breaking,
            &time_display,
            None,
            None,
            60,
            300,
        );

        assert!(layout.line1.contains("休憩中"));
    }

    #[test]
    fn test_layout_renderer_build_layout_long_breaking() {
        let renderer = LayoutRenderer::new(80);
        let time_display = TimeDisplay::new(0, 900);

        let layout = renderer.build_layout(
            TimerPhase::LongBreaking,
            &time_display,
            None,
            None,
            0,
            900,
        );

        assert!(layout.line1.contains("長期休憩中"));
    }

    #[test]
    fn test_layout_renderer_build_layout_paused() {
        let renderer = LayoutRenderer::new(80);
        let time_display = TimeDisplay::new(500, 1500);

        let layout = renderer.build_layout(
            TimerPhase::Paused,
            &time_display,
            None,
            None,
            500,
            1500,
        );

        assert!(layout.line1.contains("一時停止"));
    }

    #[test]
    fn test_layout_renderer_build_layout_stopped() {
        let renderer = LayoutRenderer::new(80);
        let time_display = TimeDisplay::new(0, 0);

        let layout = renderer.build_layout(
            TimerPhase::Stopped,
            &time_display,
            None,
            None,
            0,
            0,
        );

        assert!(layout.line1.contains("停止"));
    }

    #[test]
    fn test_layout_renderer_build_progress_bar_zero() {
        let renderer = LayoutRenderer::new(80);
        let bar = renderer.build_progress_bar(0, 100);
        assert!(bar.starts_with('['));
        assert!(bar.ends_with(']'));
        assert!(bar.contains('░'));
    }

    #[test]
    fn test_layout_renderer_build_progress_bar_half() {
        let renderer = LayoutRenderer::new(80);
        let bar = renderer.build_progress_bar(50, 100);
        assert!(bar.contains('█'));
        assert!(bar.contains('░'));
    }

    #[test]
    fn test_layout_renderer_build_progress_bar_full() {
        let renderer = LayoutRenderer::new(80);
        let bar = renderer.build_progress_bar(100, 100);
        assert!(bar.contains('█'));
        // Should have no empty chars when 100%
    }

    #[test]
    fn test_layout_renderer_build_progress_bar_zero_total() {
        let renderer = LayoutRenderer::new(80);
        let bar = renderer.build_progress_bar(50, 0);
        // When total is 0, should show empty bar
        assert!(bar.contains('░'));
        assert!(!bar.contains('█'));
    }

    #[test]
    fn test_layout_renderer_phase_style_working() {
        let (icon, label, color) = LayoutRenderer::phase_style(TimerPhase::Working);
        assert_eq!(icon, "🍅");
        assert_eq!(label, "作業中");
        assert_eq!(color, "red");
    }

    #[test]
    fn test_layout_renderer_phase_style_breaking() {
        let (icon, label, color) = LayoutRenderer::phase_style(TimerPhase::Breaking);
        assert_eq!(icon, "☕");
        assert_eq!(label, "休憩中");
        assert_eq!(color, "green");
    }

    #[test]
    fn test_layout_renderer_phase_style_long_breaking() {
        let (icon, label, color) = LayoutRenderer::phase_style(TimerPhase::LongBreaking);
        assert_eq!(icon, "🛏️");
        assert_eq!(label, "長期休憩中");
        assert_eq!(color, "blue");
    }

    #[test]
    fn test_layout_renderer_phase_style_paused() {
        let (icon, label, color) = LayoutRenderer::phase_style(TimerPhase::Paused);
        assert_eq!(icon, "⏸️");
        assert_eq!(label, "一時停止");
        assert_eq!(color, "yellow");
    }

    #[test]
    fn test_layout_renderer_phase_style_stopped() {
        let (icon, label, color) = LayoutRenderer::phase_style(TimerPhase::Stopped);
        assert_eq!(icon, "⏹");
        assert_eq!(label, "停止");
        assert_eq!(color, "white");
    }

    #[test]
    fn test_layout_renderer_set_terminal_width() {
        let mut renderer = LayoutRenderer::new(80);
        renderer.set_terminal_width(100);
        assert_eq!(renderer.terminal_width(), 100);
        assert_eq!(renderer.bar_width(), 40); // 100 * 0.4 = 40, capped at 40
    }

    #[test]
    fn test_layout_renderer_render_standalone() {
        let renderer = LayoutRenderer::new(80);
        let layout = DisplayLayout::new(
            "line1".to_string(),
            "line2".to_string(),
            Some("line3".to_string()),
        );

        let output = renderer.render_standalone(&layout);
        assert!(output.contains("line1"));
        assert!(output.contains("line2"));
        assert!(output.contains("line3"));
        assert_eq!(output.matches('\n').count(), 2);
    }

    #[test]
    fn test_layout_renderer_render_standalone_without_task() {
        let renderer = LayoutRenderer::new(80);
        let layout = DisplayLayout::new("line1".to_string(), "line2".to_string(), None);

        let output = renderer.render_standalone(&layout);
        assert!(output.contains("line1"));
        assert!(output.contains("line2"));
        assert_eq!(output.matches('\n').count(), 1);
    }
}
