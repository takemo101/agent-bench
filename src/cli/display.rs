//! Display utilities for CLI output
//!
//! Provides colored and formatted output for CLI commands.

use crate::types::{IpcResponse, TimerPhase};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::str::FromStr;

/// Display handler for CLI output
pub struct Display;

impl Display {
    // Helper to create styled progress bar
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

    /// Show status information
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
                let bar = self.create_progress_bar(
                    phase,
                    duration as u64,
                    remaining as u64,
                    data.task_name.as_deref(),
                );
                bar.finish();
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

    /// Update status information in a loop
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
                // バーの作成または更新
                let b = if let Some(b) = bar {
                    b
                } else {
                    // 初回作成
                    let new_bar = self.create_progress_bar(
                        phase,
                        duration as u64,
                        remaining as u64,
                        data.task_name.as_deref(),
                    );
                    *bar = Some(new_bar);
                    bar.as_mut().unwrap()
                };

                // 位置更新
                b.set_position(duration as u64 - remaining as u64);

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
}
