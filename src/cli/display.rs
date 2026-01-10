//! Display utilities for CLI output
//!
//! Provides colored and formatted output for CLI commands.
//! Integrates TimeDisplay, AnimationEngine, LayoutRenderer, and TerminalController
//! for enhanced visual feedback.

use crate::cli::animation::{AnimationEngine, AnimationFrame};
use crate::cli::layout::LayoutRenderer;
use crate::cli::terminal::TerminalController;
use crate::cli::time_format::TimeDisplay;
use crate::types::{IpcResponse, TimerPhase};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::str::FromStr;

/// Display handler for CLI output
///
/// Provides two display modes:
/// - Legacy mode: Uses indicatif ProgressBar (for backward compatibility)
/// - Enhanced mode: Uses LayoutRenderer + TerminalController for flicker-free animated display
pub struct Display;

/// Enhanced display state for animated updates
pub struct EnhancedDisplayState {
    /// Layout renderer for building 3-line display
    pub layout_renderer: LayoutRenderer,
    /// Terminal controller for flicker-free updates
    pub terminal_controller: TerminalController,
    /// Animation engine for phase-specific animations
    pub animation_engine: AnimationEngine,
    /// Current phase (for detecting phase changes)
    pub current_phase: Option<TimerPhase>,
}

impl EnhancedDisplayState {
    /// Create new enhanced display state
    pub fn new() -> Self {
        Self {
            layout_renderer: LayoutRenderer::default(),
            terminal_controller: TerminalController::default(),
            animation_engine: AnimationEngine::new(),
            current_phase: None,
        }
    }

    /// Update the display with new timer data
    ///
    /// Returns `true` if the loop should continue, `false` if it should stop
    pub fn update(
        &mut self,
        phase: TimerPhase,
        elapsed: u64,
        total: u64,
        task_name: Option<&str>,
    ) -> std::io::Result<bool> {
        // Stop if timer is stopped
        if phase == TimerPhase::Stopped {
            self.terminal_controller.clear()?;
            return Ok(false);
        }

        // Reset animation on phase change
        if self.current_phase != Some(phase) {
            self.animation_engine.reset();
            self.current_phase = Some(phase);
        }

        // Tick animation
        self.animation_engine.tick();

        // Get animation frame
        let frame_content = self.animation_engine.get_current_frame(phase);
        let frame = frame_content.as_ref().map(|c| AnimationFrame::new(c.as_str()));

        // Build time display
        let time_display = TimeDisplay::new(elapsed, total);

        // Build layout
        let layout = self.layout_renderer.build_layout(
            phase,
            &time_display,
            frame.as_ref(),
            task_name,
            elapsed,
            total,
        );

        // Render to terminal
        self.terminal_controller.render(&layout)?;

        Ok(true)
    }

    /// Clear the display
    pub fn clear(&mut self) -> std::io::Result<()> {
        self.terminal_controller.clear()
    }
}

impl Default for EnhancedDisplayState {
    fn default() -> Self {
        Self::new()
    }
}

impl Display {
    // Helper to create styled progress bar (legacy mode)
    fn create_progress_bar(
        &self,
        phase: TimerPhase,
        total_seconds: u64,
        remaining_seconds: u64,
        task_name: Option<&str>,
    ) -> ProgressBar {
        let (color_code, icon, label) = match phase {
            TimerPhase::Working => ("red", "🍅", "作業中"),
            TimerPhase::Breaking => ("green", "☕", "休憩中"),
            TimerPhase::LongBreaking => ("blue", "💤", "長期休憩"),
            TimerPhase::Paused => ("yellow", "⏸", "一時停止"),
            _ => ("white", "⏹", "停止"),
        };

        let template = format!(
            "{{prefix}} [{{bar:40.{}}}] {{pos}}/{{len}} ({{percent}}%)\n{{msg}}",
            color_code
        );

        let style = ProgressStyle::with_template(&template)
            .unwrap()
            .progress_chars("█░");

        let bar = ProgressBar::new(total_seconds);
        bar.set_style(style);
        // Position in indicatif is usually "completed", so total - remaining
        bar.set_position(total_seconds.saturating_sub(remaining_seconds));

        // Prefix with color
        let prefix = format!("{} {}", icon, label).color(color_code).to_string();
        bar.set_prefix(prefix);

        // Message (Task Name)
        if let Some(name) = task_name {
            bar.set_message(format!("タスク: {}", name.cyan()));
        }

        bar
    }

    /// Create a new Display instance
    pub fn new() -> Self {
        Self
    }

    /// Show success message
    pub fn show_success(&self, msg: &str) {
        println!("{} {}", "✓".green().bold(), msg.green());
    }

    /// Show start success message
    pub fn show_start_success(&self, response: IpcResponse) {
        println!("{} {}", "✓".green().bold(), response.message.green());
        if let Some(data) = response.data {
            if let Some(task) = data.task_name {
                println!("  タスク: {}", task.cyan());
            }
        }
    }

    /// Show pause success message
    pub fn show_pause_success(&self, response: IpcResponse) {
        println!("{} {}", "⏸".yellow().bold(), response.message.yellow());
    }

    /// Show resume success message
    pub fn show_resume_success(&self, response: IpcResponse) {
        println!("{} {}", "▶".green().bold(), response.message.green());
    }

    /// Show stop success message
    pub fn show_stop_success(&self, response: IpcResponse) {
        println!("{} {}", "■".red().bold(), response.message.red());
    }

    /// Show status information (one-shot display using new layout)
    pub fn show_status(&self, response: IpcResponse) {
        if let Some(data) = response.data {
            println!("{}", "=== タイマー状態 ===".bold());

            let phase = data
                .state
                .as_deref()
                .and_then(|s| TimerPhase::from_str(s).ok())
                .unwrap_or(TimerPhase::Stopped);

            // インジケーター表示（durationがある場合のみ）
            if let (Some(remaining), Some(duration)) = (data.remaining_seconds, data.duration) {
                let elapsed = (duration as u64).saturating_sub(remaining as u64);
                let total = duration as u64;

                // Use new LayoutRenderer for display
                let renderer = LayoutRenderer::default();
                let time_display = TimeDisplay::new(elapsed, total);

                let layout = renderer.build_layout(
                    phase,
                    &time_display,
                    None, // No animation for one-shot display
                    data.task_name.as_deref(),
                    elapsed,
                    total,
                );

                // Print layout lines
                for line in layout.lines() {
                    println!("{}", line);
                }
            } else {
                // 従来のテキスト表示（後方互換性のため）
                let state_display = match phase {
                    TimerPhase::Working => "作業中".green(),
                    TimerPhase::Breaking => "休憩中".cyan(),
                    TimerPhase::LongBreaking => "長い休憩中".cyan(),
                    TimerPhase::Paused => "一時停止".yellow(),
                    TimerPhase::Stopped => "停止中".red(),
                };
                println!("状態: {}", state_display);

                if let Some(remaining) = data.remaining_seconds {
                    let minutes = remaining / 60;
                    let seconds = remaining % 60;
                    println!("残り時間: {}:{:02}", minutes, seconds);
                }

                if let Some(task) = &data.task_name {
                    println!("タスク: {}", task.cyan());
                }
            }

            if let Some(count) = data.pomodoro_count {
                println!("完了ポモドーロ: {} 🍅", count);
            }
        } else {
            println!("{}", response.message);
        }
    }

    /// Show error message
    pub fn show_error(&self, msg: &str) {
        eprintln!("{} {}", "✗".red().bold(), msg.red());
    }

    /// Show install success message
    pub fn show_install_success(&self) {
        println!(
            "{} {}",
            "✓".green().bold(),
            "LaunchAgentをインストールしました".green()
        );
        println!("  次回ログイン時から自動起動します");
    }

    /// Show install failure message
    pub fn show_install_failure(&self, msg: &str) {
        eprintln!(
            "{} {}",
            "✗".red().bold(),
            "LaunchAgentのインストールに失敗しました".red()
        );
        eprintln!("  {}", msg);
    }

    /// Show uninstall success message
    pub fn show_uninstall_success(&self) {
        println!(
            "{} {}",
            "✓".green().bold(),
            "LaunchAgentをアンインストールしました".green()
        );
    }

    /// Show uninstall failure message
    pub fn show_uninstall_failure(&self, msg: &str) {
        eprintln!(
            "{} {}",
            "✗".red().bold(),
            "LaunchAgentのアンインストールに失敗しました".red()
        );
        eprintln!("  {}", msg);
    }

    /// Update status information in a loop (legacy mode using indicatif)
    /// Returns true if the loop should continue, false if it should stop
    pub fn update_status(&self, response: IpcResponse, bar: &mut Option<ProgressBar>) -> bool {
        if let Some(data) = response.data {
            let phase = data
                .state
                .as_deref()
                .and_then(|s| TimerPhase::from_str(s).ok())
                .unwrap_or(TimerPhase::Stopped);

            // 停止状態なら終了
            if phase == TimerPhase::Stopped {
                if let Some(b) = bar {
                    b.finish_with_message("停止中");
                } else {
                    println!("状態: 停止中");
                }
                return false;
            }

            if let (Some(remaining), Some(duration)) = (data.remaining_seconds, data.duration) {
                let duration_u64 = duration as u64;
                let remaining_u64 = remaining as u64;

                let b = if let Some(b) = bar {
                    if b.length() != Some(duration_u64) {
                        b.set_length(duration_u64);
                    }
                    b
                } else {
                    let new_bar = self.create_progress_bar(
                        phase,
                        duration_u64,
                        remaining_u64,
                        data.task_name.as_deref(),
                    );
                    *bar = Some(new_bar);
                    bar.as_mut().unwrap()
                };

                b.set_position(duration_u64.saturating_sub(remaining_u64));

                // フェーズ表示（Prefix）の更新
                let (color_code, icon, label) = match phase {
                    TimerPhase::Working => ("red", "🍅", "作業中"),
                    TimerPhase::Breaking => ("green", "☕", "休憩中"),
                    TimerPhase::LongBreaking => ("blue", "💤", "長期休憩"),
                    TimerPhase::Paused => ("yellow", "⏸", "一時停止"),
                    _ => ("white", "⏹", "停止"),
                };
                let prefix = format!("{} {}", icon, label).color(color_code).to_string();
                b.set_prefix(prefix);
            } else {
                // 時間情報がない場合
                println!("{}", response.message);
                return false;
            }

            true
        } else {
            // データなし
            println!("{}", response.message);
            false
        }
    }

    /// Update status using enhanced display (with animation)
    ///
    /// This method uses the new LayoutRenderer + TerminalController for
    /// flicker-free animated display. Call this in a loop with 200ms intervals.
    ///
    /// Returns `true` if the loop should continue, `false` if it should stop
    pub fn update_status_enhanced(
        &self,
        response: IpcResponse,
        state: &mut EnhancedDisplayState,
    ) -> bool {
        if let Some(data) = response.data {
            let phase = data
                .state
                .as_deref()
                .and_then(|s| TimerPhase::from_str(s).ok())
                .unwrap_or(TimerPhase::Stopped);

            if let (Some(remaining), Some(duration)) = (data.remaining_seconds, data.duration) {
                let elapsed = (duration as u64).saturating_sub(remaining as u64);
                let total = duration as u64;

                match state.update(phase, elapsed, total, data.task_name.as_deref()) {
                    Ok(should_continue) => should_continue,
                    Err(_) => {
                        // Fall back to simple text on terminal error
                        println!("状態: {:?}", phase);
                        false
                    }
                }
            } else {
                println!("{}", response.message);
                false
            }
        } else {
            println!("{}", response.message);
            false
        }
    }
}

impl Default for Display {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ResponseData;

    #[test]
    fn test_display_new() {
        let display = Display::new();
        // Just ensure it can be created
        let _ = display;
    }

    #[test]
    fn test_display_default() {
        let display = Display;
        // Just ensure it can be created
        let _ = display;
    }

    #[test]
    fn test_show_start_success() {
        let display = Display::new();
        let response = IpcResponse::success("Timer started", None);
        // This should not panic
        display.show_start_success(response);
    }

    #[test]
    fn test_show_start_success_with_task() {
        let display = Display::new();
        let response = IpcResponse::success(
            "Timer started",
            Some(ResponseData {
                state: None,
                remaining_seconds: None,
                pomodoro_count: None,
                task_name: Some("Test task".to_string()),
                duration: None,
            }),
        );
        // This should not panic
        display.show_start_success(response);
    }

    #[test]
    fn test_show_pause_success() {
        let display = Display::new();
        let response = IpcResponse::success("Timer paused", None);
        // This should not panic
        display.show_pause_success(response);
    }

    #[test]
    fn test_show_resume_success() {
        let display = Display::new();
        let response = IpcResponse::success("Timer resumed", None);
        // This should not panic
        display.show_resume_success(response);
    }

    #[test]
    fn test_show_stop_success() {
        let display = Display::new();
        let response = IpcResponse::success("Timer stopped", None);
        // This should not panic
        display.show_stop_success(response);
    }

    #[test]
    fn test_show_status_with_data() {
        let display = Display::new();
        let response = IpcResponse::success(
            "Status retrieved",
            Some(ResponseData {
                state: Some("working".to_string()),
                remaining_seconds: Some(1500),
                pomodoro_count: Some(2),
                task_name: Some("Test task".to_string()),
                duration: Some(1500),
            }),
        );
        // This should not panic
        display.show_status(response);
    }

    #[test]
    fn test_show_status_without_data() {
        let display = Display::new();
        let response = IpcResponse::success("No timer running", None);
        // This should not panic
        display.show_status(response);
    }

    #[test]
    fn test_show_error() {
        let display = Display::new();
        // This should not panic
        display.show_error("Test error");
    }

    #[test]
    fn test_show_install_success() {
        let display = Display::new();
        // This should not panic
        display.show_install_success();
    }

    #[test]
    fn test_show_install_failure() {
        let display = Display::new();
        // This should not panic
        display.show_install_failure("Test error");
    }

    #[test]
    fn test_show_uninstall_success() {
        let display = Display::new();
        // This should not panic
        display.show_uninstall_success();
    }

    #[test]
    fn test_show_uninstall_failure() {
        let display = Display::new();
        // This should not panic
        display.show_uninstall_failure("Test error");
    }

    // ========================================================================
    // Enhanced Display Tests
    // ========================================================================

    #[test]
    fn test_enhanced_display_state_new() {
        let state = EnhancedDisplayState::new();
        assert!(state.current_phase.is_none());
    }

    #[test]
    fn test_enhanced_display_state_default() {
        let state = EnhancedDisplayState::default();
        assert!(state.current_phase.is_none());
    }

    #[test]
    fn test_update_status_enhanced_stopped() {
        let display = Display::new();
        let mut state = EnhancedDisplayState::new();

        let response = IpcResponse::success(
            "Timer stopped",
            Some(ResponseData {
                state: Some("stopped".to_string()),
                remaining_seconds: Some(0),
                pomodoro_count: None,
                task_name: None,
                duration: Some(1500),
            }),
        );

        let should_continue = display.update_status_enhanced(response, &mut state);
        assert!(!should_continue);
    }

    #[test]
    fn test_update_status_enhanced_no_data() {
        let display = Display::new();
        let mut state = EnhancedDisplayState::new();

        let response = IpcResponse::success("No timer running", None);

        let should_continue = display.update_status_enhanced(response, &mut state);
        assert!(!should_continue);
    }

    #[test]
    fn test_show_status_with_layout_renderer() {
        let display = Display::new();
        // Test that show_status uses LayoutRenderer when duration is available
        let response = IpcResponse::success(
            "Status",
            Some(ResponseData {
                state: Some("working".to_string()),
                remaining_seconds: Some(1200),
                pomodoro_count: Some(1),
                task_name: Some("コーディング".to_string()),
                duration: Some(1500),
            }),
        );
        // Should not panic and should use new layout
        display.show_status(response);
    }

    #[test]
    fn test_show_status_fallback_no_duration() {
        let display = Display::new();
        // Test fallback when no duration
        let response = IpcResponse::success(
            "Status",
            Some(ResponseData {
                state: Some("working".to_string()),
                remaining_seconds: Some(1200),
                pomodoro_count: Some(1),
                task_name: Some("タスク".to_string()),
                duration: None, // No duration - should use legacy display
            }),
        );
        display.show_status(response);
    }
}
